//! Core types for the Prediction Market Terminal
//!
//! This crate defines the shared data structures used across the terminal,
//! including market representations, positions, and platform abstractions.

pub mod market;
pub mod news;
pub mod platform;
pub mod position;
pub mod error;
pub mod websocket;

pub use market::{
    MarketStatus, OrderBook, OrderBookLevel, PredictionMarket, PriceCandle, PriceHistory,
    PriceInterval, Trade, TradeHistory, TradeOutcome, TradeSide, UnifiedMarket,
};
pub use news::{MarketNewsContext, NewsFeed, NewsItem, NewsSearchParams, NewsSource};
pub use platform::Platform;
pub use position::{Balance, Portfolio, Position};
pub use error::TerminalError;
pub use websocket::{
    ClientMessage, ConnectionState, ErrorCode, OrderBookUpdateType, ServerMessage,
    SubscriptionChannel, SubscriptionKey, SubscriptionType,
};
