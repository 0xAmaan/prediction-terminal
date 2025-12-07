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
            options_json: None,
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

    /// Image URL
    #[serde(default)]
    pub image: Option<String>,

    /// Associated markets
    #[serde(default)]
    pub markets: Vec<PolymarketMarket>,
}

/// Option data for multi-outcome events (stored as JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOption {
    pub name: String,
    pub yes_price: Decimal,
    pub market_id: String,
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

        // Convert bid levels
        let bids: Vec<OrderBookLevel> = self.bids
            .iter()
            .filter_map(|level| {
                let price = Decimal::from_str(&level.price).ok()?;
                let quantity = Decimal::from_str(&level.size).ok()?;
                Some(OrderBookLevel::new(price, quantity))
            })
            .collect();

        // Convert ask levels
        let asks: Vec<OrderBookLevel> = self.asks
            .iter()
            .filter_map(|level| {
                let price = Decimal::from_str(&level.price).ok()?;
                let quantity = Decimal::from_str(&level.size).ok()?;
                Some(OrderBookLevel::new(price, quantity))
            })
            .collect();

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
        }
    }
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

        if self.is_binary() {
            // Single market - use the market's prices directly
            let market = &self.markets[0];
            let (yes_price, no_price) = market.parse_outcome_prices().unwrap_or((Decimal::ZERO, Decimal::ZERO));

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
                liquidity: self.parse_liquidity(),
                close_time: self.end_date,
                created_at: self.created_at.or(self.start_date),
                status,
                image_url: self.image.clone(),
                url,
                outcome_count: None,
                leading_outcome: None,
                is_multi_outcome: false,
                options_json: None,
            }
        } else {
            // Multi-outcome event - find the leading option
            let mut options: Vec<MarketOption> = Vec::new();
            let mut leading_option: Option<(String, Decimal)> = None;

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
                });

                // Track leading option (highest probability)
                match &leading_option {
                    None => leading_option = Some((option_name, yes_price)),
                    Some((_, current_price)) if yes_price > *current_price => {
                        leading_option = Some((option_name, yes_price));
                    }
                    _ => {}
                }
            }

            let (leading_name, yes_price) = leading_option.unwrap_or(("Unknown".to_string(), Decimal::ZERO));
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
            }
        }
    }
}
