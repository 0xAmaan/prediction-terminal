//! News aggregation clients for prediction market news
//!
//! This crate provides clients for fetching news from:
//! - Google News: Market-specific search via Google News RSS (primary for markets)
//! - RSS feeds: Curated feeds from major news sources (primary for global)
//! - Exa.ai: AI-powered news search with real-time index (optional)
//! - Firecrawl: Web scraping for full article content (optional)

pub mod error;
pub mod exa;
pub mod firecrawl;
pub mod google_news;
pub mod rss_client;
pub mod types;

// Discord integration (optional feature)
#[cfg(feature = "discord")]
pub mod discord;

pub use error::NewsError;
pub use exa::ExaClient;
pub use firecrawl::FirecrawlClient;
pub use google_news::GoogleNewsClient;
pub use rss_client::{get_curated_feeds, RssClient, RssFeed};
pub use types::ArticleContent;
