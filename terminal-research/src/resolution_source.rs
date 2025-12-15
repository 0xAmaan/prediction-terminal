//! Resolution source URL extraction and fetching
//!
//! This module provides utilities to extract URLs from resolution criteria text
//! and fetch their content to provide additional context for research.

use chrono::Utc;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::time::Duration;
use terminal_core::TerminalError;
use tracing::{info, warn};

use crate::types::ResolutionSourceData;

/// Client for fetching resolution source URLs
#[derive(Debug, Clone)]
pub struct ResolutionSourceFetcher {
    client: Client,
}

impl ResolutionSourceFetcher {
    /// Create a new resolution source fetcher
    pub fn new() -> Result<Self, TerminalError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .map_err(|e| TerminalError::internal(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client })
    }

    /// Fetch content from a URL and extract relevant text
    pub async fn fetch_url(&self, url: &str) -> Result<ResolutionSourceData, TerminalError> {
        info!("Fetching resolution source URL: {}", url);

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| TerminalError::api(format!("Failed to fetch {}: {}", url, e)))?;

        if !response.status().is_success() {
            return Err(TerminalError::api(format!(
                "Failed to fetch {}: HTTP {}",
                url,
                response.status()
            )));
        }

        let html = response
            .text()
            .await
            .map_err(|e| TerminalError::api(format!("Failed to read response from {}: {}", url, e)))?;

        // Parse HTML and extract text content
        let content = extract_text_from_html(&html, url);

        Ok(ResolutionSourceData {
            url: url.to_string(),
            content,
            fetched_at: Utc::now(),
        })
    }
}

/// Extract URLs from resolution rules text
///
/// Looks for URLs in the text that might be resolution sources.
/// Returns unique URLs found in the text.
pub fn extract_urls_from_text(text: &str) -> Vec<String> {
    // URL regex pattern - matches http:// and https:// URLs
    let url_regex = Regex::new(
        r"https?://[a-zA-Z0-9\-._~:/?#\[\]@!$&'()*+,;=%]+"
    ).expect("Invalid URL regex");

    let mut urls: HashSet<String> = HashSet::new();

    for cap in url_regex.find_iter(text) {
        let mut url = cap.as_str().to_string();

        // Clean up trailing punctuation that might have been captured
        while url.ends_with('.') || url.ends_with(',') || url.ends_with(')') || url.ends_with(']') {
            url.pop();
        }

        // Skip common non-content URLs
        if should_skip_url(&url) {
            continue;
        }

        urls.insert(url);
    }

    urls.into_iter().collect()
}

/// Check if a URL should be skipped (not useful for resolution context)
fn should_skip_url(url: &str) -> bool {
    let skip_patterns = [
        "polymarket.com",
        "kalshi.com",
        "twitter.com",
        "x.com",
        "facebook.com",
        "instagram.com",
        "youtube.com/watch", // Skip individual videos but not channels
        "reddit.com",
        "discord.com",
        "t.me",
        "mailto:",
        ".pdf",   // PDFs are harder to parse
        ".png",
        ".jpg",
        ".jpeg",
        ".gif",
        ".svg",
    ];

    let url_lower = url.to_lowercase();
    skip_patterns.iter().any(|pattern| url_lower.contains(pattern))
}

/// Extract text content from HTML
///
/// Uses smart extraction to get the most relevant content:
/// - Looks for tables (common in leaderboards)
/// - Looks for main content areas
/// - Falls back to body text
fn extract_text_from_html(html: &str, url: &str) -> String {
    let document = Html::parse_document(html);

    // Different extraction strategies based on URL
    let content = if url.contains("lmarena.ai") || url.contains("leaderboard") {
        extract_leaderboard_content(&document)
    } else {
        extract_general_content(&document)
    };

    // Truncate to reasonable length for AI context
    let max_chars = 8000;
    if content.len() > max_chars {
        format!("{}...\n[Content truncated]", &content[..max_chars])
    } else {
        content
    }
}

/// Extract content from leaderboard-style pages
fn extract_leaderboard_content(document: &Html) -> String {
    let mut content = String::new();

    // Try to find tables first (leaderboards are often in tables)
    let table_selector = Selector::parse("table").ok();
    if let Some(selector) = table_selector {
        for table in document.select(&selector) {
            // Get table headers
            if let Some(thead_selector) = Selector::parse("thead th, thead td").ok() {
                let headers: Vec<String> = table
                    .select(&thead_selector)
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if !headers.is_empty() {
                    content.push_str("Headers: ");
                    content.push_str(&headers.join(" | "));
                    content.push('\n');
                }
            }

            // Get table rows (limit to top entries for leaderboards)
            if let Some(row_selector) = Selector::parse("tbody tr").ok() {
                for (i, row) in table.select(&row_selector).enumerate() {
                    if i >= 20 {
                        // Limit to top 20 entries
                        content.push_str("... (more entries)\n");
                        break;
                    }

                    if let Some(cell_selector) = Selector::parse("td, th").ok() {
                        let cells: Vec<String> = row
                            .select(&cell_selector)
                            .map(|el| el.text().collect::<String>().trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        if !cells.is_empty() {
                            content.push_str(&format!("{}. {}\n", i + 1, cells.join(" | ")));
                        }
                    }
                }
            }
            content.push('\n');
        }
    }

    // If no tables, look for list items that might be rankings
    if content.is_empty() {
        if let Some(li_selector) = Selector::parse("li, .ranking, .leaderboard-item").ok() {
            for (i, item) in document.select(&li_selector).enumerate() {
                if i >= 20 {
                    break;
                }
                let text: String = item.text().collect::<String>().trim().to_string();
                if !text.is_empty() && text.len() < 500 {
                    content.push_str(&format!("- {}\n", text));
                }
            }
        }
    }

    // If still empty, fall back to general content
    if content.is_empty() {
        content = extract_general_content(document);
    }

    content
}

/// Extract general content from a web page
fn extract_general_content(document: &Html) -> String {
    let mut content = String::new();

    // Try to get main content area first
    let main_selectors = ["main", "article", "#content", ".content", "#main", ".main"];

    for selector_str in main_selectors {
        if let Some(selector) = Selector::parse(selector_str).ok() {
            for element in document.select(&selector) {
                let text: String = element.text().collect::<String>();
                let cleaned = clean_text(&text);
                if !cleaned.is_empty() {
                    content.push_str(&cleaned);
                    content.push('\n');
                }
            }
        }
        if !content.is_empty() {
            break;
        }
    }

    // Fall back to body if no main content found
    if content.is_empty() {
        if let Some(body_selector) = Selector::parse("body").ok() {
            for body in document.select(&body_selector) {
                let text: String = body.text().collect::<String>();
                content = clean_text(&text);
            }
        }
    }

    content
}

/// Clean and normalize text content
fn clean_text(text: &str) -> String {
    // Split into lines, trim, and filter empty lines
    let lines: Vec<&str> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    // Join with single newlines
    lines.join("\n")
}

/// Maximum number of resolution source URLs to fetch
const MAX_RESOLUTION_SOURCES: usize = 5;

/// Fetch multiple resolution source URLs concurrently
///
/// Returns successfully fetched sources (failures are logged but not returned)
pub async fn fetch_resolution_sources(
    resolution_rules: Option<&str>,
) -> Vec<ResolutionSourceData> {
    let Some(rules) = resolution_rules else {
        return Vec::new();
    };

    let urls = extract_urls_from_text(rules);
    if urls.is_empty() {
        return Vec::new();
    }

    info!("Found {} URLs in resolution rules to fetch", urls.len());

    let fetcher = match ResolutionSourceFetcher::new() {
        Ok(f) => f,
        Err(e) => {
            warn!("Failed to create resolution source fetcher: {}", e);
            return Vec::new();
        }
    };

    // Fetch URLs concurrently (limit to MAX_RESOLUTION_SOURCES)
    let fetch_futures: Vec<_> = urls
        .iter()
        .take(MAX_RESOLUTION_SOURCES)
        .map(|url| {
            let fetcher = fetcher.clone();
            let url = url.clone();
            async move {
                match fetcher.fetch_url(&url).await {
                    Ok(data) => {
                        info!(
                            "Successfully fetched resolution source: {} ({} chars)",
                            url,
                            data.content.len()
                        );
                        Some(data)
                    }
                    Err(e) => {
                        warn!("Failed to fetch resolution source {}: {}", url, e);
                        None
                    }
                }
            }
        })
        .collect();

    // Wait for all fetches to complete and collect successful results
    futures::future::join_all(fetch_futures)
        .await
        .into_iter()
        .flatten()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_urls_from_text() {
        let text = r#"
        Results from the "Arena Score" section on the Leaderboard tab of https://lmarena.ai/
        will be used to resolve this market.

        This market will resolve based on https://lmarena.ai/leaderboard/text when checked.
        "#;

        let urls = extract_urls_from_text(text);
        assert!(urls.contains(&"https://lmarena.ai/".to_string()));
        assert!(urls.contains(&"https://lmarena.ai/leaderboard/text".to_string()));
    }

    #[test]
    fn test_skip_polymarket_urls() {
        let text = "Check https://polymarket.com/event/test and https://example.com/data";
        let urls = extract_urls_from_text(text);

        assert!(!urls.iter().any(|u| u.contains("polymarket.com")));
        assert!(urls.contains(&"https://example.com/data".to_string()));
    }

    #[test]
    fn test_clean_trailing_punctuation() {
        let text = "See https://example.com/page. And also https://test.com/data,";
        let urls = extract_urls_from_text(text);

        assert!(urls.contains(&"https://example.com/page".to_string()));
        assert!(urls.contains(&"https://test.com/data".to_string()));
    }
}
