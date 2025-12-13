//! Market Cache Service
//!
//! In-memory cache with SQLite persistence for instant market lookups.
//! This is the key to fast search and market list operations.

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use terminal_core::{Platform, PredictionMarket, TerminalError};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::MarketService;

/// Cache TTL in seconds (5 minutes)
const CACHE_TTL_SECS: i64 = 300;

/// Cached market with metadata
#[derive(Debug, Clone)]
struct CachedMarket {
    market: PredictionMarket,
    updated_at: DateTime<Utc>,
}

impl CachedMarket {
    fn is_fresh(&self) -> bool {
        let age = Utc::now().signed_duration_since(self.updated_at);
        age.num_seconds() < CACHE_TTL_SECS
    }
}

/// Background refresh request
#[derive(Debug)]
pub enum RefreshRequest {
    /// Refresh a single market
    Single { platform: Platform, market_id: String },
    /// Refresh all markets for a platform
    Platform(Platform),
    /// Refresh all markets
    All,
}

/// Market cache with in-memory + SQLite backing
pub struct MarketCache {
    /// In-memory cache for instant access
    cache: Arc<RwLock<HashMap<(Platform, String), CachedMarket>>>,
    /// SQLite connection for persistence
    db: Arc<parking_lot::Mutex<Connection>>,
    /// Underlying market service for API calls
    service: Arc<MarketService>,
    /// Channel to send refresh requests to background task
    refresh_tx: mpsc::Sender<RefreshRequest>,
}

impl MarketCache {
    /// Create a new MarketCache
    ///
    /// This will:
    /// 1. Open/create the SQLite database
    /// 2. Load cached markets from DB into memory
    /// 3. Start a background refresh task
    pub async fn new<P: AsRef<Path>>(
        db_path: P,
        service: MarketService,
    ) -> Result<Self, MarketCacheError> {
        // Open database
        let conn = Connection::open(db_path.as_ref()).map_err(MarketCacheError::Database)?;

        // Initialize schema
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS markets (
                platform TEXT NOT NULL,
                market_id TEXT NOT NULL,
                ticker TEXT,
                title TEXT NOT NULL,
                data JSON NOT NULL,
                updated_at INTEGER NOT NULL,
                PRIMARY KEY (platform, market_id)
            );

            CREATE INDEX IF NOT EXISTS idx_markets_platform
            ON markets(platform, updated_at);

            CREATE INDEX IF NOT EXISTS idx_markets_title
            ON markets(title COLLATE NOCASE);
            "#,
        )
        .map_err(MarketCacheError::Database)?;

        let db = Arc::new(parking_lot::Mutex::new(conn));
        let cache = Arc::new(RwLock::new(HashMap::new()));
        let service = Arc::new(service);

        // Load existing cached markets from DB
        let loaded = Self::load_from_db(&db, &cache)?;
        info!("Loaded {} markets from cache database", loaded);

        // Create refresh channel
        let (refresh_tx, refresh_rx) = mpsc::channel::<RefreshRequest>(100);

        let market_cache = Self {
            cache: Arc::clone(&cache),
            db: Arc::clone(&db),
            service: Arc::clone(&service),
            refresh_tx,
        };

        // Spawn background refresh task
        let cache_clone = Arc::clone(&cache);
        let db_clone = Arc::clone(&db);
        let service_clone = Arc::clone(&service);
        tokio::spawn(async move {
            Self::background_refresh_task(cache_clone, db_clone, service_clone, refresh_rx).await;
        });

        Ok(market_cache)
    }

    /// Load markets from SQLite into memory
    fn load_from_db(
        db: &Arc<parking_lot::Mutex<Connection>>,
        cache: &Arc<RwLock<HashMap<(Platform, String), CachedMarket>>>,
    ) -> Result<usize, MarketCacheError> {
        let conn = db.lock();

        let mut stmt = conn
            .prepare("SELECT platform, market_id, data, updated_at FROM markets")
            .map_err(MarketCacheError::Database)?;

        let rows = stmt
            .query_map([], |row| {
                let platform_str: String = row.get(0)?;
                let market_id: String = row.get(1)?;
                let data_json: String = row.get(2)?;
                let updated_at: i64 = row.get(3)?;
                Ok((platform_str, market_id, data_json, updated_at))
            })
            .map_err(MarketCacheError::Database)?;

        let mut loaded = 0;
        let mut write_cache = cache.write();

        for row in rows.flatten() {
            let (platform_str, market_id, data_json, updated_at) = row;

            let platform = match platform_str.as_str() {
                "kalshi" => Platform::Kalshi,
                "polymarket" => Platform::Polymarket,
                _ => continue,
            };

            if let Ok(market) = serde_json::from_str::<PredictionMarket>(&data_json) {
                let updated_at =
                    DateTime::from_timestamp(updated_at, 0).unwrap_or_else(Utc::now);

                write_cache.insert(
                    (platform, market_id),
                    CachedMarket { market, updated_at },
                );
                loaded += 1;
            }
        }

        Ok(loaded)
    }

    /// Background task that handles refresh requests
    async fn background_refresh_task(
        cache: Arc<RwLock<HashMap<(Platform, String), CachedMarket>>>,
        db: Arc<parking_lot::Mutex<Connection>>,
        service: Arc<MarketService>,
        mut rx: mpsc::Receiver<RefreshRequest>,
    ) {
        info!("Market cache background refresh task started");

        while let Some(request) = rx.recv().await {
            match request {
                RefreshRequest::Single { platform, market_id } => {
                    debug!("Refreshing single market: {:?}/{}", platform, market_id);
                    if let Err(e) =
                        Self::refresh_single(&cache, &db, &service, platform, &market_id).await
                    {
                        warn!("Failed to refresh market {}: {}", market_id, e);
                    }
                }
                RefreshRequest::Platform(platform) => {
                    debug!("Refreshing all {:?} markets", platform);
                    if let Err(e) =
                        Self::refresh_platform(&cache, &db, &service, platform).await
                    {
                        warn!("Failed to refresh {:?} markets: {}", platform, e);
                    }
                }
                RefreshRequest::All => {
                    debug!("Refreshing all markets");
                    // KALSHI_DISABLED: Only refresh Polymarket while focusing on it
                    for platform in [Platform::Polymarket] {
                        if let Err(e) =
                            Self::refresh_platform(&cache, &db, &service, platform).await
                        {
                            warn!("Failed to refresh {:?} markets: {}", platform, e);
                        }
                    }
                }
            }
        }
    }

    /// Refresh a single market from API
    async fn refresh_single(
        cache: &Arc<RwLock<HashMap<(Platform, String), CachedMarket>>>,
        db: &Arc<parking_lot::Mutex<Connection>>,
        service: &Arc<MarketService>,
        platform: Platform,
        market_id: &str,
    ) -> Result<(), MarketCacheError> {
        let market = service
            .get_market(platform, market_id)
            .await
            .map_err(MarketCacheError::Api)?;

        let now = Utc::now();
        let cached = CachedMarket {
            market: market.clone(),
            updated_at: now,
        };

        // Update memory cache
        cache.write().insert((platform, market_id.to_string()), cached);

        // Update SQLite
        Self::store_market_to_db(db, platform, &market, now)?;

        Ok(())
    }

    /// Refresh all markets for a platform
    async fn refresh_platform(
        cache: &Arc<RwLock<HashMap<(Platform, String), CachedMarket>>>,
        db: &Arc<parking_lot::Mutex<Connection>>,
        service: &Arc<MarketService>,
        platform: Platform,
    ) -> Result<(), MarketCacheError> {
        let markets = service
            .get_markets_by_platform(platform, None)
            .await
            .map_err(MarketCacheError::Api)?;

        let now = Utc::now();
        let count = markets.len();

        // Batch update memory cache
        {
            let mut write_cache = cache.write();
            for market in &markets {
                let cached = CachedMarket {
                    market: market.clone(),
                    updated_at: now,
                };
                write_cache.insert((platform, market.id.clone()), cached);
            }
        }

        // Batch update SQLite
        Self::store_markets_to_db(db, platform, &markets, now)?;

        info!("Refreshed {} {:?} markets", count, platform);
        Ok(())
    }

    /// Store a single market to SQLite
    fn store_market_to_db(
        db: &Arc<parking_lot::Mutex<Connection>>,
        platform: Platform,
        market: &PredictionMarket,
        updated_at: DateTime<Utc>,
    ) -> Result<(), MarketCacheError> {
        let conn = db.lock();

        let platform_str = match platform {
            Platform::Kalshi => "kalshi",
            Platform::Polymarket => "polymarket",
        };

        let data_json =
            serde_json::to_string(market).map_err(|e| MarketCacheError::Serialization(e.to_string()))?;

        conn.execute(
            r#"
            INSERT OR REPLACE INTO markets (platform, market_id, ticker, title, data, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                platform_str,
                market.id,
                market.ticker,
                market.title,
                data_json,
                updated_at.timestamp(),
            ],
        )
        .map_err(MarketCacheError::Database)?;

        Ok(())
    }

    /// Store multiple markets to SQLite
    fn store_markets_to_db(
        db: &Arc<parking_lot::Mutex<Connection>>,
        platform: Platform,
        markets: &[PredictionMarket],
        updated_at: DateTime<Utc>,
    ) -> Result<(), MarketCacheError> {
        let conn = db.lock();

        let platform_str = match platform {
            Platform::Kalshi => "kalshi",
            Platform::Polymarket => "polymarket",
        };

        let timestamp = updated_at.timestamp();

        for market in markets {
            let data_json = match serde_json::to_string(market) {
                Ok(json) => json,
                Err(e) => {
                    warn!("Failed to serialize market {}: {}", market.id, e);
                    continue;
                }
            };

            if let Err(e) = conn.execute(
                r#"
                INSERT OR REPLACE INTO markets (platform, market_id, ticker, title, data, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                "#,
                params![
                    platform_str,
                    market.id,
                    market.ticker,
                    market.title,
                    data_json,
                    timestamp,
                ],
            ) {
                warn!("Failed to store market {}: {}", market.id, e);
            }
        }

        Ok(())
    }

    // =========================================================================
    // Public API
    // =========================================================================

    /// Get all markets, optionally filtered by platform
    ///
    /// This returns instantly from the in-memory cache.
    /// Triggers background refresh if data is stale.
    pub fn get_markets(&self, platform: Option<Platform>) -> Vec<PredictionMarket> {
        let read_cache = self.cache.read();

        let markets: Vec<PredictionMarket> = read_cache
            .iter()
            .filter(|((p, _), _cached)| {
                // Filter by platform if specified
                platform.is_none() || platform == Some(*p)
            })
            .map(|(_, cached)| cached.market.clone())
            .collect();

        // Check if we need to refresh (any stale data)
        let needs_refresh = read_cache.values().any(|c| !c.is_fresh());
        drop(read_cache);

        if needs_refresh {
            let _ = self.refresh_tx.try_send(match platform {
                Some(p) => RefreshRequest::Platform(p),
                None => RefreshRequest::All,
            });
        }

        markets
    }

    /// Search markets by title
    ///
    /// Instant search over in-memory cache.
    pub fn search_markets(
        &self,
        query: &str,
        platform: Option<Platform>,
        limit: Option<usize>,
    ) -> Vec<PredictionMarket> {
        let query_lower = query.to_lowercase();
        let read_cache = self.cache.read();

        let mut results: Vec<PredictionMarket> = read_cache
            .iter()
            .filter(|((p, _), cached)| {
                // Filter by platform if specified
                let platform_match = platform.is_none() || platform == Some(*p);
                // Filter by title match
                let title_match = cached.market.title.to_lowercase().contains(&query_lower);
                platform_match && title_match
            })
            .map(|(_, cached)| cached.market.clone())
            .collect();

        // Sort by relevance (title starts with query > contains query)
        results.sort_by(|a, b| {
            let a_starts = a.title.to_lowercase().starts_with(&query_lower);
            let b_starts = b.title.to_lowercase().starts_with(&query_lower);
            match (a_starts, b_starts) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.title.cmp(&b.title),
            }
        });

        if let Some(l) = limit {
            results.truncate(l);
        }

        results
    }

    /// Get a single market by ID
    ///
    /// Returns from cache if available and fresh.
    /// Falls back to API if not cached or stale (with background cache update).
    pub async fn get_market(
        &self,
        platform: Platform,
        market_id: &str,
    ) -> Result<PredictionMarket, TerminalError> {
        // Check cache first
        {
            let read_cache = self.cache.read();
            if let Some(cached) = read_cache.get(&(platform, market_id.to_string())) {
                if cached.is_fresh() {
                    return Ok(cached.market.clone());
                }
            }
        }

        // Cache miss or stale - fetch from API
        let market = self.service.get_market(platform, market_id).await?;

        // Update cache in background
        let _ = self.refresh_tx.try_send(RefreshRequest::Single {
            platform,
            market_id: market_id.to_string(),
        });

        Ok(market)
    }

    /// Force refresh all markets (blocking)
    pub async fn refresh_all(&self) -> Result<(), MarketCacheError> {
        // KALSHI_DISABLED: Only refresh Polymarket while focusing on it
        for platform in [Platform::Polymarket] {
            Self::refresh_platform(&self.cache, &self.db, &self.service, platform).await?;
        }
        Ok(())
    }

    /// Force refresh a platform (blocking)
    pub async fn refresh_platform_now(&self, platform: Platform) -> Result<(), MarketCacheError> {
        Self::refresh_platform(&self.cache, &self.db, &self.service, platform).await
    }

    /// Queue a background refresh
    pub fn queue_refresh(&self, request: RefreshRequest) {
        let _ = self.refresh_tx.try_send(request);
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let read_cache = self.cache.read();

        let total = read_cache.len();
        let fresh = read_cache.values().filter(|c| c.is_fresh()).count();
        let stale = total - fresh;

        let kalshi_count = read_cache
            .keys()
            .filter(|(p, _)| *p == Platform::Kalshi)
            .count();
        let poly_count = read_cache
            .keys()
            .filter(|(p, _)| *p == Platform::Polymarket)
            .count();

        let oldest = read_cache
            .values()
            .map(|c| c.updated_at)
            .min();

        CacheStats {
            total,
            fresh,
            stale,
            kalshi_count,
            polymarket_count: poly_count,
            oldest_entry: oldest,
        }
    }

    /// Get underlying market service (for non-cached operations)
    pub fn service(&self) -> &MarketService {
        &self.service
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total: usize,
    pub fresh: usize,
    pub stale: usize,
    pub kalshi_count: usize,
    pub polymarket_count: usize,
    pub oldest_entry: Option<DateTime<Utc>>,
}

/// Errors from market cache operations
#[derive(Debug, thiserror::Error)]
pub enum MarketCacheError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("API error: {0}")]
    Api(#[from] TerminalError),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(String),
}

impl Clone for MarketCache {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
            db: Arc::clone(&self.db),
            service: Arc::clone(&self.service),
            refresh_tx: self.refresh_tx.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests would go here
}
