//! News Aggregator Service
//!
//! Background service that maintains a rolling buffer of AI-enriched news
//! articles with market matching and trading signals.

use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::time::Duration;

use futures::future::join_all;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use terminal_core::{NewsFeed, NewsItem, NewsSearchParams};

use crate::news_analyzer::NewsAnalyzer;
use crate::news_service::NewsService;
use crate::websocket::WebSocketState;

/// Maximum number of enriched news items to keep in the rolling buffer
const MAX_BUFFER_SIZE: usize = 20;

/// Configuration for NewsAggregator
#[derive(Debug, Clone)]
pub struct NewsAggregatorConfig {
    /// How often to poll for news (in seconds)
    pub poll_interval_secs: u64,
    /// Maximum articles to fetch per poll
    pub articles_per_poll: usize,
    /// Whether to enable AI enrichment (requires OPENAI_API_KEY)
    pub enable_ai_enrichment: bool,
}

impl Default for NewsAggregatorConfig {
    fn default() -> Self {
        Self {
            poll_interval_secs: 30, // Poll every 30 seconds
            articles_per_poll: 20, // Fetch more articles per poll
            enable_ai_enrichment: true,
        }
    }
}

/// Background service for aggregating and broadcasting AI-enriched news
pub struct NewsAggregator {
    news_service: Arc<NewsService>,
    news_analyzer: Option<Arc<NewsAnalyzer>>,
    ws_state: Arc<WebSocketState>,
    config: NewsAggregatorConfig,
    /// Rolling buffer of enriched news items (newest first)
    buffer: RwLock<VecDeque<NewsItem>>,
    /// IDs of articles already processed (to avoid duplicates)
    seen_article_ids: RwLock<HashSet<String>>,
}

impl NewsAggregator {
    /// Create a new NewsAggregator
    pub fn new(
        news_service: Arc<NewsService>,
        news_analyzer: Option<Arc<NewsAnalyzer>>,
        ws_state: Arc<WebSocketState>,
        config: NewsAggregatorConfig,
    ) -> Self {
        info!(
            "Initializing NewsAggregator (AI enrichment: {})",
            news_analyzer.is_some() && config.enable_ai_enrichment
        );
        Self {
            news_service,
            news_analyzer,
            ws_state,
            config,
            buffer: RwLock::new(VecDeque::with_capacity(MAX_BUFFER_SIZE)),
            seen_article_ids: RwLock::new(HashSet::new()),
        }
    }

    /// Get the current news buffer as a NewsFeed
    pub async fn get_news_feed(&self) -> NewsFeed {
        let buffer = self.buffer.read().await;
        NewsFeed {
            items: buffer.iter().cloned().collect(),
            total_count: buffer.len(),
            next_cursor: None,
        }
    }

    /// Start the background polling loop
    pub async fn start(self: Arc<Self>) {
        info!(
            "Starting NewsAggregator with poll interval {}s",
            self.config.poll_interval_secs
        );

        let self_poll = Arc::clone(&self);
        let self_cleanup = Arc::clone(&self);

        // Spawn news polling task
        tokio::spawn(async move {
            self_poll.poll_news_loop().await;
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

    /// Poll for news in a loop
    async fn poll_news_loop(&self) {
        // Do initial poll immediately after a very short delay
        tokio::time::sleep(Duration::from_secs(3)).await;
        info!("Running initial news poll...");
        if let Err(e) = self.poll_and_enrich_news().await {
            error!("Initial news poll failed: {}", e);
        }

        // Then continue with regular interval
        let mut ticker = interval(Duration::from_secs(self.config.poll_interval_secs));

        loop {
            ticker.tick().await;
            if let Err(e) = self.poll_and_enrich_news().await {
                error!("Failed to poll news: {}", e);
            }
        }
    }

    /// Poll for news and enrich with AI analysis
    async fn poll_and_enrich_news(&self) -> Result<(), String> {
        debug!("Polling for news");

        let params = NewsSearchParams {
            query: None,
            limit: self.config.articles_per_poll,
            time_range: Some("24h".to_string()),
            market_id: None,
            skip_embeddings: true, // Skip embeddings, we'll do AI analysis instead
        };

        let feed = self
            .news_service
            .search_global_news(&params)
            .await
            .map_err(|e| e.to_string())?;

        // Filter out already-seen articles
        let new_articles = self.filter_new_articles(feed.items).await;

        if new_articles.is_empty() {
            debug!("No new articles to process");
            return Ok(());
        }

        info!("Processing {} new articles in parallel", new_articles.len());

        // Enrich articles in parallel for faster processing
        let enriched_articles = if self.config.enable_ai_enrichment {
            let futures: Vec<_> = new_articles
                .into_iter()
                .map(|article| self.enrich_article(article))
                .collect();
            join_all(futures).await
        } else {
            new_articles
        };

        // Add all enriched articles to buffer and broadcast
        for enriched in enriched_articles {
            self.add_to_buffer(enriched.clone()).await;
            self.ws_state.broadcast_global_news(enriched);
        }

        Ok(())
    }

    /// Enrich an article with AI analysis
    async fn enrich_article(&self, article: NewsItem) -> NewsItem {
        if let Some(analyzer) = &self.news_analyzer {
            match analyzer.analyze_news(article.clone()).await {
                Ok(enriched) => {
                    if enriched.matched_market.is_some() {
                        info!(
                            "Enriched article '{}' -> market: {}, signal: {:?}",
                            truncate(&enriched.title, 50),
                            enriched
                                .matched_market
                                .as_ref()
                                .map(|m| m.title.as_str())
                                .unwrap_or("none"),
                            enriched.price_signal
                        );
                    }
                    enriched
                }
                Err(e) => {
                    warn!("Failed to enrich article '{}': {}", truncate(&article.title, 50), e);
                    article
                }
            }
        } else {
            article
        }
    }

    /// Add an article to the rolling buffer
    async fn add_to_buffer(&self, article: NewsItem) {
        let mut buffer = self.buffer.write().await;

        // Add to front (newest first)
        buffer.push_front(article);

        // Trim buffer if it exceeds max size
        while buffer.len() > MAX_BUFFER_SIZE {
            buffer.pop_back();
        }
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

        // Keep only the most recent 500 article IDs
        if seen.len() > 500 {
            seen.clear();
            info!("Cleared seen articles cache (was {} entries)", before);
        }
    }
}

/// Truncate a string for logging
fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        &s[..max_len]
    }
}
