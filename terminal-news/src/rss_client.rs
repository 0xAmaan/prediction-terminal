//! RSS Feed Client for news aggregation
//!
//! Fetches and parses RSS/Atom feeds from curated news sources.

use chrono::{DateTime, Utc};
use reqwest::Client;
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

use terminal_core::{NewsItem, NewsSource};

use crate::error::NewsError;

/// RSS feed definition
#[derive(Debug, Clone)]
pub struct RssFeed {
    /// Name of the source
    pub name: String,
    /// RSS feed URL
    pub url: String,
    /// Category tags for relevance matching
    pub categories: Vec<String>,
    /// Base URL for favicon
    pub base_url: String,
}

impl RssFeed {
    pub fn new(name: &str, url: &str, base_url: &str, categories: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
            base_url: base_url.to_string(),
            categories: categories.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// Curated list of RSS feeds for prediction markets
pub fn get_curated_feeds() -> Vec<RssFeed> {
    vec![
        // Wire Services - Most reliable for breaking news
        RssFeed::new(
            "AP News",
            "https://feedx.net/rss/ap.xml",
            "https://apnews.com",
            &["general", "politics", "world"],
        ),
        // Financial News - Only major business/economy news, not individual stocks
        RssFeed::new(
            "CNBC Top News",
            "https://search.cnbc.com/rs/search/combinedcms/view.xml?partnerId=wrss01&id=100003114",
            "https://cnbc.com",
            &["business", "markets", "finance"],
        ),
        // Political News
        RssFeed::new(
            "Politico",
            "https://www.politico.com/rss/politicopicks.xml",
            "https://politico.com",
            &["politics", "elections", "government"],
        ),
        RssFeed::new(
            "The Hill",
            "https://thehill.com/feed/",
            "https://thehill.com",
            &["politics", "elections", "government"],
        ),
        RssFeed::new(
            "NPR Politics",
            "https://feeds.npr.org/1014/rss.xml",
            "https://npr.org",
            &["politics", "elections"],
        ),
        RssFeed::new(
            "Axios",
            "https://api.axios.com/feed/",
            "https://axios.com",
            &["politics", "tech", "business"],
        ),
        // General News
        RssFeed::new(
            "BBC News",
            "https://feeds.bbci.co.uk/news/rss.xml",
            "https://bbc.com",
            &["general", "world", "politics"],
        ),
        RssFeed::new(
            "BBC World",
            "https://feeds.bbci.co.uk/news/world/rss.xml",
            "https://bbc.com",
            &["world", "politics", "general"],
        ),
        RssFeed::new(
            "NPR News",
            "https://feeds.npr.org/1001/rss.xml",
            "https://npr.org",
            &["general", "politics", "world"],
        ),
        RssFeed::new(
            "Guardian US",
            "https://www.theguardian.com/us-news/rss",
            "https://theguardian.com",
            &["politics", "world", "general"],
        ),
        RssFeed::new(
            "CBS News",
            "https://www.cbsnews.com/latest/rss/main",
            "https://cbsnews.com",
            &["general", "politics", "world"],
        ),
        RssFeed::new(
            "ABC News",
            "https://abcnews.go.com/abcnews/topstories",
            "https://abcnews.go.com",
            &["general", "politics", "world"],
        ),
        // Tech News - Only AI-focused feeds (general tech has too much noise)
        RssFeed::new(
            "MIT Technology Review",
            "https://www.technologyreview.com/feed/",
            "https://technologyreview.com",
            &["tech", "ai", "science"],
        ),
        // Crypto specific
        RssFeed::new(
            "CoinDesk",
            "https://www.coindesk.com/arc/outboundfeeds/rss/",
            "https://coindesk.com",
            &["crypto", "bitcoin", "ethereum", "defi"],
        ),
        RssFeed::new(
            "Cointelegraph",
            "https://cointelegraph.com/rss",
            "https://cointelegraph.com",
            &["crypto", "bitcoin", "ethereum", "defi"],
        ),
        // Sports (for sports betting markets)
        RssFeed::new(
            "ESPN",
            "https://www.espn.com/espn/rss/news",
            "https://espn.com",
            &["sports", "nfl", "nba", "mlb"],
        ),
        RssFeed::new(
            "ESPN NFL",
            "https://www.espn.com/espn/rss/nfl/news",
            "https://espn.com",
            &["sports", "nfl", "football"],
        ),
        RssFeed::new(
            "ESPN NBA",
            "https://www.espn.com/espn/rss/nba/news",
            "https://espn.com",
            &["sports", "nba", "basketball"],
        ),
        RssFeed::new(
            "CBS Sports",
            "https://www.cbssports.com/rss/headlines/",
            "https://cbssports.com",
            &["sports", "nfl", "nba", "mlb"],
        ),
        // Space (for SpaceX/NASA markets only)
        RssFeed::new(
            "NASA",
            "https://www.nasa.gov/rss/dyn/breaking_news.rss",
            "https://nasa.gov",
            &["space", "science"],
        ),
        // Economics
        RssFeed::new(
            "Federal Reserve",
            "https://www.federalreserve.gov/feeds/press_all.xml",
            "https://federalreserve.gov",
            &["economics", "finance", "interest-rates", "fed"],
        ),
        // International News - for non-US markets
        RssFeed::new(
            "Reuters World",
            "https://www.reutersagency.com/feed/?taxonomy=best-regions&post_type=best",
            "https://reuters.com",
            &["world", "international", "politics"],
        ),
        RssFeed::new(
            "Al Jazeera",
            "https://www.aljazeera.com/xml/rss/all.xml",
            "https://aljazeera.com",
            &["world", "international", "middle-east", "politics"],
        ),
        RssFeed::new(
            "Guardian World",
            "https://www.theguardian.com/world/rss",
            "https://theguardian.com",
            &["world", "international", "politics"],
        ),
        RssFeed::new(
            "DW News",
            "https://rss.dw.com/rdf/rss-en-all",
            "https://dw.com",
            &["world", "international", "europe", "politics"],
        ),
    ]
}

/// RSS feed client
pub struct RssClient {
    client: Client,
    feeds: Vec<RssFeed>,
}

impl RssClient {
    /// Create a new RSS client with curated feeds
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| Client::new()),
            feeds: get_curated_feeds(),
        }
    }

    /// Create with custom feeds
    pub fn with_feeds(feeds: Vec<RssFeed>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| Client::new()),
            feeds,
        }
    }

    /// Fetch news from all feeds
    pub async fn fetch_all(&self, limit: usize) -> Result<Vec<NewsItem>, NewsError> {
        let mut all_items = Vec::new();

        for feed in &self.feeds {
            match self.fetch_feed(feed).await {
                Ok(items) => {
                    debug!("Fetched {} items from {}", items.len(), feed.name);
                    all_items.extend(items);
                }
                Err(e) => {
                    warn!("Failed to fetch feed {}: {}", feed.name, e);
                }
            }
        }

        // Sort by date, newest first
        all_items.sort_by(|a, b| b.published_at.cmp(&a.published_at));

        // Deduplicate by normalized title
        let mut seen_titles = std::collections::HashSet::new();
        all_items.retain(|item| {
            let key = normalize_title(&item.title);
            if seen_titles.contains(&key) {
                false
            } else {
                seen_titles.insert(key);
                true
            }
        });

        // Limit results
        all_items.truncate(limit);

        info!("Fetched {} total news items from RSS feeds", all_items.len());
        Ok(all_items)
    }

    /// Fetch news from feeds matching specific categories
    pub async fn fetch_by_categories(
        &self,
        categories: &[&str],
        limit: usize,
    ) -> Result<Vec<NewsItem>, NewsError> {
        let matching_feeds: Vec<&RssFeed> = self
            .feeds
            .iter()
            .filter(|feed| {
                categories
                    .iter()
                    .any(|cat| feed.categories.iter().any(|fc| fc.contains(cat)))
            })
            .collect();

        let mut all_items = Vec::new();

        for feed in matching_feeds {
            match self.fetch_feed(feed).await {
                Ok(items) => {
                    all_items.extend(items);
                }
                Err(e) => {
                    warn!("Failed to fetch feed {}: {}", feed.name, e);
                }
            }
        }

        // Sort by date, newest first
        all_items.sort_by(|a, b| b.published_at.cmp(&a.published_at));

        // Deduplicate
        let mut seen_titles = std::collections::HashSet::new();
        all_items.retain(|item| {
            let key = normalize_title(&item.title);
            if seen_titles.contains(&key) {
                false
            } else {
                seen_titles.insert(key);
                true
            }
        });

        all_items.truncate(limit);
        Ok(all_items)
    }

    /// Fetch a single RSS feed
    async fn fetch_feed(&self, feed: &RssFeed) -> Result<Vec<NewsItem>, NewsError> {
        let response = self
            .client
            .get(&feed.url)
            .header("User-Agent", "PredictionTerminal/1.0")
            .send()
            .await
            .map_err(|e| NewsError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(NewsError::ApiError {
                status: response.status().as_u16(),
                message: format!("Failed to fetch {}", feed.url),
            });
        }

        let content = response
            .bytes()
            .await
            .map_err(|e| NewsError::RequestFailed(e.to_string()))?;

        // Try parsing as RSS first, then Atom
        if let Ok(channel) = rss::Channel::read_from(&content[..]) {
            return Ok(self.parse_rss_channel(&channel, feed));
        }

        if let Ok(atom_feed) = atom_syndication::Feed::read_from(&content[..]) {
            return Ok(self.parse_atom_feed(&atom_feed, feed));
        }

        Err(NewsError::ParseError(format!(
            "Failed to parse feed: {}",
            feed.url
        )))
    }

    /// Parse RSS channel into NewsItems
    fn parse_rss_channel(&self, channel: &rss::Channel, _feed: &RssFeed) -> Vec<NewsItem> {
        channel
            .items()
            .iter()
            .filter_map(|item| {
                let title = item.title()?.to_string();
                let url = item.link()?.to_string();

                // Parse publication date
                let published_at = item
                    .pub_date()
                    .and_then(|d| DateTime::parse_from_rfc2822(d).ok())
                    .map(|d| d.with_timezone(&Utc))
                    .or_else(|| {
                        // Try extracting from URL
                        extract_date_from_url(&url)
                    })
                    .unwrap_or_else(Utc::now);

                // Skip old articles (older than 7 days)
                let age = Utc::now() - published_at;
                if age.num_days() > 7 {
                    return None;
                }

                // Generate ID from URL
                let id = {
                    let mut hasher = Sha256::new();
                    hasher.update(url.as_bytes());
                    hex::encode(&hasher.finalize()[..8])
                };

                // Get description HTML (before stripping)
                let description_html = item.description().unwrap_or_default();

                // Build summary from description
                let summary = strip_html(description_html);

                // Try to get image from multiple sources:
                // 1. Enclosure with image mime type
                // 2. Extract from description HTML <img> tags
                // 3. media:content or media:thumbnail (in extensions)
                let image_url = item
                    .enclosure()
                    .filter(|e| e.mime_type().starts_with("image/"))
                    .map(|e| e.url().to_string())
                    .or_else(|| extract_image_from_html(description_html))
                    .or_else(|| extract_media_content(item));

                // Extract source from actual article URL (more accurate than feed name)
                let source = extract_source(&url);

                Some(NewsItem {
                    id,
                    title,
                    url,
                    published_at,
                    source,
                    summary,
                    content: None,
                    image_url,
                    relevance_score: 1.0,
                    related_market_ids: vec![],
                    search_query: None,
                })
            })
            .collect()
    }

    /// Parse Atom feed into NewsItems
    fn parse_atom_feed(
        &self,
        atom_feed: &atom_syndication::Feed,
        _feed: &RssFeed,
    ) -> Vec<NewsItem> {
        atom_feed
            .entries()
            .iter()
            .filter_map(|entry| {
                let title = entry.title().to_string();
                let url = entry
                    .links()
                    .first()
                    .map(|l| l.href().to_string())
                    .unwrap_or_default();

                if url.is_empty() {
                    return None;
                }

                // Parse publication date
                let published_at = entry
                    .published()
                    .or_else(|| Some(entry.updated()))
                    .map(|d| d.with_timezone(&Utc))
                    .or_else(|| extract_date_from_url(&url))
                    .unwrap_or_else(Utc::now);

                // Skip old articles
                let age = Utc::now() - published_at;
                if age.num_days() > 7 {
                    return None;
                }

                // Generate ID
                let id = {
                    let mut hasher = Sha256::new();
                    hasher.update(url.as_bytes());
                    hex::encode(&hasher.finalize()[..8])
                };

                // Get raw content for image extraction
                let summary_html = entry.summary().map(|s| s.as_str()).unwrap_or_default();
                let content_html = entry.content().and_then(|c| c.value()).unwrap_or_default();

                // Build summary
                let summary = if !summary_html.is_empty() {
                    strip_html(summary_html)
                } else {
                    strip_html(content_html)
                };

                // Try to extract image from content
                let image_url = extract_image_from_html(content_html)
                    .or_else(|| extract_image_from_html(summary_html));

                // Extract source from actual article URL
                let source = extract_source(&url);

                Some(NewsItem {
                    id,
                    title,
                    url,
                    published_at,
                    source,
                    summary,
                    content: None,
                    image_url,
                    relevance_score: 1.0,
                    related_market_ids: vec![],
                    search_query: None,
                })
            })
            .collect()
    }
}

impl Default for RssClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract image URL from HTML content (finds first <img src="...">)
fn extract_image_from_html(html: &str) -> Option<String> {
    // Look for <img src="..." or <img src='...'
    let img_pattern = regex::Regex::new(r#"<img[^>]+src=["']([^"']+)["']"#).ok()?;
    if let Some(caps) = img_pattern.captures(html) {
        let url = caps.get(1)?.as_str().to_string();
        // Skip tiny tracking pixels and icons
        if url.contains("1x1") || url.contains("pixel") || url.contains("spacer") {
            return None;
        }
        return Some(url);
    }
    None
}

/// Extract image from RSS media:content or media:thumbnail extensions
fn extract_media_content(item: &rss::Item) -> Option<String> {
    // Check extensions for media namespace
    let extensions = item.extensions();

    // Look in media namespace
    if let Some(media) = extensions.get("media") {
        // Try media:content first
        if let Some(content_list) = media.get("content") {
            for content in content_list {
                if let Some(url) = content.attrs().get("url") {
                    // Check if it's an image
                    let medium = content.attrs().get("medium").map(|s| s.as_str());
                    let mime = content.attrs().get("type").map(|s| s.as_str());

                    if medium == Some("image") || mime.map(|m| m.starts_with("image/")).unwrap_or(false)
                       || url.ends_with(".jpg") || url.ends_with(".jpeg") || url.ends_with(".png") || url.ends_with(".webp") {
                        return Some(url.clone());
                    }
                }
            }
        }

        // Try media:thumbnail
        if let Some(thumbnail_list) = media.get("thumbnail") {
            for thumbnail in thumbnail_list {
                if let Some(url) = thumbnail.attrs().get("url") {
                    return Some(url.clone());
                }
            }
        }
    }

    None
}

/// Strip HTML tags from text
fn strip_html(html: &str) -> String {
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

    // Clean up whitespace and HTML entities
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

/// Extract date from URL patterns
fn extract_date_from_url(url: &str) -> Option<DateTime<Utc>> {
    // Pattern: /2025/12/09/ or /2025/12/9/
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

    // Pattern: /2025-12-09/
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

    None
}

/// Normalize title for deduplication
/// Extracts person names and key proper nouns to identify same-story articles from different sources
fn normalize_title(title: &str) -> String {
    // Extract multi-word proper noun phrases (likely person names like "Letitia James")
    let words: Vec<&str> = title.split_whitespace().collect();
    let mut name_phrases: Vec<String> = Vec::new();

    // Skip words that commonly start sentences or are generic titles
    let skip_words: std::collections::HashSet<&str> = [
        "The", "A", "An", "This", "That", "It", "In", "On", "At", "For", "To", "With", "From",
        "By", "As", "Is", "Are", "Was", "Were", "Will", "Would", "Could", "Should", "May",
        "Might", "Must", "Has", "Have", "Had", "Do", "Does", "Did", "Says", "Said",
        "First", "Last", "After", "Before", "Report", "Reports", "News", "Breaking", "Watch",
        "Update", "Live", "Just", "Now", "How", "Why", "What", "When", "Where", "Who",
        "WATCH", "BREAKING", "LIVE", "UPDATE", "JUST", "NOW", "US", "UK",
        // Skip titles/positions - they're not the entity itself
        "President", "Senator", "Governor", "Attorney", "General", "Secretary", "Minister",
        "Chief", "Director", "Chairman", "CEO", "Department", "Justice", "State",
    ]
    .into_iter()
    .collect();

    let mut i = 0;
    while i < words.len() {
        let word = words[i];
        let clean_word: String = word.chars().filter(|c| c.is_alphabetic()).collect();
        let is_cap = clean_word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);

        // Skip common words
        if skip_words.contains(clean_word.as_str()) {
            i += 1;
            continue;
        }

        // Found a capitalized word - look for multi-word names (2+ capitalized words together)
        if is_cap && clean_word.len() >= 3 {
            let mut phrase_words = vec![clean_word.clone()];
            let mut j = i + 1;

            while j < words.len() {
                let next = words[j];
                let next_clean: String = next.chars().filter(|c| c.is_alphabetic()).collect();
                let next_cap = next_clean.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);

                if next_cap && !skip_words.contains(next_clean.as_str()) && next_clean.len() >= 2 {
                    phrase_words.push(next_clean);
                    j += 1;
                } else {
                    break;
                }
            }

            // Only keep multi-word phrases (likely names) - "Letitia James", "New York"
            if phrase_words.len() >= 2 {
                let phrase = phrase_words.join("").to_lowercase();
                name_phrases.push(phrase);
            }

            i = j;
        } else {
            i += 1;
        }
    }

    // Sort for consistent ordering
    name_phrases.sort();

    // Use the longest name phrase as the primary key (most specific)
    if !name_phrases.is_empty() {
        // Sort by length descending, take the longest (most specific)
        name_phrases.sort_by(|a, b| b.len().cmp(&a.len()));
        // Use the longest phrase plus any others that share characters
        let primary = &name_phrases[0];
        let mut key = primary.clone();

        // Also include any other significant phrases
        for phrase in name_phrases.iter().skip(1).take(2) {
            if phrase.len() >= 6 && !primary.contains(phrase) {
                key.push('|');
                key.push_str(phrase);
            }
        }
        return key;
    }

    // Fallback: use significant words
    let lower = title.to_lowercase();
    let stop_words = ["the", "and", "for", "that", "this", "with", "from", "says", "said",
                     "will", "would", "could", "about", "after", "into", "over", "more",
                     "than", "been", "have", "were", "what", "when", "where", "which",
                     "their", "there", "these", "those", "some", "just", "also", "news",
                     "report", "reports", "again", "fails", "declines", "source", "grand",
                     "jury", "federal", "court", "indict", "department", "justice"];
    let words: Vec<&str> = lower
        .split_whitespace()
        .filter(|w| w.len() > 4)
        .filter(|w| !stop_words.contains(w))
        .take(4)
        .collect();
    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_html() {
        let html = "<p>Hello <b>world</b>!</p>";
        assert_eq!(strip_html(html), "Hello world!");
    }

    #[test]
    fn test_normalize_title() {
        let title = "The President Says This Will Change Everything";
        let normalized = normalize_title(title);
        assert!(!normalized.contains("the"));
        assert!(!normalized.contains("says"));
    }

    #[test]
    fn test_curated_feeds() {
        let feeds = get_curated_feeds();
        assert!(!feeds.is_empty());
        assert!(feeds.iter().any(|f| f.name == "Reuters"));
    }
}
