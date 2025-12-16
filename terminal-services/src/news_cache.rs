//! News Cache
//!
//! Persistent SQLite cache for news feed items with background refresh.
//! Returns instant responses from database while refreshing asynchronously.

use std::path::Path;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use tokio::sync::RwLock;
use tracing::{debug, info};

use terminal_core::{NewsFeed, NewsItem};

/// How long to keep news items in cache - show articles from last 24 hours
const NEWS_TTL_SECS: i64 = 24 * 60 * 60;

/// Cache TTL for global news feed - force refresh after this (2 minutes)
const GLOBAL_FEED_TTL_SECS: i64 = 2 * 60;

#[derive(Debug, thiserror::Error)]
pub enum NewsCacheError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// SQLite-backed news cache
pub struct NewsCache {
    db_path: String,
    /// In-memory cache of last fetch time for global feed
    last_global_fetch: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl NewsCache {
    /// Create a new news cache
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self, NewsCacheError> {
        let db_path = db_path.as_ref().to_str().unwrap().to_string();

        // Ensure parent directory exists
        if let Some(parent) = Path::new(&db_path).parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let cache = Self {
            db_path: db_path.clone(),
            last_global_fetch: Arc::new(RwLock::new(None)),
        };

        // Initialize database schema
        cache.init_db()?;

        info!("Initialized news cache at: {}", db_path);
        Ok(cache)
    }

    /// Initialize database schema
    fn init_db(&self) -> Result<(), NewsCacheError> {
        let conn = Connection::open(&self.db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS news_items (
                id TEXT PRIMARY KEY,
                feed_type TEXT NOT NULL,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                source TEXT,
                published_at INTEGER NOT NULL,
                data JSON NOT NULL,
                fetched_at INTEGER NOT NULL,
                updated_at INTEGER DEFAULT (strftime('%s', 'now'))
            )",
            [],
        )?;

        // Index for fast lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_news_feed_type ON news_items(feed_type, fetched_at DESC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_news_published ON news_items(published_at DESC)",
            [],
        )?;

        Ok(())
    }

    /// Get a database connection
    fn get_connection(&self) -> Result<Connection, NewsCacheError> {
        Ok(Connection::open(&self.db_path)?)
    }

    /// Store news items in the cache
    pub fn store_news_items(
        &self,
        feed_type: &str,
        items: &[NewsItem],
    ) -> Result<usize, NewsCacheError> {
        let conn = self.get_connection()?;
        let now = Utc::now().timestamp();
        let mut stored = 0;

        for item in items {
            let data = serde_json::to_string(item)?;

            conn.execute(
                "INSERT OR REPLACE INTO news_items
                 (id, feed_type, title, url, source, published_at, data, fetched_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    item.id,
                    feed_type,
                    item.title,
                    item.url,
                    item.source.name,
                    item.published_at.timestamp(),
                    data,
                    now,
                ],
            )?;
            stored += 1;
        }

        debug!("Stored {} news items for feed type: {}", stored, feed_type);
        Ok(stored)
    }

    /// Get cached news items for a feed type
    pub fn get_news_items(
        &self,
        feed_type: &str,
        limit: usize,
    ) -> Result<Vec<NewsItem>, NewsCacheError> {
        let conn = self.get_connection()?;

        // Get items from last fetch that aren't too old
        let cutoff = Utc::now().timestamp() - NEWS_TTL_SECS;

        let mut stmt = conn.prepare(
            "SELECT data FROM news_items
             WHERE feed_type = ?1 AND published_at > ?2
             ORDER BY published_at DESC
             LIMIT ?3",
        )?;

        let items: Result<Vec<NewsItem>, _> = stmt
            .query_map(params![feed_type, cutoff, limit as i64], |row| {
                let data: String = row.get(0)?;
                Ok(data)
            })?
            .map(|result| {
                result
                    .map_err(NewsCacheError::from)
                    .and_then(|data| serde_json::from_str(&data).map_err(NewsCacheError::from))
            })
            .collect();

        items
    }

    /// Check if global feed needs refresh
    pub async fn needs_refresh(&self) -> bool {
        let last_fetch = self.last_global_fetch.read().await;

        match *last_fetch {
            None => true, // Never fetched
            Some(last) => {
                let age = Utc::now().signed_duration_since(last);
                age.num_seconds() > GLOBAL_FEED_TTL_SECS
            }
        }
    }

    /// Mark global feed as refreshed
    pub async fn mark_refreshed(&self) {
        let mut last_fetch = self.last_global_fetch.write().await;
        *last_fetch = Some(Utc::now());
    }

    /// Get cached global news feed (instant response)
    pub fn get_cached_global_news(&self, limit: usize) -> Result<NewsFeed, NewsCacheError> {
        let items = self.get_news_items("global", limit)?;

        Ok(NewsFeed {
            items,
            total_count: 0, // Not tracked in cache
            next_cursor: None,
        })
    }

    /// Cleanup old news items
    pub fn cleanup_old_items(&self) -> Result<usize, NewsCacheError> {
        let conn = self.get_connection()?;
        let cutoff = Utc::now().timestamp() - NEWS_TTL_SECS;

        let deleted = conn.execute(
            "DELETE FROM news_items WHERE published_at < ?1",
            params![cutoff],
        )?;

        if deleted > 0 {
            info!("Cleaned up {} old news items", deleted);
        }

        Ok(deleted)
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<NewsCacheStats, NewsCacheError> {
        let conn = self.get_connection()?;

        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM news_items",
            [],
            |row| row.get(0),
        )?;

        let by_feed: Vec<(String, i64)> = {
            let mut stmt = conn.prepare(
                "SELECT feed_type, COUNT(*) FROM news_items GROUP BY feed_type",
            )?;

            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?;

            rows.collect::<Result<Vec<_>, _>>()?
        };

        Ok(NewsCacheStats { total, by_feed })
    }
}

#[derive(Debug)]
pub struct NewsCacheStats {
    pub total: i64,
    pub by_feed: Vec<(String, i64)>,
}
