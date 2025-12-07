//! Error types for the terminal

use thiserror::Error;

/// Terminal-wide error type
#[derive(Error, Debug)]
pub enum TerminalError {
    #[error("API error: {0}")]
    Api(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Platform error ({platform}): {message}")]
    Platform { platform: String, message: String },

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl TerminalError {
    pub fn api(msg: impl Into<String>) -> Self {
        TerminalError::Api(msg.into())
    }

    pub fn network(msg: impl Into<String>) -> Self {
        TerminalError::Network(msg.into())
    }

    pub fn auth(msg: impl Into<String>) -> Self {
        TerminalError::Auth(msg.into())
    }

    pub fn parse(msg: impl Into<String>) -> Self {
        TerminalError::Parse(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        TerminalError::NotFound(msg.into())
    }

    pub fn platform(platform: impl Into<String>, message: impl Into<String>) -> Self {
        TerminalError::Platform {
            platform: platform.into(),
            message: message.into(),
        }
    }

    pub fn config(msg: impl Into<String>) -> Self {
        TerminalError::Config(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        TerminalError::Internal(msg.into())
    }
}

/// Result type alias for terminal operations
pub type TerminalResult<T> = Result<T, TerminalError>;
