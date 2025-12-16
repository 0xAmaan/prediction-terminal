//! Exa.ai API client for news search

use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use sha2::{Digest, Sha256};
use tracing::{debug, info, instrument};

use terminal_core::{NewsItem, NewsSearchParams, NewsSource};

use crate::error::NewsError;
use crate::types::{
    ExaContentsOptions, ExaHighlightsOptions, ExaResult, ExaSearchRequest, ExaSearchResponse,
    ExaTextOptions,
};

/// Exa.ai API client
pub struct ExaClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl ExaClient {
    /// Create a new Exa.ai client
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.exa.ai".to_string(),
        }
    }

    /// Search for news articles
    #[instrument(skip(self), fields(query = %params.query.as_deref().unwrap_or("global news")))]
    pub async fn search_news(&self, params: &NewsSearchParams) -> Result<Vec<NewsItem>, NewsError> {
        // Query should be provided dynamically based on current trending markets
        // Fallback to general breaking news if not provided
        let query = params
            .query
            .clone()
            .unwrap_or_else(|| "breaking news today".to_string());

        // Build date filter based on time_range
        let start_date = params.time_range.as_ref().and_then(|range| {
            let now = Utc::now();
            let duration = match range.as_str() {
                "1h" => Duration::hours(1),
                "6h" => Duration::hours(6),
                "12h" => Duration::hours(12),
                "24h" => Duration::hours(24),
                "7d" => Duration::days(7),
                "30d" => Duration::days(30),
                _ => return None,
            };
            Some((now - duration).to_rfc3339())
        });

        // Use neural search but with autoprompt disabled for more precise matching
        // Keyword search causes parse errors with Exa's API
        let request = ExaSearchRequest {
            query,
            num_results: params.limit * 5, // Request 5x since we'll filter heavily
            use_autoprompt: false,         // Disable autoprompt for more literal matching
            search_type: "neural".to_string(),
            category: Some("news".to_string()),
            start_published_date: start_date,
            end_published_date: None,
            exclude_domains: None,
            // WHITELIST: Only credible, major news sources
            include_domains: Some(vec![
                // Top-tier newspapers
                "nytimes.com".to_string(),
                "washingtonpost.com".to_string(),
                "wsj.com".to_string(),
                "ft.com".to_string(),
                "economist.com".to_string(),
                // Wire services (BEST for international coverage)
                "reuters.com".to_string(),
                "apnews.com".to_string(),
                "bloomberg.com".to_string(),
                "afp.com".to_string(),
                // Broadcast news
                "bbc.com".to_string(),
                "bbc.co.uk".to_string(),
                "cnn.com".to_string(),
                "nbcnews.com".to_string(),
                "cbsnews.com".to_string(),
                "abcnews.go.com".to_string(),
                "npr.org".to_string(),
                // International news (for non-US markets)
                "aljazeera.com".to_string(),
                "dw.com".to_string(),
                "france24.com".to_string(),
                "euronews.com".to_string(),
                // Quality online news
                "theguardian.com".to_string(),
                "politico.com".to_string(),
                "politico.eu".to_string(),
                "thehill.com".to_string(),
                "axios.com".to_string(),
                "cnbc.com".to_string(),
                // Tech news (for AI/crypto markets)
                "techcrunch.com".to_string(),
                "theverge.com".to_string(),
                "wired.com".to_string(),
                "arstechnica.com".to_string(),
                // Sports (for sports betting markets)
                "espn.com".to_string(),
                "theathletic.com".to_string(),
                // Science (for space/climate markets)
                "scientificamerican.com".to_string(),
                "nature.com".to_string(),
            ]),
            contents: ExaContentsOptions {
                text: ExaTextOptions {
                    max_characters: 1000,
                    include_html_tags: false,
                },
                highlights: ExaHighlightsOptions {
                    num_sentences: 3,
                    highlights_per_url: 1,
                },
            },
        };

        info!(
            "Searching Exa.ai: query='{}', start_date={:?}, num_results={}",
            request.query.chars().take(80).collect::<String>(),
            request.start_published_date,
            request.num_results
        );

        let response = self
            .client
            .post(format!("{}/search", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| NewsError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(NewsError::ApiError {
                status: status.as_u16(),
                message: body,
            });
        }

        let exa_response: ExaSearchResponse = response
            .json()
            .await
            .map_err(|e| NewsError::ParseError(e.to_string()))?;

        info!(
            "Received {} results from Exa.ai",
            exa_response.results.len()
        );

        // Log what we're getting from Exa for debugging
        for (i, r) in exa_response.results.iter().take(5).enumerate() {
            info!(
                "Exa result {}: title='{}', date={:?}, url='{}'",
                i,
                r.title.chars().take(50).collect::<String>(),
                r.published_date,
                r.url.chars().take(60).collect::<String>()
            );
        }

        // Calculate the cutoff date for filtering old articles
        let now = Utc::now();
        let max_age = params
            .time_range
            .as_ref()
            .and_then(|range| {
                match range.as_str() {
                    "1h" => Some(Duration::hours(2)), // Allow some buffer
                    "6h" => Some(Duration::hours(12)),
                    "12h" => Some(Duration::hours(24)),
                    "24h" => Some(Duration::days(2)),
                    "7d" => Some(Duration::days(14)),
                    "30d" => Some(Duration::days(60)),
                    _ => None,
                }
            })
            .unwrap_or_else(|| Duration::days(3)); // Default to 3 days max

        let cutoff_date = now - max_age;

        // Convert results and deduplicate similar stories
        let mut items: Vec<NewsItem> = Vec::new();
        let mut seen_titles: std::collections::HashSet<String> = std::collections::HashSet::new();

        for result in exa_response.results {
            // Skip if we already have enough items
            if items.len() >= params.limit {
                break;
            }

            // CRITICAL: Extract date from URL (Exa's dates are often wrong)
            // Most news URLs contain the date like /2025/12/09/ or /2025-12-09/
            let url_date = extract_date_from_url(&result.url);

            let published_at = match url_date {
                Some(dt) => {
                    // Skip if article is too old based on URL date
                    if dt < cutoff_date {
                        debug!("Skipping old article (URL date): {} ({})", result.title, dt);
                        continue;
                    }
                    // Skip if article has a future date (indicates bad/placeholder date)
                    if dt > now {
                        debug!(
                            "Skipping article with future date: {} ({})",
                            result.title, dt
                        );
                        continue;
                    }
                    dt
                }
                None => {
                    // Fallback to Exa's date if URL doesn't contain a date
                    match &result.published_date {
                        Some(date_str) => {
                            match DateTime::parse_from_rfc3339(date_str) {
                                Ok(parsed) => {
                                    let dt = parsed.with_timezone(&Utc);
                                    if dt < cutoff_date {
                                        debug!(
                                            "Skipping old article (Exa date): {} ({})",
                                            result.title, date_str
                                        );
                                        continue;
                                    }
                                    // Skip future dates
                                    if dt > now {
                                        debug!(
                                            "Skipping article with future Exa date: {} ({})",
                                            result.title, date_str
                                        );
                                        continue;
                                    }
                                    dt
                                }
                                Err(_) => continue,
                            }
                        }
                        None => continue,
                    }
                }
            };

            // Create a normalized title for deduplication
            let title_key = normalize_title(&result.title);

            // Skip if we've seen a similar title
            if seen_titles.contains(&title_key) {
                continue;
            }

            // Skip homepage URLs
            if let Ok(url) = url::Url::parse(&result.url) {
                let path = url.path();
                if path.len() <= 5 || path == "/" {
                    continue;
                }
            }

            seen_titles.insert(title_key);
            items.push(self.convert_result_with_date(
                result,
                params.market_id.clone(),
                published_at,
            ));
        }

        // Sort by published date, newest first
        items.sort_by(|a, b| b.published_at.cmp(&a.published_at));

        Ok(items)
    }

    /// Find similar articles to a given URL
    #[instrument(skip(self))]
    pub async fn find_similar(&self, url: &str, limit: usize) -> Result<Vec<NewsItem>, NewsError> {
        #[derive(serde::Serialize)]
        struct FindSimilarRequest {
            url: String,
            #[serde(rename = "numResults")]
            num_results: usize,
            contents: ExaContentsOptions,
        }

        let request = FindSimilarRequest {
            url: url.to_string(),
            num_results: limit,
            contents: ExaContentsOptions {
                text: ExaTextOptions {
                    max_characters: 1000,
                    include_html_tags: false,
                },
                highlights: ExaHighlightsOptions {
                    num_sentences: 3,
                    highlights_per_url: 1,
                },
            },
        };

        let response = self
            .client
            .post(format!("{}/findSimilar", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| NewsError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(NewsError::ApiError {
                status: status.as_u16(),
                message: body,
            });
        }

        let exa_response: ExaSearchResponse = response
            .json()
            .await
            .map_err(|e| NewsError::ParseError(e.to_string()))?;

        let items = exa_response
            .results
            .into_iter()
            .map(|r| self.convert_result(r, None))
            .collect();

        Ok(items)
    }

    /// Convert an Exa result to a NewsItem with a pre-validated date
    fn convert_result_with_date(
        &self,
        result: ExaResult,
        market_id: Option<String>,
        published_at: DateTime<Utc>,
    ) -> NewsItem {
        // Generate a unique ID from the URL
        let id = {
            let mut hasher = Sha256::new();
            hasher.update(result.url.as_bytes());
            hex::encode(&hasher.finalize()[..8])
        };

        // Extract source from URL
        let source = extract_source(&result.url);

        // Build summary from highlights or text
        let summary = result
            .highlights
            .as_ref()
            .and_then(|h| h.first().cloned())
            .or_else(|| {
                result.text.as_ref().map(|t| {
                    let chars: String = t.chars().take(300).collect();
                    if t.len() > 300 {
                        format!("{}...", chars)
                    } else {
                        chars
                    }
                })
            })
            .unwrap_or_default();

        NewsItem {
            id,
            title: result.title,
            url: result.url,
            published_at,
            source,
            summary,
            content: result.text,
            image_url: result.image,
            relevance_score: result.score,
            related_market_ids: market_id.into_iter().collect(),
            search_query: None,
            // AI-enriched fields (set later by NewsAnalyzer)
            matched_market: None,
            price_signal: None,
            suggested_action: None,
            signal_reasoning: None,
        }
    }

    /// Convert an Exa result to a NewsItem (legacy, for find_similar)
    fn convert_result(&self, result: ExaResult, market_id: Option<String>) -> NewsItem {
        let published_at = result
            .published_date
            .as_ref()
            .and_then(|d| DateTime::parse_from_rfc3339(d).ok())
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        self.convert_result_with_date(result, market_id, published_at)
    }
}

/// Extract the actual publication date from a URL
/// Most news sites include the date in their URL structure
fn extract_date_from_url(url: &str) -> Option<DateTime<Utc>> {
    // Pattern 1: /2025/12/09/ or /2025/12/9/
    let slash_pattern = regex::Regex::new(r"/(\d{4})/(\d{1,2})/(\d{1,2})/").ok()?;
    if let Some(caps) = slash_pattern.captures(url) {
        let year: i32 = caps.get(1)?.as_str().parse().ok()?;
        let month: u32 = caps.get(2)?.as_str().parse().ok()?;
        let day: u32 = caps.get(3)?.as_str().parse().ok()?;

        if let Some(date) = chrono::NaiveDate::from_ymd_opt(year, month, day) {
            return Some(DateTime::from_naive_utc_and_offset(
                date.and_hms_opt(12, 0, 0)?,
                Utc,
            ));
        }
    }

    // Pattern 2: /2025-12-09/ or -2025-12-09-
    let dash_pattern = regex::Regex::new(r"[/-](\d{4})-(\d{2})-(\d{2})[/-]").ok()?;
    if let Some(caps) = dash_pattern.captures(url) {
        let year: i32 = caps.get(1)?.as_str().parse().ok()?;
        let month: u32 = caps.get(2)?.as_str().parse().ok()?;
        let day: u32 = caps.get(3)?.as_str().parse().ok()?;

        if let Some(date) = chrono::NaiveDate::from_ymd_opt(year, month, day) {
            return Some(DateTime::from_naive_utc_and_offset(
                date.and_hms_opt(12, 0, 0)?,
                Utc,
            ));
        }
    }

    // Pattern 3: /2025/dec/10/ (Guardian-style text month names)
    let text_month_pattern =
        regex::Regex::new(r"/(\d{4})/(jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)/(\d{1,2})/")
            .ok()?;
    if let Some(caps) = text_month_pattern.captures(&url.to_lowercase()) {
        let year: i32 = caps.get(1)?.as_str().parse().ok()?;
        let month_str = caps.get(2)?.as_str();
        let day: u32 = caps.get(3)?.as_str().parse().ok()?;

        let month: u32 = match month_str {
            "jan" => 1,
            "feb" => 2,
            "mar" => 3,
            "apr" => 4,
            "may" => 5,
            "jun" => 6,
            "jul" => 7,
            "aug" => 8,
            "sep" => 9,
            "oct" => 10,
            "nov" => 11,
            "dec" => 12,
            _ => return None,
        };

        if let Some(date) = chrono::NaiveDate::from_ymd_opt(year, month, day) {
            return Some(DateTime::from_naive_utc_and_offset(
                date.and_hms_opt(12, 0, 0)?,
                Utc,
            ));
        }
    }

    None
}

/// Normalize a title for deduplication
/// Extracts key words to detect duplicate stories from different sources
fn normalize_title(title: &str) -> String {
    let stop_words = [
        "the", "and", "for", "that", "this", "with", "from", "says", "said", "will", "would",
        "could", "about", "after", "into", "over", "more", "than", "been", "have", "were", "what",
        "when", "where", "which", "their", "there", "these", "those", "some", "just", "also",
        "news", "report", "reports",
    ];

    let lower = title.to_lowercase();
    let words: Vec<&str> = lower
        .split_whitespace()
        .filter(|w| w.len() > 3)
        .filter(|w| !stop_words.contains(w))
        .take(5)
        .collect();
    words.join(" ")
}

/// Extract source information from a URL
fn extract_source(url: &str) -> NewsSource {
    let parsed = url::Url::parse(url).ok();

    let host = parsed
        .as_ref()
        .and_then(|u| u.host_str())
        .unwrap_or("Unknown");

    // Clean up the host name for display
    let name = host
        .strip_prefix("www.")
        .unwrap_or(host)
        .split('.')
        .next()
        .unwrap_or(host)
        .to_string();

    // Capitalize first letter
    let name = if let Some(first) = name.chars().next() {
        format!("{}{}", first.to_uppercase(), &name[1..])
    } else {
        name
    };

    let base_url = parsed
        .as_ref()
        .map(|u| format!("{}://{}", u.scheme(), u.host_str().unwrap_or("")))
        .unwrap_or_else(|| url.to_string());

    NewsSource {
        name,
        url: base_url.clone(),
        favicon_url: Some(format!("{}/favicon.ico", base_url)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_source() {
        let source = extract_source("https://www.reuters.com/article/test");
        assert_eq!(source.name, "Reuters");
        assert_eq!(source.url, "https://www.reuters.com");

        let source = extract_source("https://bloomberg.com/news/test");
        assert_eq!(source.name, "Bloomberg");
    }
}
