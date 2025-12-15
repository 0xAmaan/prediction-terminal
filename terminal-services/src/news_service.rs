//! News Service
//!
//! Provides news fetching that's specifically relevant to prediction markets.
//! Fetches from RSS feeds and filters articles based on relevance to active markets.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

use terminal_core::{NewsFeed, NewsItem, NewsSearchParams, PredictionMarket};
use terminal_embedding::{EmbeddingClient, EmbeddingStore, NewsEmbedding, find_similar_markets};
use terminal_news::{
    ArticleContent, ExaClient, FirecrawlClient, GoogleNewsClient, NewsError, RssClient,
};

use crate::market_service::MarketService;

/// Cache entry with expiration
struct CacheEntry<T> {
    data: T,
    expires_at: Instant,
}

impl<T> CacheEntry<T> {
    fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

/// Configuration for NewsService
#[derive(Debug, Clone)]
pub struct NewsServiceConfig {
    /// Cache TTL for RSS feed results (in seconds)
    pub rss_cache_ttl_secs: u64,
    /// Cache TTL for market list (in seconds)
    pub market_cache_ttl_secs: u64,
    /// Minimum relevance score to include an article (0.0 - 1.0)
    pub min_relevance_score: f64,
    /// Maximum cached entries
    pub max_cache_entries: usize,
}

impl Default for NewsServiceConfig {
    fn default() -> Self {
        Self {
            rss_cache_ttl_secs: 60,     // RSS feeds don't update faster than this
            market_cache_ttl_secs: 120, // Refresh market list every 2 minutes
            min_relevance_score: 0.35,  // Minimum relevance to show article (raised from 0.15)
            max_cache_entries: 100,
        }
    }
}

/// News service for fetching market-relevant news
pub struct NewsService {
    rss: RssClient,
    /// Google News client for market-specific search (primary for markets)
    google_news: GoogleNewsClient,
    /// Exa.ai client for semantic search (market-specific news, optional fallback)
    exa: Option<ExaClient>,
    firecrawl: Option<FirecrawlClient>,
    market_service: Option<Arc<MarketService>>,
    config: NewsServiceConfig,
    /// Cache for RSS feed results
    rss_cache: RwLock<Option<CacheEntry<Vec<NewsItem>>>>,
    /// Cache for active markets
    market_cache: RwLock<Option<CacheEntry<Vec<PredictionMarket>>>>,
    /// Cache for filtered news results
    news_cache: RwLock<HashMap<String, CacheEntry<NewsFeed>>>,
    /// Cache for article content
    article_cache: RwLock<HashMap<String, CacheEntry<String>>>,
    /// Embedding client for semantic matching (optional)
    embedding_client: Option<Arc<EmbeddingClient>>,
    /// Embedding store for market embeddings (optional)
    embedding_store: Option<Arc<EmbeddingStore>>,
}

impl NewsService {
    /// Create a new NewsService
    pub fn new(
        exa_api_key: Option<String>,
        firecrawl_api_key: Option<String>,
        config: NewsServiceConfig,
    ) -> Self {
        // Initialize embedding client if OpenAI API key is available
        let embedding_client = std::env::var("OPENAI_API_KEY")
            .ok()
            .map(|key| {
                info!("Initializing embedding client with OpenAI API");
                Arc::new(EmbeddingClient::new(key))
            });

        // Initialize embedding store
        let embedding_store = EmbeddingStore::new("data/embeddings.db")
            .map(|store| {
                info!("Initialized embedding store at data/embeddings.db");
                Arc::new(store)
            })
            .ok();

        info!(
            "Initializing NewsService (Google News: enabled, Exa: {}, Firecrawl: {}, Embeddings: {})",
            exa_api_key.is_some(),
            firecrawl_api_key.is_some(),
            embedding_client.is_some() && embedding_store.is_some()
        );

        Self {
            rss: RssClient::new(),
            google_news: GoogleNewsClient::new(),
            exa: exa_api_key.map(ExaClient::new),
            firecrawl: firecrawl_api_key.map(FirecrawlClient::new),
            market_service: None,
            config,
            rss_cache: RwLock::new(None),
            market_cache: RwLock::new(None),
            news_cache: RwLock::new(HashMap::new()),
            article_cache: RwLock::new(HashMap::new()),
            embedding_client,
            embedding_store,
        }
    }

    /// Set the market service for relevance scoring
    pub fn set_market_service(&mut self, market_service: Arc<MarketService>) {
        self.market_service = Some(market_service);
    }

    /// Get cached trending markets for news filtering
    /// Only uses TOP trending markets to ensure news relevance
    async fn get_trending_markets(&self) -> Vec<PredictionMarket> {
        // Check cache first
        {
            let cache = self.market_cache.read().await;
            if let Some(entry) = cache.as_ref() {
                if !entry.is_expired() {
                    return entry.data.clone();
                }
            }
        }

        // Fetch trending markets (top 30 by volume/activity)
        let markets = if let Some(ref market_service) = self.market_service {
            // Get markets sorted by volume (trending tab logic)
            match market_service.get_all_markets(Some(30)).await {
                Ok(mut markets) => {
                    // Sort by volume (highest first) to get truly trending markets
                    markets.sort_by(|a, b| {
                        b.volume
                            .partial_cmp(&a.volume)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    // Take top 30 trending markets
                    markets.truncate(30);
                    info!(
                        "Using {} trending markets for news relevance scoring",
                        markets.len()
                    );
                    markets
                }
                Err(e) => {
                    debug!("Failed to fetch markets: {}", e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        // Update cache
        {
            let mut cache = self.market_cache.write().await;
            *cache = Some(CacheEntry::new(
                markets.clone(),
                Duration::from_secs(self.config.market_cache_ttl_secs),
            ));
        }

        markets
    }

    /// Fetch dynamic Google News feeds for top trending markets
    /// This makes the news feed PROACTIVE - hunting for news about what traders are betting on
    /// Includes both regular news AND Twitter/X posts via Google News site search
    async fn fetch_dynamic_market_feeds(
        &self,
        markets: &[PredictionMarket],
        top_n: usize,
    ) -> Vec<NewsItem> {
        if markets.is_empty() {
            return Vec::new();
        }

        let top_markets: Vec<&PredictionMarket> = markets.iter().take(top_n).collect();
        info!(
            "Generating dynamic Google News feeds (news + Twitter) for {} trending markets",
            top_markets.len()
        );

        let mut all_items = Vec::new();

        for market in &top_markets {
            // Extract keywords from market title (using simplified extraction)
            let keywords = extract_dynamic_feed_keywords(&market.title);

            if keywords.is_empty() {
                debug!("No keywords extracted from market: {}", market.title);
                continue;
            }

            info!(
                "Dynamic feed for '{}' -> keywords: {}",
                market.title, keywords
            );

            // PHASE 1: Fetch regular Google News (3 articles per market)
            match self
                .google_news
                .search_market_news(&keywords, None, 3)
                .await
            {
                Ok(items) => {
                    info!(
                        "✓ Regular news for '{}' returned {} articles",
                        keywords,
                        items.len()
                    );
                    all_items.extend(items);
                }
                Err(e) => {
                    debug!("Failed to fetch regular news for '{}': {}", keywords, e);
                }
            }

            // PHASE 2: Fetch Twitter/X posts via Google News site search (2 articles per market)
            // Google only indexes high-engagement tweets (10k+ impressions)
            // These are the tweets that move prediction markets
            let twitter_query = format!("site:x.com OR site:twitter.com {}", keywords);
            match self
                .google_news
                .search_market_news(&twitter_query, None, 2)
                .await
            {
                Ok(items) => {
                    info!(
                        "✓ Twitter feed for '{}' returned {} posts",
                        keywords,
                        items.len()
                    );
                    all_items.extend(items);
                }
                Err(e) => {
                    debug!("Failed to fetch Twitter feed for '{}': {}", keywords, e);
                }
            }
        }

        info!(
            "Total items from {} dynamic feeds (news + Twitter): {}",
            top_markets.len(),
            all_items.len()
        );

        all_items
    }

    /// Get cached RSS items or fetch fresh ones
    async fn get_rss_items(&self) -> Result<Vec<NewsItem>, NewsError> {
        // Check cache first
        {
            let cache = self.rss_cache.read().await;
            if let Some(entry) = cache.as_ref() {
                if !entry.is_expired() {
                    debug!("Using cached RSS items ({} articles)", entry.data.len());
                    return Ok(entry.data.clone());
                }
            }
        }

        // Fetch fresh RSS
        debug!("Fetching fresh RSS feeds");
        let items = self.rss.fetch_all(100).await?;
        info!("Fetched {} articles from RSS feeds", items.len());

        // Update cache
        {
            let mut cache = self.rss_cache.write().await;
            *cache = Some(CacheEntry::new(
                items.clone(),
                Duration::from_secs(self.config.rss_cache_ttl_secs),
            ));
        }

        Ok(items)
    }

    /// Search for global news relevant to prediction markets
    /// Uses STRICT entity matching - only shows news that mentions trending market topics
    /// DYNAMIC: Generates Google News RSS feeds for top trending markets on-the-fly
    #[instrument(skip(self))]
    pub async fn search_global_news(
        &self,
        params: &NewsSearchParams,
    ) -> Result<NewsFeed, NewsServiceError> {
        let cache_key = format!("global:{}", params.limit);

        // Check news cache (short TTL for responsive updates)
        {
            let cache = self.news_cache.read().await;
            if let Some(entry) = cache.get(&cache_key) {
                if !entry.is_expired() {
                    return Ok(entry.data.clone());
                }
            }
        }

        // Get TRENDING markets to drive dynamic feed generation
        let markets = self.get_trending_markets().await;

        // PHASE 1: Fetch static RSS feeds (baseline coverage)
        let static_rss_items = self.get_rss_items().await.map_err(NewsServiceError::Rss)?;
        info!(
            "Fetched {} items from static RSS feeds",
            static_rss_items.len()
        );

        // PHASE 2: Generate dynamic Google News feeds for top 5 trending markets
        let dynamic_items = self.fetch_dynamic_market_feeds(&markets, 5).await;
        info!(
            "Fetched {} items from dynamic Google News feeds for {} markets",
            dynamic_items.len(),
            std::cmp::min(5, markets.len())
        );

        // Combine static and dynamic items
        let mut all_items = static_rss_items;
        all_items.extend(dynamic_items);

        // Filter for recent articles only (last 30 days)
        let now = chrono::Utc::now();
        let thirty_days_ago = now - chrono::Duration::days(30);
        let total_count = all_items.len();
        let recent_items: Vec<NewsItem> = all_items
            .into_iter()
            .filter(|item| item.published_at >= thirty_days_ago)
            .collect();

        info!(
            "Filtered to {} recent articles (last 30 days) from {} total",
            recent_items.len(),
            total_count
        );

        // If no markets, just return top news by recency
        let items: Vec<NewsItem> = if markets.is_empty() {
            recent_items.into_iter().take(params.limit).collect()
        } else {
            // Extract key entities from trending markets
            let entities = extract_market_entities(&markets);
            info!(
                "Filtering news by {} entities from trending markets: {:?}",
                entities.len(),
                entities.iter().take(20).collect::<Vec<_>>()
            );

            // STRICT FILTER: Only include articles that mention an entity IN THE TITLE
            let mut matched_items: Vec<(NewsItem, f64, String)> = recent_items
                .into_iter()
                .filter_map(|item| {
                    let title_lower = item.title.to_lowercase();

                    // Check if title contains any entity
                    for entity in &entities {
                        let entity_lower = entity.to_lowercase();
                        if title_lower.contains(&entity_lower) {
                            // Score based on entity prominence and recency
                            let mut score: f64 = 0.7;

                            // Bonus for entity at start of title
                            if title_lower.starts_with(&entity_lower) {
                                score += 0.2;
                            }

                            // Recency bonus
                            let age_hours = (chrono::Utc::now() - item.published_at).num_hours();
                            if age_hours < 2 {
                                score += 0.1;
                            } else if age_hours < 6 {
                                score += 0.05;
                            }

                            return Some((item, score.min(1.0), entity.clone()));
                        }
                    }
                    None
                })
                .collect();

            // Sort by score (highest first), then by recency
            matched_items.sort_by(|a, b| {
                b.1.partial_cmp(&a.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| b.0.published_at.cmp(&a.0.published_at))
            });

            info!(
                "Found {} articles matching trending market entities",
                matched_items.len()
            );

            // Take top items and update their relevance scores
            matched_items
                .into_iter()
                .take(params.limit)
                .map(|(mut item, score, matched_entity)| {
                    item.relevance_score = score;
                    item.search_query = Some(matched_entity);
                    item
                })
                .collect()
        };

        // Sort by published_at descending (newest first)
        let mut sorted_items = items;
        sorted_items.sort_by(|a, b| b.published_at.cmp(&a.published_at));

        // SEMANTIC ENHANCEMENT: Add related market IDs to each article
        // Skip if requested for faster initial loads
        let enhanced_items = if params.skip_embeddings {
            sorted_items
        } else {
            let tagged_items = self.add_semantic_market_tags(sorted_items).await;

            // Filter out articles with no related markets
            // Only show news that's relevant to at least one market
            tagged_items.into_iter()
                .filter(|item| !item.related_market_ids.is_empty())
                .collect()
        };

        let feed = NewsFeed {
            total_count: enhanced_items.len(),
            items: enhanced_items,
            next_cursor: None,
        };

        // Cache results (5 second TTL for responsive updates)
        {
            let mut cache = self.news_cache.write().await;
            if cache.len() >= self.config.max_cache_entries {
                cache.retain(|_, entry| !entry.is_expired());
            }
            cache.insert(
                cache_key,
                CacheEntry::new(feed.clone(), Duration::from_secs(5)),
            );
        }

        Ok(feed)
    }

    /// Add semantic market tags to news items (populates related_market_ids)
    ///
    /// For each news article, finds all markets it's semantically similar to
    /// and populates the related_market_ids field.
    async fn add_semantic_market_tags(&self, items: Vec<NewsItem>) -> Vec<NewsItem> {
        let (Some(client), Some(store)) = (&self.embedding_client, &self.embedding_store) else {
            debug!("Semantic tagging not available, returning items unchanged");
            return items;
        };

        // Load all market embeddings
        let market_embeddings = match store.load_all_market_embeddings() {
            Ok(embs) if !embs.is_empty() => embs,
            _ => {
                debug!("No market embeddings available for semantic tagging");
                return items;
            }
        };

        let mut tagged_items = Vec::new();
        let total_items = items.len();

        for item in items {
            let mut tagged_item = item.clone();

            // Generate/retrieve embedding for this article
            let article_id = format!("{:x}", md5::compute(&item.url));

            let news_embedding = if let Ok(Some(cached)) = store.get_news_embedding(&article_id) {
                cached.embedding
            } else {
                match client.embed_news(&item.title, &item.summary).await {
                    Ok(emb) => {
                        // Cache it
                        let news_emb = NewsEmbedding::new(
                            article_id.clone(),
                            format!("{} {}", item.title, item.summary),
                            emb.clone(),
                        );
                        let _ = store.save_news_embedding(&news_emb);
                        emb
                    }
                    Err(e) => {
                        debug!("Failed to generate embedding for '{}': {}", item.title, e);
                        tagged_items.push(tagged_item);
                        continue;
                    }
                }
            };

            // Find similar markets
            let matches = find_similar_markets(
                &news_embedding,
                &market_embeddings,
                25,   // top 25 markets for better coverage
                0.38, // 38% similarity threshold - balanced to get 40-50 relevant articles
            );

            if !matches.is_empty() {
                // Populate related market IDs (limit to 5 most relevant)
                tagged_item.related_market_ids = matches
                    .iter()
                    .take(5)
                    .map(|m| m.market_id.clone())
                    .collect();

                info!(
                    "Tagged article '{}' with {} related markets (best score: {:.2})",
                    item.title,
                    tagged_item.related_market_ids.len(),
                    matches[0].score
                );
            } else {
                debug!("No matches for article: {}", item.title);
            }

            // Only include articles that match at least one market
            if !tagged_item.related_market_ids.is_empty() {
                tagged_items.push(tagged_item);
            }
        }

        info!(
            "Filtered to {} market-related articles (from {} total)",
            tagged_items.len(),
            total_items
        );

        tagged_items
    }

    /// Get contextual news for a specific market using Google News RSS
    /// Primary: Google News RSS with dynamic queries (free, unlimited, very current)
    /// Fallback: Exa.ai semantic search (optional, if API key configured)
    ///
    /// `outcome_titles` - For multi-outcome markets, the titles of each outcome
    #[instrument(skip(self, outcome_titles))]
    pub async fn get_market_news(
        &self,
        market_title: &str,
        market_id: &str,
        limit: usize,
        outcome_titles: Option<Vec<String>>,
    ) -> Result<NewsFeed, NewsServiceError> {
        let cache_key = format!("market:{}:{}", market_id, limit);

        // Check cache first (10 minute TTL to minimize redundant requests)
        {
            let cache = self.news_cache.read().await;
            if let Some(entry) = cache.get(&cache_key) {
                if !entry.is_expired() {
                    info!(
                        "Cache HIT for market news '{}': {} items",
                        market_id,
                        entry.data.items.len()
                    );
                    return Ok(entry.data.clone());
                }
            }
        }
        info!(
            "Cache MISS for market news '{}', fetching fresh...",
            market_id
        );

        info!(
            "=== MARKET NEWS SEARCH === market='{}' outcomes={:?}",
            market_title,
            outcome_titles.as_ref().map(|o| o.len()).unwrap_or(0)
        );

        // Log the query that will be sent to Google News
        info!(
            "Market: '{}' | Outcomes: {:?}",
            market_title, outcome_titles
        );

        // Extract key terms for relevance filtering
        let must_match_terms = extract_must_match_terms(market_title, outcome_titles.as_ref());
        info!(
            "Relevance filter terms for '{}': {:?}",
            market_title, must_match_terms
        );

        // Determine fetch multiplier based on market type
        // Sports markets need more raw results due to strict filtering
        let is_sports_market = market_title.to_lowercase().contains("mvp") ||
            market_title.to_lowercase().contains("nfl") ||
            market_title.to_lowercase().contains("nba") ||
            market_title.to_lowercase().contains("mlb") ||
            market_title.to_lowercase().contains("nhl") ||
            market_title.to_lowercase().contains("championship");

        let fetch_multiplier = if is_sports_market { 5 } else { 3 };

        // Try Google News RSS first (primary source - free, fast, current)
        let items = match self
            .google_news
            .search_market_news(market_title, outcome_titles.as_ref(), limit * fetch_multiplier)
            .await
        {
            Ok(results) => {
                info!(
                    "✓ Google News returned {} raw results for market '{}'",
                    results.len(),
                    market_title
                );

                // Filter for relevance and tag items with market context
                // First pass: try with 30-day limit
                let filtered_30d: Vec<NewsItem> = results
                    .iter()
                    .cloned()
                    .filter(|item| {
                        // Filter out articles older than 30 days (first pass)
                        let now = chrono::Utc::now();
                        let age_days = (now - item.published_at).num_days();
                        age_days <= 30
                    })
                    .collect();

                // Adaptive age window: expand if we got very few results
                let max_age_days = if filtered_30d.len() == 0 {
                    // No results in 30 days: try 180 days (6 months)
                    info!(
                        "0 articles in last 30 days, expanding to 180 days for market '{}'",
                        market_title
                    );
                    180
                } else if filtered_30d.len() < 3 {
                    // Few results: try 90 days (3 months)
                    info!(
                        "Only {} articles in last 30 days, expanding to 90 days for market '{}'",
                        filtered_30d.len(),
                        market_title
                    );
                    90
                } else {
                    30
                };

                // Apply actual filtering with age limit
                let filtered: Vec<NewsItem> = results
                    .into_iter()
                    .filter(|item| {
                        // Filter out articles older than max_age_days
                        let now = chrono::Utc::now();
                        let age_days = (now - item.published_at).num_days();
                        if age_days > max_age_days {
                            info!(
                                "✗ FILTERED OUT (too old): '{}' - {} days old (limit: {} days)",
                                item.title,
                                age_days,
                                max_age_days
                            );
                            return false;
                        }

                        // Filter out articles with suspiciously short titles (parsing errors)
                        if item.title.len() < 15 {
                            info!(
                                "✗ FILTERED OUT (title too short): '{}' (only {} chars)",
                                item.title,
                                item.title.len()
                            );
                            return false;
                        }

                        // If no must-match terms, accept all (let Google's relevance stand)
                        if must_match_terms.is_empty() {
                            return true;
                        }

                        let text = format!("{} {}", item.title, item.summary).to_lowercase();

                        // STRICTER FILTERING: Check for geography + topic relevance
                        // For location-specific markets (US, China, etc.), require the location
                        let market_lower = market_title.to_lowercase();
                        let has_required_geography = if market_lower.contains("us ")
                            || market_lower.contains(" us")
                            || market_lower.starts_with("us ") {
                            // Market is about US - require US/United States/America in article
                            text.contains("united states")
                                || text.contains(" us ")
                                || text.contains("u.s.")
                                || text.contains("america")
                                || text.contains("american")
                        } else if market_lower.contains("china") {
                            // Market is about China - require China in article
                            text.contains("china") || text.contains("chinese")
                        } else if market_lower.contains("europe") {
                            text.contains("europe") || text.contains("european") || text.contains("eu ")
                        } else {
                            // No specific geography requirement
                            true
                        };

                        if !has_required_geography {
                            info!(
                                "✗ FILTERED OUT (wrong geography): '{}' - doesn't mention required location",
                                item.title
                            );
                            return false;
                        }

                        // Check for league/championship keywords (always relevant if present)
                        let league_keywords = [
                            "stanley cup", "super bowl", "world series", "nba finals",
                            "world cup", "olympics", "championship", "playoffs"
                        ];
                        let has_league_keyword = league_keywords.iter().any(|kw| text.contains(kw));

                        // Check for crypto keywords (Bitcoin markets should accept general crypto news)
                        let crypto_keywords = [
                            "bitcoin", "btc", "crypto", "cryptocurrency", "blockchain",
                            "ethereum", "solana", "binance", "coinbase"
                        ];
                        let is_crypto_market = market_lower.contains("bitcoin") ||
                            market_lower.contains("btc") ||
                            market_lower.contains("crypto") ||
                            market_lower.contains("ethereum");
                        let has_crypto_keyword = crypto_keywords.iter().any(|kw| text.contains(kw));

                        // Count how many key terms match
                        let match_count = must_match_terms
                            .iter()
                            .filter(|term| text.contains(&term.to_lowercase()))
                            .count();

                        // Determine if this is a multi-outcome market (many options like sports teams)
                        let is_multi_outcome = outcome_titles.as_ref().map_or(false, |outcomes| outcomes.len() > 10);

                        // Check if this is a sports market (NFL, NBA, MLB, etc.)
                        let is_sports_market = has_league_keyword ||
                            market_lower.contains("mvp") ||
                            market_lower.contains("player") ||
                            market_lower.contains("team") ||
                            market_lower.contains("championship");

                        // Smart matching thresholds - VERY LENIENT for sports and crypto
                        let required_matches = if is_sports_market {
                            // Sports markets: accept if it mentions the league/event at all
                            // "NFL" articles are relevant for "NFL MVP" markets
                            if has_league_keyword { 0 } else { 1 }
                        } else if is_crypto_market && has_crypto_keyword {
                            // Crypto markets: accept any article with crypto keywords
                            // "Crypto Black Friday" is relevant for "Bitcoin $100k" markets
                            0
                        } else if has_league_keyword && is_multi_outcome {
                            // Championship articles are always relevant if they mention the event
                            0
                        } else if is_multi_outcome {
                            // Multi-outcome markets (elections): very lenient
                            1
                        } else if must_match_terms.len() <= 2 {
                            // Few terms: just need 1
                            1
                        } else if must_match_terms.len() == 0 {
                            // No required terms extracted (e.g., all were prices): accept all
                            0
                        } else if has_required_geography && must_match_terms.len() >= 3 {
                            // Geography verified: lenient
                            if must_match_terms.len() >= 5 { 2 } else { 1 }
                        } else {
                            // Default: need 1 match (was 2, relaxed)
                            1
                        };

                        let is_relevant = match_count >= required_matches;

                        if !is_relevant {
                            info!(
                                "✗ FILTERED OUT (not enough matches): '{}' - only {}/{} terms matched (need {}): {:?}",
                                item.title,
                                match_count,
                                must_match_terms.len(),
                                required_matches,
                                must_match_terms
                            );
                        } else {
                            info!(
                                "✓ ACCEPTED: '{}' - {}/{} terms matched",
                                item.title,
                                match_count,
                                must_match_terms.len()
                            );
                        }

                        is_relevant
                    })
                    .map(|mut item| {
                        if !item.related_market_ids.contains(&market_id.to_string()) {
                            item.related_market_ids.push(market_id.to_string());
                        }
                        item.search_query = Some(market_title.to_string());
                        item
                    })
                    .collect();

                // For multi-outcome markets (e.g., "Which movie will win?"), ensure diversity
                // Don't show 5 articles all about the same outcome
                let filtered: Vec<NewsItem> = if let Some(outcomes) = outcome_titles.as_ref() {
                    if outcomes.len() > 3 {
                        // Determine diversity limit based on market type
                        // Sports markets (many players): allow 3-4 articles per player
                        // Other multi-outcome markets: limit to 2 per outcome
                        let per_outcome_limit = if outcomes.len() > 10 {
                            // Large sports markets (NFL MVP, NBA Champion, etc.): 3 per player
                            3
                        } else if outcomes.len() > 5 {
                            // Medium markets: 2 per outcome
                            2
                        } else {
                            // Small markets: 2 per outcome
                            2
                        };

                        // Multi-outcome market: limit articles per outcome for diversity
                        let mut outcome_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
                        let mut diversified: Vec<NewsItem> = Vec::new();

                        for item in filtered {
                            // Check which outcome this article mentions
                            let title_lower = item.title.to_lowercase();
                            let summary_lower = item.summary.to_lowercase();
                            let text = format!("{} {}", title_lower, summary_lower);

                            let mut matched_outcome: Option<String> = None;
                            for outcome in outcomes {
                                let outcome_lower = outcome.to_lowercase();
                                if text.contains(&outcome_lower) {
                                    matched_outcome = Some(outcome.clone());
                                    break;
                                }
                            }

                            // Count articles per outcome and limit based on market type
                            if let Some(outcome) = matched_outcome {
                                let count = outcome_counts.entry(outcome.clone()).or_insert(0);
                                if *count < per_outcome_limit {
                                    *count += 1;
                                    diversified.push(item);
                                } else {
                                    info!("✗ DIVERSITY FILTER: Skipping '{}' - already have {} articles about '{}'", item.title, per_outcome_limit, outcome);
                                }
                            } else {
                                // Article doesn't clearly match any outcome, include it (general league news)
                                diversified.push(item);
                            }
                        }

                        diversified.into_iter().take(limit).collect()
                    } else {
                        // Binary or small market: just take the limit
                        filtered.into_iter().take(limit).collect()
                    }
                } else {
                    // No outcomes provided: just take the limit
                    filtered.into_iter().take(limit).collect()
                };

                info!(
                    "After relevance filtering: {} results for '{}'",
                    filtered.len(),
                    market_title
                );

                // If Google News returned no results, try fallbacks
                if filtered.is_empty() {
                    info!("Google News returned 0 results after filtering, trying fallbacks...");

                    // Try Exa as fallback if configured
                    if self.exa.is_some() {
                        info!("Trying Exa.ai as fallback...");
                        match self
                            .try_exa_search(market_title, market_id, limit, outcome_titles.as_ref())
                            .await
                        {
                            Ok(exa_results) if !exa_results.is_empty() => exa_results,
                            Ok(_) | Err(_) => {
                                // Don't use RSS fallback for market-specific news
                                // General RSS feeds won't have specific market news
                                info!(
                                    "No results from Google News or Exa for '{}'. Skipping RSS fallback (not relevant for specific markets).",
                                    market_title
                                );
                                vec![]
                            }
                        }
                    } else {
                        // No Exa configured - don't use RSS fallback for market news
                        info!(
                            "No Exa configured and Google News returned 0 results for '{}'. Skipping RSS fallback (not relevant for specific markets).",
                            market_title
                        );
                        vec![]
                    }
                } else {
                    filtered
                }
            }
            Err(e) => {
                info!("✗ Google News search failed for '{}': {}", market_title, e);

                // Try Exa as fallback if configured
                if self.exa.is_some() {
                    info!("Trying Exa.ai as fallback...");
                    match self
                        .try_exa_search(market_title, market_id, limit, outcome_titles.as_ref())
                        .await
                    {
                        Ok(exa_results) if !exa_results.is_empty() => exa_results,
                        Ok(_) => {
                            // Exa returned 0 results
                            info!(
                                "Exa.ai returned 0 results for '{}'. Skipping RSS fallback (not relevant for specific markets).",
                                market_title
                            );
                            vec![]
                        }
                        Err(exa_err) => {
                            info!("Exa.ai also failed: {}", exa_err);
                            // Don't use RSS fallback for market-specific news
                            info!(
                                "No results from Google News or Exa for '{}'. Skipping RSS fallback (not relevant for specific markets).",
                                market_title
                            );
                            vec![]
                        }
                    }
                } else {
                    // No Exa, don't use RSS fallback for market news
                    info!(
                        "Google News failed and no Exa configured for '{}'. Skipping RSS fallback (not relevant for specific markets).",
                        market_title
                    );
                    vec![]
                }
            }
        };

        // Sort by published_at descending (newest first)
        let mut sorted_items = items;
        sorted_items.sort_by(|a, b| b.published_at.cmp(&a.published_at));

        let feed = NewsFeed {
            total_count: sorted_items.len(),
            items: sorted_items,
            next_cursor: None,
        };

        // Cache results (5 minute TTL for market news - faster recovery from bad results)
        // Don't cache empty results as long - they might be due to overly strict filtering
        let cache_ttl = if feed.items.is_empty() {
            Duration::from_secs(60) // 1 minute for empty results
        } else {
            Duration::from_secs(300) // 5 minutes for good results
        };

        {
            let mut cache = self.news_cache.write().await;
            if cache.len() >= self.config.max_cache_entries {
                cache.retain(|_, entry| !entry.is_expired());
            }
            cache.insert(cache_key, CacheEntry::new(feed.clone(), cache_ttl));
        }

        Ok(feed)
    }

    /// Get market news with semantic matching enhancement
    ///
    /// This method combines:
    /// 1. Keyword-based matching (existing get_market_news)
    /// 2. Semantic embedding matching (finds related articles by meaning)
    ///
    /// Requires OPENAI_API_KEY environment variable for semantic matching
    #[instrument(skip(self, outcome_titles))]
    pub async fn get_market_news_with_semantics(
        &self,
        market_title: &str,
        market_id: &str,
        limit: usize,
        outcome_titles: Option<Vec<String>>,
    ) -> Result<NewsFeed, NewsServiceError> {
        // 1. Get keyword-based results (existing logic)
        let keyword_feed = self
            .get_market_news(market_title, market_id, limit, outcome_titles.clone())
            .await?;

        // 2. If semantic search is not enabled, return keyword results only
        let (Some(client), Some(store)) = (&self.embedding_client, &self.embedding_store) else {
            debug!("Semantic matching not available, returning keyword results only");
            return Ok(keyword_feed);
        };

        info!("Enhancing with semantic matching for market: {}", market_id);

        // 3. Load all market embeddings from database
        let market_embeddings = match store.load_all_market_embeddings() {
            Ok(embs) => embs,
            Err(e) => {
                debug!("Failed to load market embeddings: {}, falling back to keyword-only", e);
                return Ok(keyword_feed);
            }
        };

        if market_embeddings.is_empty() {
            info!("No market embeddings found, skipping semantic matching");
            return Ok(keyword_feed);
        }

        // 4. Get RSS items to search through
        let rss_items = match self.get_rss_items().await {
            Ok(items) => items,
            Err(e) => {
                debug!("Failed to get RSS items: {}, falling back to keyword-only", e);
                return Ok(keyword_feed);
            }
        };

        // 5. Find semantically similar articles
        let mut semantic_items: Vec<NewsItem> = Vec::new();

        for item in rss_items.iter().take(100) {
            // Generate embedding for this article (check cache first)
            let article_id = format!("{:x}", md5::compute(&item.url));

            let news_embedding = if let Ok(Some(cached)) = store.get_news_embedding(&article_id) {
                cached.embedding
            } else {
                // Generate new embedding
                match client.embed_news(&item.title, &item.summary).await {
                    Ok(emb) => {
                        // Cache it
                        let news_emb = NewsEmbedding::new(
                            article_id.clone(),
                            format!("{} {}", item.title, item.summary),
                            emb.clone(),
                        );
                        let _ = store.save_news_embedding(&news_emb);
                        emb
                    }
                    Err(e) => {
                        debug!("Failed to generate embedding for '{}': {}", item.title, e);
                        continue;
                    }
                }
            };

            // Find similar markets
            let matches = find_similar_markets(
                &news_embedding,
                &market_embeddings,
                10,   // top 10 markets
                0.70, // 70% similarity threshold
            );

            if !matches.is_empty() {
                // Collect related market IDs (limit to 5 most relevant)
                let related_markets: Vec<String> = matches
                    .iter()
                    .take(5)
                    .map(|m| m.market_id.clone())
                    .collect();

                // Check if our target market is in the matches
                for similarity_match in &matches {
                    if similarity_match.market_id == market_id {
                        let mut item_copy = item.clone();
                        item_copy.relevance_score = similarity_match.score;
                        item_copy.search_query = Some(format!("semantic:{:.2}", similarity_match.score));
                        item_copy.related_market_ids = related_markets.clone();

                        info!(
                            "Semantic match found: '{}' (score: {:.3}, {} related markets)",
                            item.title,
                            similarity_match.score,
                            related_markets.len()
                        );

                        semantic_items.push(item_copy);
                        break;
                    }
                }
            }
        }

        info!(
            "Found {} semantic matches for market '{}'",
            semantic_items.len(),
            market_id
        );

        // 6. Combine keyword and semantic results
        let combined_feed = self.combine_news_results(keyword_feed, semantic_items, limit);

        Ok(combined_feed)
    }

    /// Generate embeddings for all active markets (run this periodically)
    ///
    /// This should be called:
    /// - On startup (if embeddings DB is empty)
    /// - Daily via background job
    /// - When new markets are added
    pub async fn generate_market_embeddings(&self) -> Result<usize, NewsServiceError> {
        let (Some(client), Some(store)) = (&self.embedding_client, &self.embedding_store) else {
            return Err(NewsServiceError::NotConfigured(
                "Embedding client/store not configured".to_string(),
            ));
        };

        let Some(market_service) = &self.market_service else {
            return Err(NewsServiceError::NotConfigured(
                "Market service not set".to_string(),
            ));
        };

        info!("Generating embeddings for all active markets...");

        // Get all markets (limit to 500 most active)
        let markets = market_service
            .get_all_markets(Some(500))
            .await
            .map_err(|e| NewsServiceError::Rss(terminal_news::NewsError::RequestFailed(e.to_string())))?;

        let mut generated = 0;

        for market in markets {
            // Extract outcome titles
            let outcome_titles: Option<Vec<String>> = market.options_json.as_ref().and_then(|json| {
                serde_json::from_str::<Vec<serde_json::Value>>(json)
                    .ok()
                    .map(|options| {
                        options
                            .iter()
                            .filter_map(|opt| {
                                opt.get("title")
                                    .or_else(|| opt.get("name"))
                                    .or_else(|| opt.get("outcome"))
                                    .and_then(|n| n.as_str())
                                    .map(String::from)
                            })
                            .collect()
                    })
            });

            // Generate embedding
            match client
                .embed_market(
                    &market.title,
                    market.description.as_deref(),
                    outcome_titles.as_ref(),
                )
                .await
            {
                Ok(embedding) => {
                    let market_emb = terminal_embedding::MarketEmbedding::new(
                        market.id.clone(),
                        market.platform.to_string(),
                        market.title.clone(),
                        embedding,
                    );

                    if let Err(e) = store.save_market_embedding(&market_emb) {
                        debug!("Failed to save embedding for {}: {}", market.id, e);
                    } else {
                        generated += 1;
                        if generated % 10 == 0 {
                            info!("Generated {} market embeddings...", generated);
                        }
                    }
                }
                Err(e) => {
                    debug!("Failed to generate embedding for {}: {}", market.id, e);
                }
            }

            // Small delay to avoid rate limiting
            if generated % 50 == 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }

        info!("Completed: Generated {} market embeddings", generated);
        Ok(generated)
    }

    /// Combine keyword and semantic news results
    fn combine_news_results(
        &self,
        keyword_feed: NewsFeed,
        semantic_items: Vec<NewsItem>,
        limit: usize,
    ) -> NewsFeed {
        let mut combined_map: HashMap<String, NewsItem> = HashMap::new();

        // Add keyword results (higher priority)
        for item in keyword_feed.items {
            combined_map.insert(item.id.clone(), item);
        }

        // Add semantic results (only if not already present)
        for item in semantic_items {
            combined_map.entry(item.id.clone()).or_insert(item);
        }

        // Sort by relevance score, then by published date
        let mut items: Vec<NewsItem> = combined_map.into_values().collect();
        items.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.published_at.cmp(&a.published_at))
        });

        // Limit results
        items.truncate(limit);

        NewsFeed {
            total_count: items.len(),
            items,
            next_cursor: None,
        }
    }

    /// Try Exa.ai semantic search (optional fallback)
    async fn try_exa_search(
        &self,
        market_title: &str,
        market_id: &str,
        limit: usize,
        outcome_titles: Option<&Vec<String>>,
    ) -> Result<Vec<NewsItem>, NewsServiceError> {
        let exa = self
            .exa
            .as_ref()
            .ok_or_else(|| NewsServiceError::NotConfigured("Exa not configured".to_string()))?;

        let search_query = build_semantic_query(market_title, outcome_titles);
        let must_match_terms = extract_must_match_terms(market_title, outcome_titles);

        let params = NewsSearchParams {
            query: Some(search_query),
            limit: limit * 4,
            time_range: Some("30d".to_string()),
            market_id: Some(market_id.to_string()),
            skip_embeddings: false,
        };

        let results = exa
            .search_news(&params)
            .await
            .map_err(NewsServiceError::Rss)?;

        info!("Exa returned {} raw results", results.len());

        // Filter and tag
        let filtered: Vec<NewsItem> = results
            .into_iter()
            .filter(|item| {
                // Filter out articles older than 30 days
                let now = chrono::Utc::now();
                let age_days = (now - item.published_at).num_days();
                if age_days > 30 {
                    return false;
                }

                if must_match_terms.is_empty() {
                    return true;
                }
                let text = format!("{} {}", item.title, item.summary).to_lowercase();
                must_match_terms
                    .iter()
                    .any(|term| text.contains(&term.to_lowercase()))
            })
            .take(limit)
            .map(|mut item| {
                if !item.related_market_ids.contains(&market_id.to_string()) {
                    item.related_market_ids.push(market_id.to_string());
                }
                item.search_query = Some(market_title.to_string());
                item
            })
            .collect();

        info!("After Exa filtering: {} results", filtered.len());
        Ok(filtered)
    }

    /// Get full article content via Firecrawl (if configured)
    #[instrument(skip(self))]
    pub async fn get_article_content(&self, url: &str) -> Result<ArticleContent, NewsServiceError> {
        let Some(firecrawl) = &self.firecrawl else {
            return Err(NewsServiceError::NotConfigured(
                "Firecrawl not configured".to_string(),
            ));
        };

        // Check cache first
        {
            let cache = self.article_cache.read().await;
            if let Some(entry) = cache.get(url) {
                if !entry.is_expired() {
                    debug!("Cache hit for article: {}", url);
                    return Ok(ArticleContent {
                        markdown: entry.data.clone(),
                        title: None,
                        description: None,
                        image_url: None,
                        site_name: None,
                    });
                }
            }
        }

        // Scrape with Firecrawl
        debug!("Scraping article: {}", url);
        let content = firecrawl
            .scrape_article(url)
            .await
            .map_err(NewsServiceError::Firecrawl)?;

        // Cache the content
        {
            let mut cache = self.article_cache.write().await;
            cache.insert(
                url.to_string(),
                CacheEntry::new(content.markdown.clone(), Duration::from_secs(600)),
            );
        }

        Ok(content)
    }

    /// Cleanup expired cache entries
    pub async fn cleanup_cache(&self) {
        {
            let mut cache = self.news_cache.write().await;
            let before = cache.len();
            cache.retain(|_, entry| !entry.is_expired());
            let after = cache.len();
            if before != after {
                debug!("Cleaned up {} expired news cache entries", before - after);
            }
        }

        {
            let mut cache = self.article_cache.write().await;
            let before = cache.len();
            cache.retain(|_, entry| !entry.is_expired());
            let after = cache.len();
            if before != after {
                debug!(
                    "Cleaned up {} expired article cache entries",
                    before - after
                );
            }
        }
    }
}

/// Extract terms that MUST appear in news results for relevance
/// Only extracts HIGH-VALUE terms: proper nouns, locations, key topics
/// Also extracts names from outcome titles when available
fn extract_must_match_terms(
    market_title: &str,
    outcome_titles: Option<&Vec<String>>,
) -> Vec<String> {
    let mut terms = Vec::new();

    // Extensive stop word list - common words that don't indicate topic
    let stop_words: std::collections::HashSet<&str> = [
        // Question words
        "will",
        "what",
        "who",
        "which",
        "when",
        "where",
        "how",
        "why",
        // Common verbs
        "be",
        "is",
        "are",
        "was",
        "were",
        "been",
        "being",
        "have",
        "has",
        "had",
        "do",
        "does",
        "did",
        "will",
        "would",
        "could",
        "should",
        "may",
        "might",
        "can",
        "must",
        "shall",
        "win",
        "lose",
        "become",
        "meet",
        "reach",
        "hit",
        "make",
        "take",
        "get",
        "give",
        "find",
        "think",
        "say",
        "said",
        "go",
        "come",
        // Articles and prepositions
        "the",
        "a",
        "an",
        "in",
        "on",
        "at",
        "to",
        "for",
        "of",
        "with",
        "by",
        "from",
        "up",
        "down",
        "out",
        "off",
        "over",
        "under",
        "again",
        "further",
        "then",
        "once",
        // Common adjectives/adverbs
        "there",
        "here",
        "this",
        "that",
        "these",
        "those",
        "such",
        "other",
        "another",
        "more",
        "most",
        "some",
        "any",
        "no",
        "not",
        "only",
        "own",
        "same",
        "than",
        "too",
        "very",
        "just",
        "also",
        "now",
        "even",
        "still",
        "already",
        "always",
        "never",
        "ever",
        "often",
        "sometimes",
        "usually",
        "really",
        "actually",
        "first",
        "last",
        "next",
        "new",
        "old",
        "good",
        "bad",
        "great",
        "little",
        "big",
        "high",
        "low",
        "long",
        "short",
        "early",
        "late",
        "least",
        "less",
        "many",
        "much",
        // Time words
        "before",
        "after",
        "during",
        "while",
        "until",
        "since",
        "year",
        "years",
        "month",
        "months",
        "week",
        "weeks",
        "day",
        "days",
        "time",
        "times",
        "today",
        "tomorrow",
        "yesterday",
        "tonight",
        // Month names (temporal context, not topic-specific)
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
        "jan",
        "feb",
        "mar",
        "apr",
        "jun",
        "jul",
        "aug",
        "sep",
        "sept",
        "oct",
        "nov",
        "dec",
        // Seasons
        "spring",
        "summer",
        "fall",
        "autumn",
        "winter",
        // Year references
        "2024",
        "2025",
        "2026",
        "2027",
        // Market/prediction specific
        "market",
        "prediction",
        "price",
        "value",
        "rate",
        "percent",
        "percentage",
        // REMOVED: "election" and "presidential" - these are important topical terms
        "champion",
        "winner",
        "championship",
        "title",
        "goal",
        "goals",
        "point",
        "points",
        "score",
        "game",
        "match",
        "season",
        "country",
        "world",
        "worlds",
        "global",
        "national",
        "international",
        // Numbers as words
        "one",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine",
        "ten",
        "hundred",
        "thousand",
        "million",
        "billion",
    ]
    .into_iter()
    .collect();

    // Country adjective -> country name mappings
    let country_mappings: std::collections::HashMap<&str, &str> = [
        ("turkish", "turkey"),
        ("american", "america"),
        ("british", "britain"),
        ("french", "france"),
        ("german", "germany"),
        ("chinese", "china"),
        ("russian", "russia"),
        ("ukrainian", "ukraine"),
        ("israeli", "israel"),
        ("iranian", "iran"),
        ("brazilian", "brazil"),
        ("mexican", "mexico"),
        ("canadian", "canada"),
        ("japanese", "japan"),
        ("korean", "korea"),
        ("indian", "india"),
        ("chilean", "chile"),
        ("portuguese", "portugal"),
        ("honduran", "honduras"),
        ("venezuelan", "venezuela"),
        ("californian", "california"),
    ]
    .into_iter()
    .collect();

    // High-value topic words that should be included
    let topic_words: std::collections::HashSet<&str> = [
        // Natural disasters
        "earthquake",
        "hurricane",
        "tornado",
        "volcano",
        "tsunami",
        "wildfire",
        "flood",
        // Crypto
        "bitcoin",
        "crypto",
        "ethereum",
        "solana",
        "dogecoin",
        // Climate/Environment
        "climate",
        "emissions",
        "carbon",
        // Conflict
        "war",
        "ceasefire",
        "invasion",
        "conflict",
        // Economy
        "recession",
        "inflation",
        "unemployment",
        "gdp",
        "trillionaire",
        "billionaire",
        "fed",
        "federal",
        "reserve",
        "treasury",
        "dollar",
        "interest",
        "rates",
        "fomc",
        "powell",
        "economy",
        "economic",
        // Health
        "pandemic",
        "virus",
        "vaccine",
        "covid",
        "outbreak",
        // Space
        "asteroid",
        "spacex",
        "nasa",
        "mars",
        "moon",
        "starship",
        "rocket",
        // Tech
        "ai",
        "artificial",
        "intelligence",
        "chatgpt",
        "openai",
        // Nuclear
        "nuclear",
        "atomic",
        // Sports (specific enough to be useful)
        "basketball",
        "football",
        "baseball",
        "soccer",
        "nfl",
        "nba",
        "mlb",
        "fifa",
        "superbowl",
        "playoffs",
        "championship",
        // Religion
        "pope",
        "vatican",
        "catholic",
        "cardinal",
    ]
    .into_iter()
    .collect();

    for word in market_title.split(|c: char| !c.is_alphanumeric() && c != '$') {
        if word.len() < 3 {
            continue;
        }
        let lower = word.to_lowercase();

        // Skip stop words
        if stop_words.contains(lower.as_str()) {
            continue;
        }

        // Skip price targets and numbers (too specific for news search)
        // Examples: "$100k", "100k", "$80k", "1000"
        if word.starts_with('$') || word.chars().all(|c| c.is_numeric() || c == 'k' || c == 'm') {
            continue;
        }

        // Check if it's a country adjective
        if let Some(country) = country_mappings.get(lower.as_str()) {
            // Add country name (more likely to appear in news)
            let cap = format!("{}{}", &country[..1].to_uppercase(), &country[1..]);
            if !terms.iter().any(|t: &String| t.to_lowercase() == *country) {
                terms.push(cap);
            }
        }
        // Check if it's a high-value topic word
        else if topic_words.contains(lower.as_str()) {
            if !terms.iter().any(|t: &String| t.to_lowercase() == lower) {
                terms.push(word.to_string());
            }
        }
        // Check if it's a proper noun (capitalized) - but exclude common words
        else if word.len() >= 4
            && word
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
        {
            // Blacklist of common capitalized words that aren't useful for filtering
            let capitalized_stop: std::collections::HashSet<&str> = [
                "Sports",
                "News",
                "Media",
                "Game",
                "Games",
                "Video",
                "Cover",
                "Athlete",
                "Team",
                "Teams",
                "League",
                "Association",
                "Club",
                "Clubs",
                "College",
                "University",
                "Award",
                "Awards",
                "Prize",
                "Show",
                "Series",
                "Event",
                "Report",
                "Update",
                "Watch",
                "Live",
                "Breaking",
                "Latest",
                "Today",
            ]
            .into_iter()
            .collect();

            if !capitalized_stop.contains(word) {
                // Likely a proper noun (place, name, company, etc.)
                if !terms.iter().any(|t: &String| t.to_lowercase() == lower) {
                    terms.push(word.to_string());
                }
            }
        }
        // Extract any other meaningful content word (e.g., "gold", "oil", "stock")
        // This catches important keywords that aren't capitalized or in topic_words
        else if word.len() >= 4 {
            if !terms.iter().any(|t: &String| t.to_lowercase() == lower) {
                terms.push(word.to_string());
            }
        }
    }

    // Extract FULL NAMES from outcome titles (e.g., "Will Elon Musk be..." -> "Elon Musk")
    if let Some(outcomes) = outcome_titles {
        let skip_words: std::collections::HashSet<&str> = [
            "Will", "The", "Be", "Is", "Are", "Who", "What", "Which", "Other", "None", "First",
            "Next", "World", "Worlds",
        ]
        .into_iter()
        .collect();

        for outcome in outcomes.iter().take(5) {
            // Skip if outcome is the same as market title
            if outcome == market_title {
                continue;
            }

            // Extract consecutive capitalized words as a FULL NAME
            let words: Vec<&str> = outcome.split_whitespace().collect();
            let mut name_parts: Vec<String> = Vec::new();

            for word in &words {
                let clean: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
                let is_capitalized = clean
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false);
                let is_skip = skip_words.contains(clean.as_str());
                let is_stop = stop_words.contains(clean.to_lowercase().as_str());

                if is_capitalized && !is_skip && !is_stop && clean.len() >= 2 {
                    name_parts.push(clean);
                } else if !name_parts.is_empty() {
                    // End of consecutive capitals - save the full name if it's 2+ words
                    if name_parts.len() >= 2 {
                        let full_name = name_parts.join(" ");
                        if !terms
                            .iter()
                            .any(|t: &String| t.to_lowercase() == full_name.to_lowercase())
                        {
                            terms.push(full_name);
                        }
                        // Also add the last name separately (e.g., "Lula" from "Luiz Inácio Lula")
                        // News articles often refer to people by last name only
                        if let Some(last_name) = name_parts.last() {
                            if last_name.len() >= 3
                                && !terms
                                    .iter()
                                    .any(|t: &String| t.to_lowercase() == last_name.to_lowercase())
                            {
                                terms.push(last_name.clone());
                            }
                        }
                    }
                    name_parts.clear();
                }
            }

            // Handle name at end of string
            if name_parts.len() >= 2 {
                let full_name = name_parts.join(" ");
                if !terms
                    .iter()
                    .any(|t: &String| t.to_lowercase() == full_name.to_lowercase())
                {
                    terms.push(full_name);
                }
                // Also add the last name separately
                if let Some(last_name) = name_parts.last() {
                    if last_name.len() >= 3
                        && !terms
                            .iter()
                            .any(|t: &String| t.to_lowercase() == last_name.to_lowercase())
                    {
                        terms.push(last_name.clone());
                    }
                }
            }
        }
    }

    terms
}

/// Build a semantic search query from market title and outcomes
/// Creates a SIMPLE query focused on the key entity (country, person, topic)
/// Exa's neural search works better with simple, entity-focused queries
fn build_semantic_query(market_title: &str, outcome_titles: Option<&Vec<String>>) -> String {
    // Country adjective -> country name
    let country_adjectives: std::collections::HashMap<&str, &str> = [
        ("turkish", "Turkey"),
        ("american", "United States"),
        ("british", "United Kingdom"),
        ("french", "France"),
        ("german", "Germany"),
        ("chinese", "China"),
        ("russian", "Russia"),
        ("ukrainian", "Ukraine"),
        ("israeli", "Israel"),
        ("iranian", "Iran"),
        ("brazilian", "Brazil"),
        ("mexican", "Mexico"),
        ("canadian", "Canada"),
        ("japanese", "Japan"),
        ("korean", "Korea"),
        ("indian", "India"),
        ("chilean", "Chile"),
        ("honduran", "Honduras"),
        ("venezuelan", "Venezuela"),
        ("portuguese", "Portugal"),
    ]
    .into_iter()
    .collect();

    // Direct country names to look for
    let country_names: std::collections::HashSet<&str> = [
        "russia", "ukraine", "china", "usa", "turkey", "israel", "iran", "syria",
        "north korea", "south korea", "taiwan", "india", "pakistan", "afghanistan",
        "iraq", "yemen", "lebanon", "gaza", "palestine", "france", "germany",
        "united kingdom", "britain", "spain", "italy", "poland", "japan",
        "brazil", "mexico", "canada", "australia", "venezuela", "argentina"
    ]
    .into_iter()
    .collect();

    let generic_words: std::collections::HashSet<&str> = [
        "will", "what", "who", "which", "when", "how", "the", "be", "is", "are", "next", "first",
        "win", "become", "meet", "reach", "hit", "before", "after", "its", "their", "price",
        "champion", "winner", "goals", "country", "world", "worlds",
    ]
    .into_iter()
    .collect();

    let mut primary_entity: Option<String> = None;
    let mut found_countries: Vec<String> = Vec::new();
    let mut secondary_terms: Vec<String> = Vec::new();
    let title_lower = market_title.to_lowercase();

    // First pass: find the PRIMARY entity (country or major proper noun)
    // Check for direct country names first (handles multi-word like "North Korea")
    for country in &country_names {
        if title_lower.contains(country) {
            // Capitalize first letter of each word
            let capitalized = country
                .split_whitespace()
                .map(|w| {
                    let mut chars = w.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            found_countries.push(capitalized);
        }
    }

    // If we found countries, combine them (e.g., "Russia Ukraine" for "Russia x Ukraine ceasefire")
    if !found_countries.is_empty() {
        primary_entity = Some(found_countries.join(" "));
    }

    // If no direct country name found, check for country adjectives
    if primary_entity.is_none() {
        for word in market_title.split(|c: char| !c.is_alphanumeric()) {
            if word.len() < 3 {
                continue;
            }
            let lower = word.to_lowercase();

            // Country adjective -> use country name as primary entity
            if let Some(country) = country_adjectives.get(lower.as_str()) {
                primary_entity = Some(country.to_string());
                break;
            }
        }
    }

    // Second pass: collect proper nouns from outcomes (candidate names, etc.)
    if let Some(outcomes) = outcome_titles {
        for outcome in outcomes.iter().take(5) {
            for word in outcome.split_whitespace() {
                let clean: String = word.chars().filter(|c| c.is_alphanumeric()).collect();
                if clean.len() >= 3
                    && clean
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                    && !generic_words.contains(clean.to_lowercase().as_str())
                    && !["Will", "The", "Be", "Is", "Are", "Other", "Who", "Yes", "No", "None", "Maybe"].contains(&clean.as_str())
                {
                    if !secondary_terms
                        .iter()
                        .any(|t| t.to_lowercase() == clean.to_lowercase())
                    {
                        secondary_terms.push(clean);
                    }
                }
            }
        }
    }

    // Build a SIMPLE query - just the primary entity + "news"
    // Exa's neural search works best with simple queries
    let query = if let Some(entity) = primary_entity {
        // For country-based markets: "Turkey news" or "Turkey politics"
        format!("{} news", entity)
    } else if !secondary_terms.is_empty() {
        // For person-based markets: "Elon Musk news"
        format!(
            "{} news",
            secondary_terms
                .into_iter()
                .take(2)
                .collect::<Vec<_>>()
                .join(" ")
        )
    } else {
        // Fallback: use cleaned market title
        market_title
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .split_whitespace()
            .filter(|w| w.len() >= 3 && !generic_words.contains(&w.to_lowercase().as_str()))
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
    };

    // Return the query (even if short - "Turkey news" is only 2 words but that's fine)
    query
}

/// Extract key entities from trending markets for strict news filtering
/// Returns entities like: "Trump", "Bitcoin", "Ukraine", "Super Bowl", "NBA", etc.
fn extract_market_entities(markets: &[PredictionMarket]) -> Vec<String> {
    let mut entities = std::collections::HashSet::new();

    // High-priority entities that should always be extracted if present
    let known_entities: std::collections::HashMap<&str, Vec<&str>> = [
        // Politicians / Leaders
        ("trump", vec!["Trump"]),
        ("biden", vec!["Biden"]),
        ("harris", vec!["Harris", "Kamala"]),
        ("desantis", vec!["DeSantis"]),
        ("obama", vec!["Obama"]),
        ("pence", vec!["Pence"]),
        ("vance", vec!["Vance"]),
        ("zelensky", vec!["Zelensky", "Ukraine"]),
        ("putin", vec!["Putin", "Russia"]),
        ("xi jinping", vec!["Xi Jinping", "Xi", "China"]),
        ("netanyahu", vec!["Netanyahu", "Israel"]),
        // Tech figures
        ("elon musk", vec!["Elon Musk", "Musk", "Tesla", "SpaceX"]),
        ("musk", vec!["Musk", "Elon"]),
        ("bezos", vec!["Bezos", "Amazon"]),
        ("zuckerberg", vec!["Zuckerberg", "Meta", "Facebook"]),
        ("altman", vec!["Altman", "OpenAI", "ChatGPT"]),
        // Crypto
        ("bitcoin", vec!["Bitcoin", "BTC", "crypto"]),
        ("ethereum", vec!["Ethereum", "ETH"]),
        ("solana", vec!["Solana", "SOL"]),
        ("crypto", vec!["crypto", "cryptocurrency"]),
        // Geopolitical
        ("ukraine", vec!["Ukraine", "Zelensky", "Kyiv"]),
        ("russia", vec!["Russia", "Putin", "Moscow"]),
        ("china", vec!["China", "Beijing", "Xi"]),
        ("taiwan", vec!["Taiwan"]),
        ("israel", vec!["Israel", "Gaza", "Netanyahu"]),
        ("gaza", vec!["Gaza", "Israel", "Hamas"]),
        ("iran", vec!["Iran", "Tehran"]),
        ("ceasefire", vec!["ceasefire", "peace", "war"]),
        // Sports - exact names for sports events
        ("super bowl", vec!["Super Bowl", "NFL"]),
        ("nfl", vec!["NFL", "football"]),
        ("nba", vec!["NBA", "basketball"]),
        ("nba champion", vec!["NBA", "basketball", "championship"]),
        ("mlb", vec!["MLB", "baseball"]),
        ("premier league", vec!["Premier League", "EPL"]),
        ("champions league", vec!["Champions League", "UEFA"]),
        ("la liga", vec!["La Liga"]),
        ("world cup", vec!["World Cup", "FIFA"]),
        ("world series", vec!["World Series", "MLB"]),
        // Economics
        ("fed", vec!["Fed", "Federal Reserve", "interest rate"]),
        ("inflation", vec!["inflation"]),
        ("recession", vec!["recession"]),
        ("treasury", vec!["Treasury"]),
        // AI
        ("openai", vec!["OpenAI", "ChatGPT", "GPT"]),
        ("ai", vec!["AI", "artificial intelligence"]),
        ("chatgpt", vec!["ChatGPT", "OpenAI"]),
        // Space
        ("spacex", vec!["SpaceX", "Starship"]),
        ("nasa", vec!["NASA"]),
        ("mars", vec!["Mars"]),
        ("starship", vec!["Starship", "SpaceX"]),
        // Companies
        ("nvidia", vec!["Nvidia", "NVDA"]),
        ("tesla", vec!["Tesla", "TSLA"]),
        ("apple", vec!["Apple", "AAPL"]),
        ("google", vec!["Google", "Alphabet"]),
        ("microsoft", vec!["Microsoft", "MSFT"]),
        ("amazon", vec!["Amazon", "AMZN"]),
    ]
    .into_iter()
    .collect();

    for market in markets {
        let title_lower = market.title.to_lowercase();

        // Check for known entities in market title
        for (keyword, related_entities) in &known_entities {
            if title_lower.contains(keyword) {
                for entity in related_entities {
                    entities.insert(entity.to_string());
                }
            }
        }

        // Also extract proper nouns (consecutive capitalized words)
        let words: Vec<&str> = market.title.split_whitespace().collect();
        let mut i = 0;
        while i < words.len() {
            let word = words[i];
            let is_cap = word
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false);

            // Skip common title words
            let skip_words = [
                "Will", "What", "Who", "Which", "When", "How", "The", "Is", "Are", "Can", "Does",
                "Do",
            ];
            if is_cap && !skip_words.contains(&word) && word.len() >= 3 {
                let mut phrase_words = vec![word];
                let mut j = i + 1;

                // Collect consecutive capitalized words
                while j < words.len() {
                    let next = words[j];
                    let next_cap = next
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false);
                    if next_cap && !skip_words.contains(&next) && next.len() >= 2 {
                        phrase_words.push(next);
                        j += 1;
                    } else {
                        break;
                    }
                }

                // Add the phrase if it's meaningful
                let phrase: String = phrase_words
                    .iter()
                    .map(|w| {
                        w.chars()
                            .filter(|c| c.is_alphanumeric() || *c == '\'')
                            .collect::<String>()
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                if phrase.len() >= 3 {
                    entities.insert(phrase);
                }

                i = j;
            } else {
                i += 1;
            }
        }
    }

    // Convert to vec and sort by length (longer = more specific = better matches)
    let mut result: Vec<String> = entities.into_iter().collect();
    result.sort_by(|a, b| b.len().cmp(&a.len()));
    result
}

/// Extract keywords from market title for dynamic Google News feed generation
/// Simplified keyword extraction focused on proper nouns and key entities
fn extract_dynamic_feed_keywords(market_title: &str) -> String {
    let stop_words: std::collections::HashSet<&str> = [
        "will", "what", "who", "which", "when", "where", "how", "why", "the", "a", "an", "in",
        "on", "at", "to", "for", "of", "with", "by", "from", "be", "is", "are", "was", "were",
        "been", "being", "have", "has", "had", "do", "does", "did", "would", "could", "should",
        "may", "might", "can", "must", "win", "lose", "become", "next", "first", "before",
        "after", "market", "prediction", "hit", "reach", "price", "value", "year", "years",
        "month", "months", "2024", "2025", "2026", "2027", "january", "february", "march",
        "april", "may", "june", "july", "august", "september", "october", "november", "december",
    ]
    .into_iter()
    .collect();

    let high_value_terms: std::collections::HashSet<&str> = [
        "bitcoin", "crypto", "ethereum", "ai", "president", "election", "championship", "super",
        "bowl", "olympics", "war", "peace", "ceasefire", "fed", "nba", "nfl", "mlb", "ipo",
        "trillionaire", "billionaire",
    ]
    .into_iter()
    .collect();

    let mut keywords = Vec::new();

    for word in market_title.split(|c: char| !c.is_alphanumeric()) {
        if word.len() < 3 {
            continue;
        }

        let lower = word.to_lowercase();

        // Skip stop words
        if stop_words.contains(lower.as_str()) {
            continue;
        }

        // Include high-value terms
        if high_value_terms.contains(lower.as_str()) {
            if !keywords.iter().any(|k: &String| k.eq_ignore_ascii_case(&word)) {
                keywords.push(word.to_string());
            }
            continue;
        }

        // Include proper nouns (capitalized words)
        if word.len() >= 4
            && word
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
        {
            if !keywords.iter().any(|k: &String| k.eq_ignore_ascii_case(&word)) {
                keywords.push(word.to_string());
            }
        }
    }

    // Limit to top 5 keywords for focused search
    keywords.truncate(5);
    keywords.join(" ")
}

/// Errors that can occur in NewsService
#[derive(Debug, thiserror::Error)]
pub enum NewsServiceError {
    #[error("RSS error: {0}")]
    Rss(#[from] NewsError),

    #[error("Firecrawl error: {0}")]
    Firecrawl(NewsError),

    #[error("Service not configured: {0}")]
    NotConfigured(String),
}
