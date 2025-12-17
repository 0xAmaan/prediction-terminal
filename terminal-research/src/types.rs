use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use terminal_core::Platform;
use uuid::Uuid;

use crate::openai::SynthesizedReport;

/// Default cache TTL in hours
pub const DEFAULT_CACHE_TTL_HOURS: i64 = 24;

/// Calculate adaptive cache TTL based on market characteristics
///
/// Factors:
/// - Time to resolution: shorter TTL if market closes soon
/// - Trading volume: higher volume = shorter TTL (more active market)
/// - Default: 24 hours
pub fn calculate_cache_ttl(
    end_date: Option<chrono::DateTime<chrono::Utc>>,
    volume_24h: Option<f64>,
) -> i64 {
    // Start with default
    let mut ttl = DEFAULT_CACHE_TTL_HOURS;

    // If market ends soon (within 7 days), use shorter TTL
    if let Some(end) = end_date {
        let days_until_end = (end - chrono::Utc::now()).num_days();
        if days_until_end <= 1 {
            ttl = 1; // 1 hour for markets ending within 24h
        } else if days_until_end <= 3 {
            ttl = 4; // 4 hours for markets ending within 3 days
        } else if days_until_end <= 7 {
            ttl = 8; // 8 hours for markets ending within a week
        }
    }

    // If high volume market (>$100k in 24h), use shorter TTL
    if let Some(vol) = volume_24h {
        if vol > 100_000.0 {
            ttl = ttl.min(8); // Cap at 8 hours for high volume markets
        } else if vol > 50_000.0 {
            ttl = ttl.min(12); // Cap at 12 hours for medium-high volume
        }
    }

    // Minimum TTL of 1 hour
    ttl.max(1)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchJob {
    pub id: String,
    pub platform: Platform,
    pub market_id: String,
    pub market_title: String,
    pub status: ResearchStatus,
    pub progress: ResearchProgress,
    pub report: Option<SynthesizedReport>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub cached: bool,
    /// Cache TTL in hours (calculated based on market characteristics)
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_hours: i64,
    /// Market price when research was cached (for invalidation on significant moves)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_at_price: Option<f64>,
}

fn default_cache_ttl() -> i64 {
    DEFAULT_CACHE_TTL_HOURS
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResearchStatus {
    Pending,
    Decomposing,
    Searching,
    Analyzing,
    Synthesizing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResearchProgress {
    pub current_step: String,
    pub total_steps: u32,
    pub completed_steps: u32,
    pub current_query: Option<String>,
    pub searches_completed: u32,
    pub searches_total: u32,
}

impl ResearchJob {
    pub fn new(platform: Platform, market_id: &str, market_title: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            platform,
            market_id: market_id.to_string(),
            market_title: market_title.to_string(),
            status: ResearchStatus::Pending,
            progress: ResearchProgress::default(),
            report: None,
            error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            cached: false,
            cache_ttl_hours: DEFAULT_CACHE_TTL_HOURS,
            cached_at_price: None,
        }
    }

    pub fn cache_key(&self) -> String {
        format!("research/{:?}/{}", self.platform, self.market_id).to_lowercase()
    }

    /// Set cache TTL and price for adaptive caching
    pub fn set_cache_metadata(&mut self, ttl_hours: i64, price: Option<f64>) {
        self.cache_ttl_hours = ttl_hours;
        self.cached_at_price = price;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResearchUpdate {
    StatusChanged {
        job_id: String,
        status: ResearchStatus,
    },
    ProgressUpdate {
        job_id: String,
        progress: ResearchProgress,
    },
    Completed {
        job_id: String,
        report: SynthesizedReport,
    },
    Failed {
        job_id: String,
        error: String,
    },
    /// Follow-up research has started processing
    FollowUpStarted {
        job_id: String,
    },
    /// Document content is being streamed during follow-up research
    DocumentEditing {
        job_id: String,
        /// The content chunk being streamed
        content_chunk: String,
    },
    /// Follow-up research has completed with an updated report
    FollowUpCompleted {
        job_id: String,
        report: SynthesizedReport,
    },
}

/// Metadata for a research version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchVersion {
    /// The version key (filename like "v1702389600000.json")
    pub key: String,
    /// When this version was created
    pub created_at: DateTime<Utc>,
    /// Version number (1 = newest, increments for older versions)
    #[serde(default)]
    pub version_number: u32,
}

/// Response wrapper for list of versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchVersionList {
    pub versions: Vec<ResearchVersion>,
}

// ============================================================================
// Chat Types
// ============================================================================

/// Role of a chat message sender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    User,
    Assistant,
}

/// A single chat message in the research conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Unique message ID
    pub id: String,
    /// Who sent this message
    pub role: ChatRole,
    /// Message content
    pub content: String,
    /// When the message was created
    pub created_at: DateTime<Utc>,
    /// Whether this message triggered follow-up research
    #[serde(default)]
    pub research_triggered: bool,
}

impl ChatMessage {
    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role: ChatRole::User,
            content: content.into(),
            created_at: Utc::now(),
            research_triggered: false,
        }
    }

    /// Create a new assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role: ChatRole::Assistant,
            content: content.into(),
            created_at: Utc::now(),
            research_triggered: false,
        }
    }
}

/// Chat history for a research session
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatHistory {
    pub messages: Vec<ChatMessage>,
}

impl ChatHistory {
    /// Create empty chat history
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    /// Append a message to the history
    pub fn append(&mut self, message: ChatMessage) {
        self.messages.push(message);
    }
}

// ============================================================================
// Follow-up Research Types (Phase 4)
// ============================================================================

/// Request for follow-up research on an existing report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowUpRequest {
    /// The user's follow-up question
    pub question: String,
    /// The existing report to analyze/enhance
    pub existing_report: crate::openai::SynthesizedReport,
}

/// Response from analyzing a follow-up question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowUpResponse {
    /// The answer to the question (may be from context or new research)
    pub answer: String,
    /// Whether new research was needed to answer the question
    pub needs_research: bool,
    /// Search queries to execute if research is needed
    #[serde(default)]
    pub search_queries: Vec<String>,
}

/// An edit operation on the research document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentEdit {
    /// Section index to edit (None = append new section)
    pub section_index: Option<usize>,
    /// Type of edit operation
    pub operation: DocumentEditOperation,
    /// Content to add/replace
    pub content: String,
}

/// Type of document edit operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentEditOperation {
    /// Append content to a section or the document
    Append,
    /// Replace a section's content
    Replace,
    /// Insert a new section at the index
    Insert,
}

// ============================================================================
// Market Context Types (for AI research)
// ============================================================================

/// Context about a market's current state for AI research
///
/// Provides real-time market data (price, volume, trades, order book) so the AI
/// has accurate context when generating research.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketContext {
    /// Market title
    pub title: String,
    /// Market description
    pub description: Option<String>,
    /// Current probability/price (0.0 to 1.0)
    pub current_price: Option<f64>,
    /// Price 24 hours ago (for calculating change)
    pub price_24h_ago: Option<f64>,
    /// 24-hour trading volume in dollars
    pub volume_24h: Option<f64>,
    /// Lifetime total volume
    pub total_volume: Option<f64>,
    /// Number of unique traders
    pub num_traders: Option<u64>,
    /// Recent trades (last ~10)
    #[serde(default)]
    pub recent_trades: Vec<RecentTrade>,
    /// Order book summary
    pub order_book_summary: Option<OrderBookSummary>,
    /// Resolution rules/criteria for this market
    pub resolution_rules: Option<String>,
    /// Content fetched from resolution source URLs (e.g., leaderboard data)
    /// Format: Vec<(url, fetched_content)>
    #[serde(default)]
    pub resolution_source_content: Vec<ResolutionSourceData>,
}

/// Data fetched from a resolution source URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionSourceData {
    /// The URL that was fetched
    pub url: String,
    /// The fetched content (text extracted from the page)
    pub content: String,
    /// When this was fetched
    pub fetched_at: chrono::DateTime<chrono::Utc>,
}

/// A recent trade for market context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentTrade {
    /// Price (0.0 to 1.0)
    pub price: f64,
    /// Trade size in dollars
    pub size: f64,
    /// Trade side: "buy" or "sell"
    pub side: String,
    /// ISO 8601 timestamp
    pub timestamp: String,
}

/// Summary of order book depth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookSummary {
    /// Best bid price
    pub best_bid: Option<f64>,
    /// Best ask price
    pub best_ask: Option<f64>,
    /// Spread (ask - bid)
    pub spread: Option<f64>,
    /// Total $ within 10% of best bid
    pub bid_depth_10pct: f64,
    /// Total $ within 10% of best ask
    pub ask_depth_10pct: f64,
}

// ============================================================================
// Trading Analysis Types
// ============================================================================

/// Trading-focused analysis that accompanies the research report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingAnalysis {
    /// Estimated fair probability range low bound (0.0 to 1.0)
    pub fair_value_low: f64,
    /// Estimated fair probability range high bound (0.0 to 1.0)
    pub fair_value_high: f64,
    /// Current market price for comparison (0.0 to 1.0)
    pub current_price: f64,
    /// Calculated edge: midpoint of fair value minus current price
    /// Positive = market underpriced (buy signal), Negative = overpriced (sell signal)
    pub implied_edge: f64,
    /// How confident is the AI in this fair value estimate
    pub estimate_confidence: EstimateConfidence,
    /// Reasoning for the fair value estimate
    pub fair_value_reasoning: String,
    /// Upcoming events that could move the market
    pub catalysts: Vec<Catalyst>,
    /// Analysis of resolution rules and potential issues
    pub resolution_analysis: ResolutionAnalysis,
    /// The case against the current market consensus
    pub contrarian_case: ContrarianAnalysis,
}

/// Confidence level for fair value estimates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EstimateConfidence {
    /// Strong evidence, narrow range
    High,
    /// Decent evidence, moderate uncertainty
    Medium,
    /// Limited evidence, wide range, speculative
    Low,
}

/// An upcoming event that could move the market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalyst {
    /// When the event occurs (if known)
    pub date: Option<String>,
    /// Description of the event
    pub event: String,
    /// How much could this move the market
    pub expected_impact: CatalystImpact,
    /// Which direction if the event is favorable
    pub direction_if_positive: Option<Direction>,
}

/// Expected impact level of a catalyst
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalystImpact {
    /// Could move market 10%+
    High,
    /// Could move market 5-10%
    Medium,
    /// Could move market 1-5%
    Low,
}

/// Market direction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Bullish,
    Bearish,
}

/// Analysis of market resolution criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionAnalysis {
    /// Plain English summary of how this market resolves
    pub resolution_summary: String,
    /// The exact source/criteria used for resolution
    pub resolution_source: Option<String>,
    /// Potential ambiguities or edge cases
    pub ambiguity_flags: Vec<String>,
    /// Historical edge cases from similar markets (if any)
    pub historical_edge_cases: Vec<String>,
}

/// Analysis of contrarian viewpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContrarianAnalysis {
    /// What the market consensus appears to be
    pub consensus_view: String,
    /// The case for why consensus might be wrong
    pub contrarian_case: String,
    /// Specific reasons the crowd could be mispricing
    pub mispricing_reasons: Vec<String>,
    /// What would need to happen for contrarian view to win
    pub contrarian_triggers: Vec<String>,
}

// ============================================================================
// Edge Index Types (for filtering mispriced markets)
// ============================================================================

/// An entry in the edge index tracking a market's implied edge from research
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketEdgeEntry {
    /// Platform (polymarket, kalshi)
    pub platform: Platform,
    /// Market ID
    pub market_id: String,
    /// Market title for display
    pub title: String,
    /// Implied edge from research (positive = underpriced, negative = overpriced)
    pub implied_edge: f64,
    /// Fair value low bound
    pub fair_value_low: f64,
    /// Fair value high bound
    pub fair_value_high: f64,
    /// Current price when research was conducted
    pub current_price: f64,
    /// Confidence in the estimate
    pub estimate_confidence: EstimateConfidence,
    /// When this entry was last updated
    pub updated_at: DateTime<Utc>,
}

impl MarketEdgeEntry {
    /// Create a new edge entry from trading analysis
    pub fn from_analysis(
        platform: Platform,
        market_id: &str,
        title: &str,
        analysis: &TradingAnalysis,
    ) -> Self {
        Self {
            platform,
            market_id: market_id.to_string(),
            title: title.to_string(),
            implied_edge: analysis.implied_edge,
            fair_value_low: analysis.fair_value_low,
            fair_value_high: analysis.fair_value_high,
            current_price: analysis.current_price,
            estimate_confidence: analysis.estimate_confidence.clone(),
            updated_at: Utc::now(),
        }
    }

    /// Check if market has significant edge (>2% in either direction)
    pub fn has_edge(&self) -> bool {
        self.implied_edge.abs() > 0.02
    }

    /// Check if market is underpriced (positive edge > 2%)
    pub fn is_underpriced(&self) -> bool {
        self.implied_edge > 0.02
    }

    /// Check if market is overpriced (negative edge < -2%)
    pub fn is_overpriced(&self) -> bool {
        self.implied_edge < -0.02
    }
}

/// Index of all markets with research reports and their edge values
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EdgeIndex {
    /// All market entries with edge data
    pub entries: Vec<MarketEdgeEntry>,
    /// When this index was last updated
    pub updated_at: DateTime<Utc>,
}

impl EdgeIndex {
    /// Create a new empty index
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            updated_at: Utc::now(),
        }
    }

    /// Update or insert an entry
    pub fn upsert(&mut self, entry: MarketEdgeEntry) {
        if let Some(existing) = self
            .entries
            .iter_mut()
            .find(|e| e.platform == entry.platform && e.market_id == entry.market_id)
        {
            *existing = entry;
        } else {
            self.entries.push(entry);
        }
        self.updated_at = Utc::now();
    }

    /// Get all markets with significant edge (>2% in either direction)
    pub fn mispriced_markets(&self) -> Vec<&MarketEdgeEntry> {
        self.entries.iter().filter(|e| e.has_edge()).collect()
    }

    /// Get markets filtered by platform
    pub fn by_platform(&self, platform: Platform) -> Vec<&MarketEdgeEntry> {
        self.entries
            .iter()
            .filter(|e| e.platform == platform)
            .collect()
    }
}
