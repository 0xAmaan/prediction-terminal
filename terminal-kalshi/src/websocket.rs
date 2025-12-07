//! Kalshi WebSocket client for real-time market data
//!
//! Connects to Kalshi's WebSocket API to receive live orderbook,
//! ticker, and trade updates.
//!
//! Authentication: Uses RSA-PSS signing with SHA256 for WebSocket headers.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use rand;
use rsa::pkcs8::DecodePrivateKey;
use rsa::pss::BlindedSigningKey;
use rsa::signature::{RandomizedSigner, SignatureEncoding};
use rsa::RsaPrivateKey;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::interval;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

use terminal_core::{OrderBook, OrderBookLevel, Platform, Trade, TradeOutcome, TradeSide};

/// Kalshi WebSocket URL
const KALSHI_WS_URL: &str = "wss://api.elections.kalshi.com/trade-api/ws/v2";

/// Reconnect delay base
const RECONNECT_DELAY_BASE: Duration = Duration::from_secs(1);

/// Max reconnect attempts
const MAX_RECONNECT_ATTEMPTS: u32 = 5;

// ============================================================================
// WebSocket Message Types (matching Kalshi's protocol)
// ============================================================================

/// Command sent to Kalshi WebSocket
#[derive(Debug, Clone, Serialize)]
pub struct KalshiCommand {
    pub id: u64,
    pub cmd: String,
    pub params: KalshiCommandParams,
}

/// Parameters for subscribe/unsubscribe commands
#[derive(Debug, Clone, Serialize)]
pub struct KalshiCommandParams {
    pub channels: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_tickers: Option<Vec<String>>,
}

/// Response from Kalshi WebSocket
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KalshiResponse {
    /// Subscription confirmed
    Subscribed { id: u64, msg: SubscribedMsg },
    /// Unsubscription confirmed
    Unsubscribed { id: u64, msg: UnsubscribedMsg },
    /// Orderbook snapshot (initial state)
    OrderbookSnapshot {
        #[serde(default)]
        sid: Option<u64>,
        msg: OrderbookSnapshotMsg,
    },
    /// Orderbook delta (incremental update)
    OrderbookDelta {
        #[serde(default)]
        sid: Option<u64>,
        msg: OrderbookDeltaMsg,
    },
    /// Ticker update (price, volume)
    Ticker {
        #[serde(default)]
        sid: Option<u64>,
        msg: TickerMsg,
    },
    /// Trade executed
    Trade {
        #[serde(default)]
        sid: Option<u64>,
        msg: TradeMsg,
    },
    /// Error message
    Error { id: Option<u64>, msg: ErrorMsg },
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubscribedMsg {
    pub channel: String,
    pub sid: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnsubscribedMsg {
    pub channel: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorMsg {
    pub code: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookSnapshotMsg {
    pub market_ticker: String,
    pub yes: Vec<OrderbookLevel>,
    pub no: Vec<OrderbookLevel>,
    #[serde(default)]
    pub seq: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookDeltaMsg {
    pub market_ticker: String,
    pub price: i64,   // Price in cents (1-99)
    pub delta: i64,   // Quantity change (positive or negative)
    pub side: String, // "yes" or "no"
    #[serde(default)]
    pub seq: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderbookLevel {
    pub price: i64,    // Price in cents
    pub quantity: i64, // Number of contracts
}

#[derive(Debug, Clone, Deserialize)]
pub struct TickerMsg {
    pub market_ticker: String,
    pub yes_price: Option<i64>,
    pub no_price: Option<i64>,
    pub yes_bid: Option<i64>,
    pub yes_ask: Option<i64>,
    pub volume: Option<i64>,
    pub open_interest: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TradeMsg {
    pub market_ticker: String,
    pub trade_id: String,
    pub side: String, // "yes" or "no"
    pub price: i64,   // Price in cents
    pub count: i64,   // Number of contracts
    #[serde(default)]
    pub taker_side: Option<String>, // "buy" or "sell"
    #[serde(default)]
    pub ts: Option<i64>, // Timestamp
}

// ============================================================================
// Normalized Update Types (sent to aggregator)
// ============================================================================

/// Normalized update from Kalshi WebSocket
#[derive(Debug, Clone)]
pub enum KalshiUpdate {
    /// Full orderbook snapshot
    OrderbookSnapshot {
        market_ticker: String,
        orderbook: OrderBook,
    },
    /// Incremental orderbook update
    OrderbookDelta {
        market_ticker: String,
        side: String,
        price: Decimal,
        delta: Decimal,
        seq: Option<u64>,
    },
    /// Price/ticker update
    PriceUpdate {
        market_ticker: String,
        yes_price: Option<Decimal>,
        no_price: Option<Decimal>,
        volume: Option<Decimal>,
    },
    /// Trade executed
    Trade { market_ticker: String, trade: Trade },
    /// Connection state change
    ConnectionState {
        connected: bool,
        error: Option<String>,
    },
}

// ============================================================================
// Kalshi WebSocket Client
// ============================================================================

/// Configuration for KalshiWebSocket
#[derive(Clone)]
pub struct KalshiWebSocketConfig {
    pub api_key: Option<String>,
    pub private_key_pem: Option<String>,
    pub auto_reconnect: bool,
    pub max_reconnect_attempts: u32,
}

impl std::fmt::Debug for KalshiWebSocketConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KalshiWebSocketConfig")
            .field("api_key", &self.api_key.as_ref().map(|_| "[REDACTED]"))
            .field(
                "private_key_pem",
                &self.private_key_pem.as_ref().map(|_| "[REDACTED]"),
            )
            .field("auto_reconnect", &self.auto_reconnect)
            .field("max_reconnect_attempts", &self.max_reconnect_attempts)
            .finish()
    }
}

impl Default for KalshiWebSocketConfig {
    fn default() -> Self {
        // Load private key: try file path first, then direct env var
        let private_key_pem = if let Ok(path) = std::env::var("KALSHI_PRIVATE_KEY_FILE") {
            match std::fs::read_to_string(&path) {
                Ok(contents) => {
                    info!("[Kalshi] Loaded private key from file: {}", path);
                    Some(contents)
                }
                Err(e) => {
                    warn!("[Kalshi] Failed to read private key file '{}': {}", path, e);
                    None
                }
            }
        } else {
            std::env::var("KALSHI_PRIVATE_KEY").ok()
        };

        Self {
            api_key: std::env::var("KALSHI_API_KEY").ok(),
            private_key_pem,
            auto_reconnect: true,
            max_reconnect_attempts: MAX_RECONNECT_ATTEMPTS,
        }
    }
}

/// Sign a message using RSA-PSS with SHA256
fn sign_rsa_pss(private_key_input: &str, message: &str) -> Result<String, anyhow::Error> {
    use rsa::pkcs1::DecodeRsaPrivateKey;

    // Try to parse as PEM first
    let private_key = if private_key_input.contains("-----BEGIN") {
        // Already in PEM format - try PKCS#8 first, then PKCS#1
        RsaPrivateKey::from_pkcs8_pem(private_key_input)
            .or_else(|_| RsaPrivateKey::from_pkcs1_pem(private_key_input))
            .map_err(|e| anyhow::anyhow!("Failed to parse PEM private key: {}", e))?
    } else {
        // Raw base64 - try both PKCS#8 and PKCS#1 formats
        let clean_b64: String = private_key_input
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();

        // Try PKCS#8 format first (BEGIN PRIVATE KEY)
        let pkcs8_pem = format_as_pem(&clean_b64, "PRIVATE KEY");
        if let Ok(key) = RsaPrivateKey::from_pkcs8_pem(&pkcs8_pem) {
            key
        } else {
            // Try PKCS#1 format (BEGIN RSA PRIVATE KEY) - this is what Kalshi uses
            let pkcs1_pem = format_as_pem(&clean_b64, "RSA PRIVATE KEY");
            RsaPrivateKey::from_pkcs1_pem(&pkcs1_pem).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to parse private key (tried PKCS#8 and PKCS#1): {}",
                    e
                )
            })?
        }
    };

    // Create a blinded signing key with SHA256 for PSS
    let signing_key = BlindedSigningKey::<Sha256>::new(private_key);

    // Sign the message with randomized signer
    let mut rng = rand::thread_rng();
    let signature = signing_key.sign_with_rng(&mut rng, message.as_bytes());

    // Base64 encode the signature
    Ok(BASE64.encode(signature.to_bytes()))
}

/// Format base64 content as PEM with proper headers and line breaks
fn format_as_pem(b64_content: &str, key_type: &str) -> String {
    let mut pem = format!("-----BEGIN {}-----\n", key_type);
    for chunk in b64_content.as_bytes().chunks(64) {
        pem.push_str(std::str::from_utf8(chunk).unwrap_or(""));
        pem.push('\n');
    }
    pem.push_str(&format!("-----END {}-----\n", key_type));
    pem
}

/// Create authentication headers for Kalshi WebSocket
fn create_auth_headers(
    api_key: &str,
    private_key_pem: &str,
) -> Result<(String, String, String), anyhow::Error> {
    // Timestamp in milliseconds
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string();

    // Message to sign: timestamp + method + path
    let message = format!("{}GET/trade-api/ws/v2", timestamp);

    // Sign the message
    let signature = sign_rsa_pss(private_key_pem, &message)?;

    Ok((api_key.to_string(), signature, timestamp))
}

/// Kalshi WebSocket client
pub struct KalshiWebSocket {
    config: KalshiWebSocketConfig,
    /// Channel to send updates to the aggregator
    update_tx: broadcast::Sender<KalshiUpdate>,
    /// Currently subscribed tickers
    subscriptions: Arc<RwLock<HashSet<String>>>,
    /// Command sender to the WebSocket task
    command_tx: Option<mpsc::Sender<KalshiCommand>>,
    /// Next command ID
    next_id: Arc<RwLock<u64>>,
}

impl KalshiWebSocket {
    /// Create a new Kalshi WebSocket client
    pub fn new(config: KalshiWebSocketConfig) -> (Self, broadcast::Receiver<KalshiUpdate>) {
        let (update_tx, update_rx) = broadcast::channel(1024);

        (
            Self {
                config,
                update_tx,
                subscriptions: Arc::new(RwLock::new(HashSet::new())),
                command_tx: None,
                next_id: Arc::new(RwLock::new(1)),
            },
            update_rx,
        )
    }

    /// Start the WebSocket connection
    pub async fn start(&mut self) -> Result<(), anyhow::Error> {
        // Check if we have both API key AND private key - Kalshi WS requires RSA-PSS auth
        if self.config.api_key.is_none() || self.config.private_key_pem.is_none() {
            warn!(
                "[Kalshi WS] Missing credentials. Need both KALSHI_API_KEY and KALSHI_PRIVATE_KEY."
            );
            if self.config.api_key.is_none() {
                warn!("[Kalshi WS] KALSHI_API_KEY not set");
            }
            if self.config.private_key_pem.is_none() {
                warn!("[Kalshi WS] KALSHI_PRIVATE_KEY not set");
            }
            // Still set up the command channel so subscribe/unsubscribe don't panic
            let (command_tx, _command_rx) = mpsc::channel::<KalshiCommand>(100);
            self.command_tx = Some(command_tx);
            return Ok(());
        }

        let (command_tx, command_rx) = mpsc::channel::<KalshiCommand>(100);
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
    async fn connection_loop(
        config: KalshiWebSocketConfig,
        update_tx: broadcast::Sender<KalshiUpdate>,
        mut command_rx: mpsc::Receiver<KalshiCommand>,
        subscriptions: Arc<RwLock<HashSet<String>>>,
    ) {
        let mut reconnect_attempts = 0u32;

        // Get credentials (already validated in start())
        let api_key = match &config.api_key {
            Some(k) => k.clone(),
            None => {
                error!("[Kalshi WS] No API key configured");
                return;
            }
        };
        let private_key_pem = match &config.private_key_pem {
            Some(k) => k.clone(),
            None => {
                error!("[Kalshi WS] No private key configured");
                return;
            }
        };

        loop {
            info!("[Kalshi WS] Connecting to {}", KALSHI_WS_URL);

            // Create authentication headers using RSA-PSS signing
            let auth_result = create_auth_headers(&api_key, &private_key_pem);
            let (key, signature, timestamp) = match auth_result {
                Ok(headers) => headers,
                Err(e) => {
                    error!("[Kalshi WS] Failed to create auth headers: {}", e);
                    let _ = update_tx.send(KalshiUpdate::ConnectionState {
                        connected: false,
                        error: Some(format!("Auth error: {}", e)),
                    });
                    // Don't retry if auth fails - it's a configuration issue
                    return;
                }
            };

            // Build WebSocket request with auth headers
            let mut request = match KALSHI_WS_URL.into_client_request() {
                Ok(r) => r,
                Err(e) => {
                    error!("[Kalshi WS] Failed to build request: {}", e);
                    return;
                }
            };

            // Add Kalshi authentication headers
            let headers = request.headers_mut();
            headers.insert("KALSHI-ACCESS-KEY", key.parse().unwrap());
            headers.insert("KALSHI-ACCESS-SIGNATURE", signature.parse().unwrap());
            headers.insert("KALSHI-ACCESS-TIMESTAMP", timestamp.parse().unwrap());

            // Auth headers created successfully

            match connect_async(request).await {
                Ok((ws_stream, _)) => {
                    info!("[Kalshi WS] Connected successfully");
                    reconnect_attempts = 0;

                    // Notify connected
                    let _ = update_tx.send(KalshiUpdate::ConnectionState {
                        connected: true,
                        error: None,
                    });

                    let (mut write, mut read) = ws_stream.split();

                    // Re-subscribe to any active subscriptions
                    {
                        let subs = subscriptions.read().await;
                        if !subs.is_empty() {
                            let tickers: Vec<String> = subs.iter().cloned().collect();
                            let cmd = KalshiCommand {
                                id: 1,
                                cmd: "subscribe".to_string(),
                                params: KalshiCommandParams {
                                    channels: vec![
                                        "orderbook_delta".to_string(),
                                        "ticker".to_string(),
                                        "trade".to_string(),
                                    ],
                                    market_ticker: None,
                                    market_tickers: Some(tickers),
                                },
                            };

                            if let Ok(json) = serde_json::to_string(&cmd) {
                                if let Err(e) = write.send(Message::Text(json.into())).await {
                                    warn!("[Kalshi WS] Failed to re-subscribe: {}", e);
                                }
                            }
                        }
                    }

                    // Create heartbeat interval
                    let mut heartbeat = interval(Duration::from_secs(25));

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
                                            warn!("[Kalshi WS] Failed to send pong: {}", e);
                                            break;
                                        }
                                    }
                                    Some(Ok(Message::Close(_))) => {
                                        info!("[Kalshi WS] Connection closed by server");
                                        break;
                                    }
                                    Some(Err(e)) => {
                                        error!("[Kalshi WS] Error: {}", e);
                                        break;
                                    }
                                    None => {
                                        info!("[Kalshi WS] Stream ended");
                                        break;
                                    }
                                    _ => {}
                                }
                            }

                            // Handle outgoing commands
                            cmd = command_rx.recv() => {
                                if let Some(cmd) = cmd {
                                    if let Ok(json) = serde_json::to_string(&cmd) {
                                        // Send command
                                        if let Err(e) = write.send(Message::Text(json.into())).await {
                                            warn!("[Kalshi WS] Failed to send command: {}", e);
                                            break;
                                        }
                                    }
                                }
                            }

                            // Send periodic pings (Kalshi sends heartbeat every 10s, we respond)
                            _ = heartbeat.tick() => {
                                // Just keep the connection alive by checking read state
                                debug!("[Kalshi WS] Heartbeat tick");
                            }
                        }
                    }

                    // Notify disconnected
                    let _ = update_tx.send(KalshiUpdate::ConnectionState {
                        connected: false,
                        error: None,
                    });
                }
                Err(e) => {
                    error!("[Kalshi WS] Connection failed: {}", e);
                    let _ = update_tx.send(KalshiUpdate::ConnectionState {
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
                error!("[Kalshi WS] Max reconnect attempts reached");
                break;
            }

            let delay = RECONNECT_DELAY_BASE * 2u32.pow(reconnect_attempts - 1);
            info!(
                "[Kalshi WS] Reconnecting in {:?} (attempt {})",
                delay, reconnect_attempts
            );
            tokio::time::sleep(delay).await;
        }
    }

    /// Handle an incoming message from the WebSocket
    fn handle_message(text: &str, update_tx: &broadcast::Sender<KalshiUpdate>) {
        match serde_json::from_str::<KalshiResponse>(text) {
            Ok(response) => match response {
                KalshiResponse::Subscribed { id, msg } => {
                    info!(
                        "[Kalshi WS] Subscribed to {} (id: {}, sid: {})",
                        msg.channel, id, msg.sid
                    );
                }
                KalshiResponse::Unsubscribed { id, msg } => {
                    info!("[Kalshi WS] Unsubscribed from {} (id: {})", msg.channel, id);
                }
                KalshiResponse::OrderbookSnapshot { msg, .. } => {
                    debug!("[Kalshi WS] Orderbook snapshot for {}", msg.market_ticker);
                    let orderbook = Self::convert_orderbook_snapshot(&msg);
                    let _ = update_tx.send(KalshiUpdate::OrderbookSnapshot {
                        market_ticker: msg.market_ticker,
                        orderbook,
                    });
                }
                KalshiResponse::OrderbookDelta { msg, .. } => {
                    debug!(
                        "[Kalshi WS] Orderbook delta for {} @ {}",
                        msg.market_ticker, msg.price
                    );
                    let _ = update_tx.send(KalshiUpdate::OrderbookDelta {
                        market_ticker: msg.market_ticker,
                        side: msg.side,
                        price: Decimal::from(msg.price) / Decimal::from(100),
                        delta: Decimal::from(msg.delta),
                        seq: msg.seq,
                    });
                }
                KalshiResponse::Ticker { msg, .. } => {
                    debug!("[Kalshi WS] Ticker update for {}", msg.market_ticker);
                    let _ = update_tx.send(KalshiUpdate::PriceUpdate {
                        market_ticker: msg.market_ticker,
                        yes_price: msg.yes_price.map(|p| Decimal::from(p) / Decimal::from(100)),
                        no_price: msg.no_price.map(|p| Decimal::from(p) / Decimal::from(100)),
                        volume: msg.volume.map(Decimal::from),
                    });
                }
                KalshiResponse::Trade { msg, .. } => {
                    debug!("[Kalshi WS] Trade on {} @ {}", msg.market_ticker, msg.price);
                    let trade = Self::convert_trade(&msg);
                    let _ = update_tx.send(KalshiUpdate::Trade {
                        market_ticker: msg.market_ticker,
                        trade,
                    });
                }
                KalshiResponse::Error { msg, .. } => {
                    error!("[Kalshi WS] Error: {:?} - {}", msg.code, msg.message);
                }
            },
            Err(e) => {
                // Try parsing as raw JSON for debugging
                debug!("[Kalshi WS] Unknown message: {} (error: {})", text, e);
            }
        }
    }

    /// Convert Kalshi orderbook snapshot to internal OrderBook
    fn convert_orderbook_snapshot(msg: &OrderbookSnapshotMsg) -> OrderBook {
        let mut orderbook = OrderBook::new(msg.market_ticker.clone(), Platform::Kalshi);

        // Convert YES levels (bids are buy orders, asks are sell orders)
        // Kalshi represents orderbook differently - need to map correctly
        for level in &msg.yes {
            let price = Decimal::from(level.price) / Decimal::from(100);
            let quantity = Decimal::from(level.quantity);
            // Positive quantity = bid, negative = ask (or vice versa based on Kalshi's format)
            if level.quantity > 0 {
                orderbook
                    .yes_bids
                    .push(OrderBookLevel::new(price, quantity));
            }
        }

        for level in &msg.no {
            let price = Decimal::from(level.price) / Decimal::from(100);
            let quantity = Decimal::from(level.quantity);
            if level.quantity > 0 {
                orderbook.no_bids.push(OrderBookLevel::new(price, quantity));
            }
        }

        // Sort bids by price descending, asks by price ascending
        orderbook.yes_bids.sort_by(|a, b| b.price.cmp(&a.price));
        orderbook.no_bids.sort_by(|a, b| b.price.cmp(&a.price));

        orderbook.timestamp = Utc::now();
        orderbook
    }

    /// Convert Kalshi trade message to internal Trade
    fn convert_trade(msg: &TradeMsg) -> Trade {
        let timestamp = msg
            .ts
            .and_then(|ts| {
                chrono::DateTime::from_timestamp(ts / 1000, ((ts % 1000) * 1_000_000) as u32)
            })
            .unwrap_or_else(Utc::now);

        Trade {
            id: msg.trade_id.clone(),
            market_id: msg.market_ticker.clone(),
            platform: Platform::Kalshi,
            timestamp,
            price: Decimal::from(msg.price) / Decimal::from(100),
            quantity: Decimal::from(msg.count),
            outcome: if msg.side == "yes" {
                TradeOutcome::Yes
            } else {
                TradeOutcome::No
            },
            side: msg.taker_side.as_ref().map(|s| {
                if s == "buy" {
                    TradeSide::Buy
                } else {
                    TradeSide::Sell
                }
            }),
        }
    }

    /// Subscribe to a market
    pub async fn subscribe(&self, market_ticker: &str) -> Result<(), anyhow::Error> {
        // Track subscription
        {
            let mut subs = self.subscriptions.write().await;
            subs.insert(market_ticker.to_string());
        }

        // Send subscribe command
        if let Some(ref tx) = self.command_tx {
            let id = {
                let mut next = self.next_id.write().await;
                let id = *next;
                *next += 1;
                id
            };

            let cmd = KalshiCommand {
                id,
                cmd: "subscribe".to_string(),
                params: KalshiCommandParams {
                    channels: vec![
                        "orderbook_delta".to_string(),
                        "ticker".to_string(),
                        "trade".to_string(),
                    ],
                    market_ticker: Some(market_ticker.to_string()),
                    market_tickers: None,
                },
            };

            tx.send(cmd).await?;
        }

        Ok(())
    }

    /// Unsubscribe from a market
    pub async fn unsubscribe(&self, market_ticker: &str) -> Result<(), anyhow::Error> {
        // Remove from tracked subscriptions
        {
            let mut subs = self.subscriptions.write().await;
            subs.remove(market_ticker);
        }

        // Send unsubscribe command
        if let Some(ref tx) = self.command_tx {
            let id = {
                let mut next = self.next_id.write().await;
                let id = *next;
                *next += 1;
                id
            };

            let cmd = KalshiCommand {
                id,
                cmd: "unsubscribe".to_string(),
                params: KalshiCommandParams {
                    channels: vec![
                        "orderbook_delta".to_string(),
                        "ticker".to_string(),
                        "trade".to_string(),
                    ],
                    market_ticker: Some(market_ticker.to_string()),
                    market_tickers: None,
                },
            };

            tx.send(cmd).await?;
        }

        Ok(())
    }

    /// Get the update receiver (for cloning to other tasks)
    pub fn subscribe_updates(&self) -> broadcast::Receiver<KalshiUpdate> {
        self.update_tx.subscribe()
    }
}

impl std::fmt::Debug for KalshiWebSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KalshiWebSocket")
            .field("config", &self.config)
            .finish()
    }
}
