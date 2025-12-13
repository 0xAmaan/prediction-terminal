use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use terminal_core::Platform;
use uuid::Uuid;

use crate::openai::SynthesizedReport;

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
        }
    }

    pub fn cache_key(&self) -> String {
        format!("research/{:?}/{}", self.platform, self.market_id).to_lowercase()
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
