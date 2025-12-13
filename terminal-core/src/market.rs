//! Market data structures for prediction markets

use crate::platform::Platform;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Status of a prediction market
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketStatus {
    /// Market is open for trading
    Open,
    /// Market is closed but not yet settled
    Closed,
    /// Market has been settled with a final outcome
    Settled,
}

impl Default for MarketStatus {
    fn default() -> Self {
        MarketStatus::Open
    }
}

/// A prediction market from a specific platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionMarket {
    /// Unique identifier on the platform
    pub id: String,

    /// Which platform this market is from
    pub platform: Platform,

    /// Platform-specific ticker symbol (e.g., "KXBTC-100K-25DEC31" for Kalshi)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,

    /// Human-readable title/question
    pub title: String,

    /// Detailed description of the market
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Category (e.g., "Politics", "Crypto", "Sports")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Current YES price (0.00 - 1.00, represents probability)
    pub yes_price: Decimal,

    /// Current NO price (0.00 - 1.00, should be ~1 - yes_price)
    pub no_price: Decimal,

    /// Trading volume (in platform's native unit)
    pub volume: Decimal,

    /// Available liquidity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub liquidity: Option<Decimal>,

    /// When the market closes for trading
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_time: Option<DateTime<Utc>>,

    /// When the market was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,

    /// Current status of the market
    pub status: MarketStatus,

    /// URL to market image/icon
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,

    /// URL to the market on the platform's website
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// For multi-outcome events: number of options available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome_count: Option<usize>,

    /// For multi-outcome events: name of the leading option
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leading_outcome: Option<String>,

    /// Whether this is a multi-outcome event (more than 2 outcomes)
    #[serde(default)]
    pub is_multi_outcome: bool,

    /// For multi-outcome events: JSON array of all options
    /// Format: [{ "name": string, "yes_price": number, "market_id": string }, ...]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options_json: Option<String>,

    /// Resolution source - describes how the market will be resolved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution_source: Option<String>,

    /// Tags for categorization (e.g., "Politics", "Crypto", "AI")
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    // ========================================================================
    // Sports-specific fields
    // ========================================================================
    /// Whether this is a sports market (detected from category/title)
    #[serde(default)]
    pub is_sports: bool,

    /// Whether this game is currently live/in-progress
    #[serde(default)]
    pub is_live: bool,

    /// Current score (e.g., "13 - 6")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<String>,

    /// Current game period (e.g., "Q3", "2nd Half", "Map 2")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_period: Option<String>,

    /// Home/first team name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home_team: Option<String>,

    /// Away/second team name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub away_team: Option<String>,

    /// Home team odds (0.00 - 1.00)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub home_odds: Option<Decimal>,

    /// Away team odds (0.00 - 1.00)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub away_odds: Option<Decimal>,

    /// Spread line (e.g., "-3.5")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spread_line: Option<String>,

    /// Total/over-under line (e.g., "45.5")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_line: Option<String>,
}

impl PredictionMarket {
    /// Calculate the implied probability from the YES price
    pub fn implied_probability(&self) -> Decimal {
        self.yes_price
    }

    /// Check if this market is currently tradeable
    pub fn is_tradeable(&self) -> bool {
        self.status == MarketStatus::Open
    }
}

/// A unified market that may exist on multiple platforms
/// Used for cross-platform comparison and spread detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMarket {
    /// Internal identifier for the unified market
    pub id: String,

    /// Canonical title for this market
    pub title: String,

    /// Category for filtering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// The market on Kalshi (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kalshi: Option<PredictionMarket>,

    /// The market on Polymarket (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polymarket: Option<PredictionMarket>,

    /// Price spread between platforms (absolute difference in YES prices)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spread: Option<Decimal>,
}

impl UnifiedMarket {
    /// Create a new unified market with just a Kalshi market
    pub fn from_kalshi(market: PredictionMarket) -> Self {
        Self {
            id: format!("kalshi:{}", market.id),
            title: market.title.clone(),
            category: market.category.clone(),
            kalshi: Some(market),
            polymarket: None,
            spread: None,
        }
    }

    /// Create a new unified market with just a Polymarket market
    pub fn from_polymarket(market: PredictionMarket) -> Self {
        Self {
            id: format!("poly:{}", market.id),
            title: market.title.clone(),
            category: market.category.clone(),
            kalshi: None,
            polymarket: Some(market),
            spread: None,
        }
    }

    /// Create a matched unified market from both platforms
    pub fn matched(
        id: String,
        title: String,
        kalshi: PredictionMarket,
        polymarket: PredictionMarket,
    ) -> Self {
        let spread = (kalshi.yes_price - polymarket.yes_price).abs();
        Self {
            id,
            title,
            category: kalshi.category.clone().or(polymarket.category.clone()),
            kalshi: Some(kalshi),
            polymarket: Some(polymarket),
            spread: Some(spread),
        }
    }

    /// Calculate the spread between platforms (if both exist)
    pub fn calculate_spread(&mut self) {
        if let (Some(k), Some(p)) = (&self.kalshi, &self.polymarket) {
            self.spread = Some((k.yes_price - p.yes_price).abs());
        }
    }

    /// Check if this market exists on both platforms
    pub fn is_matched(&self) -> bool {
        self.kalshi.is_some() && self.polymarket.is_some()
    }

    /// Get the best YES price across platforms (lowest)
    pub fn best_yes_price(&self) -> Option<(Platform, Decimal)> {
        match (&self.kalshi, &self.polymarket) {
            (Some(k), Some(p)) => {
                if k.yes_price <= p.yes_price {
                    Some((Platform::Kalshi, k.yes_price))
                } else {
                    Some((Platform::Polymarket, p.yes_price))
                }
            }
            (Some(k), None) => Some((Platform::Kalshi, k.yes_price)),
            (None, Some(p)) => Some((Platform::Polymarket, p.yes_price)),
            (None, None) => None,
        }
    }

    /// Get the best NO price across platforms (lowest)
    pub fn best_no_price(&self) -> Option<(Platform, Decimal)> {
        match (&self.kalshi, &self.polymarket) {
            (Some(k), Some(p)) => {
                if k.no_price <= p.no_price {
                    Some((Platform::Kalshi, k.no_price))
                } else {
                    Some((Platform::Polymarket, p.no_price))
                }
            }
            (Some(k), None) => Some((Platform::Kalshi, k.no_price)),
            (None, Some(p)) => Some((Platform::Polymarket, p.no_price)),
            (None, None) => None,
        }
    }
}

// ============================================================================
// Order Book Types
// ============================================================================

/// A single price level in the order book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    /// Price (0.00 - 1.00 representing probability)
    pub price: Decimal,
    /// Total quantity at this level
    pub quantity: Decimal,
    /// Number of orders at this level (if available from the platform)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_count: Option<u32>,
}

impl OrderBookLevel {
    /// Create a new order book level
    pub fn new(price: Decimal, quantity: Decimal) -> Self {
        Self {
            price,
            quantity,
            order_count: None,
        }
    }
}

/// Order book snapshot for a prediction market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    /// Market identifier
    pub market_id: String,
    /// Platform
    pub platform: Platform,
    /// Timestamp of the snapshot
    pub timestamp: DateTime<Utc>,
    /// YES outcome bids (sorted by price descending - best bid first)
    pub yes_bids: Vec<OrderBookLevel>,
    /// YES outcome asks (sorted by price ascending - best ask first)
    pub yes_asks: Vec<OrderBookLevel>,
    /// NO outcome bids (sorted by price descending - best bid first)
    pub no_bids: Vec<OrderBookLevel>,
    /// NO outcome asks (sorted by price ascending - best ask first)
    pub no_asks: Vec<OrderBookLevel>,
    /// Sequence number for ordering updates (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<u64>,
}

impl OrderBook {
    /// Create an empty order book
    pub fn new(market_id: String, platform: Platform) -> Self {
        Self {
            market_id,
            platform,
            timestamp: Utc::now(),
            yes_bids: Vec::new(),
            yes_asks: Vec::new(),
            no_bids: Vec::new(),
            no_asks: Vec::new(),
            sequence: None,
        }
    }

    /// Calculate the YES spread (best ask - best bid)
    pub fn yes_spread(&self) -> Option<Decimal> {
        match (self.yes_bids.first(), self.yes_asks.first()) {
            (Some(bid), Some(ask)) => Some(ask.price - bid.price),
            _ => None,
        }
    }

    /// Calculate the YES mid price
    pub fn yes_mid_price(&self) -> Option<Decimal> {
        match (self.yes_bids.first(), self.yes_asks.first()) {
            (Some(bid), Some(ask)) => Some((bid.price + ask.price) / Decimal::from(2)),
            _ => None,
        }
    }

    /// Get the best YES bid price
    pub fn best_yes_bid(&self) -> Option<Decimal> {
        self.yes_bids.first().map(|l| l.price)
    }

    /// Get the best YES ask price
    pub fn best_yes_ask(&self) -> Option<Decimal> {
        self.yes_asks.first().map(|l| l.price)
    }
}

// ============================================================================
// Trade Types
// ============================================================================

/// Side of a trade (from the taker's perspective)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TradeSide {
    /// Buying the outcome (taker bought)
    Buy,
    /// Selling the outcome (taker sold)
    Sell,
}

/// Outcome type for trades
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TradeOutcome {
    Yes,
    No,
}

/// A single trade in a prediction market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// Trade ID (platform-specific)
    pub id: String,
    /// Market identifier
    pub market_id: String,
    /// Platform
    pub platform: Platform,
    /// Timestamp of the trade
    pub timestamp: DateTime<Utc>,
    /// Price at which the trade occurred (0.00 - 1.00)
    pub price: Decimal,
    /// Quantity traded
    pub quantity: Decimal,
    /// Which outcome was traded
    pub outcome: TradeOutcome,
    /// Trade side (buy or sell) from taker's perspective
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<TradeSide>,
    /// Transaction hash (for on-chain trades like Polymarket on Polygon)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_hash: Option<String>,
}

/// Trade history response with pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeHistory {
    /// Market identifier
    pub market_id: String,
    /// Platform
    pub platform: Platform,
    /// List of trades (most recent first)
    pub trades: Vec<Trade>,
    /// Cursor for pagination (if more trades available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

// ============================================================================
// Price History Types
// ============================================================================

/// Time interval for price candles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PriceInterval {
    /// 1 minute candles
    #[serde(rename = "1m")]
    OneMinute,
    /// 5 minute candles
    #[serde(rename = "5m")]
    FiveMinutes,
    /// 15 minute candles
    #[serde(rename = "15m")]
    FifteenMinutes,
    /// 1 hour candles
    #[serde(rename = "1h")]
    OneHour,
    /// 4 hour candles
    #[serde(rename = "4h")]
    FourHours,
    /// 1 day candles
    #[serde(rename = "1d")]
    OneDay,
}

impl PriceInterval {
    /// Get the interval duration in seconds
    pub fn to_seconds(&self) -> u32 {
        match self {
            PriceInterval::OneMinute => 60,
            PriceInterval::FiveMinutes => 300,
            PriceInterval::FifteenMinutes => 900,
            PriceInterval::OneHour => 3600,
            PriceInterval::FourHours => 14400,
            PriceInterval::OneDay => 86400,
        }
    }

    /// Parse from string representation
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "1m" => Some(PriceInterval::OneMinute),
            "5m" => Some(PriceInterval::FiveMinutes),
            "15m" => Some(PriceInterval::FifteenMinutes),
            "1h" => Some(PriceInterval::OneHour),
            "4h" => Some(PriceInterval::FourHours),
            "1d" => Some(PriceInterval::OneDay),
            _ => None,
        }
    }
}

impl Default for PriceInterval {
    fn default() -> Self {
        PriceInterval::OneHour
    }
}

/// A single OHLCV candle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceCandle {
    /// Start time of the candle
    pub timestamp: DateTime<Utc>,
    /// Opening price
    pub open: Decimal,
    /// Highest price during the period
    pub high: Decimal,
    /// Lowest price during the period
    pub low: Decimal,
    /// Closing price
    pub close: Decimal,
    /// Trading volume during the period
    pub volume: Decimal,
}

impl PriceCandle {
    /// Check if this is a bullish candle (close > open)
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    /// Get the candle body size (absolute difference between open and close)
    pub fn body_size(&self) -> Decimal {
        (self.close - self.open).abs()
    }

    /// Get the candle range (high - low)
    pub fn range(&self) -> Decimal {
        self.high - self.low
    }
}

/// Historical price data for a market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistory {
    /// Market identifier
    pub market_id: String,
    /// Platform
    pub platform: Platform,
    /// Candle interval
    pub interval: PriceInterval,
    /// Candles sorted by timestamp ascending (oldest first)
    pub candles: Vec<PriceCandle>,
}

impl PriceHistory {
    /// Create empty price history
    pub fn new(market_id: String, platform: Platform, interval: PriceInterval) -> Self {
        Self {
            market_id,
            platform,
            interval,
            candles: Vec::new(),
        }
    }

    /// Get the most recent candle
    pub fn latest(&self) -> Option<&PriceCandle> {
        self.candles.last()
    }

    /// Get the price change from first to last candle
    pub fn price_change(&self) -> Option<Decimal> {
        match (self.candles.first(), self.candles.last()) {
            (Some(first), Some(last)) => Some(last.close - first.open),
            _ => None,
        }
    }

    /// Get the price change as a percentage
    pub fn price_change_percent(&self) -> Option<Decimal> {
        match (self.candles.first(), self.candles.last()) {
            (Some(first), Some(last)) if !first.open.is_zero() => {
                Some(((last.close - first.open) / first.open) * Decimal::from(100))
            }
            _ => None,
        }
    }
}
