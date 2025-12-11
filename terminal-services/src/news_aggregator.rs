//! News Aggregator Service
//!
//! Background service that polls for news updates and broadcasts
//! new articles to WebSocket clients.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use terminal_core::{NewsItem, NewsSearchParams, Platform};

use crate::news_service::NewsService;
use crate::websocket::WebSocketState;

/// Configuration for NewsAggregator
#[derive(Debug, Clone)]
pub struct NewsAggregatorConfig {
    /// How often to poll for global news (in seconds)
    pub global_poll_interval_secs: u64,
    /// How often to poll for market-specific news (in seconds)
    pub market_poll_interval_secs: u64,
    /// Maximum articles to fetch per poll
    pub articles_per_poll: usize,
}

impl Default for NewsAggregatorConfig {
    fn default() -> Self {
        Self {
            global_poll_interval_secs: 5,  // 5 seconds for RSS
            market_poll_interval_secs: 5,  // 5 seconds for RSS
            articles_per_poll: 20,
        }
    }
}

/// Tracked market info
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct TrackedMarket {
    platform: Platform,
    market_id: String,
    title: String,
}

/// Background service for aggregating and broadcasting news
pub struct NewsAggregator {
    news_service: Arc<NewsService>,
    ws_state: Arc<WebSocketState>,
    config: NewsAggregatorConfig,
    /// Markets currently being tracked for news
    tracked_markets: RwLock<HashSet<TrackedMarket>>,
    /// IDs of articles already seen (to avoid duplicates)
    seen_article_ids: RwLock<HashSet<String>>,
}

impl NewsAggregator {
    /// Create a new NewsAggregator
    pub fn new(
        news_service: Arc<NewsService>,
        ws_state: Arc<WebSocketState>,
        config: NewsAggregatorConfig,
    ) -> Self {
        info!("Initializing NewsAggregator");
        Self {
            news_service,
            ws_state,
            config,
            tracked_markets: RwLock::new(HashSet::new()),
            seen_article_ids: RwLock::new(HashSet::new()),
        }
    }

    /// Start the background polling loops
    pub async fn start(self: Arc<Self>) {
        info!(
            "Starting NewsAggregator with global poll interval {}s, market poll interval {}s",
            self.config.global_poll_interval_secs, self.config.market_poll_interval_secs
        );

        let self_global = Arc::clone(&self);
        let self_market = Arc::clone(&self);
        let self_cleanup = Arc::clone(&self);

        // Spawn global news polling task
        tokio::spawn(async move {
            self_global.poll_global_news_loop().await;
        });

        // Spawn market-specific news polling task
        tokio::spawn(async move {
            self_market.poll_market_news_loop().await;
        });

        // Spawn cleanup task for seen article IDs (run hourly)
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(3600));
            loop {
                ticker.tick().await;
                self_cleanup.cleanup_seen_articles().await;
            }
        });
    }

    /// Track a market for news updates
    pub async fn track_market(&self, platform: Platform, market_id: String, title: String) {
        let mut markets = self.tracked_markets.write().await;
        markets.insert(TrackedMarket {
            platform,
            market_id: market_id.clone(),
            title: title.clone(),
        });
        info!("Now tracking market for news: {:?}/{}", platform, market_id);
    }

    /// Stop tracking a market
    pub async fn untrack_market(&self, platform: Platform, market_id: &str) {
        let mut markets = self.tracked_markets.write().await;
        markets.retain(|m| !(m.platform == platform && m.market_id == market_id));
        debug!(
            "Stopped tracking market for news: {:?}/{}",
            platform, market_id
        );
    }

    /// Poll for global news in a loop
    async fn poll_global_news_loop(&self) {
        let mut ticker = interval(Duration::from_secs(self.config.global_poll_interval_secs));

        // Initial poll after a short delay
        tokio::time::sleep(Duration::from_secs(2)).await;

        loop {
            ticker.tick().await;
            if let Err(e) = self.poll_global_news().await {
                error!("Failed to poll global news: {}", e);
            }
        }
    }

    /// Poll for global news once
    async fn poll_global_news(&self) -> Result<(), String> {
        debug!("Polling for global news");

        let params = NewsSearchParams {
            query: None,
            limit: self.config.articles_per_poll,
            time_range: Some("24h".to_string()),
            market_id: None,
        };

        let feed = self
            .news_service
            .search_global_news(&params)
            .await
            .map_err(|e| e.to_string())?;

        let new_articles = self.filter_new_articles(feed.items).await;

        if !new_articles.is_empty() {
            info!("Broadcasting {} new global news articles", new_articles.len());
            for article in new_articles {
                self.ws_state.broadcast_global_news(article);
            }
        }

        Ok(())
    }

    /// Poll for market-specific news in a loop
    async fn poll_market_news_loop(&self) {
        let mut ticker = interval(Duration::from_secs(self.config.market_poll_interval_secs));

        // Initial poll after a short delay
        tokio::time::sleep(Duration::from_secs(3)).await;

        loop {
            ticker.tick().await;
            if let Err(e) = self.poll_market_news().await {
                error!("Failed to poll market news: {}", e);
            }
        }
    }

    /// Poll for market-specific news once
    async fn poll_market_news(&self) -> Result<(), String> {
        let markets: Vec<TrackedMarket> = {
            let tracked = self.tracked_markets.read().await;
            tracked.iter().cloned().collect()
        };

        if markets.is_empty() {
            debug!("No markets being tracked for news");
            return Ok(());
        }

        debug!("Polling news for {} tracked markets", markets.len());

        for market in markets {
            match self
                .news_service
                .get_market_news(&market.title, &market.market_id, 5, None)
                .await
            {
                Ok(feed) => {
                    let new_articles = self.filter_new_articles(feed.items).await;

                    if !new_articles.is_empty() {
                        info!(
                            "Broadcasting {} new articles for market {:?}/{}",
                            new_articles.len(),
                            market.platform,
                            market.market_id
                        );
                        for article in new_articles {
                            self.ws_state.broadcast_market_news(
                                article,
                                market.platform,
                                market.market_id.clone(),
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to fetch news for market {:?}/{}: {}",
                        market.platform, market.market_id, e
                    );
                }
            }

            // Small delay between markets to avoid rate limiting
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        Ok(())
    }

    /// Filter out articles we've already seen
    async fn filter_new_articles(&self, articles: Vec<NewsItem>) -> Vec<NewsItem> {
        let mut seen = self.seen_article_ids.write().await;
        let mut new_articles = Vec::new();

        for article in articles {
            if !seen.contains(&article.id) {
                seen.insert(article.id.clone());
                new_articles.push(article);
            }
        }

        new_articles
    }

    /// Cleanup old seen article IDs to prevent unbounded growth
    async fn cleanup_seen_articles(&self) {
        let mut seen = self.seen_article_ids.write().await;
        let before = seen.len();

        // Keep only the most recent 1000 article IDs
        if seen.len() > 1000 {
            // Since HashSet doesn't maintain order, we just clear it
            // This is acceptable because duplicates will be rare after cleanup
            seen.clear();
            info!(
                "Cleared seen articles cache (was {} entries)",
                before
            );
        }
    }
}
