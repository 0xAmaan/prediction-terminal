//! S3 storage for caching research results
//!
//! This module provides persistent caching of research reports in AWS S3
//! to avoid duplicate API calls and provide faster responses for previously
//! researched markets.

use aws_config::BehaviorVersion;
use aws_sdk_s3::{primitives::ByteStream, Client};
use terminal_core::TerminalError;
use tracing::{info, instrument, warn};

use crate::ResearchJob;

/// S3-based storage for research results
pub struct ResearchStorage {
    client: Client,
    bucket: String,
}

impl ResearchStorage {
    /// Create a new research storage instance
    ///
    /// Requires AWS credentials to be configured via environment variables:
    /// - AWS_ACCESS_KEY_ID
    /// - AWS_SECRET_ACCESS_KEY
    /// - AWS_REGION
    /// - RESEARCH_S3_BUCKET (optional, defaults to "prediction-terminal-research")
    pub async fn new() -> Result<Self, TerminalError> {
        let config = aws_config::defaults(BehaviorVersion::latest())
            .load()
            .await;
        let client = Client::new(&config);

        let bucket = std::env::var("RESEARCH_S3_BUCKET")
            .unwrap_or_else(|_| "prediction-terminal-research".to_string());

        info!("Research storage initialized with bucket: {}", bucket);

        Ok(Self { client, bucket })
    }

    /// Get a cached research job by its cache key
    ///
    /// Returns None if:
    /// - The cache entry doesn't exist
    /// - The cache entry has expired (older than 24 hours)
    /// - There was an error reading/parsing the cache
    #[instrument(skip(self))]
    pub async fn get_cached(&self, key: &str) -> Result<Option<ResearchJob>, TerminalError> {
        let result = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await;

        match result {
            Ok(output) => {
                let bytes = output
                    .body
                    .collect()
                    .await
                    .map_err(|e| TerminalError::internal(format!("Failed to read S3 body: {}", e)))?
                    .into_bytes();

                let job: ResearchJob = serde_json::from_slice(&bytes).map_err(|e| {
                    TerminalError::parse(format!("Failed to parse cached research: {}", e))
                })?;

                // Check if cache is still valid (24 hours)
                let age = chrono::Utc::now() - job.updated_at;
                if age.num_hours() > 24 {
                    info!("Cache expired for key: {} (age: {} hours)", key, age.num_hours());
                    return Ok(None);
                }

                info!("Cache hit for key: {}", key);
                Ok(Some(job))
            }
            Err(e) => {
                let error_str = e.to_string();
                // NoSuchKey means not cached, not an error
                if error_str.contains("NoSuchKey") || error_str.contains("NotFound") {
                    info!("Cache miss for key: {}", key);
                    Ok(None)
                } else {
                    warn!("S3 error for key {}: {}", key, error_str);
                    Err(TerminalError::internal(format!("S3 error: {}", e)))
                }
            }
        }
    }

    /// Save a research job to the cache
    #[instrument(skip(self, job))]
    pub async fn save(&self, job: &ResearchJob) -> Result<(), TerminalError> {
        let key = job.cache_key();
        let body = serde_json::to_vec(job)
            .map_err(|e| TerminalError::internal(format!("Failed to serialize job: {}", e)))?;

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(ByteStream::from(body))
            .content_type("application/json")
            .send()
            .await
            .map_err(|e| TerminalError::internal(format!("S3 put error: {}", e)))?;

        info!("Saved research to cache: {}", key);
        Ok(())
    }

    /// Get the cache key for a platform and market ID
    pub fn cache_key(platform: terminal_core::Platform, market_id: &str) -> String {
        format!("research/{:?}/{}", platform, market_id).to_lowercase()
    }
}
