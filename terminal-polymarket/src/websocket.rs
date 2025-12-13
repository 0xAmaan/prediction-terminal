//! Polymarket WebSocket client for real-time market data
//!
//! Connects to Polymarket's WebSocket API to receive live orderbook,
//! price, and trade updates via the market channel.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::interval;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

use terminal_core::{OrderBook, OrderBookLevel, Platform, Trade, TradeOutcome, TradeSide};
use crate::client::PolymarketCredentials;

/// Polymarket WebSocket URL (market channel - no auth required)
/// Note: The URL includes /ws/market path for the market channel subscription
const POLYMARKET_WS_URL: &str = "wss://ws-subscriptions-clob.polymarket.com/ws/market";

/// Reconnect delay base
const RECONNECT_DELAY_BASE: Duration = Duration::from_secs(1);

/// Max reconnect attempts
const MAX_RECONNECT_ATTEMPTS: u32 = 5;

/// Ping interval (Polymarket expects pings every 10s)
const PING_INTERVAL: Duration = Duration::from_secs(10);

// ============================================================================
// WebSocket Message Types (matching Polymarket's protocol)
// ============================================================================

/// Subscribe message for market channel
#[derive(Debug, Clone, Serialize)]
pub struct MarketSubscribeMessage {
    pub assets_ids: Vec<String>,
    #[serde(rename = "type")]
    pub msg_type: String,
}

/// Subscribe message for user channel (authenticated)
#[derive(Debug, Clone, Serialize)]
pub struct UserSubscribeMessage {
    pub markets: Vec<String>,
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthObject>,
}

/// Auth object for authenticated channels
#[derive(Debug, Clone, Serialize)]
pub struct AuthObject {
    #[serde(rename = "apiKey")]
    pub api_key: String,
    pub secret: String,
    pub passphrase: String,
}

/// Response from Polymarket WebSocket
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum PolymarketResponse {
    /// Book snapshot/update message
    Book(BookMessage),
    /// Price change message
    PriceChange(PriceChangeMessage),
    /// Last trade price message
    LastTradePrice(LastTradePriceMessage),
    /// Tick size change message
    TickSizeChange(TickSizeChangeMessage),
    /// Error message
    Error(ErrorMessage),
    /// Generic acknowledgment
    Ack(AckMessage),
}

#[derive(Debug, Clone, Deserialize)]
pub struct BookMessage {
    #[serde(rename = "event_type")]
    pub event_type: Option<String>,
    #[serde(rename = "asset_id")]
    pub asset_id: String,
    pub market: Option<String>,
    pub bids: Vec<BookLevel>,
    pub asks: Vec<BookLevel>,
    pub timestamp: Option<String>,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BookLevel {
    pub price: String,
    pub size: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PriceChangeMessage {
    #[serde(rename = "event_type")]
    pub event_type: Option<String>,
    #[serde(rename = "asset_id")]
    pub asset_id: String,
    pub changes: Vec<PriceChangeItem>,
    #[serde(rename = "best_bid")]
    pub best_bid: Option<String>,
    #[serde(rename = "best_ask")]
    pub best_ask: Option<String>,
    pub timestamp: Option<String>,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PriceChangeItem {
    pub price: String,
    pub size: String,
    pub side: String, // "BUY" or "SELL"
}

#[derive(Debug, Clone, Deserialize)]
pub struct LastTradePriceMessage {
    #[serde(rename = "event_type")]
    pub event_type: Option<String>,
    #[serde(rename = "asset_id")]
    pub asset_id: String,
    pub price: String,
    pub size: Option<String>,
    pub side: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TickSizeChangeMessage {
    #[serde(rename = "event_type")]
    pub event_type: Option<String>,
    #[serde(rename = "asset_id")]
    pub asset_id: String,
    #[serde(rename = "tick_size")]
    pub tick_size: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorMessage {
    pub error: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AckMessage {
    #[serde(rename = "type")]
    pub msg_type: Option<String>,
    pub message: Option<String>,
}

// ============================================================================
// Normalized Update Types (sent to aggregator)
// ============================================================================

/// Normalized update from Polymarket WebSocket
#[derive(Debug, Clone)]
pub enum PolymarketUpdate {
    /// Full orderbook snapshot
    OrderbookSnapshot {
        asset_id: String,
        orderbook: OrderBook,
    },
    /// Price change (orderbook delta)
    PriceChange {
        asset_id: String,
        changes: Vec<(Decimal, Decimal, String)>, // (price, size, side)
        best_bid: Option<Decimal>,
        best_ask: Option<Decimal>,
    },
    /// Last trade price
    Trade {
        asset_id: String,
        trade: Trade,
    },
    /// Connection state change
    ConnectionState {
        connected: bool,
        error: Option<String>,
    },
}

// ============================================================================
// Polymarket WebSocket Client
// ============================================================================

/// Configuration for PolymarketWebSocket
#[derive(Clone, Debug)]
pub struct PolymarketWebSocketConfig {
    pub credentials: Option<PolymarketCredentials>,
    pub auto_reconnect: bool,
    pub max_reconnect_attempts: u32,
}

impl Default for PolymarketWebSocketConfig {
    fn default() -> Self {
        Self {
            credentials: PolymarketCredentials::from_env(),
            auto_reconnect: true,
            max_reconnect_attempts: MAX_RECONNECT_ATTEMPTS,
        }
    }
}

/// Polymarket WebSocket client
pub struct PolymarketWebSocket {
    config: PolymarketWebSocketConfig,
    /// Channel to send updates to the aggregator
    update_tx: broadcast::Sender<PolymarketUpdate>,
    /// Currently subscribed asset IDs (token IDs)
    subscriptions: Arc<RwLock<HashSet<String>>>,
    /// Command sender to the WebSocket task
    command_tx: Option<mpsc::Sender<WebSocketCommand>>,
}

/// Commands sent to the WebSocket task
#[derive(Debug)]
pub enum WebSocketCommand {
    Subscribe { asset_ids: Vec<String> },
    Unsubscribe { asset_ids: Vec<String> },
}

impl PolymarketWebSocket {
    /// Create a new Polymarket WebSocket client
    pub fn new(config: PolymarketWebSocketConfig) -> (Self, broadcast::Receiver<PolymarketUpdate>) {
        let (update_tx, update_rx) = broadcast::channel(1024);

        (
            Self {
                config,
                update_tx,
                subscriptions: Arc::new(RwLock::new(HashSet::new())),
                command_tx: None,
            },
            update_rx,
        )
    }

    /// Start the WebSocket connection
    /// Note: Connection is lazy - it only connects when subscriptions are added
    pub async fn start(&mut self) -> Result<(), anyhow::Error> {
        let (command_tx, command_rx) = mpsc::channel::<WebSocketCommand>(100);
        self.command_tx = Some(command_tx);

        let config = self.config.clone();
        let update_tx = self.update_tx.clone();
        let subscriptions = Arc::clone(&self.subscriptions);

        tokio::spawn(async move {
            Self::connection_loop(config, update_tx, command_rx, subscriptions).await;
        });

        Ok(())
    }

    /// Main connection loop with reconnection logic
    /// Note: This uses lazy connection - waits for first subscription before connecting
    async fn connection_loop(
        config: PolymarketWebSocketConfig,
        update_tx: broadcast::Sender<PolymarketUpdate>,
        mut command_rx: mpsc::Receiver<WebSocketCommand>,
        subscriptions: Arc<RwLock<HashSet<String>>>,
    ) {
        let mut reconnect_attempts = 0u32;
        let mut pending_subscribe: Option<Vec<String>>;

        // Wait for first subscription command before connecting
        // This prevents the server from closing idle connections
        info!("[Polymarket WS] Waiting for subscriptions before connecting...");
        loop {
            match command_rx.recv().await {
                Some(WebSocketCommand::Subscribe { asset_ids }) => {
                    if !asset_ids.is_empty() {
                        info!("[Polymarket WS] Got subscription request, will connect");
                        pending_subscribe = Some(asset_ids);
                        break;
                    }
                }
                Some(WebSocketCommand::Unsubscribe { .. }) => {
                    // Ignore unsubscribe before connected
                }
                None => {
                    info!("[Polymarket WS] Command channel closed, exiting");
                    return;
                }
            }
        }

        // Now we have a subscription, enter the main connection loop
        loop {
            info!("[Polymarket WS] Connecting to {}", POLYMARKET_WS_URL);

            match connect_async(POLYMARKET_WS_URL).await {
                Ok((ws_stream, _)) => {
                    info!("[Polymarket WS] Connected successfully");
                    reconnect_attempts = 0;

                    // Notify connected
                    let _ = update_tx.send(PolymarketUpdate::ConnectionState {
                        connected: true,
                        error: None,
                    });

                    let (mut write, mut read) = ws_stream.split();

                    // Send pending subscription first (if any)
                    if let Some(asset_ids) = pending_subscribe.take() {
                        let msg = MarketSubscribeMessage {
                            assets_ids: asset_ids,
                            msg_type: "market".to_string(),
                        };
                        if let Ok(json) = serde_json::to_string(&msg) {
                            info!("[Polymarket WS] Subscribing: {}", json);
                            if let Err(e) = write.send(Message::Text(json.into())).await {
                                warn!("[Polymarket WS] Failed to subscribe: {}", e);
                            }
                        }
                    }

                    // Re-subscribe to any active subscriptions (on reconnect)
                    {
                        let subs = subscriptions.read().await;
                        if !subs.is_empty() {
                            let asset_ids: Vec<String> = subs.iter().cloned().collect();
                            let msg = MarketSubscribeMessage {
                                assets_ids: asset_ids,
                                msg_type: "market".to_string(),
                            };

                            if let Ok(json) = serde_json::to_string(&msg) {
                                // Re-subscribing after reconnect
                                if let Err(e) = write.send(Message::Text(json.into())).await {
                                    warn!("[Polymarket WS] Failed to re-subscribe: {}", e);
                                }
                            }
                        }
                    }

                    // Create ping interval - start after first successful subscribe
                    let mut ping_timer = interval(PING_INTERVAL);

                    loop {
                        tokio::select! {
                            // Handle incoming messages
                            msg = read.next() => {
                                match msg {
                                    Some(Ok(Message::Text(text))) => {
                                        Self::handle_message(&text, &update_tx);
                                    }
                                    Some(Ok(Message::Ping(data))) => {
                                        // Respond to ping
                                        if let Err(e) = write.send(Message::Pong(data)).await {
                                            warn!("[Polymarket WS] Failed to send pong: {}", e);
                                            break;
                                        }
                                    }
                                    Some(Ok(Message::Close(_))) => {
                                        info!("[Polymarket WS] Connection closed by server");
                                        break;
                                    }
                                    Some(Err(e)) => {
                                        error!("[Polymarket WS] Error: {}", e);
                                        break;
                                    }
                                    None => {
                                        info!("[Polymarket WS] Stream ended");
                                        break;
                                    }
                                    _ => {}
                                }
                            }

                            // Handle outgoing commands
                            cmd = command_rx.recv() => {
                                if let Some(cmd) = cmd {
                                    match cmd {
                                        WebSocketCommand::Subscribe { asset_ids } => {
                                            let msg = MarketSubscribeMessage {
                                                assets_ids: asset_ids,
                                                msg_type: "market".to_string(),
                                            };
                                            if let Ok(json) = serde_json::to_string(&msg) {
                                                // Subscribing to market
                                                if let Err(e) = write.send(Message::Text(json.into())).await {
                                                    warn!("[Polymarket WS] Failed to send subscribe: {}", e);
                                                    break;
                                                }
                                            }
                                        }
                                        WebSocketCommand::Unsubscribe { asset_ids: _ } => {
                                            // Polymarket doesn't have explicit unsubscribe
                                            // We just stop processing updates for those assets
                                            // Unsubscribe (local only - Polymarket doesn't support server-side unsub)
                                        }
                                    }
                                }
                            }

                            // Send periodic pings to keep connection alive
                            _ = ping_timer.tick() => {
                                // Send keepalive ping
                                if let Err(e) = write.send(Message::Text("PING".to_string().into())).await {
                                    warn!("[Polymarket WS] Failed to send ping: {}", e);
                                    break;
                                }
                            }
                        }
                    }

                    // Notify disconnected
                    let _ = update_tx.send(PolymarketUpdate::ConnectionState {
                        connected: false,
                        error: None,
                    });
                }
                Err(e) => {
                    error!("[Polymarket WS] Connection failed: {}", e);
                    let _ = update_tx.send(PolymarketUpdate::ConnectionState {
                        connected: false,
                        error: Some(e.to_string()),
                    });
                }
            }

            // Reconnection logic
            if !config.auto_reconnect {
                break;
            }

            reconnect_attempts += 1;
            if reconnect_attempts > config.max_reconnect_attempts {
                error!("[Polymarket WS] Max reconnect attempts reached");
                break;
            }

            let delay = RECONNECT_DELAY_BASE * 2u32.pow(reconnect_attempts - 1);
            info!("[Polymarket WS] Reconnecting in {:?} (attempt {})", delay, reconnect_attempts);
            tokio::time::sleep(delay).await;
        }
    }

    /// Handle an incoming message from the WebSocket
    fn handle_message(text: &str, update_tx: &broadcast::Sender<PolymarketUpdate>) {
        // Skip ping/pong responses
        if text == "PONG" || text.is_empty() {
            return;
        }

        // Try to detect message type by looking for key fields
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
            // Check for error
            if json.get("error").is_some() {
                if let Ok(err) = serde_json::from_value::<ErrorMessage>(json) {
                    error!("[Polymarket WS] Error: {} - {:?}", err.error, err.message);
                }
                return;
            }

            // Check event_type if present
            let event_type = json.get("event_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match event_type {
                "book" | "" if json.get("bids").is_some() && json.get("asks").is_some() => {
                    if let Ok(book) = serde_json::from_value::<BookMessage>(json) {
                        debug!("[Polymarket WS] Book update for {}", book.asset_id);
                        let orderbook = Self::convert_book_message(&book);
                        let _ = update_tx.send(PolymarketUpdate::OrderbookSnapshot {
                            asset_id: book.asset_id,
                            orderbook,
                        });
                    }
                }
                "price_change" | "" if json.get("changes").is_some() => {
                    if let Ok(price_change) = serde_json::from_value::<PriceChangeMessage>(json) {
                        debug!("[Polymarket WS] Price change for {}", price_change.asset_id);
                        let changes: Vec<(Decimal, Decimal, String)> = price_change.changes
                            .iter()
                            .filter_map(|c| {
                                let price = c.price.parse::<Decimal>().ok()?;
                                let size = c.size.parse::<Decimal>().ok()?;
                                Some((price, size, c.side.clone()))
                            })
                            .collect();

                        let _ = update_tx.send(PolymarketUpdate::PriceChange {
                            asset_id: price_change.asset_id,
                            changes,
                            best_bid: price_change.best_bid.and_then(|b| b.parse().ok()),
                            best_ask: price_change.best_ask.and_then(|a| a.parse().ok()),
                        });
                    }
                }
                "last_trade_price" | "" if json.get("price").is_some() && !json.get("bids").is_some() => {
                    if let Ok(trade) = serde_json::from_value::<LastTradePriceMessage>(json) {
                        debug!("[Polymarket WS] Last trade price for {}", trade.asset_id);
                        let trade_obj = Self::convert_trade_message(&trade);
                        let _ = update_tx.send(PolymarketUpdate::Trade {
                            asset_id: trade.asset_id,
                            trade: trade_obj,
                        });
                    }
                }
                _ => {
                    debug!("[Polymarket WS] Unknown message type: {}", text);
                }
            }
        } else {
            debug!("[Polymarket WS] Failed to parse message: {}", text);
        }
    }

    /// Convert book message to internal OrderBook
    fn convert_book_message(msg: &BookMessage) -> OrderBook {
        let mut orderbook = OrderBook::new(msg.asset_id.clone(), Platform::Polymarket);

        // Convert bids (buy orders)
        for level in &msg.bids {
            if let (Ok(price), Ok(size)) = (level.price.parse::<Decimal>(), level.size.parse::<Decimal>()) {
                orderbook.yes_bids.push(OrderBookLevel::new(price, size));
            }
        }

        // Convert asks (sell orders)
        for level in &msg.asks {
            if let (Ok(price), Ok(size)) = (level.price.parse::<Decimal>(), level.size.parse::<Decimal>()) {
                orderbook.yes_asks.push(OrderBookLevel::new(price, size));
            }
        }

        // Sort bids by price descending, asks by price ascending
        orderbook.yes_bids.sort_by(|a, b| b.price.cmp(&a.price));
        orderbook.yes_asks.sort_by(|a, b| a.price.cmp(&b.price));

        orderbook.timestamp = Utc::now();
        orderbook
    }

    /// Convert last trade price message to internal Trade
    fn convert_trade_message(msg: &LastTradePriceMessage) -> Trade {
        let timestamp = msg.timestamp
            .as_ref()
            .and_then(|ts| ts.parse::<i64>().ok())
            .map(|ts| {
                if ts > 10_000_000_000 {
                    chrono::DateTime::from_timestamp(ts / 1000, ((ts % 1000) * 1_000_000) as u32)
                } else {
                    chrono::DateTime::from_timestamp(ts, 0)
                }
            })
            .flatten()
            .unwrap_or_else(Utc::now);

        let price = msg.price.parse::<Decimal>().unwrap_or(Decimal::ZERO);
        let quantity = msg.size
            .as_ref()
            .and_then(|s| s.parse::<Decimal>().ok())
            .unwrap_or(Decimal::ONE);

        let side = msg.side.as_ref().map(|s| {
            if s.to_uppercase() == "BUY" { TradeSide::Buy } else { TradeSide::Sell }
        });

        Trade {
            id: format!("{}-{}", msg.asset_id, timestamp.timestamp_millis()),
            market_id: msg.asset_id.clone(),
            platform: Platform::Polymarket,
            timestamp,
            price,
            quantity,
            outcome: TradeOutcome::Yes, // Polymarket uses token for outcome
            side,
            transaction_hash: None, // WebSocket last_trade_price doesn't include tx hash
        }
    }

    /// Subscribe to market updates for given asset IDs (token IDs)
    pub async fn subscribe(&self, asset_ids: Vec<String>) -> Result<(), anyhow::Error> {
        // Track subscriptions
        {
            let mut subs = self.subscriptions.write().await;
            for id in &asset_ids {
                subs.insert(id.clone());
            }
        }

        // Send subscribe command
        if let Some(ref tx) = self.command_tx {
            tx.send(WebSocketCommand::Subscribe { asset_ids }).await?;
        }

        Ok(())
    }

    /// Unsubscribe from market updates for given asset IDs
    pub async fn unsubscribe(&self, asset_ids: Vec<String>) -> Result<(), anyhow::Error> {
        // Remove from tracked subscriptions
        {
            let mut subs = self.subscriptions.write().await;
            for id in &asset_ids {
                subs.remove(id);
            }
        }

        // Send unsubscribe command
        if let Some(ref tx) = self.command_tx {
            tx.send(WebSocketCommand::Unsubscribe { asset_ids }).await?;
        }

        Ok(())
    }

    /// Get the update receiver (for cloning to other tasks)
    pub fn subscribe_updates(&self) -> broadcast::Receiver<PolymarketUpdate> {
        self.update_tx.subscribe()
    }
}

impl std::fmt::Debug for PolymarketWebSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PolymarketWebSocket")
            .field("config", &self.config)
            .finish()
    }
}
