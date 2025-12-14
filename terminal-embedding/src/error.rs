//! Error types for embedding operations

use thiserror::Error;

pub type Result<T> = std::result::Result<T, EmbeddingError>;

#[derive(Debug, Error)]
pub enum EmbeddingError {
    #[error("OpenAI API error: {0}")]
    OpenAI(#[from] async_openai::error::OpenAIError),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),

    #[error("Invalid embedding dimension: expected {expected}, got {actual}")]
    InvalidDimension { expected: usize, actual: usize },

    #[error("Embedding not found: {0}")]
    NotFound(String),

    #[error("Configuration error: {0}")]
    Config(String),
}
