//! Kalshi API client
//!
//! Provides methods for interacting with the Kalshi REST API.

use crate::types::{
    BalanceResponse, EventMarketsResponse, EventResponse, EventsResponse, MarketResponse,
    MarketsResponse, OrderbookResponse, PositionsResponse, TradesResponse,
};
use reqwest::Client;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::time::Duration;
use terminal_core::{
    Balance, OrderBook, Platform, Position, PredictionMarket, TerminalError, TradeHistory,
};
use tracing::{debug, instrument, warn};

/// Base URL for Kalshi API
const KALSHI_API_BASE: &str = "https://api.elections.kalshi.com/trade-api/v2";
const KALSHI_DEMO_API_BASE: &str = "https://demo-api.kalshi.co/trade-api/v2";

/// Kalshi API client
#[derive(Clone)]
pub struct KalshiClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl KalshiClient {
    /// Create a new Kalshi client (unauthenticated, for public endpoints)
    pub fn new(use_demo: bool) -> Self {
        let base_url = if use_demo {
            KALSHI_DEMO_API_BASE
        } else {
            KALSHI_API_BASE
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.to_string(),
            api_key: None,
        }
    }

    /// Create a new authenticated Kalshi client
    pub fn with_auth(api_key: String, use_demo: bool) -> Self {
        let mut client = Self::new(use_demo);
        client.api_key = Some(api_key);
        client
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// List markets from Kalshi
    #[instrument(skip(self))]
    pub async fn list_markets(
        &self,
        status: Option<&str>,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<Vec<PredictionMarket>, TerminalError> {
        let mut url = format!("{}/markets", self.base_url);

        // Build query params
        let mut params = Vec::new();
        if let Some(s) = status {
            params.push(format!("status={}", s));
        }
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }
        if let Some(c) = cursor {
            params.push(format!("cursor={}", c));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        debug!("Fetching Kalshi markets from: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| TerminalError::network(format!("Failed to fetch markets: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TerminalError::api(format!(
                "Kalshi API error ({}): {}",
                status, body
            )));
        }

        let markets_response: MarketsResponse = response.json().await.map_err(|e| {
            TerminalError::parse(format!("Failed to parse markets response: {}", e))
        })?;

        let markets = markets_response
            .markets
            .into_iter()
            .map(|m| m.to_prediction_market())
            .collect();

        Ok(markets)
    }

    /// Get a single market by ticker or event_ticker
    ///
    /// This method first tries to fetch as a market ticker. If that fails with 404,
    /// it tries to fetch as an event ticker (for grouped multi-outcome markets).
    #[instrument(skip(self))]
    pub async fn get_market(&self, ticker: &str) -> Result<PredictionMarket, TerminalError> {
        debug!("Fetching Kalshi market: {}", ticker);

        // First, try to fetch as a market ticker
        let url = format!("{}/markets/{}", self.base_url, ticker);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| TerminalError::network(format!("Failed to fetch market: {}", e)))?;

        let status_code = response.status().as_u16();

        if response.status().is_success() {
            let market_response: MarketResponse = response.json().await.map_err(|e| {
                TerminalError::parse(format!("Failed to parse market response: {}", e))
            })?;
            return Ok(market_response.market.to_prediction_market());
        }

        // If 404, try to fetch as an event ticker (for grouped markets)
        if status_code == 404 {
            debug!("Market {} not found, trying as event ticker", ticker);
            return self.get_event_as_market(ticker).await;
        }

        // Other error
        let body = response.text().await.unwrap_or_default();
        Err(TerminalError::api(format!(
            "Kalshi API error ({}): {}",
            status_code, body
        )))
    }

    /// Fetch an event and its markets, returning a grouped multi-outcome PredictionMarket
    async fn get_event_as_market(
        &self,
        event_ticker: &str,
    ) -> Result<PredictionMarket, TerminalError> {
        use crate::types::markets_to_multi_outcome;

        debug!("Fetching Kalshi event: {}", event_ticker);

        // Fetch the event (includes markets in the response)
        let event_url = format!("{}/events/{}", self.base_url, event_ticker);

        let event_response = self
            .client
            .get(&event_url)
            .send()
            .await
            .map_err(|e| TerminalError::network(format!("Failed to fetch event: {}", e)))?;

        let event_status = event_response.status();

        if event_status.as_u16() == 404 {
            return Err(TerminalError::not_found(format!(
                "Neither market nor event found: {}",
                event_ticker
            )));
        }

        if !event_status.is_success() {
            let body = event_response.text().await.unwrap_or_default();
            return Err(TerminalError::api(format!(
                "Kalshi API error ({}): {}",
                event_status, body
            )));
        }

        let event_data: EventResponse = event_response
            .json()
            .await
            .map_err(|e| TerminalError::parse(format!("Failed to parse event response: {}", e)))?;

        let event_title = event_data.event.title.clone();
        let markets = event_data.markets;

        debug!("Event {} has {} markets", event_ticker, markets.len());

        if markets.is_empty() {
            return Err(TerminalError::not_found(format!(
                "Event {} has no markets",
                event_ticker
            )));
        }

        // If only one market, return it directly
        if markets.len() == 1 {
            return Ok(markets.into_iter().next().unwrap().to_prediction_market());
        }

        // Multiple markets - build a multi-outcome market
        Ok(markets_to_multi_outcome(
            event_ticker,
            markets,
            event_title.as_ref(),
        ))
    }

    /// Fetch events and build a lookup of event_ticker -> series_ticker
    #[instrument(skip(self))]
    pub async fn fetch_series_lookup(&self) -> Result<HashMap<String, String>, TerminalError> {
        let mut lookup = HashMap::new();
        let mut cursor: Option<String> = None;
        let limit = 200u32;

        loop {
            let url = {
                let mut url = format!("{}/events?limit={}", self.base_url, limit);
                if let Some(ref c) = cursor {
                    url.push_str(&format!("&cursor={}", c));
                }
                url
            };

            debug!(
                "Fetching Kalshi events for series lookup, cursor: {:?}",
                cursor
            );

            let response =
                self.client.get(&url).send().await.map_err(|e| {
                    TerminalError::network(format!("Failed to fetch events: {}", e))
                })?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(TerminalError::api(format!(
                    "Kalshi API error ({}): {}",
                    status, body
                )));
            }

            let events_response: EventsResponse = response
                .json()
                .await
                .map_err(|e| TerminalError::parse(format!("Failed to parse events: {}", e)))?;

            for event in events_response.events {
                if let Some(series_ticker) = event.series_ticker {
                    lookup.insert(event.event_ticker, series_ticker);
                }
            }

            // Check for more pages
            match events_response.cursor {
                Some(c) if !c.is_empty() => cursor = Some(c),
                _ => break,
            }
        }

        debug!("Built series lookup with {} entries", lookup.len());
        Ok(lookup)
    }

    /// List all markets (handles pagination)
    /// Fetches more markets than requested to sort by volume client-side,
    /// since Kalshi API doesn't support server-side volume sorting
    #[instrument(skip(self))]
    pub async fn list_all_markets(
        &self,
        status: Option<&str>,
        max_markets: Option<usize>,
    ) -> Result<Vec<PredictionMarket>, TerminalError> {
        let mut all_markets = Vec::new();
        let mut cursor: Option<String> = None;
        let limit = 100u32;
        let requested = max_markets.unwrap_or(100);
        // Fetch more markets to get better volume coverage, then sort and truncate
        // Fetch up to 5x requested or 500, whichever is larger
        let fetch_limit = std::cmp::max(requested * 5, 500);

        loop {
            let url = {
                let mut url = format!("{}/markets?limit={}", self.base_url, limit);
                if let Some(s) = status {
                    url.push_str(&format!("&status={}", s));
                }
                if let Some(ref c) = cursor {
                    url.push_str(&format!("&cursor={}", c));
                }
                url
            };

            debug!("Fetching Kalshi markets page, cursor: {:?}", cursor);

            let response =
                self.client.get(&url).send().await.map_err(|e| {
                    TerminalError::network(format!("Failed to fetch markets: {}", e))
                })?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(TerminalError::api(format!(
                    "Kalshi API error ({}): {}",
                    status, body
                )));
            }

            let markets_response: MarketsResponse = response
                .json()
                .await
                .map_err(|e| TerminalError::parse(format!("Failed to parse markets: {}", e)))?;

            let markets: Vec<PredictionMarket> = markets_response
                .markets
                .into_iter()
                .map(|m| m.to_prediction_market())
                .collect();

            all_markets.extend(markets);

            // Check if we should stop fetching
            if all_markets.len() >= fetch_limit {
                break;
            }

            // Check for more pages
            match markets_response.cursor {
                Some(c) if !c.is_empty() => cursor = Some(c),
                _ => break,
            }
        }

        debug!(
            "Fetched {} Kalshi markets, sorting by volume",
            all_markets.len()
        );

        // Sort by volume descending (highest volume first)
        all_markets.sort_by(|a, b| b.volume.cmp(&a.volume));

        // Truncate to requested limit
        all_markets.truncate(requested);

        debug!(
            "Returning top {} Kalshi markets by volume",
            all_markets.len()
        );
        Ok(all_markets)
    }

    /// List markets grouped by event, with proper event titles
    ///
    /// This fetches both events (for canonical titles) and markets,
    /// then groups markets by event_ticker. Events with multiple markets
    /// become multi-outcome cards with the event title.
    #[instrument(skip(self))]
    pub async fn list_markets_grouped(
        &self,
        status: Option<&str>,
        max_markets: Option<usize>,
    ) -> Result<Vec<PredictionMarket>, TerminalError> {
        use crate::types::group_markets_by_event;

        // 1. Fetch events to build title lookup (event_ticker -> title)
        let event_titles = self.fetch_event_titles(status).await?;

        // 2. Fetch raw markets (not converted to PredictionMarket yet)
        let raw_markets = self.fetch_raw_markets(status, max_markets).await?;
        let raw_count = raw_markets.len();

        // 3. Group markets by event and convert
        let grouped = group_markets_by_event(raw_markets, &event_titles);

        debug!(
            "Grouped {} raw markets into {} cards",
            raw_count,
            grouped.len()
        );

        // 4. Sort by volume descending
        let mut result = grouped;
        result.sort_by(|a, b| b.volume.cmp(&a.volume));

        // 5. Truncate to requested limit
        let limit = max_markets.unwrap_or(100);
        result.truncate(limit);

        Ok(result)
    }

    /// Fetch event titles from Kalshi events endpoint
    ///
    /// Only fetches a single page to avoid rate limiting.
    /// Falls back to market titles for events not in this page.
    async fn fetch_event_titles(
        &self,
        status: Option<&str>,
    ) -> Result<HashMap<String, String>, TerminalError> {
        use crate::types::EventsResponse;

        let mut titles = HashMap::new();
        // Only fetch one page of 200 events to avoid rate limiting
        // The grouping logic falls back to market titles if event title not found
        let limit = 200u32;

        let url = {
            let mut url = format!("{}/events?limit={}", self.base_url, limit);
            if let Some(s) = status {
                url.push_str(&format!("&status={}", s));
            }
            url
        };

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| TerminalError::network(format!("Failed to fetch events: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            // Don't fail completely - just return empty map and use market titles
            warn!("Failed to fetch events for titles ({}): {}", status, body);
            return Ok(titles);
        }

        let events_response: EventsResponse = response
            .json()
            .await
            .map_err(|e| TerminalError::parse(format!("Failed to parse events: {}", e)))?;

        for event in events_response.events {
            if let Some(title) = event.title {
                titles.insert(event.event_ticker, title);
            }
        }

        Ok(titles)
    }

    /// Fetch raw KalshiMarket objects (not converted to PredictionMarket)
    ///
    /// Limits pagination to avoid rate limiting - max 2 pages (200 markets)
    async fn fetch_raw_markets(
        &self,
        status: Option<&str>,
        max_markets: Option<usize>,
    ) -> Result<Vec<crate::types::KalshiMarket>, TerminalError> {
        use crate::types::MarketsResponse;

        let mut all_markets = Vec::new();
        let mut cursor: Option<String> = None;
        let limit = 100u32;
        let requested = max_markets.unwrap_or(100);
        // Limit to 2 pages max (200 markets) to avoid rate limiting
        // After grouping, this should give us plenty of unique events
        let fetch_limit = std::cmp::min(requested * 2, 200);
        let max_pages = 2;
        let mut pages_fetched = 0;

        loop {
            let url = {
                let mut url = format!("{}/markets?limit={}", self.base_url, limit);
                if let Some(s) = status {
                    url.push_str(&format!("&status={}", s));
                }
                if let Some(ref c) = cursor {
                    url.push_str(&format!("&cursor={}", c));
                }
                url
            };

            let response =
                self.client.get(&url).send().await.map_err(|e| {
                    TerminalError::network(format!("Failed to fetch markets: {}", e))
                })?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(TerminalError::api(format!(
                    "Kalshi API error ({}): {}",
                    status, body
                )));
            }

            let markets_response: MarketsResponse = response
                .json()
                .await
                .map_err(|e| TerminalError::parse(format!("Failed to parse markets: {}", e)))?;

            all_markets.extend(markets_response.markets);
            pages_fetched += 1;

            // Check if we should stop fetching
            if all_markets.len() >= fetch_limit || pages_fetched >= max_pages {
                break;
            }

            // Check for more pages
            match markets_response.cursor {
                Some(c) if !c.is_empty() => cursor = Some(c),
                _ => break,
            }
        }

        Ok(all_markets)
    }

    // ========================================================================
    // Order Book Methods
    // ========================================================================

    /// Get the order book for a market
    #[instrument(skip(self))]
    pub async fn get_orderbook(&self, ticker: &str) -> Result<OrderBook, TerminalError> {
        let url = format!("{}/markets/{}/orderbook", self.base_url, ticker);

        debug!("Fetching Kalshi orderbook for: {}", ticker);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| TerminalError::network(format!("Failed to fetch orderbook: {}", e)))?;

        if response.status().as_u16() == 404 {
            return Err(TerminalError::not_found(format!(
                "Market not found: {}",
                ticker
            )));
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TerminalError::api(format!(
                "Kalshi API error ({}): {}",
                status, body
            )));
        }

        let orderbook_response: OrderbookResponse = response
            .json()
            .await
            .map_err(|e| TerminalError::parse(format!("Failed to parse orderbook: {}", e)))?;

        Ok(orderbook_response.orderbook.to_order_book(ticker))
    }

    // ========================================================================
    // Trade History Methods
    // ========================================================================

    /// Get recent trades for a market
    #[instrument(skip(self))]
    pub async fn get_trades(
        &self,
        ticker: &str,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<TradeHistory, TerminalError> {
        // Kalshi uses query param for ticker, not path param
        let mut url = format!("{}/markets/trades?ticker={}", self.base_url, ticker);

        // Add additional query params
        if let Some(l) = limit {
            url.push_str(&format!("&limit={}", l));
        }
        if let Some(c) = cursor {
            url.push_str(&format!("&cursor={}", c));
        }

        debug!("Fetching Kalshi trades for: {}", ticker);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| TerminalError::network(format!("Failed to fetch trades: {}", e)))?;

        if response.status().as_u16() == 404 {
            return Err(TerminalError::not_found(format!(
                "Market not found: {}",
                ticker
            )));
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TerminalError::api(format!(
                "Kalshi API error ({}): {}",
                status, body
            )));
        }

        let trades_response: TradesResponse = response
            .json()
            .await
            .map_err(|e| TerminalError::parse(format!("Failed to parse trades: {}", e)))?;

        let trades = trades_response
            .trades
            .into_iter()
            .map(|t| t.to_trade(ticker))
            .collect();

        Ok(TradeHistory {
            market_id: ticker.to_string(),
            platform: Platform::Kalshi,
            trades,
            next_cursor: trades_response.cursor,
        })
    }

    // ========================================================================
    // Related Markets Methods
    // ========================================================================

    /// Get markets related to the same event
    #[instrument(skip(self))]
    pub async fn get_related_markets(
        &self,
        event_ticker: &str,
    ) -> Result<Vec<PredictionMarket>, TerminalError> {
        let url = format!("{}/events/{}/markets", self.base_url, event_ticker);

        debug!("Fetching related markets for event: {}", event_ticker);

        let response =
            self.client.get(&url).send().await.map_err(|e| {
                TerminalError::network(format!("Failed to fetch event markets: {}", e))
            })?;

        if response.status().as_u16() == 404 {
            return Err(TerminalError::not_found(format!(
                "Event not found: {}",
                event_ticker
            )));
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TerminalError::api(format!(
                "Kalshi API error ({}): {}",
                status, body
            )));
        }

        let markets_response: EventMarketsResponse = response
            .json()
            .await
            .map_err(|e| TerminalError::parse(format!("Failed to parse event markets: {}", e)))?;

        let markets = markets_response
            .markets
            .into_iter()
            .map(|m| m.to_prediction_market())
            .collect();

        Ok(markets)
    }

    // ========================================================================
    // Authenticated Methods (Portfolio)
    // ========================================================================

    /// Check if the client is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.api_key.is_some()
    }

    /// Helper to ensure authentication
    fn require_auth(&self) -> Result<&str, TerminalError> {
        self.api_key
            .as_deref()
            .ok_or_else(|| TerminalError::auth("Kalshi authentication required"))
    }

    /// Get the user's balance (requires authentication)
    #[instrument(skip(self))]
    pub async fn get_balance(&self) -> Result<Balance, TerminalError> {
        let token = self.require_auth()?;
        let url = format!("{}/portfolio/balance", self.base_url);

        debug!("Fetching Kalshi portfolio balance");

        let response = self
            .client
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| TerminalError::network(format!("Failed to fetch balance: {}", e)))?;

        if response.status().as_u16() == 401 {
            return Err(TerminalError::auth("Invalid or expired Kalshi API key"));
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TerminalError::api(format!(
                "Kalshi API error ({}): {}",
                status, body
            )));
        }

        let balance_response: BalanceResponse = response
            .json()
            .await
            .map_err(|e| TerminalError::parse(format!("Failed to parse balance: {}", e)))?;

        // Convert cents to dollars
        let balance_usd = Decimal::from(balance_response.balance) / Decimal::from(100);

        Ok(Balance::new(
            Platform::Kalshi,
            balance_usd,
            Decimal::ZERO,
            "USD",
        ))
    }

    /// Get the user's positions (requires authentication)
    #[instrument(skip(self))]
    pub async fn get_positions(&self) -> Result<Vec<Position>, TerminalError> {
        use terminal_core::position::Outcome;

        let token = self.require_auth()?;
        let url = format!("{}/portfolio/positions", self.base_url);

        debug!("Fetching Kalshi portfolio positions");

        let response = self
            .client
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| TerminalError::network(format!("Failed to fetch positions: {}", e)))?;

        if response.status().as_u16() == 401 {
            return Err(TerminalError::auth("Invalid or expired Kalshi API key"));
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TerminalError::api(format!(
                "Kalshi API error ({}): {}",
                status, body
            )));
        }

        let positions_response: PositionsResponse = response
            .json()
            .await
            .map_err(|e| TerminalError::parse(format!("Failed to parse positions: {}", e)))?;

        // Convert Kalshi positions to terminal-core positions
        let positions: Vec<Position> = positions_response
            .market_positions
            .into_iter()
            .filter(|p| p.position != 0)
            .map(|p| {
                let (outcome, quantity) = if p.position > 0 {
                    (Outcome::Yes, Decimal::from(p.position))
                } else {
                    (Outcome::No, Decimal::from(-p.position))
                };

                Position {
                    market_id: p.ticker.clone(),
                    market_title: p.ticker.clone(), // Would need market lookup for title
                    platform: Platform::Kalshi,
                    outcome,
                    quantity,
                    avg_price: Decimal::ZERO, // Not provided in this endpoint
                    current_price: Decimal::ZERO,
                    unrealized_pnl: Decimal::from(p.realized_pnl) / Decimal::from(100),
                }
            })
            .collect();

        Ok(positions)
    }
}

impl std::fmt::Debug for KalshiClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KalshiClient")
            .field("base_url", &self.base_url)
            .field("authenticated", &self.api_key.is_some())
            .finish()
    }
}
