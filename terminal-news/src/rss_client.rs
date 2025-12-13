//! RSS Feed Client for news aggregation
//!
//! Fetches and parses RSS/Atom feeds from curated news sources.

use chrono::{DateTime, Utc};
use reqwest::Client;
use sha2::{Digest, Sha256};
use tracing::{info, warn};

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
        let mut source_counts = std::collections::HashMap::new();

        for feed in &self.feeds {
            match self.fetch_feed(feed).await {
                Ok(items) => {
                    let count = items.len();
                    info!("✓ Fetched {} items from {}", count, feed.name);
                    *source_counts.entry(feed.name.clone()).or_insert(0) += count;
                    all_items.extend(items);
                }
                Err(e) => {
                    warn!("✗ Failed to fetch feed {}: {}", feed.name, e);
                    *source_counts.entry(feed.name.clone()).or_insert(0) += 0;
                }
            }
        }

        info!(
            "Raw fetch results: {} items from {} feeds",
            all_items.len(),
            self.feeds.len()
        );

        // Sort by date, newest first
        all_items.sort_by(|a, b| b.published_at.cmp(&a.published_at));

        // Deduplicate by normalized title
        let mut seen_titles = std::collections::HashSet::new();
        let before_dedup = all_items.len();
        all_items.retain(|item| {
            let key = normalize_title(&item.title);
            if seen_titles.contains(&key) {
                false
            } else {
                seen_titles.insert(key);
                true
            }
        });
        info!(
            "After deduplication: {} items (removed {} duplicates)",
            all_items.len(),
            before_dedup - all_items.len()
        );

        // Diversify sources using round-robin selection
        // Group by source
        let mut by_source: std::collections::HashMap<String, Vec<NewsItem>> =
            std::collections::HashMap::new();
        for item in all_items {
            by_source
                .entry(item.source.name.clone())
                .or_insert_with(Vec::new)
                .push(item);
        }

        // Sort each source's items by date
        for items in by_source.values_mut() {
            items.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        }

        info!("Collected items from {} sources", by_source.len());

        // Round-robin selection to ensure source diversity
        let mut diversified_items = Vec::new();
        let mut source_indices: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        // Continue until we have enough items or exhausted all sources
        while diversified_items.len() < limit {
            let mut added_this_round = false;

            // Try to take one item from each source in round-robin fashion
            for (source, items) in &by_source {
                let idx = source_indices.entry(source.clone()).or_insert(0);
                if *idx < items.len() {
                    diversified_items.push(items[*idx].clone());
                    *idx += 1;
                    added_this_round = true;

                    if diversified_items.len() >= limit {
                        break;
                    }
                }
            }

            // If we couldn't add any items this round, we've exhausted all sources
            if !added_this_round {
                break;
            }
        }

        // Final sort by date to ensure newest items appear first
        diversified_items.sort_by(|a, b| b.published_at.cmp(&a.published_at));

        // Log source distribution in final results
        let mut final_sources = std::collections::HashMap::new();
        for item in &diversified_items {
            *final_sources.entry(item.source.name.clone()).or_insert(0) += 1;
        }
        info!(
            "Final {} items by source: {:?}",
            diversified_items.len(),
            final_sources
        );

        // Backfill missing thumbnails by fetching article pages
        let items_without_images: Vec<usize> = diversified_items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.image_url.is_none())
            .map(|(i, _)| i)
            .collect();

        if !items_without_images.is_empty() {
            info!(
                "Attempting to fetch thumbnails for {} articles without images",
                items_without_images.len()
            );

            // Limit concurrent fetches to avoid overwhelming servers
            let mut fetch_count = 0;
            for idx in items_without_images {
                if fetch_count >= 10 {
                    // Limit to 10 additional fetches to keep response time reasonable
                    break;
                }

                if let Ok(image_url) = self
                    .fetch_article_thumbnail(&diversified_items[idx].url)
                    .await
                {
                    diversified_items[idx].image_url = Some(image_url);
                    fetch_count += 1;
                }
            }

            if fetch_count > 0 {
                info!("Successfully fetched {} additional thumbnails", fetch_count);
            }
        }

        Ok(diversified_items)
    }

    /// Fetch thumbnail from actual article page (for items without images from RSS)
    async fn fetch_article_thumbnail(&self, url: &str) -> Result<String, NewsError> {
        let response = self
            .client
            .get(url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            )
            .timeout(std::time::Duration::from_secs(5)) // 5 second timeout
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

                // Parse publication date - try multiple formats
                let published_at = item
                    .pub_date()
                    .and_then(|d| {
                        // Try RFC 2822 format first (most common for RSS)
                        DateTime::parse_from_rfc2822(d)
                            .ok()
                            .map(|dt| dt.with_timezone(&Utc))
                            .or_else(|| {
                                // Try RFC 3339 / ISO 8601 format (used by Atom feeds)
                                DateTime::parse_from_rfc3339(d)
                                    .ok()
                                    .map(|dt| dt.with_timezone(&Utc))
                            })
                            .or_else(|| {
                                // Try custom date format parsing
                                chrono::NaiveDateTime::parse_from_str(d, "%Y-%m-%d %H:%M:%S")
                                    .ok()
                                    .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc))
                            })
                    })
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

/// Extract image URL from HTML content (finds best quality image with multiple strategies)
fn extract_image_from_html(html: &str) -> Option<String> {
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

    // Strategy 4: Look for all <img src="..." tags
    let img_pattern = regex::Regex::new(r#"<img[^>]+src=["']([^"']+)["']"#).ok()?;

    let mut candidates: Vec<String> = img_pattern
        .captures_iter(html)
        .filter_map(|caps| caps.get(1).map(|m| m.as_str().to_string()))
        .filter(|url| {
            // Filter out tracking pixels, icons, and other junk
            !url.contains("1x1")
                && !url.contains("pixel")
                && !url.contains("spacer")
                && !url.contains("icon")
                && !url.contains("logo")
                && !url.contains("avatar")
                && !url.ends_with(".svg")
                && !url.ends_with(".gif")
                && !url.contains("blank")
        })
        .collect();

    if candidates.is_empty() {
        return None;
    }

    // Prefer larger images (likely article images, not thumbnails)
    // Images with "image", "photo", "article" in URL are usually better
    candidates.sort_by(|a, b| {
        let a_score = score_image_url(a);
        let b_score = score_image_url(b);
        b_score.cmp(&a_score) // Descending
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

    // YouTube short URL
    let youtube_short = regex::Regex::new(r#"youtu\.be/([a-zA-Z0-9_-]+)"#).ok()?;
    if let Some(caps) = youtube_short.captures(html) {
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

/// Score image URL quality (higher is better)
fn score_image_url(url: &str) -> i32 {
    let mut score = 0;
    let lower = url.to_lowercase();

    // Prefer images with these keywords
    if lower.contains("image") || lower.contains("img") {
        score += 10;
    }
    if lower.contains("photo") {
        score += 10;
    }
    if lower.contains("article") {
        score += 10;
    }
    if lower.contains("content") {
        score += 5;
    }
    if lower.contains("media") {
        score += 5;
    }

    // Size indicators (larger is better for thumbnails)
    if lower.contains("large") || lower.contains("big") {
        score += 8;
    }
    if lower.contains("medium") {
        score += 5;
    }
    if lower.contains("original") {
        score += 10;
    }
    if lower.contains("_1200") || lower.contains("1920") || lower.contains("2048") {
        score += 15;
    }
    if lower.contains("_800") || lower.contains("1024") {
        score += 10;
    }

    // Penalize small images
    if lower.contains("thumb") || lower.contains("small") {
        score -= 5;
    }
    if lower.contains("_50") || lower.contains("_100") || lower.contains("_150") {
        score -= 10;
    }

    // Image format preference (jpg/webp > png > gif)
    if lower.ends_with(".jpg") || lower.ends_with(".jpeg") || lower.ends_with(".webp") {
        score += 5;
    }

    score
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

                    if medium == Some("image")
                        || mime.map(|m| m.starts_with("image/")).unwrap_or(false)
                        || url.ends_with(".jpg")
                        || url.ends_with(".jpeg")
                        || url.ends_with(".png")
                        || url.ends_with(".webp")
                    {
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
        "The",
        "A",
        "An",
        "This",
        "That",
        "It",
        "In",
        "On",
        "At",
        "For",
        "To",
        "With",
        "From",
        "By",
        "As",
        "Is",
        "Are",
        "Was",
        "Were",
        "Will",
        "Would",
        "Could",
        "Should",
        "May",
        "Might",
        "Must",
        "Has",
        "Have",
        "Had",
        "Do",
        "Does",
        "Did",
        "Says",
        "Said",
        "First",
        "Last",
        "After",
        "Before",
        "Report",
        "Reports",
        "News",
        "Breaking",
        "Watch",
        "Update",
        "Live",
        "Just",
        "Now",
        "How",
        "Why",
        "What",
        "When",
        "Where",
        "Who",
        "WATCH",
        "BREAKING",
        "LIVE",
        "UPDATE",
        "JUST",
        "NOW",
        "US",
        "UK",
        // Skip titles/positions - they're not the entity itself
        "President",
        "Senator",
        "Governor",
        "Attorney",
        "General",
        "Secretary",
        "Minister",
        "Chief",
        "Director",
        "Chairman",
        "CEO",
        "Department",
        "Justice",
        "State",
    ]
    .into_iter()
    .collect();

    let mut i = 0;
    while i < words.len() {
        let word = words[i];
        let clean_word: String = word.chars().filter(|c| c.is_alphabetic()).collect();
        let is_cap = clean_word
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false);

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
                let next_cap = next_clean
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);

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
    let stop_words = [
        "the",
        "and",
        "for",
        "that",
        "this",
        "with",
        "from",
        "says",
        "said",
        "will",
        "would",
        "could",
        "about",
        "after",
        "into",
        "over",
        "more",
        "than",
        "been",
        "have",
        "were",
        "what",
        "when",
        "where",
        "which",
        "their",
        "there",
        "these",
        "those",
        "some",
        "just",
        "also",
        "news",
        "report",
        "reports",
        "again",
        "fails",
        "declines",
        "source",
        "grand",
        "jury",
        "federal",
        "court",
        "indict",
        "department",
        "justice",
    ];
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
