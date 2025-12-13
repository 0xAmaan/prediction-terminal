//! Business logic services for the Prediction Market Terminal
//!
//! This crate provides the service layer that orchestrates data
//! from multiple platform clients and provides unified market views.

pub mod aggregator;
pub mod candle_service;
pub mod market_cache;
pub mod market_service;
pub mod market_stats;
pub mod trade_collector;
pub mod trade_storage;
pub mod websocket;

pub use aggregator::{AggregatorConfig, AggregatorHealth, ConnectionHealth, MarketDataAggregator};
pub use candle_service::CandleService;
pub use market_cache::{CacheStats, MarketCache, MarketCacheError, RefreshRequest};
pub use market_service::{MarketService, OutcomePriceHistory};
pub use market_stats::{MarketStats, MarketStatsService, Timeframe};
pub use trade_collector::{TradeCollector, TradeCollectorConfig};
pub use trade_storage::{
    MarketTradeStats, OrderbookSnapshot, PriceSnapshot, StoredCandle, StoredPrice, TradeStorage,
    TxnCounts,
};
pub use websocket::{SubscriptionEvent, SubscriptionManager, WebSocketState};
