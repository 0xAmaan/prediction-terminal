//! Market service for aggregating markets from multiple platforms

use std::sync::Arc;
use terminal_core::{OrderBook, Platform, PredictionMarket, TerminalError, TradeHistory, UnifiedMarket};
use terminal_kalshi::KalshiClient;
use terminal_polymarket::PolymarketClient;
use tracing::{debug, info, instrument, warn};

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
                info!("Fetching Kalshi markets");
                // Kalshi uses "open" status for active markets
                self.kalshi.list_all_markets(Some("open"), limit).await
            }
            Platform::Polymarket => {
                info!("Fetching Polymarket events");
                // Use events endpoint for proper grouping (not individual market options)
                self.polymarket.list_all_events(true, limit).await
            }
        }
    }

    /// Get all markets from all platforms
    #[instrument(skip(self))]
    pub async fn get_all_markets(
        &self,
        limit_per_platform: Option<usize>,
    ) -> Result<Vec<PredictionMarket>, TerminalError> {
        info!("Fetching markets from all platforms");

        // Fetch from both platforms concurrently
        let kalshi_future = self.kalshi.list_all_markets(Some("open"), limit_per_platform);
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
        info!("Fetching related markets for {} on {:?}", market_id, platform);

        let markets = match platform {
            Platform::Kalshi => {
                // For Kalshi, we need to get the event_ticker from the market first
                let market = self.kalshi.get_market(market_id).await?;
                // The ticker contains the event info, extract it
                // Format: TICKER or EVENT_TICKER-VARIANT
                if let Some(ticker) = &market.ticker {
                    // Try to get event markets
                    self.kalshi.get_related_markets(ticker).await.unwrap_or_default()
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
                        self.polymarket.get_related_markets(slug).await.unwrap_or_default()
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
}

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
