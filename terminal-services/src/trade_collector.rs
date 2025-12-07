//! Trade Collector Service
//!
//! Background service that collects trades from platform APIs and stores them
//! in the trade storage for historical price data generation.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use terminal_core::Platform;

use crate::market_service::MarketService;
use crate::trade_storage::TradeStorage;
use crate::websocket::WebSocketState;

/// Configuration for the trade collector
#[derive(Debug, Clone)]
pub struct TradeCollectorConfig {
    /// How often to poll for new trades (in seconds)
    pub poll_interval_secs: u64,
    /// Maximum number of trades to fetch per request
    pub trades_per_request: u32,
    /// Whether to collect from Kalshi
    pub collect_kalshi: bool,
    /// Whether to collect from Polymarket
    pub collect_polymarket: bool,
}

impl Default for TradeCollectorConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: 10,
            trades_per_request: 50,
            collect_kalshi: true,
            collect_polymarket: true,
        }
    }
}

/// Background service for collecting trades from platform APIs
pub struct TradeCollector {
    market_service: Arc<MarketService>,
    storage: Arc<TradeStorage>,
    ws_state: Option<Arc<WebSocketState>>,
    config: TradeCollectorConfig,
    /// Markets currently being tracked
    tracked_markets: RwLock<HashSet<(Platform, String)>>,
}

impl TradeCollector {
    /// Create a new TradeCollector
    pub fn new(
        market_service: Arc<MarketService>,
        storage: Arc<TradeStorage>,
        ws_state: Option<Arc<WebSocketState>>,
        config: TradeCollectorConfig,
    ) -> Self {
        Self {
            market_service,
            storage,
            ws_state,
            config,
            tracked_markets: RwLock::new(HashSet::new()),
        }
    }

    /// Add a market to be tracked
    pub async fn track_market(&self, platform: Platform, market_id: String) {
        let mut markets = self.tracked_markets.write().await;
        markets.insert((platform, market_id.clone()));
        info!(
            "Now tracking market: {:?}/{}",
            platform, market_id
        );
    }

    /// Remove a market from tracking
    pub async fn untrack_market(&self, platform: Platform, market_id: &str) {
        let mut markets = self.tracked_markets.write().await;
        markets.remove(&(platform, market_id.to_string()));
        debug!(
            "Stopped tracking market: {:?}/{}",
            platform, market_id
        );
    }

    /// Start the background collection loop
    ///
    /// This runs indefinitely, polling for new trades at the configured interval.
    pub async fn start(self: Arc<Self>) {
        info!(
            "Starting trade collector with {}s poll interval",
            self.config.poll_interval_secs
        );

        let mut ticker = interval(Duration::from_secs(self.config.poll_interval_secs));

        loop {
            ticker.tick().await;

            // Get current tracked markets
            let markets: Vec<(Platform, String)> = {
                let tracked = self.tracked_markets.read().await;
                tracked.iter().cloned().collect()
            };

            if markets.is_empty() {
                debug!("No markets being tracked, skipping collection cycle");
                continue;
            }

            debug!("Collecting trades for {} markets", markets.len());

            // Collect trades for each market
            for (platform, market_id) in markets {
                if let Err(e) = self.collect_market_trades(platform, &market_id).await {
                    warn!(
                        "Failed to collect trades for {:?}/{}: {}",
                        platform, market_id, e
                    );
                }
            }
        }
    }

    /// Collect trades for a single market
    pub async fn collect_market_trades(
        &self,
        platform: Platform,
        market_id: &str,
    ) -> Result<usize, TradeCollectorError> {
        // Check if we should collect for this platform
        match platform {
            Platform::Kalshi if !self.config.collect_kalshi => return Ok(0),
            Platform::Polymarket if !self.config.collect_polymarket => return Ok(0),
            _ => {}
        }

        // Get the latest trade we have to determine where to start
        let latest_trade = self.storage.get_latest_trade(platform, market_id)?;
        let latest_timestamp = latest_trade.map(|t| t.timestamp);

        debug!(
            "Collecting trades for {:?}/{}, latest: {:?}",
            platform, market_id, latest_timestamp
        );

        // Fetch trades from the API
        let trade_history = self
            .market_service
            .get_trades(platform, market_id, Some(self.config.trades_per_request), None)
            .await
            .map_err(|e| TradeCollectorError::Api(e.to_string()))?;

        if trade_history.trades.is_empty() {
            debug!("No trades returned for {:?}/{}", platform, market_id);
            return Ok(0);
        }

        // Filter to only new trades
        let new_trades: Vec<_> = trade_history
            .trades
            .into_iter()
            .filter(|t| {
                if let Some(latest) = latest_timestamp {
                    t.timestamp > latest || !self.storage.trade_exists(&t.id).unwrap_or(true)
                } else {
                    true
                }
            })
            .collect();

        if new_trades.is_empty() {
            debug!("No new trades for {:?}/{}", platform, market_id);
            return Ok(0);
        }

        let new_trade_count = new_trades.len();

        // Store the trades
        let stored = self.storage.store_trades(&new_trades)?;
        info!(
            "Stored {}/{} new trades for {:?}/{}",
            stored, new_trade_count, platform, market_id
        );

        // Broadcast new trades via WebSocket if available
        if let Some(ref ws_state) = self.ws_state {
            for trade in &new_trades {
                ws_state.broadcast_trade(trade.clone());
            }
        }

        Ok(stored)
    }

    /// Backfill trades for a market
    ///
    /// Fetches historical trades using pagination until no more are available.
    pub async fn backfill_market(
        &self,
        platform: Platform,
        market_id: &str,
        max_pages: usize,
    ) -> Result<usize, TradeCollectorError> {
        info!(
            "Starting backfill for {:?}/{} (max {} pages)",
            platform, market_id, max_pages
        );

        let mut total_stored = 0;
        let mut cursor: Option<String> = None;

        for page in 0..max_pages {
            let trade_history = self
                .market_service
                .get_trades(
                    platform,
                    market_id,
                    Some(self.config.trades_per_request),
                    cursor.as_deref(),
                )
                .await
                .map_err(|e| TradeCollectorError::Api(e.to_string()))?;

            if trade_history.trades.is_empty() {
                info!(
                    "Backfill complete for {:?}/{} after {} pages, {} trades",
                    platform, market_id, page, total_stored
                );
                break;
            }

            // Store trades (INSERT OR IGNORE to avoid duplicates)
            let stored = self.storage.store_trades(&trade_history.trades)?;
            total_stored += stored;

            debug!(
                "Backfill page {}: stored {} trades for {:?}/{}",
                page, stored, platform, market_id
            );

            // Check if there are more pages
            match trade_history.next_cursor {
                Some(next) => cursor = Some(next),
                None => {
                    info!(
                        "Backfill complete for {:?}/{} (no more pages), {} trades",
                        platform, market_id, total_stored
                    );
                    break;
                }
            }

            // Small delay between pages to avoid rate limiting
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(total_stored)
    }
}

/// Errors that can occur during trade collection
#[derive(Debug, thiserror::Error)]
pub enum TradeCollectorError {
    #[error("Storage error: {0}")]
    Storage(#[from] crate::trade_storage::TradeStorageError),

    #[error("API error: {0}")]
    Api(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_track_untrack_market() {
        let storage = Arc::new(TradeStorage::new_in_memory().unwrap());
        let market_service = Arc::new(MarketService::new(None, None));
        let collector = TradeCollector::new(
            market_service,
            storage,
            None,
            TradeCollectorConfig::default(),
        );

        collector
            .track_market(Platform::Kalshi, "test-market".to_string())
            .await;

        let markets = collector.tracked_markets.read().await;
        assert!(markets.contains(&(Platform::Kalshi, "test-market".to_string())));
        drop(markets);

        collector
            .untrack_market(Platform::Kalshi, "test-market")
            .await;

        let markets = collector.tracked_markets.read().await;
        assert!(!markets.contains(&(Platform::Kalshi, "test-market".to_string())));
    }
}
