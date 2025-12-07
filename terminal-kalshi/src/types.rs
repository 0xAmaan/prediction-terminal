//! Kalshi API response types
//!
//! These types mirror the Kalshi API responses and are converted
//! to terminal-core types for use in the application.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Response from GET /markets
#[derive(Debug, Clone, Deserialize)]
pub struct MarketsResponse {
    pub markets: Vec<KalshiMarket>,
    #[serde(default)]
    pub cursor: Option<String>,
}

/// Response from GET /events
#[derive(Debug, Clone, Deserialize)]
pub struct EventsResponse {
    pub events: Vec<KalshiEvent>,
    #[serde(default)]
    pub cursor: Option<String>,
}

/// Response from GET /events/{event_ticker}
#[derive(Debug, Clone, Deserialize)]
pub struct EventResponse {
    pub event: KalshiEvent,
}

/// A Kalshi event from the API
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KalshiEvent {
    /// Event ticker (unique identifier)
    pub event_ticker: String,

    /// Series ticker this event belongs to
    #[serde(default)]
    pub series_ticker: Option<String>,

    /// Event title
    #[serde(default)]
    pub title: Option<String>,

    /// Category
    #[serde(default)]
    pub category: Option<String>,
}

/// Response from GET /markets/{ticker}
#[derive(Debug, Clone, Deserialize)]
pub struct MarketResponse {
    pub market: KalshiMarket,
}

/// A Kalshi market from the API
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KalshiMarket {
    /// Market ticker (unique identifier)
    pub ticker: String,

    /// Event ticker this market belongs to
    #[serde(default)]
    pub event_ticker: Option<String>,

    /// Market title
    pub title: String,

    /// Short description
    #[serde(default)]
    pub subtitle: Option<String>,

    /// Current YES price in cents (1-99)
    #[serde(default)]
    pub yes_bid: Option<i64>,

    /// Current YES ask in cents
    #[serde(default)]
    pub yes_ask: Option<i64>,

    /// Last traded YES price in cents
    #[serde(default)]
    pub last_price: Option<i64>,

    /// NO bid price in cents
    #[serde(default)]
    pub no_bid: Option<i64>,

    /// NO ask price in cents
    #[serde(default)]
    pub no_ask: Option<i64>,

    /// 24h volume
    #[serde(default)]
    pub volume: Option<i64>,

    /// Total volume
    #[serde(default)]
    pub volume_24h: Option<i64>,

    /// Open interest
    #[serde(default)]
    pub open_interest: Option<i64>,

    /// Market status
    #[serde(default)]
    pub status: Option<String>,

    /// When the market closes
    #[serde(default)]
    pub close_time: Option<DateTime<Utc>>,

    /// When the market was created
    #[serde(default)]
    pub created_time: Option<DateTime<Utc>>,

    /// When the market opened for trading
    #[serde(default)]
    pub open_time: Option<DateTime<Utc>>,

    /// When the market expires
    #[serde(default)]
    pub expiration_time: Option<DateTime<Utc>>,

    /// Category
    #[serde(default)]
    pub category: Option<String>,

    /// Result if settled (yes/no)
    #[serde(default)]
    pub result: Option<String>,

    /// Image URL
    #[serde(default)]
    pub image_url: Option<String>,
}

impl KalshiMarket {
    /// Convert cents to decimal probability (0.00 - 1.00)
    pub fn cents_to_decimal(cents: Option<i64>) -> Decimal {
        match cents {
            Some(c) => Decimal::from(c) / Decimal::from(100),
            None => Decimal::ZERO,
        }
    }

    /// Extract series ticker from event_ticker by stripping numeric suffix
    /// e.g., "KXELONMARS-99" -> "KXELONMARS", "KXFUSION" -> "KXFUSION"
    fn extract_series_ticker(event_ticker: &str) -> &str {
        // Find the last hyphen
        if let Some(pos) = event_ticker.rfind('-') {
            // Check if everything after the hyphen starts with a digit
            let suffix = &event_ticker[pos + 1..];
            if suffix.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                return &event_ticker[..pos];
            }
        }
        event_ticker
    }

    /// Get the YES price as a decimal
    pub fn yes_price(&self) -> Decimal {
        // Prefer last_price, then yes_bid, then midpoint of bid/ask
        if let Some(price) = self.last_price {
            return Self::cents_to_decimal(Some(price));
        }
        if let Some(bid) = self.yes_bid {
            if let Some(ask) = self.yes_ask {
                return Self::cents_to_decimal(Some((bid + ask) / 2));
            }
            return Self::cents_to_decimal(Some(bid));
        }
        Self::cents_to_decimal(self.yes_ask)
    }

    /// Get the NO price as a decimal
    pub fn no_price(&self) -> Decimal {
        // NO price is typically 1 - YES price
        Decimal::ONE - self.yes_price()
    }

    /// Convert to terminal-core PredictionMarket
    pub fn to_prediction_market(&self) -> terminal_core::PredictionMarket {
        use terminal_core::{MarketStatus, Platform, PredictionMarket};

        let status = match self.status.as_deref() {
            Some("active") | Some("open") => MarketStatus::Open,
            Some("closed") => MarketStatus::Closed,
            Some("settled") | Some("finalized") => MarketStatus::Settled,
            _ => MarketStatus::Open,
        };

        // URL format: https://kalshi.com/markets/{series_ticker}
        // Extract series_ticker from event_ticker by stripping numeric suffix
        // e.g., KXELONMARS-99 -> KXELONMARS
        let series_ticker = self.event_ticker
            .as_ref()
            .map(|et| Self::extract_series_ticker(et))
            .unwrap_or(&self.ticker);
        let url = Some(format!("https://kalshi.com/markets/{}", series_ticker.to_lowercase()));

        PredictionMarket {
            id: self.ticker.clone(),
            platform: Platform::Kalshi,
            ticker: Some(self.ticker.clone()),
            title: self.title.clone(),
            description: self.subtitle.clone(),
            category: self.category.clone(),
            yes_price: self.yes_price(),
            no_price: self.no_price(),
            volume: Decimal::from(self.volume.unwrap_or(0)),
            liquidity: self.open_interest.map(Decimal::from),
            close_time: self.close_time.or(self.expiration_time),
            created_at: self.created_time.or(self.open_time),
            status,
            image_url: self.image_url.clone(),
            url,
            // Kalshi markets are always binary (Yes/No)
            outcome_count: None,
            leading_outcome: None,
            is_multi_outcome: false,
            options_json: None,
        }
    }
}

/// Response from GET /portfolio/positions
#[derive(Debug, Clone, Deserialize)]
pub struct PositionsResponse {
    #[serde(default)]
    pub market_positions: Vec<KalshiPosition>,
    #[serde(default)]
    pub event_positions: Vec<KalshiEventPosition>,
    #[serde(default)]
    pub cursor: Option<String>,
}

/// A position in a Kalshi market
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KalshiPosition {
    /// Market ticker
    pub ticker: String,

    /// Number of YES contracts (positive) or NO contracts (negative)
    #[serde(default)]
    pub position: i64,

    /// Total contracts traded
    #[serde(default)]
    pub total_traded: i64,

    /// Number of resting orders
    #[serde(default)]
    pub resting_orders_count: i64,

    /// Realized P&L in cents
    #[serde(default)]
    pub realized_pnl: i64,

    /// Market exposure
    #[serde(default)]
    pub market_exposure: i64,
}

/// Position at the event level
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KalshiEventPosition {
    pub event_ticker: String,
    #[serde(default)]
    pub event_exposure: i64,
    #[serde(default)]
    pub total_cost: i64,
    #[serde(default)]
    pub realized_pnl: i64,
}

/// Response from GET /portfolio/balance
#[derive(Debug, Clone, Deserialize)]
pub struct BalanceResponse {
    pub balance: i64, // in cents
}

// ============================================================================
// Order Book Types
// ============================================================================

/// Response from GET /markets/{ticker}/orderbook
#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookResponse {
    pub orderbook: KalshiOrderbook,
}

/// Kalshi orderbook structure
///
/// Kalshi returns orders as arrays of [price_cents, quantity] for YES and NO outcomes.
/// These represent resting limit orders (bids) for each outcome.
/// The asks for YES are derived from NO bids (buying NO = selling YES).
#[derive(Debug, Clone, Deserialize)]
pub struct KalshiOrderbook {
    /// YES orders: list of [price_cents, quantity] pairs
    #[serde(default)]
    pub yes: Option<Vec<Vec<i64>>>,
    /// NO orders: list of [price_cents, quantity] pairs
    #[serde(default)]
    pub no: Option<Vec<Vec<i64>>>,
}

impl KalshiOrderbook {
    /// Convert to terminal-core OrderBook
    ///
    /// Kalshi orderbook: YES and NO arrays are resting limit orders (bids) for each outcome.
    /// - YES bids = orders to buy YES
    /// - NO bids = orders to buy NO (which is equivalent to selling YES)
    /// So: YES asks are derived from NO bids (at inverted prices: 100 - price)
    pub fn to_order_book(
        &self,
        market_id: &str,
    ) -> terminal_core::OrderBook {
        use terminal_core::{OrderBook, OrderBookLevel, Platform};
        use chrono::Utc;

        let mut order_book = OrderBook::new(market_id.to_string(), Platform::Kalshi);
        order_book.timestamp = Utc::now();

        // YES bids: direct from yes array
        if let Some(yes_orders) = &self.yes {
            order_book.yes_bids = yes_orders
                .iter()
                .filter(|level| level.len() >= 2)
                .map(|level| {
                    OrderBookLevel::new(
                        Decimal::from(level[0]) / Decimal::from(100),
                        Decimal::from(level[1]),
                    )
                })
                .collect();
            // Sort bids descending by price (best bid first)
            order_book.yes_bids.sort_by(|a, b| b.price.cmp(&a.price));
        }

        // YES asks: derived from NO orders (inverted price: 100 - no_price)
        if let Some(no_orders) = &self.no {
            order_book.yes_asks = no_orders
                .iter()
                .filter(|level| level.len() >= 2)
                .map(|level| {
                    // NO bid at X cents = YES ask at (100 - X) cents
                    let no_price = level[0];
                    let yes_ask_price = 100 - no_price;
                    OrderBookLevel::new(
                        Decimal::from(yes_ask_price) / Decimal::from(100),
                        Decimal::from(level[1]),
                    )
                })
                .collect();
            // Sort asks ascending by price (best ask first)
            order_book.yes_asks.sort_by(|a, b| a.price.cmp(&b.price));
        }

        // NO bids: direct from no array
        if let Some(no_orders) = &self.no {
            order_book.no_bids = no_orders
                .iter()
                .filter(|level| level.len() >= 2)
                .map(|level| {
                    OrderBookLevel::new(
                        Decimal::from(level[0]) / Decimal::from(100),
                        Decimal::from(level[1]),
                    )
                })
                .collect();
            order_book.no_bids.sort_by(|a, b| b.price.cmp(&a.price));
        }

        // NO asks: derived from YES orders (inverted price)
        if let Some(yes_orders) = &self.yes {
            order_book.no_asks = yes_orders
                .iter()
                .filter(|level| level.len() >= 2)
                .map(|level| {
                    let yes_price = level[0];
                    let no_ask_price = 100 - yes_price;
                    OrderBookLevel::new(
                        Decimal::from(no_ask_price) / Decimal::from(100),
                        Decimal::from(level[1]),
                    )
                })
                .collect();
            order_book.no_asks.sort_by(|a, b| a.price.cmp(&b.price));
        }

        order_book
    }
}

// ============================================================================
// Trade History Types
// ============================================================================

/// Response from GET /markets/{ticker}/trades
#[derive(Debug, Clone, Deserialize)]
pub struct TradesResponse {
    pub trades: Vec<KalshiTrade>,
    #[serde(default)]
    pub cursor: Option<String>,
}

/// A single Kalshi trade
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KalshiTrade {
    /// Trade ID
    #[serde(default)]
    pub trade_id: Option<String>,

    /// Market ticker
    #[serde(default)]
    pub ticker: Option<String>,

    /// YES price in cents
    #[serde(default)]
    pub yes_price: Option<i64>,

    /// NO price in cents
    #[serde(default)]
    pub no_price: Option<i64>,

    /// Number of contracts traded
    #[serde(default)]
    pub count: Option<i64>,

    /// When the trade occurred
    #[serde(default)]
    pub created_time: Option<DateTime<Utc>>,

    /// Taker side ("yes" or "no")
    #[serde(default)]
    pub taker_side: Option<String>,
}

impl KalshiTrade {
    /// Convert to terminal-core Trade
    pub fn to_trade(&self, market_id: &str) -> terminal_core::Trade {
        use terminal_core::{Platform, Trade, TradeOutcome, TradeSide};
        use chrono::Utc;

        let (outcome, side) = match self.taker_side.as_deref() {
            Some("yes") => (TradeOutcome::Yes, Some(TradeSide::Buy)),
            Some("no") => (TradeOutcome::No, Some(TradeSide::Buy)),
            _ => (TradeOutcome::Yes, None),
        };

        let price = self.yes_price
            .map(|p| Decimal::from(p) / Decimal::from(100))
            .unwrap_or(Decimal::ZERO);

        Trade {
            id: self.trade_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            market_id: market_id.to_string(),
            platform: Platform::Kalshi,
            timestamp: self.created_time.unwrap_or_else(Utc::now),
            price,
            quantity: Decimal::from(self.count.unwrap_or(0)),
            outcome,
            side,
        }
    }
}

// ============================================================================
// Event Markets Response (for related markets)
// ============================================================================

/// Response from GET /events/{event_ticker}/markets
#[derive(Debug, Clone, Deserialize)]
pub struct EventMarketsResponse {
    pub markets: Vec<KalshiMarket>,
    #[serde(default)]
    pub cursor: Option<String>,
}
