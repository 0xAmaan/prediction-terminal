//! Core types for embeddings

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Embedding vector (1536 dimensions for text-embedding-3-small)
pub type EmbeddingVector = Vec<f32>;

/// Market embedding with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketEmbedding {
    /// Unique market identifier
    pub market_id: String,
    /// Platform (kalshi, polymarket)
    pub platform: String,
    /// Text used to generate embedding
    pub embedding_text: String,
    /// The embedding vector (1536 dimensions)
    pub embedding: EmbeddingVector,
    /// Embedding dimension (should be 1536)
    pub dimension: usize,
    /// Model used (text-embedding-3-small)
    pub model: String,
    /// When this embedding was created
    pub created_at: DateTime<Utc>,
    /// When this embedding was last updated
    pub updated_at: DateTime<Utc>,
}

/// News article embedding (cached)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsEmbedding {
    /// Article identifier (SHA256 hash of URL)
    pub article_id: String,
    /// The embedding vector
    pub embedding: EmbeddingVector,
    /// Text used to generate embedding
    pub embedding_text: String,
    /// When this embedding was created
    pub created_at: DateTime<Utc>,
    /// When this embedding expires (24h TTL)
    pub expires_at: DateTime<Utc>,
}

/// Similarity match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityMatch {
    /// Market ID
    pub market_id: String,
    /// Platform
    pub platform: String,
    /// Cosine similarity score (0.0 - 1.0)
    pub score: f64,
}

impl MarketEmbedding {
    /// Create a new market embedding
    pub fn new(
        market_id: String,
        platform: String,
        embedding_text: String,
        embedding: EmbeddingVector,
    ) -> Self {
        let now = Utc::now();
        Self {
            market_id,
            platform,
            embedding_text,
            dimension: embedding.len(),
            embedding,
            model: "text-embedding-3-small".to_string(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl NewsEmbedding {
    /// Create a new news embedding with 24h TTL
    pub fn new(article_id: String, embedding_text: String, embedding: EmbeddingVector) -> Self {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::hours(24);
        Self {
            article_id,
            embedding,
            embedding_text,
            created_at: now,
            expires_at,
        }
    }

    /// Check if this embedding has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}
