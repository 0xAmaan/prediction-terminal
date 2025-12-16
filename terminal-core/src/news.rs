//! News data structures for prediction market news aggregation

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::Platform;

/// Price signal indicating whether a market appears mispriced based on news
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PriceSignal {
    /// News suggests the market is underpriced (probability should be higher)
    Underpriced,
    /// News suggests the market is overpriced (probability should be lower)
    Overpriced,
    /// News doesn't clearly indicate mispricing
    Neutral,
}

/// Suggested trading action based on news analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SuggestedAction {
    /// Consider buying YES shares
    Buy,
    /// Consider selling YES shares (or buying NO)
    Sell,
    /// No clear action suggested
    Hold,
}

/// Information about a market matched to a news item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedMarket {
    /// Platform the market is on
    pub platform: Platform,
    /// Market identifier
    pub market_id: String,
    /// Market title for display
    pub title: String,
    /// Current YES price (0.0 - 1.0)
    pub current_price: f64,
    /// URL to the market
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Specific outcome that's affected (for multi-outcome markets, e.g., "Susie Wiles")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
}

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

    // ========================================================================
    // AI-enriched fields for trading signals
    // ========================================================================
    /// The market this news is most relevant to (AI-matched)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_market: Option<MatchedMarket>,
    /// Price signal: does this news suggest the market is mispriced?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_signal: Option<PriceSignal>,
    /// Suggested trading action based on the news
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_action: Option<SuggestedAction>,
    /// Brief AI reasoning for the signal (1-2 sentences)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_reasoning: Option<String>,
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
