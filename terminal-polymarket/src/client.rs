//! Polymarket API client
//!
//! Provides methods for interacting with the Polymarket Gamma API
//! for market data retrieval.

use crate::types::{
    ClobOrderbookResponse, DataApiTrade, PolymarketEvent, PolymarketMarket, PriceHistoryPoint,
    PricesHistoryResponse, CLOB_API_BASE, DATA_API_BASE,
};
use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE},
    Engine as _,
};
use hmac::{Hmac, Mac};
use reqwest::Client;
use sha2::Sha256;
use std::time::Duration;
use terminal_core::{OrderBook, PredictionMarket, TerminalError, TradeHistory};
use tracing::{debug, instrument};

/// Base URL for Polymarket Gamma API
const GAMMA_API_BASE: &str = "https://gamma-api.polymarket.com";

/// API credentials for authenticated CLOB requests
#[derive(Clone, Debug)]
pub struct PolymarketCredentials {
    pub api_key: String,
    pub api_secret: String,
    pub passphrase: String,
}

impl PolymarketCredentials {
    /// Create new credentials
    pub fn new(api_key: String, api_secret: String, passphrase: String) -> Self {
        Self {
            api_key,
            api_secret,
            passphrase,
        }
    }

    /// Create credentials from environment variables
    pub fn from_env() -> Option<Self> {
        let api_key = std::env::var("POLY_API_KEY").ok()?;
        let api_secret = std::env::var("POLY_SECRET").ok()?;
        let passphrase = std::env::var("POLY_PASSPHRASE").ok()?;

        Some(Self {
            api_key,
            api_secret,
            passphrase,
        })
    }

    /// Build HMAC signature for a request
    ///
    /// Message format: {timestamp}{method}{path}{body}
    /// - Secret: base64 decoded (standard or URL-safe)
    /// - Algorithm: HMAC-SHA256
    /// - Output: base64 URL-safe encoded
    pub fn build_signature(
        &self,
        timestamp: &str,
        method: &str,
        path: &str,
        body: &str,
    ) -> Result<String, TerminalError> {
        // Decode the secret from base64 (try standard first, then URL-safe)
        let secret_bytes = STANDARD
            .decode(&self.api_secret)
            .or_else(|_| URL_SAFE.decode(&self.api_secret))
            .map_err(|e| TerminalError::auth(format!("Failed to decode API secret: {}", e)))?;

        // Build the message: timestamp + method + path + body
        let message = format!("{}{}{}{}", timestamp, method, path, body);

        tracing::debug!(
            "HMAC message: timestamp={}, method={}, path={}, body_len={}",
            timestamp,
            method,
            path,
            body.len()
        );

        // Create HMAC-SHA256
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(&secret_bytes)
            .map_err(|e| TerminalError::auth(format!("Failed to create HMAC: {}", e)))?;

        mac.update(message.as_bytes());

        // Get the result and base64 URL-safe encode it
        let result = mac.finalize();
        let signature = URL_SAFE.encode(result.into_bytes());

        Ok(signature)
    }
}

/// Polymarket API client
#[derive(Clone)]
pub struct PolymarketClient {
    client: Client,
    base_url: String,
    clob_url: String,
    data_api_url: String,
    credentials: Option<PolymarketCredentials>,
}

impl PolymarketClient {
    /// Create a new Polymarket client without authentication
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        // Try to load credentials from environment (kept for future auth features)
        let credentials = PolymarketCredentials::from_env();
        if let Some(ref creds) = credentials {
            tracing::info!(
                "Loaded Polymarket credentials from environment (api_key: {}...)",
                &creds.api_key[..8]
            );
        }

        Self {
            client,
            base_url: GAMMA_API_BASE.to_string(),
            clob_url: CLOB_API_BASE.to_string(),
            data_api_url: DATA_API_BASE.to_string(),
            credentials,
        }
    }

    /// Create a new Polymarket client with explicit credentials
    pub fn with_credentials(credentials: PolymarketCredentials) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: GAMMA_API_BASE.to_string(),
            clob_url: CLOB_API_BASE.to_string(),
            data_api_url: DATA_API_BASE.to_string(),
            credentials: Some(credentials),
        }
    }

    /// Check if the client has authentication credentials
    pub fn has_credentials(&self) -> bool {
        self.credentials.is_some()
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// List markets from Polymarket
    #[instrument(skip(self))]
    pub async fn list_markets(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        active_only: bool,
    ) -> Result<Vec<PredictionMarket>, TerminalError> {
        let mut url = format!("{}/markets", self.base_url);

        // Build query params
        let mut params = Vec::new();
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }
        if let Some(o) = offset {
            params.push(format!("offset={}", o));
        }
        if active_only {
            params.push("active=true".to_string());
            params.push("closed=false".to_string());
        }
        // Order by volume for most relevant markets
        params.push("order=volume".to_string());
        params.push("ascending=false".to_string());

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        debug!("Fetching Polymarket markets from: {}", url);

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
                "Polymarket API error ({}): {}",
                status, body
            )));
        }

        let markets: Vec<PolymarketMarket> = response.json().await.map_err(|e| {
            TerminalError::parse(format!("Failed to parse markets response: {}", e))
        })?;

        let prediction_markets = markets
            .into_iter()
            .map(|m| m.to_prediction_market())
            .collect();

        Ok(prediction_markets)
    }

    /// List events from Polymarket (events contain grouped markets)
    #[instrument(skip(self))]
    pub async fn list_events(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        active_only: bool,
    ) -> Result<Vec<PolymarketEvent>, TerminalError> {
        let mut url = format!("{}/events", self.base_url);

        // Build query params
        let mut params = Vec::new();
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }
        if let Some(o) = offset {
            params.push(format!("offset={}", o));
        }
        if active_only {
            params.push("closed=false".to_string());
        }
        // Order by volume descending for highest volume first
        params.push("order=volume".to_string());
        params.push("ascending=false".to_string());

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        debug!("Fetching Polymarket events from: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| TerminalError::network(format!("Failed to fetch events: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TerminalError::api(format!(
                "Polymarket API error ({}): {}",
                status, body
            )));
        }

        let events: Vec<PolymarketEvent> = response
            .json()
            .await
            .map_err(|e| TerminalError::parse(format!("Failed to parse events response: {}", e)))?;

        Ok(events)
    }

    /// Get a single market by ID
    ///
    /// Tries both /markets and /events endpoints since multi-outcome events
    /// use event IDs which are only fetchable via /events
    #[instrument(skip(self))]
    pub async fn get_market(&self, id: &str) -> Result<PredictionMarket, TerminalError> {
        // First try /markets endpoint (for individual market IDs)
        let url = format!("{}/markets?id={}", self.base_url, id);
        debug!("Fetching Polymarket market: {}", id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| TerminalError::network(format!("Failed to fetch market: {}", e)))?;

        if response.status().is_success() {
            let markets: Vec<PolymarketMarket> = response.json().await.map_err(|e| {
                TerminalError::parse(format!("Failed to parse market response: {}", e))
            })?;

            if let Some(market) = markets.into_iter().next() {
                return Ok(market.to_prediction_market());
            }
        }

        // If not found in /markets, try /events endpoint (for multi-outcome event IDs)
        debug!("Market not found in /markets, trying /events for: {}", id);
        let events_url = format!("{}/events?id={}", self.base_url, id);

        let response = self
            .client
            .get(&events_url)
            .send()
            .await
            .map_err(|e| TerminalError::network(format!("Failed to fetch event: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TerminalError::api(format!(
                "Polymarket API error ({}): {}",
                status, body
            )));
        }

        let events: Vec<PolymarketEvent> = response
            .json()
            .await
            .map_err(|e| TerminalError::parse(format!("Failed to parse event response: {}", e)))?;

        events
            .into_iter()
            .next()
            .map(|e| e.to_prediction_market())
            .ok_or_else(|| TerminalError::not_found(format!("Market not found: {}", id)))
    }

    /// List all markets with pagination (uses /markets endpoint - individual options)
    #[instrument(skip(self))]
    pub async fn list_all_markets(
        &self,
        active_only: bool,
        max_markets: Option<usize>,
    ) -> Result<Vec<PredictionMarket>, TerminalError> {
        let mut all_markets = Vec::new();
        let mut offset = 0u32;
        let limit = 100u32;
        let max = max_markets.unwrap_or(500);

        loop {
            debug!("Fetching Polymarket markets page, offset: {}", offset);

            let markets = self
                .list_markets(Some(limit), Some(offset), active_only)
                .await?;

            if markets.is_empty() {
                break;
            }

            all_markets.extend(markets);

            if all_markets.len() >= max {
                all_markets.truncate(max);
                break;
            }

            offset += limit;
        }

        debug!("Fetched {} Polymarket markets total", all_markets.len());
        Ok(all_markets)
    }

    /// List all events with pagination (uses /events endpoint - proper event grouping)
    /// This is the preferred method as it returns events with their markets grouped
    #[instrument(skip(self))]
    pub async fn list_all_events(
        &self,
        active_only: bool,
        max_events: Option<usize>,
    ) -> Result<Vec<PredictionMarket>, TerminalError> {
        let mut all_events = Vec::new();
        let mut offset = 0u32;
        let limit = 100u32;
        let max = max_events.unwrap_or(500);

        loop {
            debug!("Fetching Polymarket events page, offset: {}", offset);

            let events = self
                .list_events(Some(limit), Some(offset), active_only)
                .await?;

            if events.is_empty() {
                break;
            }

            // Convert events to PredictionMarkets
            let markets: Vec<PredictionMarket> = events
                .into_iter()
                .filter(|e| !e.markets.is_empty()) // Skip events with no markets
                .map(|e| e.to_prediction_market())
                .collect();

            all_events.extend(markets);

            if all_events.len() >= max {
                all_events.truncate(max);
                break;
            }

            offset += limit;
        }

        debug!("Fetched {} Polymarket events total", all_events.len());
        Ok(all_events)
    }

    // ========================================================================
    // CLOB API Methods (Order Book, Trades)
    // ========================================================================

    /// Get the order book for a token from the CLOB API
    ///
    /// # Arguments
    /// * `token_id` - The CLOB token ID (YES or NO token)
    /// * `is_yes_token` - Whether this is the YES token (affects how we populate the order book)
    #[instrument(skip(self))]
    pub async fn get_orderbook(
        &self,
        token_id: &str,
        is_yes_token: bool,
    ) -> Result<OrderBook, TerminalError> {
        let url = format!("{}/book?token_id={}", self.clob_url, token_id);

        debug!("Fetching Polymarket orderbook from: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| TerminalError::network(format!("Failed to fetch orderbook: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TerminalError::api(format!(
                "CLOB API error ({}): {}",
                status, body
            )));
        }

        let clob_book: ClobOrderbookResponse = response.json().await.map_err(|e| {
            TerminalError::parse(format!("Failed to parse orderbook response: {}", e))
        })?;

        Ok(clob_book.to_order_book(token_id, is_yes_token))
    }

    /// Get the condition ID for a market (binary events only)
    ///
    /// For binary markets, this returns the conditionId from the first market.
    /// Multi-outcome markets have multiple condition IDs (one per outcome).
    #[instrument(skip(self))]
    pub async fn get_condition_id(&self, event_id: &str) -> Result<String, TerminalError> {
        let event = self.get_event_by_id(event_id).await?;

        // Get the first market's condition ID
        event
            .markets
            .first()
            .and_then(|m| m.condition_id.clone())
            .ok_or_else(|| {
                TerminalError::not_found(format!("No condition ID found for event {}", event_id))
            })
    }

    /// Get recent trades for an event from the public data API
    ///
    /// # Arguments
    /// * `event_id` - The event ID (NOT token ID - this is the numeric event ID from Gamma API)
    /// * `limit` - Maximum number of trades to return (default 50)
    ///
    /// Note: This uses the public data API which doesn't require authentication.
    /// The API requires a condition ID (0x...) to filter by market, so we look it up first.
    #[instrument(skip(self))]
    pub async fn get_trades(
        &self,
        event_id: &str,
        limit: Option<u32>,
    ) -> Result<TradeHistory, TerminalError> {
        use terminal_core::Platform;

        // First, look up the condition ID for this event
        let condition_id = self.get_condition_id(event_id).await?;

        let limit = limit.unwrap_or(50);
        // Use market={conditionId} to filter trades by this specific market
        let url = format!(
            "{}/trades?market={}&limit={}",
            self.data_api_url, condition_id, limit
        );

        debug!("Fetching Polymarket trades from public API: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| TerminalError::network(format!("Failed to fetch trades: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TerminalError::api(format!(
                "Data API error ({}): {}",
                status, body
            )));
        }

        // The public data API returns an array directly, not wrapped in { data: [...] }
        let trades: Vec<DataApiTrade> = response
            .json()
            .await
            .map_err(|e| TerminalError::parse(format!("Failed to parse trades response: {}", e)))?;

        let trades = trades.into_iter().map(|t| t.to_trade(event_id)).collect();

        Ok(TradeHistory {
            market_id: event_id.to_string(),
            platform: Platform::Polymarket,
            trades,
            next_cursor: None, // Public API doesn't have cursor pagination
        })
    }

    /// Get an event by ID (returns raw event data with markets)
    #[instrument(skip(self))]
    pub async fn get_event_by_id(&self, event_id: &str) -> Result<PolymarketEvent, TerminalError> {
        let url = format!("{}/events?id={}", self.base_url, event_id);

        debug!("Fetching Polymarket event by ID: {}", event_id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| TerminalError::network(format!("Failed to fetch event: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TerminalError::api(format!(
                "Polymarket API error ({}): {}",
                status, body
            )));
        }

        let events: Vec<PolymarketEvent> = response
            .json()
            .await
            .map_err(|e| TerminalError::parse(format!("Failed to parse event response: {}", e)))?;

        events
            .into_iter()
            .next()
            .ok_or_else(|| TerminalError::not_found(format!("Event not found: {}", event_id)))
    }

    /// Get the CLOB token ID for an event (for binary events only)
    ///
    /// Returns the YES token ID for the first market in the event.
    /// This is used to fetch orderbook/trades from the CLOB API.
    #[instrument(skip(self))]
    pub async fn get_clob_token_id(&self, event_id: &str) -> Result<String, TerminalError> {
        let event = self.get_event_by_id(event_id).await?;

        // Get the first market's clob_token_ids
        let market = event.markets.first().ok_or_else(|| {
            TerminalError::not_found(format!("Event {} has no markets", event_id))
        })?;

        market.yes_token_id().ok_or_else(|| {
            TerminalError::not_found(format!("Market {} has no CLOB token ID", market.id))
        })
    }

    /// Get an event by slug (for related markets)
    #[instrument(skip(self))]
    pub async fn get_event(&self, slug: &str) -> Result<PolymarketEvent, TerminalError> {
        let url = format!("{}/events?slug={}", self.base_url, slug);

        debug!("Fetching Polymarket event: {}", slug);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| TerminalError::network(format!("Failed to fetch event: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TerminalError::api(format!(
                "Polymarket API error ({}): {}",
                status, body
            )));
        }

        let events: Vec<PolymarketEvent> = response
            .json()
            .await
            .map_err(|e| TerminalError::parse(format!("Failed to parse event response: {}", e)))?;

        events
            .into_iter()
            .next()
            .ok_or_else(|| TerminalError::not_found(format!("Event not found: {}", slug)))
    }

    /// Get related markets from the same event
    ///
    /// # Arguments
    /// * `event_slug` - The event slug (extracted from market URL)
    #[instrument(skip(self))]
    pub async fn get_related_markets(
        &self,
        event_slug: &str,
    ) -> Result<Vec<PredictionMarket>, TerminalError> {
        let event = self.get_event(event_slug).await?;

        // Convert all markets in the event to PredictionMarkets
        let markets = event
            .markets
            .iter()
            .map(|m| m.to_prediction_market())
            .collect();

        Ok(markets)
    }

    // ========================================================================
    // Price History API Methods
    // ========================================================================

    /// Get price history for a token from the CLOB API
    ///
    /// # Arguments
    /// * `token_id` - The CLOB token ID (YES token)
    /// * `interval` - Duration string: "1m", "1h", "6h", "1d", "1w", "max"
    /// * `fidelity_override` - Optional fidelity in minutes (lower = more data points)
    #[instrument(skip(self))]
    pub async fn get_prices_history(
        &self,
        token_id: &str,
        interval: &str,
        fidelity_override: Option<u32>,
    ) -> Result<Vec<PriceHistoryPoint>, TerminalError> {
        // Use override if provided, otherwise map interval to default fidelity
        // Fidelity controls data granularity - lower = more data points
        let fidelity = fidelity_override.unwrap_or_else(|| match interval {
            "1m" => 1,
            "1h" => 1,
            "6h" => 5,
            "1d" => 15,
            "1w" => 60,
            "max" => 1440, // 1 day in minutes
            _ => 60,       // default to hourly
        });

        let url = format!(
            "{}/prices-history?market={}&interval={}&fidelity={}",
            self.clob_url, token_id, interval, fidelity
        );

        debug!("Fetching Polymarket price history from: {}", url);

        let response =
            self.client.get(&url).send().await.map_err(|e| {
                TerminalError::network(format!("Failed to fetch price history: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TerminalError::api(format!(
                "CLOB API error ({}): {}",
                status, body
            )));
        }

        let prices_response: PricesHistoryResponse = response.json().await.map_err(|e| {
            TerminalError::parse(format!("Failed to parse price history response: {}", e))
        })?;

        Ok(prices_response.history)
    }

    /// Get trades for a specific outcome within an event
    ///
    /// # Arguments
    /// * `condition_id` - The condition ID for the outcome
    /// * `limit` - Maximum number of trades to return (default 50)
    #[instrument(skip(self))]
    pub async fn get_outcome_trades(
        &self,
        condition_id: &str,
        limit: Option<u32>,
    ) -> Result<TradeHistory, TerminalError> {
        use terminal_core::Platform;

        let limit = limit.unwrap_or(50);
        let url = format!(
            "{}/trades?market={}&limit={}",
            self.data_api_url, condition_id, limit
        );

        debug!("Fetching Polymarket outcome trades from: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| TerminalError::network(format!("Failed to fetch trades: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TerminalError::api(format!(
                "Data API error ({}): {}",
                status, body
            )));
        }

        let trades: Vec<DataApiTrade> = response
            .json()
            .await
            .map_err(|e| TerminalError::parse(format!("Failed to parse trades response: {}", e)))?;

        let trades = trades
            .into_iter()
            .map(|t| t.to_trade(condition_id))
            .collect();

        Ok(TradeHistory {
            market_id: condition_id.to_string(),
            platform: Platform::Polymarket,
            trades,
            next_cursor: None,
        })
    }
}

impl Default for PolymarketClient {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for PolymarketClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PolymarketClient")
            .field("base_url", &self.base_url)
            .finish()
    }
}
