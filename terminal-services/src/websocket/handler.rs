//! WebSocket connection handler
//!
//! Handles individual WebSocket connections, message parsing,
//! and subscription management.

use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use terminal_core::{
    ClientMessage, ErrorCode, MarketNewsContext, NewsItem, Platform, ServerMessage,
    SubscriptionChannel, SubscriptionKey,
};
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, error, info, warn};

use super::subscription::{BroadcastMessage, ClientId, SubscriptionManager};
use crate::MarketService;

/// Subscription event for notifying the aggregator
#[derive(Debug, Clone)]
pub enum SubscriptionEvent {
    /// A client subscribed to a market
    Subscribe {
        platform: Platform,
        market_id: String,
    },
    /// A client unsubscribed from a market (when no clients remain)
    Unsubscribe {
        platform: Platform,
        market_id: String,
    },
}

/// Shared state for WebSocket handlers
#[derive(Clone)]
pub struct WebSocketState {
    /// Subscription manager
    pub subscriptions: Arc<SubscriptionManager>,
    /// Market service for data fetching
    pub market_service: MarketService,
    /// Channel to notify aggregator of subscription changes
    subscription_event_tx: Option<mpsc::Sender<SubscriptionEvent>>,
}

impl WebSocketState {
    /// Create new WebSocket state
    pub fn new(market_service: MarketService) -> Self {
        Self {
            subscriptions: Arc::new(SubscriptionManager::new()),
            market_service,
            subscription_event_tx: None,
        }
    }

    /// Set the subscription event sender (for aggregator integration)
    pub fn set_subscription_event_sender(&mut self, tx: mpsc::Sender<SubscriptionEvent>) {
        self.subscription_event_tx = Some(tx);
    }

    /// Get a subscription event receiver
    pub fn create_subscription_event_channel() -> (
        mpsc::Sender<SubscriptionEvent>,
        mpsc::Receiver<SubscriptionEvent>,
    ) {
        mpsc::channel(256)
    }

    /// Handle a new WebSocket connection
    ///
    /// This is called when a WebSocket upgrade is successful.
    /// It spawns tasks to handle incoming messages and broadcast outgoing messages.
    pub async fn handle_connection<S>(&self, socket: S)
    where
        S: futures_util::Stream<
                Item = Result<
                    tokio_tungstenite::tungstenite::Message,
                    tokio_tungstenite::tungstenite::Error,
                >,
            > + futures_util::Sink<
                tokio_tungstenite::tungstenite::Message,
                Error = tokio_tungstenite::tungstenite::Error,
            > + Send
            + 'static,
    {
        let client_id = self.subscriptions.new_client_id();
        info!("New WebSocket connection: {}", client_id);

        let (mut ws_sender, mut ws_receiver) = socket.split();

        // Subscribe to broadcast channel
        let mut broadcast_rx = self.subscriptions.subscribe_broadcast();

        // Clone state for the message handler
        let subscriptions = Arc::clone(&self.subscriptions);
        let subscriptions_for_broadcast = Arc::clone(&self.subscriptions);

        // Spawn task to forward broadcast messages to this client
        let (outgoing_tx, mut outgoing_rx) = tokio::sync::mpsc::channel::<ServerMessage>(100);
        let outgoing_tx_clone = outgoing_tx.clone();

        // Task: Forward broadcast messages to client (filtered by subscription)
        tokio::spawn(async move {
            loop {
                match broadcast_rx.recv().await {
                    Ok(BroadcastMessage { key, message }) => {
                        // Check if this client is subscribed to this key
                        if subscriptions_for_broadcast.is_subscribed(client_id, &key) {
                            if outgoing_tx_clone.send(message).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!("Client {} lagged {} messages", client_id, n);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
        });

        // Task: Send outgoing messages to WebSocket
        let send_task = tokio::spawn(async move {
            while let Some(message) = outgoing_rx.recv().await {
                let json = match serde_json::to_string(&message) {
                    Ok(j) => j,
                    Err(e) => {
                        error!("Failed to serialize message: {}", e);
                        continue;
                    }
                };

                if ws_sender
                    .send(tokio_tungstenite::tungstenite::Message::Text(json.into()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });

        // Task: Receive and process incoming messages
        let recv_task = {
            let outgoing_tx = outgoing_tx.clone();
            let subscription_event_tx = self.subscription_event_tx.clone();
            async move {
                while let Some(result) = ws_receiver.next().await {
                    match result {
                        Ok(msg) => {
                            if let Err(e) = Self::handle_message(
                                client_id,
                                msg,
                                &subscriptions,
                                &outgoing_tx,
                                &subscription_event_tx,
                            )
                            .await
                            {
                                warn!("Error handling message from {}: {}", client_id, e);
                            }
                        }
                        Err(e) => {
                            debug!("WebSocket error for {}: {}", client_id, e);
                            break;
                        }
                    }
                }
            }
        };

        // Wait for either task to complete (connection closed)
        tokio::select! {
            _ = send_task => {}
            _ = recv_task => {}
        }

        // Clean up subscriptions
        self.subscriptions.remove_client(client_id);
        info!("WebSocket connection closed: {}", client_id);
    }

    /// Handle an incoming WebSocket message
    async fn handle_message(
        client_id: ClientId,
        msg: tokio_tungstenite::tungstenite::Message,
        subscriptions: &Arc<SubscriptionManager>,
        outgoing_tx: &tokio::sync::mpsc::Sender<ServerMessage>,
        subscription_event_tx: &Option<mpsc::Sender<SubscriptionEvent>>,
    ) -> Result<(), String> {
        use tokio_tungstenite::tungstenite::Message;

        match msg {
            Message::Text(text) => {
                let client_msg: ClientMessage =
                    serde_json::from_str(&text).map_err(|e| format!("Invalid JSON: {}", e))?;

                match client_msg {
                    ClientMessage::Subscribe { subscription } => {
                        // Check if this is a new subscription for this market
                        let key = SubscriptionKey::from(&subscription);
                        let is_first = subscriptions.is_first_subscription(&key);

                        subscriptions.subscribe(client_id, &subscription);

                        // Notify aggregator if this is the first subscription for this market
                        // (only for market-specific subscriptions, not global news)
                        if is_first && !subscription.is_news() {
                            if let (Some(platform), Some(market_id)) =
                                (subscription.platform(), subscription.market_id())
                            {
                                if let Some(ref tx) = subscription_event_tx {
                                    let _ = tx
                                        .send(SubscriptionEvent::Subscribe {
                                            platform,
                                            market_id: market_id.to_string(),
                                        })
                                        .await;
                                }
                            }
                        }

                        // Send confirmation
                        let _ = outgoing_tx
                            .send(ServerMessage::Subscribed { subscription })
                            .await;
                    }
                    ClientMessage::Unsubscribe { subscription } => {
                        subscriptions.unsubscribe(client_id, &subscription);

                        // Check if any clients remain subscribed to this market
                        // (only for market-specific subscriptions, not global news)
                        let key = SubscriptionKey::from(&subscription);
                        if !subscriptions.has_any_subscribers(&key) && !subscription.is_news() {
                            if let (Some(platform), Some(market_id)) =
                                (subscription.platform(), subscription.market_id())
                            {
                                if let Some(ref tx) = subscription_event_tx {
                                    let _ = tx
                                        .send(SubscriptionEvent::Unsubscribe {
                                            platform,
                                            market_id: market_id.to_string(),
                                        })
                                        .await;
                                }
                            }
                        }

                        // Send confirmation
                        let _ = outgoing_tx
                            .send(ServerMessage::Unsubscribed { subscription })
                            .await;
                    }
                    ClientMessage::Ping { timestamp } => {
                        let _ = outgoing_tx
                            .send(ServerMessage::Pong {
                                client_timestamp: timestamp,
                                server_timestamp: Utc::now().timestamp_millis(),
                            })
                            .await;
                    }
                }
            }
            Message::Ping(_data) => {
                // Handled automatically by tungstenite
                debug!("Received ping from {}", client_id);
            }
            Message::Pong(_) => {
                // Expected response to our pings
                debug!("Received pong from {}", client_id);
            }
            Message::Close(_) => {
                debug!("Received close from {}", client_id);
            }
            Message::Binary(_) => {
                // We don't support binary messages
                let _ = outgoing_tx
                    .send(ServerMessage::Error {
                        code: ErrorCode::InvalidMessage,
                        message: "Binary messages not supported".to_string(),
                    })
                    .await;
            }
            Message::Frame(_) => {
                // Raw frames not supported
            }
        }

        Ok(())
    }

    /// Broadcast a price update to all subscribed clients
    pub fn broadcast_price_update(
        &self,
        platform: terminal_core::Platform,
        market_id: String,
        yes_price: rust_decimal::Decimal,
        no_price: rust_decimal::Decimal,
    ) {
        let key = SubscriptionKey {
            platform,
            market_id: market_id.clone(),
            channel: terminal_core::SubscriptionChannel::Price,
        };

        self.subscriptions.broadcast(
            key,
            ServerMessage::PriceUpdate {
                platform,
                market_id,
                yes_price,
                no_price,
                timestamp: Utc::now(),
            },
        );
    }

    /// Broadcast an order book update to all subscribed clients
    pub fn broadcast_orderbook_update(
        &self,
        platform: terminal_core::Platform,
        market_id: String,
        orderbook: terminal_core::OrderBook,
    ) {
        let key = SubscriptionKey {
            platform,
            market_id: market_id.clone(),
            channel: terminal_core::SubscriptionChannel::OrderBook,
        };

        self.subscriptions.broadcast(
            key,
            ServerMessage::OrderBookUpdate {
                platform,
                market_id,
                update_type: terminal_core::OrderBookUpdateType::Snapshot,
                yes_bids: orderbook.yes_bids,
                yes_asks: orderbook.yes_asks,
                no_bids: orderbook.no_bids,
                no_asks: orderbook.no_asks,
                timestamp: Utc::now(),
            },
        );
    }

    /// Broadcast a trade update to all subscribed clients
    pub fn broadcast_trade(&self, trade: terminal_core::Trade) {
        let key = SubscriptionKey {
            platform: trade.platform,
            market_id: trade.market_id.clone(),
            channel: SubscriptionChannel::Trades,
        };

        self.subscriptions.broadcast(
            key,
            ServerMessage::TradeUpdate {
                platform: trade.platform,
                market_id: trade.market_id.clone(),
                trade,
            },
        );
    }

    /// Broadcast a global news update to all clients subscribed to global news
    pub fn broadcast_global_news(&self, item: NewsItem) {
        let key = SubscriptionKey {
            platform: Platform::Kalshi, // Placeholder for global news
            market_id: "__global_news__".to_string(),
            channel: SubscriptionChannel::GlobalNews,
        };

        self.subscriptions.broadcast(
            key,
            ServerMessage::NewsUpdate {
                item,
                market_context: None,
            },
        );
    }

    /// Broadcast a market-specific news update
    pub fn broadcast_market_news(&self, item: NewsItem, platform: Platform, market_id: String) {
        let key = SubscriptionKey {
            platform,
            market_id: market_id.clone(),
            channel: SubscriptionChannel::MarketNews,
        };

        self.subscriptions.broadcast(
            key,
            ServerMessage::NewsUpdate {
                item,
                market_context: Some(MarketNewsContext {
                    platform,
                    market_id,
                }),
            },
        );
    }
}

impl std::fmt::Debug for WebSocketState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSocketState")
            .field("subscriptions", &self.subscriptions)
            .finish()
    }
}
