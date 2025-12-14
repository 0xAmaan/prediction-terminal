//! Embedding and Semantic Matching for Prediction Markets
//!
//! This crate provides semantic understanding for news-to-market matching using
//! vector embeddings from OpenAI's text-embedding-3-small model.
//!
//! ## Features
//! - Generate embeddings for markets and news articles
//! - Calculate cosine similarity between embeddings
//! - Store and retrieve embeddings from SQLite
//! - Find semantically similar markets for news articles

pub mod client;
pub mod error;
pub mod similarity;
pub mod store;
pub mod types;

pub use client::EmbeddingClient;
pub use error::{EmbeddingError, Result};
pub use similarity::{cosine_similarity, find_similar_markets};
pub use store::EmbeddingStore;
pub use types::{EmbeddingVector, MarketEmbedding, NewsEmbedding, SimilarityMatch};
