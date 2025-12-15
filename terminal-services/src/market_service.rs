//! Market service for aggregating markets from multiple platforms

use std::sync::Arc;
use terminal_core::{
    OrderBook, Platform, PredictionMarket, TerminalError, TradeHistory, UnifiedMarket,
};
use terminal_kalshi::KalshiClient;
use terminal_polymarket::{MarketFilter, MarketOption, PolymarketClient, PriceHistoryPoint};
use tracing::{debug, info, instrument, warn};

/// Minimal struct for parsing binary market options_json to extract clob_token_id
/// Binary markets only have: {"name": "Yes", "clob_token_id": "..."}
/// Multi-outcome markets have the full MarketOption fields (yes_price, market_id, etc.)
#[derive(Debug, Clone, serde::Deserialize)]
struct MinimalOption {
    #[serde(default)]
    #[allow(dead_code)]
    name: Option<String>,
    #[serde(default)]
    clob_token_id: Option<String>,
}

/// Service for fetching and aggregating markets across platforms
pub struct MarketService {
    kalshi: Arc<KalshiClient>,
    polymarket: Arc<PolymarketClient>,
}

impl MarketService {
    /// Create a new market service
    pub fn new(kalshi: KalshiClient, polymarket: PolymarketClient) -> Self {
        Self {
            kalshi: Arc::new(kalshi),
            polymarket: Arc::new(polymarket),
        }
    }

    /// Get all markets from a specific platform
    #[instrument(skip(self))]
    pub async fn get_markets_by_platform(
        &self,
        platform: Platform,
        limit: Option<usize>,
    ) -> Result<Vec<PredictionMarket>, TerminalError> {
        match platform {
            Platform::Kalshi => {
                info!("Fetching Kalshi markets (grouped by event)");
                // Use grouped method to combine multi-outcome events into single cards
                self.kalshi.list_markets_grouped(Some("open"), limit).await
            }
            Platform::Polymarket => {
                info!("Fetching Polymarket events");
                // Use events endpoint for proper grouping (not individual market options)
                self.polymarket.list_all_events(true, limit).await
            }
        }
    }

    /// Get Polymarket markets with a specific filter applied
    ///
    /// This queries Polymarket's API with proper server-side filtering/sorting,
    /// rather than fetching all markets and filtering client-side.
    #[instrument(skip(self))]
    pub async fn get_filtered_markets(
        &self,
        filter: MarketFilter,
        limit: Option<usize>,
    ) -> Result<Vec<PredictionMarket>, TerminalError> {
        info!("Fetching filtered Polymarket markets with filter: {:?}", filter);

        // Query Polymarket with the filter
        let events = self
            .polymarket
            .list_filtered_events(filter, limit.map(|l| l as u32))
            .await?;

        // Convert PolymarketEvents to PredictionMarkets
        let markets: Vec<PredictionMarket> = events
            .into_iter()
            .map(|e| e.to_prediction_market())
            .collect();

        debug!("Got {} markets for filter {:?}", markets.len(), filter);
        Ok(markets)
    }

    /// Get all markets from all platforms
    #[instrument(skip(self))]
    pub async fn get_all_markets(
        &self,
        limit_per_platform: Option<usize>,
    ) -> Result<Vec<PredictionMarket>, TerminalError> {
        info!("Fetching markets from all platforms");

        // Fetch from both platforms concurrently
        // Use grouped method for Kalshi to combine multi-outcome events
        let kalshi_future = self
            .kalshi
            .list_markets_grouped(Some("open"), limit_per_platform);
        // Use events endpoint for Polymarket (proper grouping, not individual options)
        let poly_future = self.polymarket.list_all_events(true, limit_per_platform);

        let (kalshi_result, poly_result) = tokio::join!(kalshi_future, poly_future);

        let mut all_markets = Vec::new();

        // Add Kalshi markets (log errors but don't fail)
        match kalshi_result {
            Ok(markets) => {
                debug!("Got {} Kalshi markets", markets.len());
                all_markets.extend(markets);
            }
            Err(e) => {
                warn!("Failed to fetch Kalshi markets: {}", e);
            }
        }

        // Add Polymarket events
        match poly_result {
            Ok(events) => {
                debug!("Got {} Polymarket events", events.len());
                all_markets.extend(events);
            }
            Err(e) => {
                warn!("Failed to fetch Polymarket events: {}", e);
            }
        }

        info!("Total markets fetched: {}", all_markets.len());
        Ok(all_markets)
    }

    /// Get a single market by ID and platform
    #[instrument(skip(self))]
    pub async fn get_market(
        &self,
        platform: Platform,
        id: &str,
    ) -> Result<PredictionMarket, TerminalError> {
        match platform {
            Platform::Kalshi => self.kalshi.get_market(id).await,
            Platform::Polymarket => self.polymarket.get_market(id).await,
        }
    }

    /// Search markets by title (simple substring match)
    #[instrument(skip(self))]
    pub async fn search_markets(
        &self,
        query: &str,
        platform: Option<Platform>,
        limit: Option<usize>,
    ) -> Result<Vec<PredictionMarket>, TerminalError> {
        let query_lower = query.to_lowercase();

        let markets = match platform {
            Some(p) => self.get_markets_by_platform(p, None).await?,
            None => self.get_all_markets(None).await?,
        };

        let mut filtered: Vec<PredictionMarket> = markets
            .into_iter()
            .filter(|m| m.title.to_lowercase().contains(&query_lower))
            .collect();

        if let Some(l) = limit {
            filtered.truncate(l);
        }

        Ok(filtered)
    }

    /// Get markets as unified markets (for potential matching)
    #[instrument(skip(self))]
    pub async fn get_unified_markets(
        &self,
        limit_per_platform: Option<usize>,
    ) -> Result<Vec<UnifiedMarket>, TerminalError> {
        let markets = self.get_all_markets(limit_per_platform).await?;

        // For now, just wrap each market as a unified market
        // TODO: Add market matching logic
        let unified: Vec<UnifiedMarket> = markets
            .into_iter()
            .map(|m| match m.platform {
                Platform::Kalshi => UnifiedMarket::from_kalshi(m),
                Platform::Polymarket => UnifiedMarket::from_polymarket(m),
            })
            .collect();

        Ok(unified)
    }

    // ========================================================================
    // Order Book & Trade Methods
    // ========================================================================

    /// Get order book for a market
    ///
    /// For Kalshi, the market_id is the ticker.
    /// For Polymarket, the market_id is the event ID - we'll look up the CLOB token ID.
    #[instrument(skip(self))]
    pub async fn get_orderbook(
        &self,
        platform: Platform,
        market_id: &str,
    ) -> Result<OrderBook, TerminalError> {
        info!("Fetching orderbook for {} on {:?}", market_id, platform);

        match platform {
            Platform::Kalshi => self.kalshi.get_orderbook(market_id).await,
            Platform::Polymarket => {
                // For Polymarket, market_id is the event ID
                // We need to look up the CLOB token ID first
                let token_id = self.polymarket.get_clob_token_id(market_id).await?;
                debug!("Resolved event {} to token ID: {}", market_id, token_id);
                self.polymarket.get_orderbook(&token_id, true).await
            }
        }
    }

    /// Get recent trades for a market
    ///
    /// For Kalshi, the market_id is the ticker.
    /// For Polymarket, the market_id is the event ID (uses public data API, no auth needed).
    #[instrument(skip(self))]
    pub async fn get_trades(
        &self,
        platform: Platform,
        market_id: &str,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<TradeHistory, TerminalError> {
        info!("Fetching trades for {} on {:?}", market_id, platform);

        match platform {
            Platform::Kalshi => self.kalshi.get_trades(market_id, limit, cursor).await,
            Platform::Polymarket => {
                // For Polymarket, use the public data API with the event ID directly
                self.polymarket.get_trades(market_id, limit).await
            }
        }
    }

    /// Get related markets (e.g., other markets in the same event)
    #[instrument(skip(self))]
    pub async fn get_related_markets(
        &self,
        platform: Platform,
        market_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<PredictionMarket>, TerminalError> {
        info!(
            "Fetching related markets for {} on {:?}",
            market_id, platform
        );

        let markets = match platform {
            Platform::Kalshi => {
                // For Kalshi, we need to get the event_ticker from the market first
                let market = self.kalshi.get_market(market_id).await?;
                // The ticker contains the event info, extract it
                // Format: TICKER or EVENT_TICKER-VARIANT
                if let Some(ticker) = &market.ticker {
                    // Try to get event markets
                    self.kalshi
                        .get_related_markets(ticker)
                        .await
                        .unwrap_or_default()
                } else {
                    Vec::new()
                }
            }
            Platform::Polymarket => {
                // For Polymarket, we need to extract the event slug from the market URL
                let market = self.polymarket.get_market(market_id).await?;
                if let Some(url) = &market.url {
                    // URL format: https://polymarket.com/event/{slug}
                    if let Some(slug) = url.strip_prefix("https://polymarket.com/event/") {
                        self.polymarket
                            .get_related_markets(slug)
                            .await
                            .unwrap_or_default()
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }
        };

        // Apply limit if specified
        let markets = if let Some(l) = limit {
            markets.into_iter().take(l).collect()
        } else {
            markets
        };

        Ok(markets)
    }

    // ========================================================================
    // Multi-Outcome / Price History Methods (Polymarket-specific for now)
    // ========================================================================

    /// Get price history for multiple outcomes (top N by price)
    ///
    /// Returns price history for each of the top N outcomes in a multi-outcome market.
    /// Only supported for Polymarket currently.
    #[instrument(skip(self))]
    pub async fn get_multi_outcome_prices(
        &self,
        platform: Platform,
        event_id: &str,
        top: usize,
        interval: &str,
    ) -> Result<Vec<OutcomePriceHistory>, TerminalError> {
        info!(
            "Fetching multi-outcome prices for {} on {:?}",
            event_id, platform
        );

        match platform {
            Platform::Kalshi => {
                // Get the event to access options
                let market = self.kalshi.get_market(event_id).await?;

                // Parse options from options_json
                let options: Vec<serde_json::Value> = if let Some(json) = &market.options_json {
                    serde_json::from_str(json).unwrap_or_default()
                } else {
                    return Err(TerminalError::not_found(
                        "No options found for market".to_string(),
                    ));
                };

                // Extract series ticker from event_id (e.g., "KXNEWPOPE-70" -> "KXNEWPOPE")
                let series_ticker = terminal_kalshi::types::KalshiMarket::extract_series_ticker_static(event_id);

                // Sort by yes_price descending and take top N
                let mut sorted_options = options.clone();
                sorted_options.sort_by(|a, b| {
                    let price_a: f64 = a["yes_price"].as_str()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);
                    let price_b: f64 = b["yes_price"].as_str()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0.0);
                    price_b.partial_cmp(&price_a).unwrap_or(std::cmp::Ordering::Equal)
                });
                let top_options: Vec<_> = sorted_options.into_iter().take(top).collect();

                // Fetch price history for each outcome
                let mut results = Vec::new();
                for (idx, option) in top_options.into_iter().enumerate() {
                    let market_ticker = option["market_id"].as_str().unwrap_or("");
                    let option_name = option["name"].as_str().unwrap_or("Unknown").to_string();

                    if !market_ticker.is_empty() {
                        match self.kalshi.get_candlesticks(series_ticker, market_ticker, interval).await {
                            Ok(history) => {
                                // Convert Kalshi PriceHistoryPoint to Polymarket format
                                let history: Vec<PriceHistoryPoint> = history
                                    .into_iter()
                                    .map(|p| PriceHistoryPoint { t: p.t, p: p.p })
                                    .collect();

                                results.push(OutcomePriceHistory {
                                    name: option_name,
                                    market_id: market_ticker.to_string(),
                                    color: OUTCOME_COLORS[idx % OUTCOME_COLORS.len()].to_string(),
                                    history,
                                });
                            }
                            Err(e) => {
                                warn!("Failed to fetch price history for {}: {}", option_name, e);
                            }
                        }
                    }
                }

                Ok(results)
            }
            Platform::Polymarket => {
                // Get the event to access options
                let market = self.polymarket.get_market(event_id).await?;

                // Parse options from options_json (multi-outcome markets have full MarketOption fields)
                let options: Vec<MarketOption> = if let Some(json) = &market.options_json {
                    serde_json::from_str(json).unwrap_or_default()
                } else {
                    return Err(TerminalError::not_found(
                        "No options found for market".to_string(),
                    ));
                };

                // Sort by yes_price descending and take top N
                let mut sorted_options = options.clone();
                sorted_options.sort_by(|a, b| {
                    b.yes_price
                        .partial_cmp(&a.yes_price)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                let top_options: Vec<_> = sorted_options.into_iter().take(top).collect();

                // Fetch price history for each outcome
                let mut results = Vec::new();
                for (idx, option) in top_options.into_iter().enumerate() {
                    if let Some(token_id) = &option.clob_token_id {
                        match self.polymarket.get_prices_history(token_id, interval, None).await {
                            Ok(history) => {
                                results.push(OutcomePriceHistory {
                                    name: option.name.clone(),
                                    market_id: option.market_id.clone(),
                                    color: OUTCOME_COLORS[idx % OUTCOME_COLORS.len()].to_string(),
                                    history,
                                });
                            }
                            Err(e) => {
                                warn!("Failed to fetch price history for {}: {}", option.name, e);
                            }
                        }
                    }
                }

                Ok(results)
            }
        }
    }

    /// Get orderbook for a specific outcome within a multi-outcome event
    #[instrument(skip(self))]
    pub async fn get_outcome_orderbook(
        &self,
        platform: Platform,
        _event_id: &str,
        token_id: &str,
    ) -> Result<OrderBook, TerminalError> {
        info!(
            "Fetching outcome orderbook for token {} on {:?}",
            token_id, platform
        );

        match platform {
            Platform::Kalshi => {
                // For Kalshi, the token_id is actually the ticker
                self.kalshi.get_orderbook(token_id).await
            }
            Platform::Polymarket => {
                // For Polymarket, use the CLOB token ID directly
                self.polymarket.get_orderbook(token_id, true).await
            }
        }
    }

    /// Get trades for a specific outcome within a multi-outcome event
    #[instrument(skip(self))]
    pub async fn get_outcome_trades(
        &self,
        platform: Platform,
        _event_id: &str,
        condition_id: &str,
        limit: Option<u32>,
    ) -> Result<TradeHistory, TerminalError> {
        info!(
            "Fetching outcome trades for condition {} on {:?}",
            condition_id, platform
        );

        match platform {
            Platform::Kalshi => {
                // For Kalshi, condition_id is the ticker
                self.kalshi.get_trades(condition_id, limit, None).await
            }
            Platform::Polymarket => {
                // For Polymarket, use the condition ID to filter trades
                self.polymarket
                    .get_outcome_trades(condition_id, limit)
                    .await
            }
        }
    }

    /// Get price history for a specific outcome
    #[instrument(skip(self))]
    pub async fn get_outcome_prices(
        &self,
        platform: Platform,
        token_id: &str,
        interval: &str,
    ) -> Result<Vec<PriceHistoryPoint>, TerminalError> {
        info!(
            "Fetching outcome prices for token {} on {:?}",
            token_id, platform
        );

        match platform {
            Platform::Kalshi => {
                // For Kalshi, token_id is the market ticker (e.g., "KXTRILLION-25-MUSK")
                // Extract series_ticker from market_ticker for the candlesticks API
                let series_ticker =
                    terminal_kalshi::types::KalshiMarket::extract_series_from_market_ticker(
                        token_id,
                    );

                let history = self
                    .kalshi
                    .get_candlesticks(series_ticker, token_id, interval)
                    .await?;

                // Convert Kalshi PriceHistoryPoint to terminal-core format
                Ok(history
                    .into_iter()
                    .map(|p| PriceHistoryPoint { t: p.t, p: p.p })
                    .collect())
            }
            Platform::Polymarket => self.polymarket.get_prices_history(token_id, interval, None).await,
        }
    }

    /// Get native price history for a market or outcome
    ///
    /// This fetches price history directly from the platform's API (complete coverage),
    /// rather than building from stored trades (partial coverage).
    ///
    /// For Polymarket:
    /// - If market_id is an event ID (UUID format), looks up the YES token's clob_token_id
    /// - If market_id is already a token ID (numeric string), uses it directly
    ///
    /// # Arguments
    /// * `platform` - The trading platform
    /// * `market_id` - Market identifier (event ID or token ID for Polymarket)
    /// * `interval` - Duration string: "1m", "1h", "6h", "1d", "1w", "max"
    /// * `fidelity` - Optional fidelity override in minutes (lower = more data points)
    #[instrument(skip(self))]
    pub async fn get_native_price_history(
        &self,
        platform: Platform,
        market_id: &str,
        interval: &str,
        fidelity: Option<u32>,
    ) -> Result<Vec<PriceHistoryPoint>, TerminalError> {
        info!(
            "Fetching native price history for {} on {:?}",
            market_id, platform
        );

        match platform {
            Platform::Kalshi => {
                // For Kalshi, extract series ticker and use candlesticks API
                let series_ticker =
                    terminal_kalshi::types::KalshiMarket::extract_series_from_market_ticker(
                        market_id,
                    );

                let history = self
                    .kalshi
                    .get_candlesticks(series_ticker, market_id, interval)
                    .await?;

                Ok(history
                    .into_iter()
                    .map(|p| PriceHistoryPoint { t: p.t, p: p.p })
                    .collect())
            }
            Platform::Polymarket => {
                // Polymarket token IDs are very long numeric strings (77+ digits)
                // Event IDs are UUIDs (with hyphens) or shorter alphanumeric strings
                let is_token_id = market_id.len() > 50 && market_id.chars().all(|c| c.is_ascii_digit());

                let token_id = if is_token_id {
                    // Already a token ID (e.g., from multi-outcome individual outcome)
                    debug!("Using market_id directly as token ID: {}", market_id);
                    market_id.to_string()
                } else {
                    // Event ID - look up the YES token's clob_token_id
                    let market = self.polymarket.get_market(market_id).await?;

                    let options: Vec<MinimalOption> = if let Some(json) = &market.options_json {
                        serde_json::from_str(json).map_err(|e| {
                            TerminalError::parse(format!("Failed to parse options_json: {}", e))
                        })?
                    } else {
                        return Err(TerminalError::not_found(
                            "No options found for market".to_string(),
                        ));
                    };

                    options
                        .first()
                        .and_then(|o| o.clob_token_id.clone())
                        .ok_or_else(|| {
                            TerminalError::not_found("No clob_token_id found for market".to_string())
                        })?
                };

                debug!("Fetching price history for token ID: {}", token_id);
                self.polymarket.get_prices_history(&token_id, interval, fidelity).await
            }
        }
    }
}

/// Price history for a single outcome
#[derive(Debug, Clone, serde::Serialize)]
pub struct OutcomePriceHistory {
    pub name: String,
    pub market_id: String,
    pub color: String,
    pub history: Vec<PriceHistoryPoint>,
}

/// Color palette for outcome chart lines
const OUTCOME_COLORS: &[&str] = &[
    "#22c55e", // green
    "#3b82f6", // blue
    "#f59e0b", // amber
    "#8b5cf6", // purple
    "#ef4444", // red
    "#06b6d4", // cyan
    "#ec4899", // pink
    "#14b8a6", // teal
];

impl Clone for MarketService {
    fn clone(&self) -> Self {
        Self {
            kalshi: Arc::clone(&self.kalshi),
            polymarket: Arc::clone(&self.polymarket),
        }
    }
}

impl std::fmt::Debug for MarketService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MarketService").finish()
    }
}
