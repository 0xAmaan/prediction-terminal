//! SQLite storage for embeddings using rusqlite

use std::path::Path;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use rusqlite::{params, Connection};
use tracing::{debug, info, instrument};

use crate::{
    error::{EmbeddingError, Result},
    types::{MarketEmbedding, NewsEmbedding},
};

/// SQLite store for embeddings
pub struct EmbeddingStore {
    conn: Arc<Mutex<Connection>>,
}

impl EmbeddingStore {
    /// Create a new embedding store
    ///
    /// # Arguments
    /// * `database_path` - Path to SQLite database file
    #[instrument(skip(database_path))]
    pub fn new<P: AsRef<Path> + std::fmt::Debug>(database_path: P) -> Result<Self> {
        info!("Opening embedding database: {:?}", database_path.as_ref());
        let conn = Connection::open(database_path.as_ref()).map_err(|e| {
            EmbeddingError::Database(format!("Failed to open database: {}", e))
        })?;

        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        store.init_tables()?;
        Ok(store)
    }

    /// Create an in-memory store (for testing)
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| EmbeddingError::Database(format!("Failed to create in-memory DB: {}", e)))?;

        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        store.init_tables()?;
        Ok(store)
    }

    /// Initialize database tables
    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Market embeddings table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS market_embeddings (
                market_id TEXT PRIMARY KEY,
                platform TEXT NOT NULL,
                embedding_text TEXT NOT NULL,
                embedding BLOB NOT NULL,
                dimension INTEGER NOT NULL,
                model TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| EmbeddingError::Database(e.to_string()))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_market_embeddings_platform
             ON market_embeddings(platform)",
            [],
        )
        .map_err(|e| EmbeddingError::Database(e.to_string()))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_market_embeddings_updated
             ON market_embeddings(updated_at)",
            [],
        )
        .map_err(|e| EmbeddingError::Database(e.to_string()))?;

        // News embeddings cache table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS news_embeddings (
                article_id TEXT PRIMARY KEY,
                embedding_text TEXT NOT NULL,
                embedding BLOB NOT NULL,
                created_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL
            )",
            [],
        )
        .map_err(|e| EmbeddingError::Database(e.to_string()))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_news_embeddings_expires
             ON news_embeddings(expires_at)",
            [],
        )
        .map_err(|e| EmbeddingError::Database(e.to_string()))?;

        info!("Embedding database tables initialized");
        Ok(())
    }

    /// Save or update a market embedding
    #[instrument(skip(self, embedding))]
    pub fn save_market_embedding(&self, embedding: &MarketEmbedding) -> Result<()> {
        let embedding_bytes = bincode::serialize(&embedding.embedding)?;
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO market_embeddings
             (market_id, platform, embedding_text, embedding, dimension, model, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(market_id) DO UPDATE SET
                embedding = excluded.embedding,
                embedding_text = excluded.embedding_text,
                platform = excluded.platform,
                dimension = excluded.dimension,
                model = excluded.model,
                updated_at = excluded.updated_at",
            params![
                &embedding.market_id,
                &embedding.platform,
                &embedding.embedding_text,
                &embedding_bytes,
                embedding.dimension as i64,
                &embedding.model,
                embedding.created_at.timestamp(),
                embedding.updated_at.timestamp(),
            ],
        )
        .map_err(|e| EmbeddingError::Database(e.to_string()))?;

        debug!("Saved embedding for market: {}", embedding.market_id);
        Ok(())
    }

    /// Get a specific market embedding
    #[instrument(skip(self))]
    pub fn get_market_embedding(&self, market_id: &str) -> Result<MarketEmbedding> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT market_id, platform, embedding_text, embedding, dimension, model, created_at, updated_at
                 FROM market_embeddings
                 WHERE market_id = ?",
            )
            .map_err(|e| EmbeddingError::Database(e.to_string()))?;

        let result = stmt
            .query_row(params![market_id], |row| {
                let embedding_bytes: Vec<u8> = row.get(3)?;
                let embedding: Vec<f32> = bincode::deserialize(&embedding_bytes)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                let created_at: i64 = row.get(6)?;
                let updated_at: i64 = row.get(7)?;

                Ok(MarketEmbedding {
                    market_id: row.get(0)?,
                    platform: row.get(1)?,
                    embedding_text: row.get(2)?,
                    embedding,
                    dimension: row.get::<_, i64>(4)? as usize,
                    model: row.get(5)?,
                    created_at: chrono::DateTime::from_timestamp(created_at, 0)
                        .unwrap_or_else(|| Utc::now()),
                    updated_at: chrono::DateTime::from_timestamp(updated_at, 0)
                        .unwrap_or_else(|| Utc::now()),
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    EmbeddingError::NotFound(market_id.to_string())
                }
                _ => EmbeddingError::Database(e.to_string()),
            })?;

        Ok(result)
    }

    /// Load all market embeddings
    ///
    /// Returns a vector of (market_id, platform, embedding) tuples
    /// Useful for batch similarity calculations
    #[instrument(skip(self))]
    pub fn load_all_market_embeddings(&self) -> Result<Vec<(String, String, Vec<f32>)>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT market_id, platform, embedding
                 FROM market_embeddings
                 ORDER BY updated_at DESC",
            )
            .map_err(|e| EmbeddingError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let market_id: String = row.get(0)?;
                let platform: String = row.get(1)?;
                let embedding_bytes: Vec<u8> = row.get(2)?;
                Ok((market_id, platform, embedding_bytes))
            })
            .map_err(|e| EmbeddingError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let (market_id, platform, embedding_bytes) =
                row.map_err(|e| EmbeddingError::Database(e.to_string()))?;
            let embedding: Vec<f32> = bincode::deserialize(&embedding_bytes)?;
            results.push((market_id, platform, embedding));
        }

        info!("Loaded {} market embeddings", results.len());
        Ok(results)
    }

    /// Load market embeddings for a specific platform
    #[instrument(skip(self))]
    pub fn load_market_embeddings_by_platform(
        &self,
        platform: &str,
    ) -> Result<Vec<(String, String, Vec<f32>)>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT market_id, platform, embedding
                 FROM market_embeddings
                 WHERE platform = ?
                 ORDER BY updated_at DESC",
            )
            .map_err(|e| EmbeddingError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![platform], |row| {
                let market_id: String = row.get(0)?;
                let platform: String = row.get(1)?;
                let embedding_bytes: Vec<u8> = row.get(2)?;
                Ok((market_id, platform, embedding_bytes))
            })
            .map_err(|e| EmbeddingError::Database(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let (market_id, platform, embedding_bytes) =
                row.map_err(|e| EmbeddingError::Database(e.to_string()))?;
            let embedding: Vec<f32> = bincode::deserialize(&embedding_bytes)?;
            results.push((market_id, platform, embedding));
        }

        debug!("Loaded {} embeddings for platform {}", results.len(), platform);
        Ok(results)
    }

    /// Delete a market embedding
    pub fn delete_market_embedding(&self, market_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "DELETE FROM market_embeddings WHERE market_id = ?",
            params![market_id],
        )
        .map_err(|e| EmbeddingError::Database(e.to_string()))?;

        debug!("Deleted embedding for market: {}", market_id);
        Ok(())
    }

    /// Save a news embedding to cache
    #[instrument(skip(self, embedding))]
    pub fn save_news_embedding(&self, embedding: &NewsEmbedding) -> Result<()> {
        let embedding_bytes = bincode::serialize(&embedding.embedding)?;
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO news_embeddings
             (article_id, embedding_text, embedding, created_at, expires_at)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(article_id) DO UPDATE SET
                embedding = excluded.embedding,
                embedding_text = excluded.embedding_text,
                created_at = excluded.created_at,
                expires_at = excluded.expires_at",
            params![
                &embedding.article_id,
                &embedding.embedding_text,
                &embedding_bytes,
                embedding.created_at.timestamp(),
                embedding.expires_at.timestamp(),
            ],
        )
        .map_err(|e| EmbeddingError::Database(e.to_string()))?;

        debug!("Cached news embedding for article: {}", embedding.article_id);
        Ok(())
    }

    /// Get a cached news embedding
    #[instrument(skip(self))]
    pub fn get_news_embedding(&self, article_id: &str) -> Result<Option<NewsEmbedding>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT article_id, embedding_text, embedding, created_at, expires_at
                 FROM news_embeddings
                 WHERE article_id = ?",
            )
            .map_err(|e| EmbeddingError::Database(e.to_string()))?;

        match stmt.query_row(params![article_id], |row| {
            let embedding_bytes: Vec<u8> = row.get(2)?;
            let embedding: Vec<f32> = bincode::deserialize(&embedding_bytes)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

            let created_at: i64 = row.get(3)?;
            let expires_at: i64 = row.get(4)?;

            Ok(NewsEmbedding {
                article_id: row.get(0)?,
                embedding_text: row.get(1)?,
                embedding,
                created_at: chrono::DateTime::from_timestamp(created_at, 0)
                    .unwrap_or_else(|| Utc::now()),
                expires_at: chrono::DateTime::from_timestamp(expires_at, 0)
                    .unwrap_or_else(|| Utc::now()),
            })
        }) {
            Ok(news_emb) => {
                // Check if expired
                if news_emb.is_expired() {
                    debug!("News embedding expired, returning None");
                    Ok(None)
                } else {
                    Ok(Some(news_emb))
                }
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(EmbeddingError::Database(e.to_string())),
        }
    }

    /// Clean up expired news embeddings
    #[instrument(skip(self))]
    pub fn cleanup_expired_news(&self) -> Result<usize> {
        let now = Utc::now().timestamp();
        let conn = self.conn.lock().unwrap();

        let deleted = conn
            .execute("DELETE FROM news_embeddings WHERE expires_at < ?", params![now])
            .map_err(|e| EmbeddingError::Database(e.to_string()))?;

        if deleted > 0 {
            info!("Cleaned up {} expired news embeddings", deleted);
        }

        Ok(deleted)
    }

    /// Get statistics about stored embeddings
    pub fn get_stats(&self) -> Result<EmbeddingStats> {
        let conn = self.conn.lock().unwrap();

        let market_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM market_embeddings", [], |row| row.get(0))
            .unwrap_or(0);

        let news_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM news_embeddings", [], |row| row.get(0))
            .unwrap_or(0);

        // Get database size (page_count * page_size)
        let page_count: i64 = conn
            .query_row("PRAGMA page_count", [], |row| row.get(0))
            .unwrap_or(0);
        let page_size: i64 = conn
            .query_row("PRAGMA page_size", [], |row| row.get(0))
            .unwrap_or(4096);
        let total_size = page_count * page_size;

        Ok(EmbeddingStats {
            market_count: market_count as usize,
            news_count: news_count as usize,
            database_size_bytes: total_size as usize,
        })
    }
}

/// Statistics about embedding storage
#[derive(Debug, Clone)]
pub struct EmbeddingStats {
    pub market_count: usize,
    pub news_count: usize,
    pub database_size_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MarketEmbedding;

    fn create_test_store() -> EmbeddingStore {
        EmbeddingStore::new_in_memory().expect("Failed to create test store")
    }

    #[test]
    fn test_save_and_load_market_embedding() {
        let store = create_test_store();

        let embedding = MarketEmbedding::new(
            "test_market".to_string(),
            "polymarket".to_string(),
            "Will Bitcoin hit $100k?".to_string(),
            vec![0.1; 1536],
        );

        store.save_market_embedding(&embedding).expect("Failed to save");

        let loaded = store
            .get_market_embedding("test_market")
            .expect("Failed to load");

        assert_eq!(loaded.market_id, "test_market");
        assert_eq!(loaded.platform, "polymarket");
        assert_eq!(loaded.embedding.len(), 1536);
    }

    #[test]
    fn test_load_all_market_embeddings() {
        let store = create_test_store();

        for i in 0..5 {
            let embedding = MarketEmbedding::new(
                format!("market_{}", i),
                "polymarket".to_string(),
                format!("Market {}", i),
                vec![0.1; 1536],
            );
            store.save_market_embedding(&embedding).unwrap();
        }

        let all = store.load_all_market_embeddings().unwrap();
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_news_embedding_cache() {
        let store = create_test_store();

        let news_emb = NewsEmbedding::new(
            "article_1".to_string(),
            "Test article".to_string(),
            vec![0.5; 1536],
        );

        store.save_news_embedding(&news_emb).unwrap();

        let loaded = store.get_news_embedding("article_1").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().article_id, "article_1");
    }
}
