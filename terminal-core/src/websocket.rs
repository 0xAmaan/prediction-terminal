//! WebSocket message types for real-time market data
//!
//! These types define the protocol for WebSocket communication between
//! the server and clients.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{NewsFeed, OrderBookLevel, Platform, Trade};

// ============================================================================
// Client -> Server Messages
// ============================================================================

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Subscribe to market updates
    Subscribe {
        /// Subscription type
        subscription: SubscriptionType,
    },
    /// Unsubscribe from market updates
    Unsubscribe {
        /// Subscription type to unsubscribe from
        subscription: SubscriptionType,
    },
    /// Ping to keep connection alive
    Ping {
        /// Client timestamp
        timestamp: i64,
    },
}

/// Types of subscriptions available
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SubscriptionType {
    /// Subscribe to price updates for a market
    Price {
        platform: Platform,
        market_id: String,
    },
    /// Subscribe to order book updates for a market
    OrderBook {
        platform: Platform,
        market_id: String,
    },
    /// Subscribe to trade updates for a market
    Trades {
        platform: Platform,
        market_id: String,
    },
}

impl SubscriptionType {
    /// Get the platform for this subscription
    pub fn platform(&self) -> Platform {
        match self {
            Self::Price { platform, .. } => *platform,
            Self::OrderBook { platform, .. } => *platform,
            Self::Trades { platform, .. } => *platform,
        }
    }

    /// Get the market ID for this subscription
    pub fn market_id(&self) -> &str {
        match self {
            Self::Price { market_id, .. } => market_id,
            Self::OrderBook { market_id, .. } => market_id,
            Self::Trades { market_id, .. } => market_id,
        }
    }
}

// ============================================================================
// Server -> Client Messages
// ============================================================================

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Subscription confirmed
    Subscribed {
        subscription: SubscriptionType,
    },
    /// Unsubscription confirmed
    Unsubscribed {
        subscription: SubscriptionType,
    },
    /// Price update for a market
    PriceUpdate {
        platform: Platform,
        market_id: String,
        yes_price: Decimal,
        no_price: Decimal,
        timestamp: DateTime<Utc>,
    },
    /// Order book snapshot or update
    OrderBookUpdate {
        platform: Platform,
        market_id: String,
        /// Type of update (snapshot or delta)
        update_type: OrderBookUpdateType,
        yes_bids: Vec<OrderBookLevel>,
        yes_asks: Vec<OrderBookLevel>,
        no_bids: Vec<OrderBookLevel>,
        no_asks: Vec<OrderBookLevel>,
        timestamp: DateTime<Utc>,
    },
    /// New trade occurred
    TradeUpdate {
        platform: Platform,
        market_id: String,
        trade: Trade,
    },
    /// News update for a market
    NewsUpdate {
        feed: NewsFeed,
    },
    /// Error message
    Error {
        code: ErrorCode,
        message: String,
    },
    /// Pong response to client ping
    Pong {
        /// Echo back client timestamp
        client_timestamp: i64,
        /// Server timestamp
        server_timestamp: i64,
    },
    /// Connection status update
    ConnectionStatus {
        platform: Platform,
        status: ConnectionState,
    },
}

/// Type of order book update
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderBookUpdateType {
    /// Full order book snapshot
    Snapshot,
    /// Incremental update (delta)
    Delta,
}

/// Error codes for WebSocket errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// Invalid message format
    InvalidMessage,
    /// Unknown subscription type
    UnknownSubscription,
    /// Market not found
    MarketNotFound,
    /// Platform connection error
    PlatformError,
    /// Rate limit exceeded
    RateLimited,
    /// Internal server error
    InternalError,
}

/// Connection state for platform connections
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    /// Connected and receiving data
    Connected,
    /// Attempting to connect
    Connecting,
    /// Disconnected, will attempt reconnect
    Disconnected,
    /// Connection failed, not retrying
    Failed,
}

// ============================================================================
// Subscription Key (for internal use)
// ============================================================================

/// Unique key for a subscription (used in subscription manager)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubscriptionKey {
    pub platform: Platform,
    pub market_id: String,
    pub channel: SubscriptionChannel,
}

/// Channel type for subscriptions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubscriptionChannel {
    Price,
    OrderBook,
    Trades,
    News,
}

impl From<&SubscriptionType> for SubscriptionKey {
    fn from(sub: &SubscriptionType) -> Self {
        match sub {
            SubscriptionType::Price { platform, market_id } => Self {
                platform: *platform,
                market_id: market_id.clone(),
                channel: SubscriptionChannel::Price,
            },
            SubscriptionType::OrderBook { platform, market_id } => Self {
                platform: *platform,
                market_id: market_id.clone(),
                channel: SubscriptionChannel::OrderBook,
            },
            SubscriptionType::Trades { platform, market_id } => Self {
                platform: *platform,
                market_id: market_id.clone(),
                channel: SubscriptionChannel::Trades,
            },
        }
    }
}
