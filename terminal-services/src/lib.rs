//! Business logic services for the Prediction Market Terminal
//!
//! This crate provides the service layer that orchestrates data
//! from multiple platform clients and provides unified market views.

pub mod aggregator;
pub mod candle_service;
pub mod market_service;
pub mod news_aggregator;
pub mod news_service;
pub mod trade_collector;
pub mod trade_storage;
pub mod websocket;

pub use aggregator::{AggregatorConfig, AggregatorHealth, ConnectionHealth, MarketDataAggregator};
pub use candle_service::CandleService;
pub use market_service::{MarketService, OutcomePriceHistory};
pub use news_aggregator::{NewsAggregator, NewsAggregatorConfig};
pub use news_service::{NewsService, NewsServiceConfig, NewsServiceError};
pub use trade_collector::{TradeCollector, TradeCollectorConfig};
pub use trade_storage::TradeStorage;
pub use websocket::{SubscriptionEvent, SubscriptionManager, WebSocketState};
