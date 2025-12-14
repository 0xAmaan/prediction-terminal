//! News data structures for prediction market news aggregation

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::Platform;

/// Source of a news article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsSource {
    /// Name of the news source (e.g., "Reuters", "Bloomberg")
    pub name: String,
    /// URL of the source's website
    pub url: String,
    /// URL to the source's favicon
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon_url: Option<String>,
}

/// A news article related to prediction markets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    /// Unique identifier (hash of URL)
    pub id: String,
    /// Article title
    pub title: String,
    /// Article URL
    pub url: String,
    /// Publication date
    pub published_at: DateTime<Utc>,
    /// Source information
    pub source: NewsSource,
    /// Brief summary/excerpt
    pub summary: String,
    /// Full article content (if scraped via Firecrawl)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Article thumbnail/image URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// Relevance score from search (0.0 - 1.0)
    pub relevance_score: f64,
    /// Related market IDs (if contextual news)
    #[serde(default)]
    pub related_market_ids: Vec<String>,
    /// Search query that found this article
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_query: Option<String>,
}

/// News feed response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsFeed {
    /// List of news items
    pub items: Vec<NewsItem>,
    /// Total number of results available
    pub total_count: usize,
    /// Cursor for pagination (if more results available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Parameters for searching news
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NewsSearchParams {
    /// Search query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    /// Maximum number of results to return
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Time range filter (e.g., "24h", "7d", "30d")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_range: Option<String>,
    /// Related market ID (for contextual news)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_id: Option<String>,
    /// Skip expensive embedding generation for faster responses
    #[serde(default)]
    pub skip_embeddings: bool,
}

fn default_limit() -> usize {
    20
}

/// Context for market-specific news updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketNewsContext {
    /// Platform the market is on
    pub platform: Platform,
    /// Market identifier
    pub market_id: String,
}
