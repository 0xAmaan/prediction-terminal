//! Polymarket API response types
//!
//! These types mirror the Polymarket Gamma API responses and are converted
//! to terminal-core types for use in the application.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Base URL for the public data API (no auth required)
pub const DATA_API_BASE: &str = "https://data-api.polymarket.com";

/// A Polymarket market from the Gamma API
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolymarketMarket {
    /// Unique identifier
    pub id: String,

    /// Market question
    pub question: String,

    /// URL slug
    #[serde(default)]
    pub slug: Option<String>,

    /// Condition ID (used for CLOB)
    #[serde(default)]
    pub condition_id: Option<String>,

    /// End date
    #[serde(default)]
    pub end_date: Option<DateTime<Utc>>,

    /// When the market was created
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,

    /// Category
    #[serde(default)]
    pub category: Option<String>,

    /// Total liquidity
    #[serde(default)]
    pub liquidity: Option<String>,

    /// Total volume
    #[serde(default)]
    pub volume: Option<String>,

    /// Numeric volume (some responses have this)
    #[serde(default)]
    pub volume_num: Option<f64>,

    /// Numeric liquidity
    #[serde(default)]
    pub liquidity_num: Option<f64>,

    /// Outcomes (typically "Yes" and "No")
    #[serde(default)]
    pub outcomes: Option<String>,

    /// Outcome prices as JSON string (e.g., "[0.65, 0.35]")
    #[serde(default)]
    pub outcome_prices: Option<String>,

    /// Whether the market is active
    #[serde(default)]
    pub active: Option<bool>,

    /// Whether the market is closed
    #[serde(default)]
    pub closed: Option<bool>,

    /// Market image URL
    #[serde(default)]
    pub image: Option<String>,

    /// Market icon URL
    #[serde(default)]
    pub icon: Option<String>,

    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// CLOB token IDs (for trading)
    #[serde(default)]
    pub clob_token_ids: Option<String>,

    /// Group item title (for multi-outcome events, e.g., "Jurassic World" in "Top grossing movie")
    #[serde(default)]
    pub group_item_title: Option<String>,

    /// Events this market belongs to (contains event slugs)
    #[serde(default)]
    pub events: Option<Vec<MarketEvent>>,

    /// Resolution source - describes how the market will be resolved
    #[serde(default, rename = "resolutionSource")]
    pub resolution_source: Option<String>,
}

/// Event reference within a market
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketEvent {
    /// Event slug
    #[serde(default)]
    pub slug: Option<String>,

    /// Event title
    #[serde(default)]
    pub title: Option<String>,
}

/// A tag from the Polymarket API
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolymarketTag {
    /// Tag ID
    pub id: String,

    /// Human-readable label (e.g., "Politics", "Crypto", "AI")
    pub label: String,

    /// URL slug
    pub slug: String,
}

/// Market filter options for the frontend tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketFilter {
    /// All markets (default - sorted by volume)
    All,
    /// Trending markets (highest volume)
    Trending,
    /// Markets ending soon (sorted by end date ascending)
    Expiring,
    /// Newest markets (sorted by creation date descending)
    New,
    /// Crypto markets (tag_id = 21)
    Crypto,
    /// Politics markets (tag_id = 2)
    Politics,
    /// Sports markets (tag_id = 1)
    Sports,
}

impl MarketFilter {
    /// Get the Polymarket tag ID for category filters
    pub fn tag_id(&self) -> Option<u32> {
        match self {
            MarketFilter::Crypto => Some(21),
            MarketFilter::Politics => Some(2),
            MarketFilter::Sports => Some(1),
            _ => None,
        }
    }

    /// Check if this filter requires date-based ordering
    pub fn needs_date_ordering(&self) -> bool {
        matches!(self, MarketFilter::Expiring | MarketFilter::New)
    }
}

impl std::str::FromStr for MarketFilter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "all" => Ok(MarketFilter::All),
            "trending" => Ok(MarketFilter::Trending),
            "expiring" => Ok(MarketFilter::Expiring),
            "new" => Ok(MarketFilter::New),
            "crypto" => Ok(MarketFilter::Crypto),
            "politics" => Ok(MarketFilter::Politics),
            "sports" => Ok(MarketFilter::Sports),
            _ => Err(format!("Unknown filter: {}", s)),
        }
    }
}

impl std::fmt::Display for MarketFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketFilter::All => write!(f, "all"),
            MarketFilter::Trending => write!(f, "trending"),
            MarketFilter::Expiring => write!(f, "expiring"),
            MarketFilter::New => write!(f, "new"),
            MarketFilter::Crypto => write!(f, "crypto"),
            MarketFilter::Politics => write!(f, "politics"),
            MarketFilter::Sports => write!(f, "sports"),
        }
    }
}

impl PolymarketMarket {
    /// Parse outcome prices from the JSON string
    /// The API returns prices in various formats:
    /// - JSON array of strings: "[\"0.0115\", \"0.9885\"]"
    /// - JSON array of numbers: "[0.0115, 0.9885]"
    /// - Comma-separated: "0.0115, 0.9885"
    pub fn parse_outcome_prices(&self) -> Option<(Decimal, Decimal)> {
        let prices_str = self.outcome_prices.as_ref()?;

        // Try to parse as JSON array of strings first (most common format)
        if let Ok(prices) = serde_json::from_str::<Vec<String>>(prices_str) {
            if prices.len() >= 2 {
                let yes = Decimal::from_str(&prices[0]).unwrap_or(Decimal::ZERO);
                let no = Decimal::from_str(&prices[1]).unwrap_or(Decimal::ZERO);
                return Some((yes, no));
            }
        }

        // Try to parse as JSON array of numbers
        if let Ok(prices) = serde_json::from_str::<Vec<f64>>(prices_str) {
            if prices.len() >= 2 {
                let yes = Decimal::from_str(&prices[0].to_string()).unwrap_or(Decimal::ZERO);
                let no = Decimal::from_str(&prices[1].to_string()).unwrap_or(Decimal::ZERO);
                return Some((yes, no));
            }
        }

        // Try parsing as comma-separated (fallback)
        let parts: Vec<&str> = prices_str.trim_matches(|c| c == '[' || c == ']').split(',').collect();
        if parts.len() >= 2 {
            let yes = Decimal::from_str(parts[0].trim().trim_matches('"')).unwrap_or(Decimal::ZERO);
            let no = Decimal::from_str(parts[1].trim().trim_matches('"')).unwrap_or(Decimal::ZERO);
            return Some((yes, no));
        }

        None
    }

    /// Parse volume from string to Decimal
    pub fn parse_volume(&self) -> Decimal {
        if let Some(v) = self.volume_num {
            return Decimal::from_str(&v.to_string()).unwrap_or(Decimal::ZERO);
        }

        self.volume
            .as_ref()
            .and_then(|v| Decimal::from_str(v).ok())
            .unwrap_or(Decimal::ZERO)
    }

    /// Parse liquidity from string to Decimal
    pub fn parse_liquidity(&self) -> Option<Decimal> {
        if let Some(l) = self.liquidity_num {
            return Some(Decimal::from_str(&l.to_string()).unwrap_or(Decimal::ZERO));
        }

        self.liquidity
            .as_ref()
            .and_then(|l| Decimal::from_str(l).ok())
    }

    /// Parse CLOB token IDs from the JSON string
    /// Returns (yes_token_id, no_token_id) if available
    pub fn parse_clob_token_ids(&self) -> Option<(String, String)> {
        let ids_str = self.clob_token_ids.as_ref()?;

        // Parse as JSON array of strings
        if let Ok(ids) = serde_json::from_str::<Vec<String>>(ids_str) {
            if ids.len() >= 2 {
                return Some((ids[0].clone(), ids[1].clone()));
            }
        }

        None
    }

    /// Get the YES token ID for CLOB API calls
    pub fn yes_token_id(&self) -> Option<String> {
        self.parse_clob_token_ids().map(|(yes, _)| yes)
    }

    /// Get the NO token ID for CLOB API calls
    pub fn no_token_id(&self) -> Option<String> {
        self.parse_clob_token_ids().map(|(_, no)| no)
    }

    /// Check if category indicates a sports market
    pub fn is_sports_category(category: Option<&str>) -> bool {
        let sports_keywords = [
            "nfl", "nba", "mlb", "nhl", "mls", "soccer", "football", "basketball",
            "baseball", "hockey", "tennis", "boxing", "ufc", "mma", "cricket",
            "f1", "formula 1", "golf", "esports", "counter strike", "valorant",
            "league of legends", "la liga", "premier league", "champions league",
            "serie a", "bundesliga", "ncaa", "college football", "college basketball",
        ];

        if let Some(cat) = category {
            let cat_lower = cat.to_lowercase();
            sports_keywords.iter().any(|kw| cat_lower.contains(kw))
        } else {
            false
        }
    }

    /// Parse team names from title if it matches "Team A vs Team B" pattern
    pub fn parse_teams_from_title(title: &str) -> Option<(String, String)> {
        // Try "vs" first, then "v"
        let separators = [" vs ", " vs. ", " v ", " v. "];

        for sep in separators {
            if let Some(pos) = title.to_lowercase().find(sep) {
                let team_a = title[..pos].trim().to_string();
                let team_b = title[pos + sep.len()..].trim();
                // Remove any trailing question mark or extra text
                let team_b = team_b.split('?').next().unwrap_or(team_b).trim().to_string();
                if !team_a.is_empty() && !team_b.is_empty() {
                    return Some((team_a, team_b));
                }
            }
        }
        None
    }

    /// Convert to terminal-core PredictionMarket
    pub fn to_prediction_market(&self) -> terminal_core::PredictionMarket {
        use terminal_core::{MarketStatus, Platform, PredictionMarket};

        let (yes_price, no_price) = self.parse_outcome_prices().unwrap_or((Decimal::ZERO, Decimal::ZERO));

        let status = match (self.active, self.closed) {
            (_, Some(true)) => MarketStatus::Closed,
            (Some(false), _) => MarketStatus::Closed,
            _ => MarketStatus::Open,
        };

        // For grouped markets, use event slug for URL; otherwise use market slug
        let (url, title) = if let Some(group_item) = &self.group_item_title {
            // This is a grouped market (e.g., "Jurassic World" within "Top grossing movie 2025")
            let event_slug = self.events
                .as_ref()
                .and_then(|e| e.first())
                .and_then(|e| e.slug.clone());

            let event_title = self.events
                .as_ref()
                .and_then(|e| e.first())
                .and_then(|e| e.title.clone());

            let url = event_slug.map(|s| format!("https://polymarket.com/event/{}", s));

            // Format as "Event Title → Option" for better clarity
            let title = if let Some(event_title) = event_title {
                format!("{} → {}", event_title, group_item)
            } else {
                self.question.clone()
            };

            (url, title)
        } else {
            // Regular market
            let url = self.slug.as_ref().map(|s| format!("https://polymarket.com/event/{}", s));
            (url, self.question.clone())
        };

        // Sports detection
        let is_sports = Self::is_sports_category(self.category.as_deref())
            || Self::parse_teams_from_title(&title).is_some();

        let (home_team, away_team, home_odds, away_odds) = if is_sports {
            if let Some((team_a, team_b)) = Self::parse_teams_from_title(&title) {
                (Some(team_a), Some(team_b), Some(yes_price), Some(no_price))
            } else {
                (None, None, None, None)
            }
        } else {
            (None, None, None, None)
        };

        // Create options_json with token ID for sparkline fetching
        // clob_token_ids format is either a JSON array "[\"YES_TOKEN\", \"NO_TOKEN\"]" or a single token
        let options_json = self.clob_token_ids.as_ref().and_then(|ids| {
            let yes_token = if ids.starts_with('[') {
                // Parse as JSON array and get first (YES) token
                serde_json::from_str::<Vec<String>>(ids)
                    .ok()
                    .and_then(|tokens| tokens.first().cloned())
            } else {
                // Single token string
                Some(ids.clone())
            };

            yes_token.and_then(|token_id| {
                serde_json::to_string(&vec![serde_json::json!({
                    "name": "Yes",
                    "clob_token_id": token_id
                })])
                .ok()
            })
        });

        PredictionMarket {
            id: self.id.clone(),
            platform: Platform::Polymarket,
            ticker: self.condition_id.clone(),
            title,
            description: self.description.clone(),
            category: self.category.clone(),
            yes_price,
            no_price,
            volume: self.parse_volume(),
            volume_24hr: None, // Individual markets don't have 24hr volume in API
            liquidity: self.parse_liquidity(),
            close_time: self.end_date,
            created_at: self.created_at,
            status,
            image_url: self.image.clone().or(self.icon.clone()),
            url,
            // Individual markets are binary by default
            outcome_count: None,
            leading_outcome: None,
            is_multi_outcome: false,
            options_json,
            // Sports fields
            is_sports,
            is_live: false, // Polymarket API doesn't provide live game data
            score: None,
            game_period: None,
            home_team,
            away_team,
            home_odds,
            away_odds,
            spread_line: None,
            total_line: None,
            resolution_source: self.resolution_source.clone(),
            // Individual markets don't have tags - tags are on events
            tags: Vec::new(),
        }
    }
}

/// A Polymarket event (contains multiple markets)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolymarketEvent {
    /// Event ID
    pub id: String,

    /// Event title
    pub title: String,

    /// Event slug
    #[serde(default)]
    pub slug: Option<String>,

    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// Start date
    #[serde(default)]
    pub start_date: Option<DateTime<Utc>>,

    /// End date
    #[serde(default)]
    pub end_date: Option<DateTime<Utc>>,

    /// When the event was created
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,

    /// Category
    #[serde(default)]
    pub category: Option<String>,

    /// Whether the event is active
    #[serde(default)]
    pub active: Option<bool>,

    /// Whether the event is closed
    #[serde(default)]
    pub closed: Option<bool>,

    /// Total liquidity
    #[serde(default)]
    pub liquidity: Option<f64>,

    /// Total volume
    #[serde(default)]
    pub volume: Option<f64>,

    /// 24-hour volume (directly from Polymarket API)
    #[serde(default, rename = "volume24hr")]
    pub volume_24hr: Option<f64>,

    /// Image URL
    #[serde(default)]
    pub image: Option<String>,

    /// Associated markets
    #[serde(default)]
    pub markets: Vec<PolymarketMarket>,

    /// Resolution source - describes how the event will be resolved
    #[serde(default, rename = "resolutionSource")]
    pub resolution_source: Option<String>,

    /// Tags associated with this event (e.g., "Politics", "Crypto", "AI")
    #[serde(default)]
    pub tags: Vec<PolymarketTag>,
}

/// Option data for multi-outcome events (stored as JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOption {
    pub name: String,
    pub yes_price: Decimal,
    pub market_id: String,
    /// YES token ID for orderbook and price history API calls
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clob_token_id: Option<String>,
    /// Condition ID for trades API filtering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_id: Option<String>,
}

// ============================================================================
// CLOB API Types (for order book and trades)
// ============================================================================

/// CLOB API base URL
pub const CLOB_API_BASE: &str = "https://clob.polymarket.com";

/// Response from GET /book
#[derive(Debug, Clone, Deserialize)]
pub struct ClobOrderbookResponse {
    /// Market identifier
    #[serde(default)]
    pub market: Option<String>,
    /// Asset/token ID
    #[serde(default)]
    pub asset_id: Option<String>,
    /// Hash of the order book state
    #[serde(default)]
    pub hash: Option<String>,
    /// Timestamp as string
    #[serde(default)]
    pub timestamp: Option<String>,
    /// Bid levels (buy orders)
    #[serde(default)]
    pub bids: Vec<ClobOrderLevel>,
    /// Ask levels (sell orders)
    #[serde(default)]
    pub asks: Vec<ClobOrderLevel>,
}

/// A single level in the CLOB order book
#[derive(Debug, Clone, Deserialize)]
pub struct ClobOrderLevel {
    /// Price as string (0.00 - 1.00)
    pub price: String,
    /// Size/quantity as string
    pub size: String,
}

impl ClobOrderbookResponse {
    /// Convert to terminal-core OrderBook
    pub fn to_order_book(&self, market_id: &str, is_yes_token: bool) -> terminal_core::OrderBook {
        use terminal_core::{OrderBook, OrderBookLevel, Platform};
        use chrono::Utc;
        use std::str::FromStr;

        let mut order_book = OrderBook::new(market_id.to_string(), Platform::Polymarket);
        order_book.timestamp = Utc::now();

        // Convert bid levels and sort descending (best/highest bid first)
        let mut bids: Vec<OrderBookLevel> = self.bids
            .iter()
            .filter_map(|level| {
                let price = Decimal::from_str(&level.price).ok()?;
                let quantity = Decimal::from_str(&level.size).ok()?;
                Some(OrderBookLevel::new(price, quantity))
            })
            .collect();
        bids.sort_by(|a, b| b.price.cmp(&a.price)); // Descending: best bid first

        // Convert ask levels and sort ascending (best/lowest ask first)
        let mut asks: Vec<OrderBookLevel> = self.asks
            .iter()
            .filter_map(|level| {
                let price = Decimal::from_str(&level.price).ok()?;
                let quantity = Decimal::from_str(&level.size).ok()?;
                Some(OrderBookLevel::new(price, quantity))
            })
            .collect();
        asks.sort_by(|a, b| a.price.cmp(&b.price)); // Ascending: best ask first

        if is_yes_token {
            order_book.yes_bids = bids;
            order_book.yes_asks = asks;
        } else {
            order_book.no_bids = bids;
            order_book.no_asks = asks;
        }

        order_book
    }
}

/// Response from GET /trades
#[derive(Debug, Clone, Deserialize)]
pub struct ClobTradesResponse {
    #[serde(default)]
    pub data: Vec<ClobTrade>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

/// A single trade from the CLOB API
#[derive(Debug, Clone, Deserialize)]
pub struct ClobTrade {
    /// Trade ID
    #[serde(default)]
    pub id: Option<String>,
    /// Asset/token ID
    #[serde(default)]
    pub asset_id: Option<String>,
    /// Price as string
    #[serde(default)]
    pub price: Option<String>,
    /// Size/quantity as string
    #[serde(default)]
    pub size: Option<String>,
    /// Trade side ("BUY" or "SELL")
    #[serde(default)]
    pub side: Option<String>,
    /// Timestamp as Unix timestamp (seconds or milliseconds)
    #[serde(default)]
    pub timestamp: Option<i64>,
    /// Transaction hash
    #[serde(default)]
    pub transaction_hash: Option<String>,
}

impl ClobTrade {
    /// Convert to terminal-core Trade
    pub fn to_trade(&self, market_id: &str) -> terminal_core::Trade {
        use terminal_core::{Platform, Trade, TradeOutcome, TradeSide};
        use chrono::{TimeZone, Utc};
        use std::str::FromStr;

        let side = match self.side.as_deref() {
            Some("BUY") => Some(TradeSide::Buy),
            Some("SELL") => Some(TradeSide::Sell),
            _ => None,
        };

        let price = self.price
            .as_ref()
            .and_then(|p| Decimal::from_str(p).ok())
            .unwrap_or(Decimal::ZERO);

        let quantity = self.size
            .as_ref()
            .and_then(|s| Decimal::from_str(s).ok())
            .unwrap_or(Decimal::ZERO);

        // Parse timestamp (could be seconds or milliseconds)
        let timestamp = self.timestamp
            .map(|ts| {
                if ts > 10_000_000_000 {
                    // Milliseconds
                    Utc.timestamp_millis_opt(ts).single().unwrap_or_else(Utc::now)
                } else {
                    // Seconds
                    Utc.timestamp_opt(ts, 0).single().unwrap_or_else(Utc::now)
                }
            })
            .unwrap_or_else(Utc::now);

        Trade {
            id: self.id.clone()
                .or(self.transaction_hash.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            market_id: market_id.to_string(),
            platform: Platform::Polymarket,
            timestamp,
            price,
            quantity,
            outcome: TradeOutcome::Yes, // Default to YES for the token
            side,
            transaction_hash: self.transaction_hash.clone(),
        }
    }
}

// ============================================================================
// Public Data API Types (https://data-api.polymarket.com)
// ============================================================================

/// A trade from the public data API (no authentication required)
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataApiTrade {
    /// Side (BUY or SELL)
    pub side: String,
    /// Asset/token ID
    pub asset: String,
    /// Condition ID (market hash)
    pub condition_id: String,
    /// Trade size (amount)
    pub size: f64,
    /// Trade price
    pub price: f64,
    /// Unix timestamp (seconds)
    pub timestamp: i64,
    /// Event title
    #[serde(default)]
    pub title: Option<String>,
    /// Outcome (Yes/No)
    #[serde(default)]
    pub outcome: Option<String>,
    /// Transaction hash
    #[serde(default)]
    pub transaction_hash: Option<String>,
}

impl DataApiTrade {
    /// Convert to terminal-core Trade
    pub fn to_trade(&self, market_id: &str) -> terminal_core::Trade {
        use terminal_core::{Platform, Trade, TradeOutcome, TradeSide};
        use chrono::{TimeZone, Utc};

        let side = match self.side.as_str() {
            "BUY" => Some(TradeSide::Buy),
            "SELL" => Some(TradeSide::Sell),
            _ => None,
        };

        let outcome = match self.outcome.as_deref() {
            Some("Yes") => TradeOutcome::Yes,
            Some("No") => TradeOutcome::No,
            _ => TradeOutcome::Yes,
        };

        let timestamp = if self.timestamp > 10_000_000_000 {
            Utc.timestamp_millis_opt(self.timestamp).single().unwrap_or_else(Utc::now)
        } else {
            Utc.timestamp_opt(self.timestamp, 0).single().unwrap_or_else(Utc::now)
        };

        Trade {
            id: self.transaction_hash.clone()
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            market_id: market_id.to_string(),
            platform: Platform::Polymarket,
            timestamp,
            price: Decimal::from_str(&self.price.to_string()).unwrap_or(Decimal::ZERO),
            quantity: Decimal::from_str(&self.size.to_string()).unwrap_or(Decimal::ZERO),
            outcome,
            side,
            transaction_hash: self.transaction_hash.clone(),
        }
    }
}

// ============================================================================
// Price History Types (from CLOB API /prices-history)
// ============================================================================

/// Response from GET /prices-history
#[derive(Debug, Clone, Deserialize)]
pub struct PricesHistoryResponse {
    /// List of timestamp/price pairs
    pub history: Vec<PriceHistoryPoint>,
}

/// A single price point from the CLOB API
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PriceHistoryPoint {
    /// Unix timestamp in seconds
    pub t: i64,
    /// Price (0.0 - 1.0)
    pub p: f64,
}

impl PolymarketEvent {
    /// Check if this is a binary event (single market with 2 outcomes)
    pub fn is_binary(&self) -> bool {
        self.markets.len() == 1
    }

    /// Parse volume from f64 to Decimal
    fn parse_volume(&self) -> Decimal {
        self.volume
            .map(|v| Decimal::from_str(&v.to_string()).unwrap_or(Decimal::ZERO))
            .unwrap_or(Decimal::ZERO)
    }

    /// Parse liquidity from f64 to Decimal
    fn parse_liquidity(&self) -> Option<Decimal> {
        self.liquidity
            .map(|l| Decimal::from_str(&l.to_string()).unwrap_or(Decimal::ZERO))
    }

    /// Convert event to a PredictionMarket
    /// For multi-outcome events, shows the leading option's probability
    pub fn to_prediction_market(&self) -> terminal_core::PredictionMarket {
        use terminal_core::{MarketStatus, Platform, PredictionMarket};

        let status = match (self.active, self.closed) {
            (_, Some(true)) => MarketStatus::Closed,
            (Some(false), _) => MarketStatus::Closed,
            _ => MarketStatus::Open,
        };

        let url = self.slug.as_ref().map(|s| format!("https://polymarket.com/event/{}", s));

        // Sports detection (use PolymarketMarket's helper methods)
        let is_sports = PolymarketMarket::is_sports_category(self.category.as_deref())
            || PolymarketMarket::parse_teams_from_title(&self.title).is_some();

        let (home_team, away_team) = if is_sports {
            PolymarketMarket::parse_teams_from_title(&self.title)
                .map(|(a, b)| (Some(a), Some(b)))
                .unwrap_or((None, None))
        } else {
            (None, None)
        };

        // Extract tag labels
        let tags: Vec<String> = self.tags.iter().map(|t| t.label.clone()).collect();

        if self.is_binary() {
            // Single market - use the market's prices directly
            let market = &self.markets[0];
            let (yes_price, no_price) = market.parse_outcome_prices().unwrap_or((Decimal::ZERO, Decimal::ZERO));

            let (home_odds, away_odds) = if is_sports && home_team.is_some() {
                (Some(yes_price), Some(no_price))
            } else {
                (None, None)
            };

            // Create options_json with token ID for sparkline fetching
            let options_json = market.yes_token_id().and_then(|token_id| {
                serde_json::to_string(&vec![serde_json::json!({
                    "name": "Yes",
                    "clob_token_id": token_id
                })])
                .ok()
            });

            PredictionMarket {
                id: self.id.clone(),
                platform: Platform::Polymarket,
                ticker: market.condition_id.clone(),
                title: self.title.clone(),
                description: self.description.clone(),
                category: self.category.clone(),
                yes_price,
                no_price,
                volume: self.parse_volume(),
                volume_24hr: self.volume_24hr.map(|v| Decimal::from_str(&v.to_string()).unwrap_or(Decimal::ZERO)),
                liquidity: self.parse_liquidity(),
                close_time: self.end_date,
                created_at: self.created_at.or(self.start_date),
                status,
                image_url: self.image.clone(),
                url,
                outcome_count: None,
                leading_outcome: None,
                is_multi_outcome: false,
                options_json,
                // For binary events, resolution rules may be in child market description
                resolution_source: self.resolution_source.clone()
                    .filter(|s| !s.is_empty())
                    .or_else(|| {
                        self.markets.first()
                            .and_then(|m| m.description.clone())
                            .filter(|d| d.len() > 100) // Detailed rules are longer
                    }),
                // Sports fields
                is_sports,
                is_live: false,
                score: None,
                game_period: None,
                home_team,
                away_team,
                home_odds,
                away_odds,
                spread_line: None,
                total_line: None,
                tags: tags.clone(),
            }
        } else {
            // Multi-outcome event - find the leading option
            let mut options: Vec<MarketOption> = Vec::new();
            let mut leading_option: Option<(String, Decimal, Option<String>)> = None;

            for market in &self.markets {
                let (yes_price, _) = market.parse_outcome_prices().unwrap_or((Decimal::ZERO, Decimal::ZERO));

                // Use group_item_title if available, otherwise use question
                let option_name = market.group_item_title
                    .clone()
                    .unwrap_or_else(|| market.question.clone());

                options.push(MarketOption {
                    name: option_name.clone(),
                    yes_price,
                    market_id: market.id.clone(),
                    clob_token_id: market.yes_token_id(),
                    condition_id: market.condition_id.clone(),
                });

                // Track leading option (highest probability) with its description
                match &leading_option {
                    None => leading_option = Some((option_name, yes_price, market.description.clone())),
                    Some((_, current_price, _)) if yes_price > *current_price => {
                        leading_option = Some((option_name, yes_price, market.description.clone()));
                    }
                    _ => {}
                }
            }

            let (leading_name, yes_price, leading_description) = leading_option.unwrap_or(("Unknown".to_string(), Decimal::ZERO, None));
            let no_price = Decimal::ONE - yes_price;

            // Serialize options to JSON for detail view
            let options_json = serde_json::to_string(&options).ok();

            PredictionMarket {
                id: self.id.clone(),
                platform: Platform::Polymarket,
                ticker: None, // Multi-outcome events don't have a single condition_id
                title: self.title.clone(),
                description: self.description.clone(),
                category: self.category.clone(),
                yes_price,
                no_price,
                volume: self.parse_volume(),
                volume_24hr: self.volume_24hr.map(|v| Decimal::from_str(&v.to_string()).unwrap_or(Decimal::ZERO)),
                liquidity: self.parse_liquidity(),
                close_time: self.end_date,
                created_at: self.created_at.or(self.start_date),
                status,
                image_url: self.image.clone(),
                url,
                outcome_count: Some(self.markets.len()),
                leading_outcome: Some(leading_name),
                is_multi_outcome: true,
                options_json,
                // For multi-outcome events, use leading outcome's resolution rules
                resolution_source: self.resolution_source.clone()
                    .filter(|s| !s.is_empty())
                    .or_else(|| {
                        // Use leading market's description (the one with highest probability)
                        leading_description.clone()
                            .filter(|d| d.len() > 100) // Detailed rules are longer
                    }),
                // Sports fields - multi-outcome sports are less common, but support them
                is_sports,
                is_live: false,
                score: None,
                game_period: None,
                home_team: None, // Multi-outcome doesn't map to 2 teams
                away_team: None,
                home_odds: None,
                away_odds: None,
                spread_line: None,
                total_line: None,
                tags,
            }
        }
    }
}
