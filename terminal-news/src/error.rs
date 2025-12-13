//! Error types for the news module

use thiserror::Error;

/// Errors that can occur in the news module
#[derive(Debug, Error)]
pub enum NewsError {
    /// HTTP request failed
    #[error("Request failed: {0}")]
    RequestFailed(String),

    /// API returned an error response
    #[error("API error (status {status}): {message}")]
    ApiError {
        /// HTTP status code
        status: u16,
        /// Error message from API
        message: String,
    },

    /// Failed to parse API response
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Article scraping failed
    #[error("Scrape failed: {0}")]
    ScrapeFailed(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimited,

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}
