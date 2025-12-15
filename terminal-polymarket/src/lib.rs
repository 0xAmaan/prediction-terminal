//! Polymarket integration for the Prediction Market Terminal
//!
//! This crate provides a client for interacting with the Polymarket Gamma API,
//! which provides market data without requiring authentication.
//!
//! For authenticated endpoints (like trades), set environment variables:
//! - `POLY_API_KEY` - Your Polymarket API key
//! - `POLY_SECRET` - Your Polymarket API secret
//! - `POLY_PASSPHRASE` - Your Polymarket API passphrase

pub mod client;
pub mod types;
pub mod websocket;

pub use client::{PolymarketClient, PolymarketCredentials};
pub use types::{MarketFilter, MarketOption, PriceHistoryPoint};
pub use websocket::{PolymarketWebSocket, PolymarketWebSocketConfig, PolymarketUpdate};
