//! News aggregation clients for prediction market news
//!
//! This crate provides clients for fetching news from:
//! - RSS feeds: Curated feeds from major news sources (primary)
//! - Exa.ai: AI-powered news search with real-time index (optional)
//! - Firecrawl: Web scraping for full article content (optional)

pub mod error;
pub mod exa;
pub mod firecrawl;
pub mod rss_client;
pub mod types;

pub use error::NewsError;
pub use exa::ExaClient;
pub use firecrawl::FirecrawlClient;
pub use rss_client::{RssClient, RssFeed, get_curated_feeds};
pub use types::ArticleContent;
