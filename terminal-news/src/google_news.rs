//! Google News RSS client for market-specific news
//!
//! Fetches news from Google News RSS API using dynamic queries.
//! Google News provides fresh, relevant news with excellent search capabilities.

use reqwest::Client;
use tracing::info;

use terminal_core::NewsItem;

use crate::error::NewsError;
use crate::rss_client::RssFeed;

/// Google News RSS client
pub struct GoogleNewsClient {
    client: Client,
    base_url: String,
}

impl GoogleNewsClient {
    /// Create a new Google News client
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .user_agent("Mozilla/5.0 (compatible; PredictionTerminal/1.0)")
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url: "https://news.google.com/rss/search".to_string(),
        }
    }

    /// Search Google News for a specific market
    ///
    /// # Arguments
    /// * `market_title` - The market question/title
    /// * `outcome_titles` - Optional list of outcome names (for multi-outcome markets)
    /// * `limit` - Maximum number of results
    ///
    /// # Returns
    /// Vec of NewsItems from Google News RSS
    pub async fn search_market_news(
        &self,
        market_title: &str,
        outcome_titles: Option<&Vec<String>>,
        limit: usize,
    ) -> Result<Vec<NewsItem>, NewsError> {
        // Build optimized search query
        let query = build_google_query(market_title, outcome_titles);

        info!(
            "Google News search: market='{}', query='{}', limit={}",
            market_title, query, limit
        );

        // Construct Google News RSS URL
        let url = format!(
            "{}?q={}&hl=en&gl=US&ceid=US:en",
            self.base_url,
            urlencoding::encode(&query)
        );

        info!("Fetching Google News RSS: {}", url);

        // Fetch the RSS feed
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| NewsError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(NewsError::ApiError {
                status: response.status().as_u16(),
                message: format!("Google News returned status {}", response.status()),
            });
        }

        let content = response
            .bytes()
            .await
            .map_err(|e| NewsError::RequestFailed(e.to_string()))?;

        // Parse RSS feed (Google News uses standard RSS 2.0 format)
        let channel = rss::Channel::read_from(&content[..]).map_err(|e| {
            NewsError::ParseError(format!("Failed to parse Google News RSS: {}", e))
        })?;

        // Convert to NewsItems using the shared RssFeed logic
        let feed = RssFeed {
            name: "Google News".to_string(),
            url: url.clone(),
            base_url: "https://news.google.com".to_string(),
            categories: vec!["search".to_string()],
        };

        let mut items = parse_google_news_channel(&channel, &feed);

        info!(
            "Google News returned {} items for '{}'",
            items.len(),
            market_title
        );

        // Limit results
        items.truncate(limit);

        // Backfill missing thumbnails by fetching article pages
        let items_without_images: Vec<usize> = items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.image_url.is_none())
            .map(|(i, _)| i)
            .collect();

        if !items_without_images.is_empty() {
            info!(
                "Attempting to fetch thumbnails for {} Google News articles without images",
                items_without_images.len()
            );

            // Limit concurrent fetches
            let mut fetch_count = 0;
            for idx in items_without_images {
                if fetch_count >= 5 {
                    // Limit to 5 for Google News (already more targeted)
                    break;
                }

                if let Ok(image_url) = self.fetch_article_thumbnail(&items[idx].url).await {
                    items[idx].image_url = Some(image_url);
                    fetch_count += 1;
                }
            }

            if fetch_count > 0 {
                info!(
                    "Successfully fetched {} additional thumbnails for Google News",
                    fetch_count
                );
            }
        }

        Ok(items)
    }

    /// Fetch thumbnail from actual article page (for items without images from Google News RSS)
    async fn fetch_article_thumbnail(&self, url: &str) -> Result<String, NewsError> {
        let response = self
            .client
            .get(url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            )
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| NewsError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(NewsError::ApiError {
                status: response.status().as_u16(),
                message: format!("Failed to fetch article"),
            });
        }

        let html = response
            .text()
            .await
            .map_err(|e| NewsError::RequestFailed(e.to_string()))?;

        // Try to extract og:image or twitter:image from the actual article page
        extract_meta_image(&html)
            .or_else(|| extract_video_thumbnail(&html))
            .or_else(|| extract_link_image(&html))
            .ok_or_else(|| NewsError::ParseError("No thumbnail found".to_string()))
    }
}

impl Default for GoogleNewsClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Build an optimized Google News search query from market context
///
/// Strategy:
/// 1. Extract key entities (people, places, companies, events)
/// 2. Add context terms from outcomes
/// 3. Add domain-specific context words for better relevance
/// 4. Keep query focused (6-10 key terms)
/// 5. Google News does its own semantic matching
fn build_google_query(market_title: &str, outcome_titles: Option<&Vec<String>>) -> String {
    let mut terms = Vec::new();

    // Extract proper nouns and key terms from market title
    let title_terms = extract_key_terms(market_title);
    terms.extend(title_terms);

    // Add key terms from outcomes (people names, companies, etc.)
    // Skip price-based outcomes (e.g., "$7,000", "$10,000")
    if let Some(outcomes) = outcome_titles {
        // Check if outcomes are price targets (most start with $ or are numbers)
        let are_prices = outcomes.iter().take(5).all(|o| {
            let trimmed = o.trim();
            trimmed.starts_with('$') || trimmed.chars().all(|c| c.is_numeric() || c == ',' || c == '.')
        });

        // Only extract terms from non-price outcomes
        if !are_prices {
            for outcome in outcomes.iter().take(5) {
                let outcome_terms = extract_key_terms(outcome);
                for term in outcome_terms {
                    // Avoid duplicates
                    if !terms.iter().any(|t| t.eq_ignore_ascii_case(&term)) {
                        terms.push(term);
                    }
                }
            }
        }
    }

    // Add domain-specific context terms for better relevance
    let lower_title = market_title.to_lowercase();
    let context_terms = get_context_terms(&lower_title);
    for context in context_terms {
        if !terms.iter().any(|t| t.eq_ignore_ascii_case(context)) {
            terms.push(context.to_string());
        }
    }

    // Expand abbreviations to full names for better search
    let expanded: Vec<String> = terms
        .into_iter()
        .map(|term| {
            // Expand common abbreviations
            match term.as_str() {
                "US" => "United States".to_string(),
                "UK" => "United Kingdom".to_string(),
                "EU" => "European Union".to_string(),
                _ => term,
            }
        })
        .collect();

    // Limit to top 10 terms (focused but specific)
    let mut final_terms = expanded;
    final_terms.truncate(10);

    // Join with spaces
    final_terms.join(" ")
}

/// Get domain-specific context terms based on market topic
/// This adds relevant keywords to make the query more specific
fn get_context_terms(market_title_lower: &str) -> Vec<&'static str> {
    let mut context = Vec::new();

    // Climate/Environment
    if market_title_lower.contains("climate") || market_title_lower.contains("emission") {
        context.extend_from_slice(&["emissions", "carbon", "Paris agreement", "target"]);
    }

    // Elections/Politics
    if market_title_lower.contains("election") || market_title_lower.contains("president") {
        context.extend_from_slice(&["polls", "campaign", "candidate"]);
    }

    // Economics/Markets
    if market_title_lower.contains("recession") || market_title_lower.contains("gdp") {
        context.extend_from_slice(&["economy", "growth", "inflation"]);
    }

    // Crypto/Blockchain
    if market_title_lower.contains("bitcoin") || market_title_lower.contains("crypto") {
        context.extend_from_slice(&["cryptocurrency", "blockchain", "price"]);
    }

    // AI/Technology
    if market_title_lower.contains(" ai ") || market_title_lower.contains("artificial intelligence")
    {
        context.extend_from_slice(&["artificial intelligence", "technology", "OpenAI"]);
    }

    // Space
    if market_title_lower.contains("mars") || market_title_lower.contains("spacex") {
        context.extend_from_slice(&["SpaceX", "NASA", "rocket", "launch"]);
    }

    // Sports
    if market_title_lower.contains("super bowl") || market_title_lower.contains("nfl") {
        context.extend_from_slice(&["NFL", "football", "playoffs"]);
    }
    if market_title_lower.contains("nba") || market_title_lower.contains("basketball") {
        context.extend_from_slice(&["NBA", "basketball", "championship"]);
    }

    // IPO/Business
    if market_title_lower.contains("ipo") {
        context.extend_from_slice(&["IPO", "stock", "listing", "public"]);
    }

    // Take only top 3 context terms to keep query focused
    context.truncate(3);
    context
}

/// Extract key terms from text (proper nouns, important words)
fn extract_key_terms(text: &str) -> Vec<String> {
    let mut terms = Vec::new();

    // Stop words and overly specific terms to filter out
    let stop_words: std::collections::HashSet<&str> = [
        // Question words
        "will",
        "what",
        "who",
        "which",
        "when",
        "where",
        "how",
        "why",
        // Articles and common words
        "the",
        "a",
        "an",
        "in",
        "on",
        "at",
        "to",
        "for",
        "of",
        "with",
        "by",
        "from",
        "be",
        "is",
        "are",
        "was",
        "were",
        "been",
        "being",
        "have",
        "has",
        "had",
        "do",
        "does",
        "did",
        "will",
        "would",
        "could",
        "should",
        "may",
        "might",
        "can",
        "must",
        "win",
        "lose",
        "become",
        "next",
        "first",
        "before",
        "after",
        "market",
        "prediction",
        "value",
        "hit",
        "reach",
        // Month names - too specific for news search
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
        "jan",
        "feb",
        "mar",
        "apr",
        "jun",
        "jul",
        "aug",
        "sep",
        "sept",
        "oct",
        "nov",
        "dec",
    ]
    .into_iter()
    .collect();

    // High-value terms that should be included even if not capitalized
    let important_terms: std::collections::HashSet<&str> = [
        "bitcoin",
        "btc",
        "ethereum",
        "crypto",
        "ai",
        "trillionaire",
        "billionaire",
        "president",
        "election",
        "championship",
        "super",
        "bowl",
        "olympics",
        "earthquake",
        "hurricane",
        "war",
        "peace",
        "treaty",
        "ceasefire",
        "ipo",
        "acquisition",
        "merger",
        "bankruptcy",
        // Country/region codes - CRITICAL for geography-specific markets
        "us",
        "uk",
        "eu",
        "un",
        "uae",
        "china",
        "russia",
        "india",
        "japan",
        // Sports leagues
        "nba",
        "nfl",
        "mlb",
        "nhl",
        "fifa",
        "uefa",
        // Economic indicators
        "gdp",
        "cpi",
        "fed",
    ]
    .into_iter()
    .collect();

    // Split into words and extract meaningful terms
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut i = 0;

    while i < words.len() {
        let word = words[i];
        let clean: String = word
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '$')
            .collect();
        let lower = clean.to_lowercase();

        // Check if it's an important term FIRST (before length check)
        // This allows short but critical terms like "US", "UK", "AI", "EU"
        if important_terms.contains(lower.as_str()) {
            if !terms
                .iter()
                .any(|t: &String| t.eq_ignore_ascii_case(&clean))
            {
                terms.push(clean);
            }
            i += 1;
            continue;
        }

        // Skip stop words and very short words
        if clean.len() < 3 || stop_words.contains(lower.as_str()) {
            i += 1;
            continue;
        }

        // Skip date-like patterns (numbers, date ranges like "8-14")
        let is_date_like = clean.chars().all(|c| c.is_numeric() || c == '-');
        if is_date_like {
            i += 1;
            continue;
        }

        // Skip years (2024, 2025, 2026, etc.) - too specific for general news
        if clean.len() == 4 && clean.chars().all(|c| c.is_numeric()) {
            if let Ok(year) = clean.parse::<i32>() {
                if year >= 2020 && year <= 2100 {
                    i += 1;
                    continue;
                }
            }
        }

        // Skip year ranges (2025-26, 2024-25, etc.)
        if clean.contains('-') && clean.len() <= 7 {
            let parts: Vec<&str> = clean.split('-').collect();
            if parts.len() == 2 {
                let all_numeric = parts.iter().all(|p| p.chars().all(|c| c.is_numeric()));
                if all_numeric {
                    i += 1;
                    continue;
                }
            }
        }

        // Skip prediction-specific terms that don't appear in news
        let prediction_terms: std::collections::HashSet<&str> = [
            "winner",
            "champion", // Keep "championship" in important_terms
            "announce",
            "prediction",
            "predict",
            "odds",
            "favorite",
        ]
        .into_iter()
        .collect();

        if prediction_terms.contains(lower.as_str()) {
            i += 1;
            continue;
        }

        // Check if it's a proper noun (capitalized) or special format (like $100k)
        let is_capitalized = word
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false);
        let has_dollar = clean.starts_with('$');
        let has_number = clean.chars().any(|c| c.is_numeric());

        if is_capitalized || has_dollar {
            // Look for consecutive capitalized words (person names, places)
            let mut phrase_words = vec![clean.clone()];
            let mut j = i + 1;

            // Collect consecutive capitalized words (up to 3)
            while j < words.len() && phrase_words.len() < 3 {
                let next = words[j];
                let next_clean: String = next.chars().filter(|c| c.is_alphanumeric()).collect();
                let next_cap = next
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);

                if next_cap
                    && next_clean.len() >= 2
                    && !stop_words.contains(next_clean.to_lowercase().as_str())
                {
                    phrase_words.push(next_clean);
                    j += 1;
                } else {
                    break;
                }
            }

            // Add the phrase
            let phrase = phrase_words.join(" ");
            if !terms.iter().any(|t| t.eq_ignore_ascii_case(&phrase)) {
                terms.push(phrase);
            }

            i = j;
        } else if has_number && clean.len() >= 3 {
            // Numbers like "2025", "100k", "50%" are useful
            if !terms.iter().any(|t| t.eq_ignore_ascii_case(&clean)) {
                terms.push(clean);
            }
            i += 1;
        } else {
            i += 1;
        }
    }

    terms
}

/// Parse Google News RSS channel into NewsItems
/// Google News RSS has a specific format with source attribution
fn parse_google_news_channel(channel: &rss::Channel, _feed: &RssFeed) -> Vec<NewsItem> {
    use chrono::{DateTime, Utc};
    use sha2::{Digest, Sha256};

    channel
        .items()
        .iter()
        .filter_map(|item| {
            let title = item.title()?.to_string();
            let url = item.link()?.to_string();

            // Skip Polymarket URLs (these are market links, not news articles)
            if url.contains("polymarket.com") || url.contains("kalshi.com") {
                return None;
            }

            // Also filter by title patterns (Google News may use redirect URLs)
            // Polymarket titles end with "Predict..." or contain "Predictions & Odds"
            if title.contains("Predict...")
                || title.contains("Predictions & Odds")
                || title.contains("Prediction & Odds")
                || (title.contains("polymarket.com") || title.contains("kalshi.com")) {
                return None;
            }

            // Parse publication date - try multiple formats
            let published_at = item
                .pub_date()
                .and_then(|d| {
                    DateTime::parse_from_rfc2822(d)
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc))
                        .or_else(|| {
                            DateTime::parse_from_rfc3339(d)
                                .ok()
                                .map(|dt| dt.with_timezone(&Utc))
                        })
                });

            // Check for updated date in Dublin Core extensions
            let updated_at = item
                .dublin_core_ext()
                .and_then(|dc| dc.dates().first())
                .and_then(|d| {
                    DateTime::parse_from_rfc3339(d)
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc))
                        .or_else(|| {
                            DateTime::parse_from_rfc2822(d)
                                .ok()
                                .map(|dt| dt.with_timezone(&Utc))
                        })
                });

            // Use the most recent date (prefer updated over published)
            let published_at = match (published_at, updated_at) {
                (Some(pub_date), Some(upd_date)) => {
                    if upd_date > pub_date {
                        upd_date
                    } else {
                        pub_date
                    }
                }
                (Some(pub_date), None) => pub_date,
                (None, Some(upd_date)) => upd_date,
                (None, None) => Utc::now(),
            };

            // Generate ID from URL
            let id = {
                let mut hasher = Sha256::new();
                hasher.update(url.as_bytes());
                hex::encode(&hasher.finalize()[..8])
            };

            // Extract summary from description
            let summary = item
                .description()
                .map(|d| strip_html_simple(d))
                .unwrap_or_default();

            // Extract source from title (Google News includes source in title like "Title - Source")
            let (clean_title, source_name) = extract_source_from_google_title(&title);

            // Try to extract image from description
            let image_url = item
                .description()
                .and_then(|d| extract_image_from_html_simple(d));

            Some(NewsItem {
                id,
                title: clean_title,
                url,
                published_at,
                source: terminal_core::NewsSource {
                    name: source_name,
                    url: "https://news.google.com".to_string(),
                    favicon_url: None,
                },
                summary,
                content: None,
                image_url,
                relevance_score: 0.9, // Google News pre-filters for relevance
                related_market_ids: vec![],
                search_query: None,
                // AI-enriched fields (set later by NewsAnalyzer)
                matched_market: None,
                price_signal: None,
                suggested_action: None,
                signal_reasoning: None,
            })
        })
        .collect()
}

/// Extract source name from Google News title format: "Article Title - Source Name"
fn extract_source_from_google_title(title: &str) -> (String, String) {
    if let Some(pos) = title.rfind(" - ") {
        let clean_title = title[..pos].trim().to_string();
        let source = title[pos + 3..].trim().to_string();
        (clean_title, source)
    } else {
        (title.to_string(), "Google News".to_string())
    }
}

/// Simple HTML stripping
fn strip_html_simple(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    result
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Extract image URL from HTML (improved quality selection with multiple strategies)
fn extract_image_from_html_simple(html: &str) -> Option<String> {
    // Strategy 1: Try Open Graph and Twitter Card meta tags (highest quality)
    if let Some(url) = extract_meta_image(html) {
        return Some(url);
    }

    // Strategy 2: Try video thumbnails (YouTube, Vimeo, etc.)
    if let Some(url) = extract_video_thumbnail(html) {
        return Some(url);
    }

    // Strategy 3: Try <link rel="image_src"> tag
    if let Some(url) = extract_link_image(html) {
        return Some(url);
    }

    // Strategy 4: Find all <img> tags and score them
    let img_pattern = regex::Regex::new(r#"<img[^>]+src=["']([^"']+)["']"#).ok()?;

    let mut candidates: Vec<String> = img_pattern
        .captures_iter(html)
        .filter_map(|caps| caps.get(1).map(|m| m.as_str().to_string()))
        .filter(|url| {
            // Filter out junk images
            !url.contains("1x1")
                && !url.contains("pixel")
                && !url.contains("spacer")
                && !url.contains("icon")
                && !url.contains("logo")
                && !url.ends_with(".svg")
                && !url.ends_with(".gif")
        })
        .collect();

    if candidates.is_empty() {
        return None;
    }

    // Score and pick best image
    candidates.sort_by(|a, b| {
        let a_score = score_image_quality(a);
        let b_score = score_image_quality(b);
        b_score.cmp(&a_score)
    });

    candidates.into_iter().next()
}

/// Extract image from Open Graph or Twitter Card meta tags
fn extract_meta_image(html: &str) -> Option<String> {
    // Try og:image first (Open Graph)
    let og_pattern =
        regex::Regex::new(r#"<meta[^>]+property=["']og:image["'][^>]+content=["']([^"']+)["']"#)
            .ok()?;
    if let Some(caps) = og_pattern.captures(html) {
        if let Some(url) = caps.get(1) {
            return Some(url.as_str().to_string());
        }
    }

    // Try reversed attribute order
    let og_pattern_rev =
        regex::Regex::new(r#"<meta[^>]+content=["']([^"']+)["'][^>]+property=["']og:image["']"#)
            .ok()?;
    if let Some(caps) = og_pattern_rev.captures(html) {
        if let Some(url) = caps.get(1) {
            return Some(url.as_str().to_string());
        }
    }

    // Try twitter:image
    let twitter_pattern =
        regex::Regex::new(r#"<meta[^>]+name=["']twitter:image["'][^>]+content=["']([^"']+)["']"#)
            .ok()?;
    if let Some(caps) = twitter_pattern.captures(html) {
        if let Some(url) = caps.get(1) {
            return Some(url.as_str().to_string());
        }
    }

    // Try reversed attribute order
    let twitter_pattern_rev =
        regex::Regex::new(r#"<meta[^>]+content=["']([^"']+)["'][^>]+name=["']twitter:image["']"#)
            .ok()?;
    if let Some(caps) = twitter_pattern_rev.captures(html) {
        if let Some(url) = caps.get(1) {
            return Some(url.as_str().to_string());
        }
    }

    None
}

/// Extract video thumbnail from YouTube, Vimeo, or other video embeds
fn extract_video_thumbnail(html: &str) -> Option<String> {
    // YouTube embed: extract video ID and construct thumbnail URL
    let youtube_pattern = regex::Regex::new(r#"youtube\.com/embed/([a-zA-Z0-9_-]+)"#).ok()?;
    if let Some(caps) = youtube_pattern.captures(html) {
        if let Some(video_id) = caps.get(1) {
            return Some(format!(
                "https://img.youtube.com/vi/{}/maxresdefault.jpg",
                video_id.as_str()
            ));
        }
    }

    // YouTube watch URL
    let youtube_watch = regex::Regex::new(r#"youtube\.com/watch\?v=([a-zA-Z0-9_-]+)"#).ok()?;
    if let Some(caps) = youtube_watch.captures(html) {
        if let Some(video_id) = caps.get(1) {
            return Some(format!(
                "https://img.youtube.com/vi/{}/maxresdefault.jpg",
                video_id.as_str()
            ));
        }
    }

    // Vimeo embed
    let vimeo_pattern = regex::Regex::new(r#"player\.vimeo\.com/video/(\d+)"#).ok()?;
    if let Some(caps) = vimeo_pattern.captures(html) {
        if let Some(video_id) = caps.get(1) {
            // Note: Vimeo thumbnails require API call, but we can try the common pattern
            return Some(format!("https://vumbnail.com/{}.jpg", video_id.as_str()));
        }
    }

    // Generic <video> tag with poster attribute
    let video_poster = regex::Regex::new(r#"<video[^>]+poster=["']([^"']+)["']"#).ok()?;
    if let Some(caps) = video_poster.captures(html) {
        if let Some(url) = caps.get(1) {
            return Some(url.as_str().to_string());
        }
    }

    None
}

/// Extract image from <link rel="image_src"> tag
fn extract_link_image(html: &str) -> Option<String> {
    let link_pattern =
        regex::Regex::new(r#"<link[^>]+rel=["']image_src["'][^>]+href=["']([^"']+)["']"#).ok()?;
    if let Some(caps) = link_pattern.captures(html) {
        if let Some(url) = caps.get(1) {
            return Some(url.as_str().to_string());
        }
    }

    // Try reversed attribute order
    let link_pattern_rev =
        regex::Regex::new(r#"<link[^>]+href=["']([^"']+)["'][^>]+rel=["']image_src["']"#).ok()?;
    if let Some(caps) = link_pattern_rev.captures(html) {
        if let Some(url) = caps.get(1) {
            return Some(url.as_str().to_string());
        }
    }

    None
}

/// Score image URL for quality (higher = better)
fn score_image_quality(url: &str) -> i32 {
    let mut score = 0;
    let lower = url.to_lowercase();

    if lower.contains("image") || lower.contains("img") {
        score += 10;
    }
    if lower.contains("photo") || lower.contains("article") {
        score += 10;
    }
    if lower.contains("large") || lower.contains("_1200") || lower.contains("1920") {
        score += 15;
    }
    if lower.contains("medium") || lower.contains("_800") {
        score += 8;
    }
    if lower.contains("thumb") || lower.contains("small") {
        score -= 5;
    }
    if lower.ends_with(".jpg") || lower.ends_with(".webp") {
        score += 5;
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_google_query() {
        let query = build_google_query(
            "Will Elon Musk become the world's first trillionaire?",
            Some(&vec![
                "Elon Musk".to_string(),
                "Jeff Bezos".to_string(),
                "Bernard Arnault".to_string(),
            ]),
        );
        assert!(query.contains("Elon Musk"));
        assert!(query.contains("trillionaire"));
    }

    #[test]
    fn test_extract_key_terms() {
        let terms = extract_key_terms("Will Bitcoin hit $100k in 2025?");
        assert!(terms.iter().any(|t| t.contains("Bitcoin")));
        assert!(terms
            .iter()
            .any(|t| t.contains("$100k") || t.contains("100k")));
    }

    #[test]
    fn test_extract_source_from_google_title() {
        let (title, source) =
            extract_source_from_google_title("Bitcoin surges past $100k - CoinDesk");
        assert_eq!(title, "Bitcoin surges past $100k");
        assert_eq!(source, "CoinDesk");
    }
}
