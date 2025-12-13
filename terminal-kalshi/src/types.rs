//! Kalshi API response types
//!
//! These types mirror the Kalshi API responses and are converted
//! to terminal-core types for use in the application.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
/// This endpoint returns both the event AND its markets in a single response
#[derive(Debug, Clone, Deserialize)]
pub struct EventResponse {
    pub event: KalshiEvent,
    #[serde(default)]
    pub markets: Vec<KalshiMarket>,
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

    /// Primary resolution rules
    #[serde(default)]
    pub rules_primary: Option<String>,
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
        Self::extract_series_ticker_static(event_ticker)
    }

    /// Static version of extract_series_ticker for use outside impl
    pub fn extract_series_ticker_static(event_ticker: &str) -> &str {
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

    /// Extract series ticker from a full market ticker
    /// e.g., "KXTRILLION-25-MUSK" -> "KXTRILLION"
    /// e.g., "KXNEWPOPE-70-PPIZ" -> "KXNEWPOPE"
    pub fn extract_series_from_market_ticker(market_ticker: &str) -> &str {
        // Find the first hyphen followed by a digit
        for (i, c) in market_ticker.char_indices() {
            if c == '-' {
                if let Some(next_char) = market_ticker[i + 1..].chars().next() {
                    if next_char.is_ascii_digit() {
                        return &market_ticker[..i];
                    }
                }
            }
        }
        market_ticker
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
    fn parse_teams_from_title(title: &str) -> Option<(String, String)> {
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

        // Sports detection
        let is_sports = Self::is_sports_category(self.category.as_deref())
            || Self::parse_teams_from_title(&self.title).is_some();

        let (home_team, away_team, home_odds, away_odds) = if is_sports {
            if let Some((team_a, team_b)) = Self::parse_teams_from_title(&self.title) {
                let yes = self.yes_price();
                let no = self.no_price();
                (Some(team_a), Some(team_b), Some(yes), Some(no))
            } else {
                (None, None, None, None)
            }
        } else {
            (None, None, None, None)
        };

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
            resolution_source: self.rules_primary.clone(),
            // Sports fields
            is_sports,
            is_live: false, // Kalshi API doesn't provide live game data
            score: None,
            game_period: None,
            home_team,
            away_team,
            home_odds,
            away_odds,
            spread_line: None,
            total_line: None,
            // Kalshi doesn't have tags like Polymarket
            tags: Vec::new(),
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
            transaction_hash: None, // Kalshi doesn't have on-chain transactions
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

// ============================================================================
// Market Grouping Functions
// ============================================================================

/// Group markets by event_ticker and convert to multi-outcome PredictionMarkets
///
/// This function takes raw Kalshi markets and groups them by their event.
/// Events with multiple markets become multi-outcome cards, while
/// single-market events remain as binary cards.
pub fn group_markets_by_event(
    markets: Vec<KalshiMarket>,
    event_titles: &HashMap<String, String>,
) -> Vec<terminal_core::PredictionMarket> {
    let mut event_groups: HashMap<String, Vec<KalshiMarket>> = HashMap::new();
    let mut standalone: Vec<KalshiMarket> = Vec::new();

    // Group markets by event_ticker
    for market in markets {
        match &market.event_ticker {
            Some(et) => event_groups.entry(et.clone()).or_default().push(market),
            None => standalone.push(market),
        }
    }

    let mut result: Vec<terminal_core::PredictionMarket> = Vec::new();

    // Process grouped events
    for (event_ticker, group) in event_groups {
        if group.len() == 1 {
            // Single market in event = binary card
            result.push(group.into_iter().next().unwrap().to_prediction_market());
        } else {
            // Multiple markets = multi-outcome card
            let event_title = event_titles.get(&event_ticker);
            result.push(markets_to_multi_outcome(&event_ticker, group, event_title));
        }
    }

    // Add standalone markets (no event_ticker)
    for market in standalone {
        result.push(market.to_prediction_market());
    }

    result
}

/// Convert a group of related markets into a single multi-outcome PredictionMarket
pub fn markets_to_multi_outcome(
    event_ticker: &str,
    markets: Vec<KalshiMarket>,
    event_title: Option<&String>,
) -> terminal_core::PredictionMarket {
    use terminal_core::{MarketStatus, Platform, PredictionMarket};

    let first = &markets[0];

    // Build outcomes JSON with market-level titles (these are the specific options)
    // Frontend expects: name, market_id, yes_price, clob_token_id, condition_id
    let outcomes: Vec<serde_json::Value> = markets
        .iter()
        .map(|m| {
            let price = m.yes_price();
            serde_json::json!({
                "name": m.title.clone(),
                "market_id": m.ticker.clone(),
                "yes_price": price.to_string(),
                "clob_token_id": m.ticker.clone(),  // Kalshi uses ticker for orderbook/chart
                "condition_id": m.ticker.clone(),   // Kalshi uses ticker for trades
            })
        })
        .collect();

    // Find leading outcome (highest yes price)
    let leader = markets
        .iter()
        .max_by(|a, b| a.yes_price().cmp(&b.yes_price()))
        .unwrap();
    let leader_price = leader.yes_price();

    // Sum volume across all markets in the event
    let total_volume: i64 = markets
        .iter()
        .map(|m| m.volume.unwrap_or(0))
        .sum();

    // Use earliest close time from all markets
    let close_time = markets
        .iter()
        .filter_map(|m| m.close_time.or(m.expiration_time))
        .min();

    // Use earliest created time
    let created_at = markets
        .iter()
        .filter_map(|m| m.created_time.or(m.open_time))
        .min();

    // Status from first market (they should all be the same)
    let status = match first.status.as_deref() {
        Some("active") | Some("open") => MarketStatus::Open,
        Some("closed") => MarketStatus::Closed,
        Some("settled") | Some("finalized") => MarketStatus::Settled,
        _ => MarketStatus::Open,
    };

    // URL uses series ticker derived from event_ticker
    let series_ticker = KalshiMarket::extract_series_ticker_static(event_ticker);
    let url = Some(format!(
        "https://kalshi.com/markets/{}",
        series_ticker.to_lowercase()
    ));

    // Sports detection
    let is_sports = KalshiMarket::is_sports_category(first.category.as_deref());

    PredictionMarket {
        id: event_ticker.to_string(),
        platform: Platform::Kalshi,
        ticker: Some(event_ticker.to_string()),
        // Use event title if available, fallback to first market title
        title: event_title.cloned().unwrap_or_else(|| first.title.clone()),
        description: first.subtitle.clone(),
        category: first.category.clone(),
        yes_price: leader_price,
        no_price: Decimal::ONE - leader_price,
        volume: Decimal::from(total_volume),
        liquidity: None,
        close_time,
        created_at,
        status,
        image_url: first.image_url.clone(),
        url,
        // Multi-outcome fields
        is_multi_outcome: true,
        outcome_count: Some(markets.len()),
        leading_outcome: Some(leader.title.clone()),
        options_json: Some(serde_json::to_string(&outcomes).unwrap_or_default()),
        resolution_source: leader.rules_primary.clone(),
        // Sports fields
        is_sports,
        is_live: false,
        score: None,
        game_period: None,
        home_team: None,
        away_team: None,
        home_odds: None,
        away_odds: None,
        spread_line: None,
        total_line: None,
        // Kalshi doesn't have tags like Polymarket
        tags: Vec::new(),
    }
}

// ============================================================================
// Candlesticks Types (for price history)
// ============================================================================

/// Response from GET /series/{series}/markets/{ticker}/candlesticks
#[derive(Debug, Clone, Deserialize)]
pub struct CandlesticksResponse {
    pub candlesticks: Vec<KalshiCandlestick>,
}

/// A single candlestick from Kalshi API
#[derive(Debug, Clone, Deserialize)]
pub struct KalshiCandlestick {
    /// End timestamp for this period (Unix timestamp)
    pub end_period_ts: i64,
    /// YES ask OHLC data
    pub yes_ask: Option<CandlestickOHLC>,
    /// YES bid OHLC data
    pub yes_bid: Option<CandlestickOHLC>,
    /// Volume during this period
    #[serde(default)]
    pub volume: i64,
    /// Open interest
    #[serde(default)]
    pub open_interest: i64,
}

/// OHLC (Open/High/Low/Close) data for a candlestick
#[derive(Debug, Clone, Deserialize)]
pub struct CandlestickOHLC {
    /// Close price in cents (0-99)
    pub close: Option<i64>,
    /// High price in cents
    pub high: Option<i64>,
    /// Low price in cents
    pub low: Option<i64>,
    /// Open price in cents
    pub open: Option<i64>,
}

/// A single price point for charts (matches Polymarket format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistoryPoint {
    /// Unix timestamp in seconds
    pub t: i64,
    /// Price (0.0 - 1.0)
    pub p: f64,
}
