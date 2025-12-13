//! WebSocket infrastructure for real-time market data
//!
//! This module provides WebSocket server functionality for streaming
//! market data to connected clients.

mod handler;
mod subscription;

pub use handler::{SubscriptionEvent, WebSocketState};
pub use subscription::SubscriptionManager;
