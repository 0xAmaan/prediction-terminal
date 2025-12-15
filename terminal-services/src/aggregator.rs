//! Market Data Aggregator
//!
//! Coordinates WebSocket connections to Kalshi and Polymarket exchanges,
//! normalizes incoming data, and broadcasts updates to connected frontend clients.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use tokio::sync::{broadcast, RwLock};
use tracing::{info, warn};

use terminal_core::{OrderBook, OrderBookLevel, Platform};
use terminal_kalshi::{KalshiUpdate, KalshiWebSocket, KalshiWebSocketConfig};
use terminal_polymarket::{PolymarketUpdate, PolymarketWebSocket, PolymarketWebSocketConfig};

use crate::websocket::{SubscriptionEvent, WebSocketState};
use crate::MarketService;
use crate::TradeStorage;

/// Health status for a connection
#[derive(Debug, Clone, Serialize)]
pub struct ConnectionHealth {
    pub platform: String,
    pub connected: bool,
    pub last_message_time: Option<DateTime<Utc>>,
    pub message_count: u64,
    pub is_stale: bool,
}

/// Overall aggregator health
#[derive(Debug, Clone, Serialize)]
pub struct AggregatorHealth {
    pub kalshi: ConnectionHealth,
    pub polymarket: ConnectionHealth,
    pub active_subscriptions: usize,
    pub healthy: bool,
}

/// Stale threshold - if no message for this duration, consider connection stale
const STALE_THRESHOLD_SECS: u64 = 60;

/// Configuration for the MarketDataAggregator
#[derive(Clone, Debug)]
pub struct AggregatorConfig {
    pub kalshi_enabled: bool,
    pub polymarket_enabled: bool,
}

impl Default for AggregatorConfig {
    fn default() -> Self {
        Self {
            kalshi_enabled: true,
            polymarket_enabled: true,
        }
    }
}

/// Health metrics for a single connection (atomic for thread-safe access)
#[derive(Debug, Default)]
pub struct ConnectionMetrics {
    connected: AtomicBool,
    last_message_epoch_ms: AtomicU64,
    message_count: AtomicU64,
}

impl ConnectionMetrics {
    fn new() -> Self {
        Self {
            connected: AtomicBool::new(false),
            last_message_epoch_ms: AtomicU64::new(0),
            message_count: AtomicU64::new(0),
        }
    }

    fn set_connected(&self, connected: bool) {
        self.connected.store(connected, Ordering::SeqCst);
    }

    fn record_message(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.last_message_epoch_ms.store(now, Ordering::SeqCst);
        self.message_count.fetch_add(1, Ordering::SeqCst);
    }

    fn get_health(&self, platform: &str) -> ConnectionHealth {
        let connected = self.connected.load(Ordering::SeqCst);
        let last_ms = self.last_message_epoch_ms.load(Ordering::SeqCst);
        let message_count = self.message_count.load(Ordering::SeqCst);

        let last_message_time = if last_ms > 0 {
            DateTime::from_timestamp(
                (last_ms / 1000) as i64,
                ((last_ms % 1000) * 1_000_000) as u32,
            )
        } else {
            None
        };

        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let is_stale = if connected && last_ms > 0 {
            (now_ms - last_ms) > (STALE_THRESHOLD_SECS * 1000)
        } else {
            !connected
        };

        ConnectionHealth {
            platform: platform.to_string(),
            connected,
            last_message_time,
            message_count,
            is_stale,
        }
    }
}

/// Manages connections to exchange WebSockets and aggregates data
pub struct MarketDataAggregator {
    config: AggregatorConfig,
    /// WebSocket state for broadcasting to frontend clients
    ws_state: Arc<WebSocketState>,
    /// Market service for lookups (ticker <-> market_id mapping)
    _market_service: MarketService,
    /// Kalshi WebSocket client
    kalshi_ws: Option<KalshiWebSocket>,
    /// Polymarket WebSocket client
    polymarket_ws: Option<PolymarketWebSocket>,
    /// Mapping from market_id to Kalshi ticker
    kalshi_ticker_map: Arc<RwLock<HashMap<String, String>>>,
    /// Mapping from market_id to Polymarket token_id
    polymarket_token_map: Arc<RwLock<HashMap<String, String>>>,
    /// Active subscriptions per platform
    active_subscriptions: Arc<RwLock<HashMap<Platform, HashSet<String>>>>,
    /// Local orderbook cache for applying deltas
    orderbook_cache: Arc<RwLock<HashMap<String, OrderBook>>>,
    /// Health metrics for Kalshi connection
    kalshi_metrics: Arc<ConnectionMetrics>,
    /// Health metrics for Polymarket connection
    polymarket_metrics: Arc<ConnectionMetrics>,
    /// Trade storage for persisting prices and orderbook snapshots
    trade_storage: Option<Arc<TradeStorage>>,
}

impl MarketDataAggregator {
    /// Create a new MarketDataAggregator
    pub fn new(
        config: AggregatorConfig,
        ws_state: Arc<WebSocketState>,
        market_service: MarketService,
    ) -> Self {
        Self {
            config,
            ws_state,
            _market_service: market_service,
            kalshi_ws: None,
            polymarket_ws: None,
            kalshi_ticker_map: Arc::new(RwLock::new(HashMap::new())),
            polymarket_token_map: Arc::new(RwLock::new(HashMap::new())),
            active_subscriptions: Arc::new(RwLock::new(HashMap::new())),
            orderbook_cache: Arc::new(RwLock::new(HashMap::new())),
            kalshi_metrics: Arc::new(ConnectionMetrics::new()),
            polymarket_metrics: Arc::new(ConnectionMetrics::new()),
            trade_storage: None,
        }
    }

    /// Set trade storage for price and orderbook persistence
    pub fn set_trade_storage(&mut self, storage: Arc<TradeStorage>) {
        self.trade_storage = Some(storage);
    }

    /// Start orderbook snapshot background task
    fn start_snapshot_task(
        orderbook_cache: Arc<RwLock<HashMap<String, OrderBook>>>,
        ticker_map: Arc<RwLock<HashMap<String, String>>>,
        _token_map: Arc<RwLock<HashMap<String, String>>>,
        storage: Arc<TradeStorage>,
    ) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));

            loop {
                interval.tick().await;

                // Read current orderbooks
                let orderbooks = {
                    let cache = orderbook_cache.read().await;
                    cache.clone()
                };

                if orderbooks.is_empty() {
                    continue;
                }

                // Get ticker mappings for platform detection
                let kalshi_tickers: HashSet<String> = {
                    let map = ticker_map.read().await;
                    map.values().cloned().collect()
                };

                // Snapshot each orderbook
                for (market_id, book) in orderbooks {
                    // Determine platform (if in kalshi ticker map, it's Kalshi)
                    let platform = if kalshi_tickers.contains(&market_id) {
                        Platform::Kalshi
                    } else {
                        Platform::Polymarket
                    };

                    // Serialize orderbook levels to JSON
                    let yes_bids = serde_json::to_string(&book.yes_bids).unwrap_or_default();
                    let yes_asks = serde_json::to_string(&book.yes_asks).unwrap_or_default();
                    let no_bids = serde_json::to_string(&book.no_bids).unwrap_or_default();
                    let no_asks = serde_json::to_string(&book.no_asks).unwrap_or_default();

                    // Store snapshot
                    if let Err(e) = storage.store_orderbook_snapshot(
                        platform,
                        &market_id,
                        &yes_bids,
                        &yes_asks,
                        &no_bids,
                        &no_asks,
                    ) {
                        warn!("[Aggregator] Failed to store orderbook snapshot for {}: {}", market_id, e);
                    }

                    // Also store current price from best bid/ask
                    let yes_price = book.yes_bids.first().map(|l| {
                        l.price.try_into().unwrap_or_else(|_| l.price.to_string().parse().unwrap_or(0.0))
                    });
                    let no_price = book.no_bids.first().map(|l| {
                        l.price.try_into().unwrap_or_else(|_| l.price.to_string().parse().unwrap_or(0.0))
                    });

                    if yes_price.is_some() || no_price.is_some() {
                        if let Err(e) = storage.store_price(platform, &market_id, yes_price, no_price) {
                            warn!("[Aggregator] Failed to store price for {}: {}", market_id, e);
                        }
                    }
                }

                // Prune old snapshots once per day (check on each tick, but only act if needed)
                // This is a lightweight check
                static LAST_PRUNE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let last = LAST_PRUNE.load(std::sync::atomic::Ordering::SeqCst);

                if now - last > 86400 {
                    // Prune snapshots older than 7 days
                    match storage.prune_orderbook_snapshots(7) {
                        Ok(deleted) => {
                            if deleted > 0 {
                                info!("[Aggregator] Pruned {} old orderbook snapshots", deleted);
                            }
                        }
                        Err(e) => {
                            warn!("[Aggregator] Failed to prune orderbook snapshots: {}", e);
                        }
                    }
                    LAST_PRUNE.store(now, std::sync::atomic::Ordering::SeqCst);
                }
            }
        });
    }

    /// Get health status for all connections
    pub async fn get_health(&self) -> AggregatorHealth {
        let kalshi_health = self.kalshi_metrics.get_health("kalshi");
        let polymarket_health = self.polymarket_metrics.get_health("polymarket");

        let active_subs = {
            let subs = self.active_subscriptions.read().await;
            subs.values().map(|s| s.len()).sum()
        };

        let healthy = (!self.config.kalshi_enabled || (!kalshi_health.is_stale))
            && (!self.config.polymarket_enabled || (!polymarket_health.is_stale));

        AggregatorHealth {
            kalshi: kalshi_health,
            polymarket: polymarket_health,
            active_subscriptions: active_subs,
            healthy,
        }
    }

    /// Start the aggregator and exchange WebSocket connections
    pub async fn start(&mut self) -> Result<(), anyhow::Error> {
        info!("[Aggregator] Starting MarketDataAggregator");

        // Start Kalshi WebSocket
        if self.config.kalshi_enabled {
            let kalshi_config = KalshiWebSocketConfig::default();
            let (mut kalshi_ws, kalshi_rx) = KalshiWebSocket::new(kalshi_config);

            kalshi_ws.start().await?;

            // Spawn task to process Kalshi updates
            let ws_state = Arc::clone(&self.ws_state);
            let ticker_map = Arc::clone(&self.kalshi_ticker_map);
            let orderbook_cache = Arc::clone(&self.orderbook_cache);
            let metrics = Arc::clone(&self.kalshi_metrics);

            tokio::spawn(async move {
                Self::process_kalshi_updates(
                    kalshi_rx,
                    ws_state,
                    ticker_map,
                    orderbook_cache,
                    metrics,
                )
                .await;
            });

            self.kalshi_ws = Some(kalshi_ws);
            info!("[Aggregator] Kalshi WebSocket started");
        }

        // Start Polymarket WebSocket
        if self.config.polymarket_enabled {
            let polymarket_config = PolymarketWebSocketConfig::default();
            let (mut polymarket_ws, polymarket_rx) = PolymarketWebSocket::new(polymarket_config);

            polymarket_ws.start().await?;

            // Spawn task to process Polymarket updates
            let ws_state = Arc::clone(&self.ws_state);
            let token_map = Arc::clone(&self.polymarket_token_map);
            let orderbook_cache = Arc::clone(&self.orderbook_cache);
            let metrics = Arc::clone(&self.polymarket_metrics);

            tokio::spawn(async move {
                Self::process_polymarket_updates(
                    polymarket_rx,
                    ws_state,
                    token_map,
                    orderbook_cache,
                    metrics,
                )
                .await;
            });

            self.polymarket_ws = Some(polymarket_ws);
            info!("[Aggregator] Polymarket WebSocket started");
        }

        // Start orderbook snapshot task if storage is configured
        if let Some(ref storage) = self.trade_storage {
            Self::start_snapshot_task(
                Arc::clone(&self.orderbook_cache),
                Arc::clone(&self.kalshi_ticker_map),
                Arc::clone(&self.polymarket_token_map),
                Arc::clone(storage),
            );
            info!("[Aggregator] Orderbook snapshot task started");
        }

        // Start periodic health logging task (every 60 seconds)
        let kalshi_metrics = Arc::clone(&self.kalshi_metrics);
        let polymarket_metrics = Arc::clone(&self.polymarket_metrics);
        let ws_state = Arc::clone(&self.ws_state);
        let kalshi_enabled = self.config.kalshi_enabled;
        let polymarket_enabled = self.config.polymarket_enabled;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;

                let kalshi_health = kalshi_metrics.get_health("kalshi");
                let poly_health = polymarket_metrics.get_health("polymarket");
                let sub_count = ws_state.subscriptions.total_subscriptions();

                let kalshi_status = if !kalshi_enabled {
                    "disabled".to_string()
                } else if kalshi_health.connected {
                    format!("connected ({}s ago)",
                        kalshi_health.last_message_time
                            .map(|t| (Utc::now() - t).num_seconds())
                            .unwrap_or(0))
                } else {
                    "disconnected".to_string()
                };

                let poly_status = if !polymarket_enabled {
                    "disabled".to_string()
                } else if poly_health.connected {
                    format!("connected ({}s ago)",
                        poly_health.last_message_time
                            .map(|t| (Utc::now() - t).num_seconds())
                            .unwrap_or(0))
                } else {
                    "disconnected".to_string()
                };

                info!(
                    "[Aggregator] Health: Kalshi={}, Polymarket={}, subscriptions={}",
                    kalshi_status, poly_status, sub_count
                );
            }
        });
        info!("[Aggregator] Health logging task started");

        Ok(())
    }

    /// Subscribe to a market on the appropriate exchange
    pub async fn subscribe(
        &self,
        platform: Platform,
        market_id: &str,
    ) -> Result<(), anyhow::Error> {
        info!(
            "[Aggregator] Subscribing to {:?} market: {}",
            platform, market_id
        );

        match platform {
            Platform::Kalshi => {
                if let Some(ref ws) = self.kalshi_ws {
                    // For Kalshi, market_id IS the ticker
                    let ticker = market_id.to_string();

                    // Store mapping
                    {
                        let mut map = self.kalshi_ticker_map.write().await;
                        map.insert(market_id.to_string(), ticker.clone());
                    }

                    // Track subscription
                    {
                        let mut subs = self.active_subscriptions.write().await;
                        subs.entry(Platform::Kalshi)
                            .or_insert_with(HashSet::new)
                            .insert(market_id.to_string());
                    }

                    ws.subscribe(&ticker).await?;
                }
            }
            Platform::Polymarket => {
                if let Some(ref ws) = self.polymarket_ws {
                    // For Polymarket, we need to look up the token_id
                    // The market_id might be the event_id, we need the CLOB token_id
                    let token_id = self.get_polymarket_token_id(market_id).await?;

                    // Store mapping (token_id -> market_id for reverse lookup)
                    {
                        let mut map = self.polymarket_token_map.write().await;
                        map.insert(token_id.clone(), market_id.to_string());
                    }

                    // Track subscription
                    {
                        let mut subs = self.active_subscriptions.write().await;
                        subs.entry(Platform::Polymarket)
                            .or_insert_with(HashSet::new)
                            .insert(market_id.to_string());
                    }

                    ws.subscribe(vec![token_id]).await?;
                }
            }
        }

        Ok(())
    }

    /// Unsubscribe from a market
    pub async fn unsubscribe(
        &self,
        platform: Platform,
        market_id: &str,
    ) -> Result<(), anyhow::Error> {
        info!(
            "[Aggregator] Unsubscribing from {:?} market: {}",
            platform, market_id
        );

        match platform {
            Platform::Kalshi => {
                if let Some(ref ws) = self.kalshi_ws {
                    let ticker = {
                        let map = self.kalshi_ticker_map.read().await;
                        map.get(market_id).cloned()
                    };

                    if let Some(ticker) = ticker {
                        ws.unsubscribe(&ticker).await?;
                    }

                    // Remove from tracking
                    {
                        let mut map = self.kalshi_ticker_map.write().await;
                        map.remove(market_id);
                    }
                    {
                        let mut subs = self.active_subscriptions.write().await;
                        if let Some(set) = subs.get_mut(&Platform::Kalshi) {
                            set.remove(market_id);
                        }
                    }
                }
            }
            Platform::Polymarket => {
                if let Some(ref ws) = self.polymarket_ws {
                    // Find and remove from token map
                    let token_id = {
                        let map = self.polymarket_token_map.read().await;
                        map.iter()
                            .find(|(_, v)| *v == market_id)
                            .map(|(k, _)| k.clone())
                    };

                    if let Some(token_id) = token_id {
                        ws.unsubscribe(vec![token_id.clone()]).await?;

                        let mut map = self.polymarket_token_map.write().await;
                        map.remove(&token_id);
                    }

                    // Remove from tracking
                    {
                        let mut subs = self.active_subscriptions.write().await;
                        if let Some(set) = subs.get_mut(&Platform::Polymarket) {
                            set.remove(market_id);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get Polymarket token_id for a market
    async fn get_polymarket_token_id(&self, market_id: &str) -> Result<String, anyhow::Error> {
        // Try to get from cache first
        {
            let map = self.polymarket_token_map.read().await;
            // Check reverse lookup (token_id -> market_id stored, find by value)
            for (token, mid) in map.iter() {
                if mid == market_id {
                    return Ok(token.clone());
                }
            }
        }

        // Fetch from API
        let client = terminal_polymarket::PolymarketClient::new();
        match client.get_clob_token_id(market_id).await {
            Ok(token_id) => Ok(token_id),
            Err(e) => {
                warn!(
                    "[Aggregator] Failed to get token_id for {}: {}",
                    market_id, e
                );
                // Fall back to using market_id as token_id (some markets use condition_id directly)
                Ok(market_id.to_string())
            }
        }
    }

    /// Process Kalshi WebSocket updates
    async fn process_kalshi_updates(
        mut rx: broadcast::Receiver<KalshiUpdate>,
        ws_state: Arc<WebSocketState>,
        _ticker_map: Arc<RwLock<HashMap<String, String>>>,
        orderbook_cache: Arc<RwLock<HashMap<String, OrderBook>>>,
        metrics: Arc<ConnectionMetrics>,
    ) {
        info!("[Aggregator] Starting Kalshi update processor");

        loop {
            match rx.recv().await {
                Ok(update) => {
                    // Record message for health tracking
                    metrics.record_message();

                    match update {
                        KalshiUpdate::OrderbookSnapshot {
                            market_ticker,
                            orderbook,
                        } => {
                            // Kalshi orderbook snapshot received

                            // Cache the orderbook
                            {
                                let mut cache = orderbook_cache.write().await;
                                cache.insert(market_ticker.clone(), orderbook.clone());
                            }

                            // Broadcast to clients
                            ws_state.broadcast_orderbook_update(
                                Platform::Kalshi,
                                market_ticker,
                                orderbook,
                            );
                        }
                        KalshiUpdate::OrderbookDelta {
                            market_ticker,
                            side,
                            price,
                            delta,
                            seq: _,
                        } => {
                            // Kalshi orderbook delta received

                            // Apply delta to cached orderbook
                            let updated_book = {
                                let mut cache = orderbook_cache.write().await;
                                if let Some(book) = cache.get_mut(&market_ticker) {
                                    // Apply the delta
                                    let levels = if side == "yes" {
                                        &mut book.yes_bids
                                    } else {
                                        &mut book.no_bids
                                    };

                                    // Find and update or add the level
                                    if let Some(level) =
                                        levels.iter_mut().find(|l| l.price == price)
                                    {
                                        level.quantity += delta;
                                        if level.quantity <= Decimal::ZERO {
                                            levels.retain(|l| l.price != price);
                                        }
                                    } else if delta > Decimal::ZERO {
                                        levels.push(OrderBookLevel::new(price, delta));
                                        levels.sort_by(|a, b| b.price.cmp(&a.price));
                                    }

                                    book.timestamp = Utc::now();
                                    Some(book.clone())
                                } else {
                                    None
                                }
                            };

                            // Broadcast updated orderbook
                            if let Some(book) = updated_book {
                                ws_state.broadcast_orderbook_update(
                                    Platform::Kalshi,
                                    market_ticker,
                                    book,
                                );
                            }
                        }
                        KalshiUpdate::PriceUpdate {
                            market_ticker,
                            yes_price,
                            no_price,
                            volume: _,
                        } => {
                            // Kalshi price update received

                            if let (Some(yes), Some(no)) = (yes_price, no_price) {
                                ws_state.broadcast_price_update(
                                    Platform::Kalshi,
                                    market_ticker,
                                    yes,
                                    no,
                                );
                            }
                        }
                        KalshiUpdate::Trade {
                            market_ticker: _,
                            trade,
                        } => {
                            // Kalshi trade received
                            ws_state.broadcast_trade(trade);
                        }
                        KalshiUpdate::ConnectionState { connected, error } => {
                            metrics.set_connected(connected);
                            if connected {
                                info!("[Aggregator] Kalshi WebSocket connected");
                            } else {
                                warn!("[Aggregator] Kalshi WebSocket disconnected: {:?}", error);
                            }
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!("[Aggregator] Kalshi update receiver lagged {} messages", n);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    info!("[Aggregator] Kalshi update channel closed");
                    break;
                }
            }
        }
    }

    /// Process Polymarket WebSocket updates
    async fn process_polymarket_updates(
        mut rx: broadcast::Receiver<PolymarketUpdate>,
        ws_state: Arc<WebSocketState>,
        token_map: Arc<RwLock<HashMap<String, String>>>,
        orderbook_cache: Arc<RwLock<HashMap<String, OrderBook>>>,
        metrics: Arc<ConnectionMetrics>,
    ) {
        info!("[Aggregator] Starting Polymarket update processor");

        loop {
            match rx.recv().await {
                Ok(update) => {
                    // Record message for health tracking
                    metrics.record_message();

                    match update {
                        PolymarketUpdate::OrderbookSnapshot {
                            asset_id,
                            orderbook,
                        } => {
                            // Look up market_id from token_id
                            let market_id = {
                                let map = token_map.read().await;
                                map.get(&asset_id)
                                    .cloned()
                                    .unwrap_or_else(|| asset_id.clone())
                            };

                            // Polymarket orderbook snapshot received

                            // Cache the orderbook
                            {
                                let mut cache = orderbook_cache.write().await;
                                cache.insert(market_id.clone(), orderbook.clone());
                            }

                            // Broadcast to clients
                            ws_state.broadcast_orderbook_update(
                                Platform::Polymarket,
                                market_id,
                                orderbook,
                            );
                        }
                        PolymarketUpdate::PriceChange {
                            asset_id,
                            changes,
                            best_bid,
                            best_ask,
                        } => {
                            let market_id = {
                                let map = token_map.read().await;
                                map.get(&asset_id)
                                    .cloned()
                                    .unwrap_or_else(|| asset_id.clone())
                            };

                            // Polymarket price change received

                            // Apply changes to cached orderbook
                            let updated_book = {
                                let mut cache = orderbook_cache.write().await;
                                if let Some(book) = cache.get_mut(&market_id) {
                                    for (price, size, side) in changes {
                                        let levels = if side == "BUY" {
                                            &mut book.yes_bids
                                        } else {
                                            &mut book.yes_asks
                                        };

                                        if let Some(level) =
                                            levels.iter_mut().find(|l| l.price == price)
                                        {
                                            level.quantity = size;
                                            if level.quantity <= Decimal::ZERO {
                                                levels.retain(|l| l.price != price);
                                            }
                                        } else if size > Decimal::ZERO {
                                            levels.push(OrderBookLevel::new(price, size));
                                        }
                                    }

                                    // Sort
                                    book.yes_bids.sort_by(|a, b| b.price.cmp(&a.price));
                                    book.yes_asks.sort_by(|a, b| a.price.cmp(&b.price));
                                    book.timestamp = Utc::now();

                                    Some(book.clone())
                                } else {
                                    None
                                }
                            };

                            // Broadcast updated orderbook
                            if let Some(book) = updated_book {
                                ws_state.broadcast_orderbook_update(
                                    Platform::Polymarket,
                                    market_id.clone(),
                                    book,
                                );
                            }

                            // Also broadcast price update if we have best bid/ask
                            if let (Some(bid), Some(_ask)) = (best_bid, best_ask) {
                                // YES price is typically the best bid
                                let yes_price = bid;
                                let no_price = Decimal::ONE - bid;

                                ws_state.broadcast_price_update(
                                    Platform::Polymarket,
                                    market_id,
                                    yes_price,
                                    no_price,
                                );
                            }
                        }
                        PolymarketUpdate::Trade { asset_id, trade } => {
                            let market_id = {
                                let map = token_map.read().await;
                                map.get(&asset_id)
                                    .cloned()
                                    .unwrap_or_else(|| asset_id.clone())
                            };

                            // Polymarket trade received

                            // Update trade with correct market_id
                            let mut trade = trade;
                            trade.market_id = market_id;

                            ws_state.broadcast_trade(trade);
                        }
                        PolymarketUpdate::ConnectionState { connected, error } => {
                            metrics.set_connected(connected);
                            if connected {
                                info!("[Aggregator] Polymarket WebSocket connected");
                            } else {
                                warn!(
                                    "[Aggregator] Polymarket WebSocket disconnected: {:?}",
                                    error
                                );
                            }
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!(
                        "[Aggregator] Polymarket update receiver lagged {} messages",
                        n
                    );
                }
                Err(broadcast::error::RecvError::Closed) => {
                    info!("[Aggregator] Polymarket update channel closed");
                    break;
                }
            }
        }
    }

    /// Check if a market is actively subscribed
    pub async fn is_subscribed(&self, platform: Platform, market_id: &str) -> bool {
        let subs = self.active_subscriptions.read().await;
        subs.get(&platform)
            .map(|set| set.contains(market_id))
            .unwrap_or(false)
    }

    /// Process subscription events from WebSocketState
    pub async fn process_subscription_events(
        &self,
        mut rx: tokio::sync::mpsc::Receiver<SubscriptionEvent>,
    ) {
        info!("[Aggregator] Starting subscription event processor");

        while let Some(event) = rx.recv().await {
            match event {
                SubscriptionEvent::Subscribe {
                    platform,
                    market_id,
                } => {
                    info!(
                        "[Aggregator] Received subscribe event for {:?}:{}",
                        platform, market_id
                    );
                    if let Err(e) = self.subscribe(platform, &market_id).await {
                        warn!(
                            "[Aggregator] Failed to subscribe to {:?}:{}: {}",
                            platform, market_id, e
                        );
                    }
                }
                SubscriptionEvent::Unsubscribe {
                    platform,
                    market_id,
                } => {
                    info!(
                        "[Aggregator] Received unsubscribe event for {:?}:{}",
                        platform, market_id
                    );
                    if let Err(e) = self.unsubscribe(platform, &market_id).await {
                        warn!(
                            "[Aggregator] Failed to unsubscribe from {:?}:{}: {}",
                            platform, market_id, e
                        );
                    }
                }
            }
        }

        info!("[Aggregator] Subscription event processor stopped");
    }
}

impl std::fmt::Debug for MarketDataAggregator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MarketDataAggregator")
            .field("config", &self.config)
            .finish()
    }
}
