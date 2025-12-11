//! Subscription manager for WebSocket connections
//!
//! Manages client subscriptions and broadcasts updates to interested clients.

use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use terminal_core::{ServerMessage, SubscriptionKey, SubscriptionType};
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

/// Unique identifier for a WebSocket client connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientId(pub u64);

impl std::fmt::Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "client-{}", self.0)
    }
}

/// Manages subscriptions and message broadcasting
pub struct SubscriptionManager {
    /// Next client ID to assign
    next_client_id: AtomicU64,
    /// Map of subscription key -> set of subscribed client IDs
    subscriptions: DashMap<SubscriptionKey, HashSet<ClientId>>,
    /// Map of client ID -> set of subscription keys
    client_subscriptions: DashMap<ClientId, HashSet<SubscriptionKey>>,
    /// Broadcast channel for sending messages to all clients
    /// Each client receives messages and filters based on their subscriptions
    broadcast_tx: broadcast::Sender<BroadcastMessage>,
}

/// A message with its target subscription info
#[derive(Debug, Clone)]
pub struct BroadcastMessage {
    /// The subscription key this message is for
    pub key: SubscriptionKey,
    /// The actual message
    pub message: ServerMessage,
}

impl SubscriptionManager {
    /// Create a new subscription manager
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(1024);
        Self {
            next_client_id: AtomicU64::new(1),
            subscriptions: DashMap::new(),
            client_subscriptions: DashMap::new(),
            broadcast_tx,
        }
    }

    /// Generate a new unique client ID
    pub fn new_client_id(&self) -> ClientId {
        ClientId(self.next_client_id.fetch_add(1, Ordering::SeqCst))
    }

    /// Subscribe to a broadcast channel
    pub fn subscribe_broadcast(&self) -> broadcast::Receiver<BroadcastMessage> {
        self.broadcast_tx.subscribe()
    }

    /// Register a client subscription
    pub fn subscribe(&self, client_id: ClientId, subscription: &SubscriptionType) {
        let key = SubscriptionKey::from(subscription);

        // Add to subscription -> clients map
        self.subscriptions
            .entry(key.clone())
            .or_default()
            .insert(client_id);

        // Add to client -> subscriptions map
        self.client_subscriptions
            .entry(client_id)
            .or_default()
            .insert(key.clone());

        debug!(
            "Client {} subscribed to {:?}",
            client_id, key
        );
    }

    /// Unsubscribe a client from a subscription
    pub fn unsubscribe(&self, client_id: ClientId, subscription: &SubscriptionType) {
        let key = SubscriptionKey::from(subscription);

        // Remove from subscription -> clients map
        if let Some(mut clients) = self.subscriptions.get_mut(&key) {
            clients.remove(&client_id);
            if clients.is_empty() {
                drop(clients);
                self.subscriptions.remove(&key);
            }
        }

        // Remove from client -> subscriptions map
        if let Some(mut subs) = self.client_subscriptions.get_mut(&client_id) {
            subs.remove(&key);
        }

        debug!(
            "Client {} unsubscribed from {:?}",
            client_id, key
        );
    }

    /// Remove all subscriptions for a client (on disconnect)
    pub fn remove_client(&self, client_id: ClientId) {
        // Get all subscriptions for this client
        if let Some((_, subscriptions)) = self.client_subscriptions.remove(&client_id) {
            // Remove client from each subscription
            for key in subscriptions {
                if let Some(mut clients) = self.subscriptions.get_mut(&key) {
                    clients.remove(&client_id);
                    if clients.is_empty() {
                        drop(clients);
                        self.subscriptions.remove(&key);
                    }
                }
            }
        }

        info!("Client {} disconnected, removed all subscriptions", client_id);
    }

    /// Check if a client is subscribed to a specific subscription
    pub fn is_subscribed(&self, client_id: ClientId, key: &SubscriptionKey) -> bool {
        self.client_subscriptions
            .get(&client_id)
            .map(|subs| subs.contains(key))
            .unwrap_or(false)
    }

    /// Get the number of subscribers for a subscription
    pub fn subscriber_count(&self, key: &SubscriptionKey) -> usize {
        self.subscriptions
            .get(key)
            .map(|clients| clients.len())
            .unwrap_or(0)
    }

    /// Check if this would be the first subscription for a given key
    /// (i.e., no clients are currently subscribed)
    pub fn is_first_subscription(&self, key: &SubscriptionKey) -> bool {
        !self.subscriptions.contains_key(key)
    }

    /// Check if any clients are subscribed to a given key
    pub fn has_any_subscribers(&self, key: &SubscriptionKey) -> bool {
        self.subscriptions
            .get(key)
            .map(|clients| !clients.is_empty())
            .unwrap_or(false)
    }

    /// Check if any clients are subscribed to a specific platform/market combination
    /// (across all channels: price, orderbook, trades)
    pub fn has_any_market_subscribers(
        &self,
        platform: terminal_core::Platform,
        market_id: &str,
    ) -> bool {
        self.subscriptions.iter().any(|entry| {
            entry.key().platform == platform && entry.key().market_id == market_id
        })
    }

    /// Broadcast a message to all subscribers of a subscription
    pub fn broadcast(&self, key: SubscriptionKey, message: ServerMessage) {
        let broadcast_msg = BroadcastMessage { key, message };

        // Send to broadcast channel - receivers will filter based on subscriptions
        if let Err(e) = self.broadcast_tx.send(broadcast_msg) {
            warn!("Failed to broadcast message: {} (no receivers)", e);
        }
    }

    /// Broadcast a message to all connected clients (no subscription filtering)
    ///
    /// This is used for global messages like research updates that should
    /// be sent to all clients regardless of their subscriptions.
    pub fn broadcast_to_all(&self, message: ServerMessage) {
        // Use a special "global" key that all clients will accept
        // We create a dummy key that won't match any real subscriptions,
        // but the handler.rs will need to be updated to recognize this pattern
        let broadcast_msg = BroadcastMessage {
            key: SubscriptionKey {
                platform: terminal_core::Platform::Kalshi, // placeholder
                market_id: "__global__".to_string(),
                channel: terminal_core::SubscriptionChannel::Price, // placeholder
            },
            message,
        };

        if let Err(e) = self.broadcast_tx.send(broadcast_msg) {
            warn!("Failed to broadcast global message: {} (no receivers)", e);
        }
    }

    /// Get all subscriptions for a specific platform/market
    pub fn get_market_subscriptions(
        &self,
        platform: terminal_core::Platform,
        market_id: &str,
    ) -> Vec<SubscriptionKey> {
        self.subscriptions
            .iter()
            .filter(|entry| {
                entry.key().platform == platform && entry.key().market_id == market_id
            })
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Check if there are any active subscriptions
    pub fn has_subscriptions(&self) -> bool {
        !self.subscriptions.is_empty()
    }

    /// Get total number of active subscriptions
    pub fn total_subscriptions(&self) -> usize {
        self.subscriptions.len()
    }

    /// Get total number of connected clients
    pub fn total_clients(&self) -> usize {
        self.client_subscriptions.len()
    }
}

impl Default for SubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SubscriptionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubscriptionManager")
            .field("total_subscriptions", &self.total_subscriptions())
            .field("total_clients", &self.total_clients())
            .finish()
    }
}

/// Create a shared subscription manager
#[allow(dead_code)]
pub fn create_subscription_manager() -> Arc<SubscriptionManager> {
    Arc::new(SubscriptionManager::new())
}
