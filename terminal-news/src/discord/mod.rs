//! Discord integration for news aggregation
//!
//! This module provides Discord Gateway integration to surface high-engagement
//! messages from prediction market communities as news items.

#[cfg(feature = "discord")]
pub mod config;

#[cfg(feature = "discord")]
pub mod engagement;

#[cfg(feature = "discord")]
pub mod message_converter;

#[cfg(feature = "discord")]
pub mod client;

#[cfg(feature = "discord")]
pub use config::{DiscordConfig, ServerConfig, EngagementThreshold, ConfigError};

#[cfg(feature = "discord")]
pub use engagement::{EngagementMetrics, EngagementTracker, calculate_relevance_score};

#[cfg(feature = "discord")]
pub use message_converter::discord_message_to_news_item;

#[cfg(feature = "discord")]
pub use client::DiscordClient;
