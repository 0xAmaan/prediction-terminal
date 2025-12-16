//! WebSocket infrastructure for real-time market data
//!
//! This module provides WebSocket server functionality for streaming
//! market data to connected clients.

mod subscription;
mod handler;

pub use subscription::SubscriptionManager;
pub use handler::{SubscriptionEvent, TradeSubscriptionEvent, WebSocketState};
