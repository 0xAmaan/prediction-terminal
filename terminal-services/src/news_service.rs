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
}

impl NewsService {
    /// Create a new NewsService
    pub fn new(
        exa_api_key: Option<String>,
        firecrawl_api_key: Option<String>,
        config: NewsServiceConfig,
    ) -> Self {
        info!(
            "Initializing NewsService (Google News: enabled, Exa: {}, Firecrawl: {})",
            exa_api_key.is_some(),
            firecrawl_api_key.is_some()
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

        // Get RSS items and TRENDING markets only
        let all_items = self.get_rss_items().await.map_err(NewsServiceError::Rss)?;
        let markets = self.get_trending_markets().await;

        // If no markets, just return top news by recency
        let items: Vec<NewsItem> = if markets.is_empty() {
            all_items.into_iter().take(params.limit).collect()
        } else {
            // Extract key entities from trending markets
            let entities = extract_market_entities(&markets);
            info!(
                "Filtering news by {} entities from trending markets: {:?}",
                entities.len(),
                entities.iter().take(20).collect::<Vec<_>>()
            );

            // STRICT FILTER: Only include articles that mention an entity IN THE TITLE
            let mut matched_items: Vec<(NewsItem, f64, String)> = all_items
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

        let feed = NewsFeed {
            total_count: items.len(),
            items,
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

        // Try Google News RSS first (primary source - free, fast, current)
        let items = match self
            .google_news
            .search_market_news(market_title, outcome_titles.as_ref(), limit * 3)
            .await
        {
            Ok(results) => {
                info!(
                    "✓ Google News returned {} raw results for market '{}'",
                    results.len(),
                    market_title
                );

                // Filter for relevance and tag items with market context
                let filtered: Vec<NewsItem> = results
                    .into_iter()
                    .filter(|item| {
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

                        // Count how many key terms match
                        let match_count = must_match_terms
                            .iter()
                            .filter(|term| text.contains(&term.to_lowercase()))
                            .count();

                        // More lenient matching: require fewer matches if we already verified geography
                        let required_matches = if must_match_terms.len() <= 2 {
                            1 // For markets with few terms, just need 1
                        } else if has_required_geography && must_match_terms.len() >= 3 {
                            // If geography is correct, be more lenient (1-2 terms)
                            if must_match_terms.len() >= 5 { 2 } else { 1 }
                        } else {
                            2 // Default: need 2 matches
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
                    .take(limit)
                    .map(|mut item| {
                        if !item.related_market_ids.contains(&market_id.to_string()) {
                            item.related_market_ids.push(market_id.to_string());
                        }
                        item.search_query = Some(market_title.to_string());
                        item
                    })
                    .collect();

                info!(
                    "After relevance filtering: {} results for '{}'",
                    filtered.len(),
                    market_title
                );

                // If Google News returned no results, try fallbacks
                if filtered.is_empty() {
                    info!("Google News returned 0 results, trying fallbacks...");

                    // Try Exa as fallback if configured
                    if self.exa.is_some() {
                        info!("Trying Exa.ai as fallback...");
                        match self
                            .try_exa_search(market_title, market_id, limit, outcome_titles.as_ref())
                            .await
                        {
                            Ok(exa_results) if !exa_results.is_empty() => exa_results,
                            Ok(_) | Err(_) => {
                                // Final fallback: RSS search
                                info!("Final fallback: RSS search...");
                                self.fallback_rss_search(
                                    market_title,
                                    market_id,
                                    limit,
                                    outcome_titles.as_ref(),
                                )
                                .await?
                            }
                        }
                    } else {
                        // No Exa, go straight to RSS fallback
                        info!("No Exa configured, using RSS fallback...");
                        self.fallback_rss_search(
                            market_title,
                            market_id,
                            limit,
                            outcome_titles.as_ref(),
                        )
                        .await?
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
                        Ok(exa_results) => exa_results,
                        Err(exa_err) => {
                            info!("Exa.ai also failed: {}", exa_err);
                            // Final fallback: RSS search
                            info!("Final fallback: RSS search...");
                            self.fallback_rss_search(
                                market_title,
                                market_id,
                                limit,
                                outcome_titles.as_ref(),
                            )
                            .await?
                        }
                    }
                } else {
                    // No Exa, go straight to RSS fallback
                    info!("No Exa configured, using RSS fallback...");
                    self.fallback_rss_search(
                        market_title,
                        market_id,
                        limit,
                        outcome_titles.as_ref(),
                    )
                    .await?
                }
            }
        };

        let feed = NewsFeed {
            total_count: items.len(),
            items,
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
            time_range: Some("7d".to_string()),
            market_id: Some(market_id.to_string()),
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

    /// Fallback RSS-based search when Exa is not available
    async fn fallback_rss_search(
        &self,
        market_title: &str,
        market_id: &str,
        limit: usize,
        outcome_titles: Option<&Vec<String>>,
    ) -> Result<Vec<NewsItem>, NewsServiceError> {
        let all_items = self.get_rss_items().await.map_err(NewsServiceError::Rss)?;

        // Use must-match terms for strict filtering (same as Exa post-filter)
        let must_match_terms = extract_must_match_terms(market_title, outcome_titles);

        info!(
            "RSS fallback for '{}' with must-match terms: {:?}",
            market_title, must_match_terms
        );

        // Require at least one must-match term to be present
        let mut matched: Vec<NewsItem> = all_items
            .into_iter()
            .filter(|item| {
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
                item.related_market_ids.push(market_id.to_string());
                item.search_query = Some(market_title.to_string());
                item
            })
            .collect();

        matched.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        Ok(matched)
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
        // Market/prediction specific
        "market",
        "prediction",
        "price",
        "value",
        "rate",
        "percent",
        "percentage",
        "election",
        "presidential",
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

    for word in market_title.split(|c: char| !c.is_alphanumeric()) {
        if word.len() < 3 {
            continue;
        }
        let lower = word.to_lowercase();

        // Skip stop words
        if stop_words.contains(lower.as_str()) {
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

    let generic_words: std::collections::HashSet<&str> = [
        "will", "what", "who", "which", "when", "how", "the", "be", "is", "are", "next", "first",
        "win", "become", "meet", "reach", "hit", "before", "after", "its", "their", "price",
        "champion", "winner", "goals", "country", "world", "worlds",
    ]
    .into_iter()
    .collect();

    let mut primary_entity: Option<String> = None;
    let mut secondary_terms: Vec<String> = Vec::new();

    // First pass: find the PRIMARY entity (country or major proper noun)
    for word in market_title.split(|c: char| !c.is_alphanumeric()) {
        if word.len() < 3 {
            continue;
        }
        let lower = word.to_lowercase();

        // Country adjective -> use country name as primary entity
        if let Some(country) = country_adjectives.get(lower.as_str()) {
            if primary_entity.is_none() {
                primary_entity = Some(country.to_string());
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
                    && !["Will", "The", "Be", "Is", "Are", "Other", "Who"].contains(&clean.as_str())
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
