//! Kalshi integration for the Prediction Market Terminal
//!
//! This crate provides a client for interacting with the Kalshi API,
//! including market data retrieval, portfolio management, and real-time
//! WebSocket streaming.

pub mod client;
pub mod types;
pub mod websocket;

pub use client::KalshiClient;
pub use websocket::{KalshiWebSocket, KalshiWebSocketConfig, KalshiUpdate};
