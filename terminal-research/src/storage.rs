//! S3 storage for caching research results
//!
//! This module provides persistent caching of research reports in AWS S3
//! to avoid duplicate API calls and provide faster responses for previously
//! researched markets.
//!
//! ## Storage Structure
//!
//! Research is stored with versioning support:
//! - `research/{platform}/{market_id}/current.json` - always the latest version
//! - `research/{platform}/{market_id}/v{timestamp}.json` - historical versions
//! - `research/{platform}/{market_id}/chat.json` - chat history (Phase 3)

use aws_config::BehaviorVersion;
use aws_sdk_s3::{primitives::ByteStream, Client};
use chrono::DateTime;
use terminal_core::{Platform, TerminalError};
use tracing::{info, instrument, warn};

use crate::types::{ChatHistory, ChatMessage, ResearchVersion};
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
    ///
    /// This method looks for the new versioned format (`{key}/current.json`) first,
    /// then falls back to the old format (`{key}`) for backwards compatibility.
    #[instrument(skip(self))]
    pub async fn get_cached(&self, key: &str) -> Result<Option<ResearchJob>, TerminalError> {
        // Try new versioned format first: research/{platform}/{market_id}/current.json
        let versioned_key = format!("{}/current.json", key);
        if let Some(job) = self.get_object(&versioned_key).await? {
            return Ok(Some(job));
        }

        // Fall back to old format: research/{platform}/{market_id}
        // (for backwards compatibility with existing cache)
        if let Some(job) = self.get_object(key).await? {
            info!("Found research in old format, will migrate on next save");
            return Ok(Some(job));
        }

        Ok(None)
    }

    /// Get a single object from S3 and parse as ResearchJob
    async fn get_object(&self, key: &str) -> Result<Option<ResearchJob>, TerminalError> {
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
    ///
    /// This saves to both `current.json` and a timestamped version file.
    #[instrument(skip(self, job))]
    pub async fn save(&self, job: &ResearchJob) -> Result<(), TerminalError> {
        self.save_version(job).await
    }

    /// Save a research job as a new version
    ///
    /// Saves to:
    /// - `{cache_key}/current.json` - the latest version
    /// - `{cache_key}/v{timestamp}.json` - historical version
    #[instrument(skip(self, job))]
    pub async fn save_version(&self, job: &ResearchJob) -> Result<(), TerminalError> {
        let base_key = job.cache_key();
        let body = serde_json::to_vec(job)
            .map_err(|e| TerminalError::internal(format!("Failed to serialize job: {}", e)))?;

        // Save to current.json
        let current_key = format!("{}/current.json", base_key);
        self.put_object(&current_key, body.clone()).await?;
        info!("Saved research to cache: {}", current_key);

        // Save to timestamped version
        let timestamp = job.updated_at.timestamp_millis();
        let version_key = format!("{}/v{}.json", base_key, timestamp);
        self.put_object(&version_key, body).await?;
        info!("Saved research version: {}", version_key);

        Ok(())
    }

    /// Put a single object to S3
    async fn put_object(&self, key: &str, body: Vec<u8>) -> Result<(), TerminalError> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(body))
            .content_type("application/json")
            .send()
            .await
            .map_err(|e| TerminalError::internal(format!("S3 put error: {}", e)))?;
        Ok(())
    }

    /// List all versions for a platform and market ID
    ///
    /// Returns versions sorted by creation time (newest first).
    #[instrument(skip(self))]
    pub async fn list_versions(
        &self,
        platform: Platform,
        market_id: &str,
    ) -> Result<Vec<ResearchVersion>, TerminalError> {
        let prefix = format!(
            "research/{:?}/{}/v",
            platform, market_id
        )
        .to_lowercase();

        let result = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(&prefix)
            .send()
            .await
            .map_err(|e| TerminalError::internal(format!("S3 list error: {}", e)))?;

        let mut versions: Vec<ResearchVersion> = result
            .contents()
            .iter()
            .filter_map(|obj| {
                let key = obj.key()?;
                // Extract timestamp from key like "research/kalshi/abc/v1702389600000.json"
                let filename = key.rsplit('/').next()?;
                if !filename.starts_with('v') || !filename.ends_with(".json") {
                    return None;
                }
                let timestamp_str = &filename[1..filename.len() - 5]; // Remove "v" and ".json"
                let timestamp_millis: i64 = timestamp_str.parse().ok()?;
                let created_at = DateTime::from_timestamp_millis(timestamp_millis)?;

                Some(ResearchVersion {
                    key: filename.to_string(),
                    created_at,
                    version_number: 0, // Will be set after sorting
                })
            })
            .collect();

        // Sort by creation time, newest first
        versions.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Add version numbers (1 = newest)
        for (i, version) in versions.iter_mut().enumerate() {
            version.version_number = (i + 1) as u32;
        }

        info!(
            "Found {} versions for {}/{}",
            versions.len(),
            platform,
            market_id
        );
        Ok(versions)
    }

    /// Get a specific version by its key
    ///
    /// The version_key should be the filename like "v1702389600000.json"
    #[instrument(skip(self))]
    pub async fn get_version(
        &self,
        platform: Platform,
        market_id: &str,
        version_key: &str,
    ) -> Result<Option<ResearchJob>, TerminalError> {
        let full_key = format!(
            "research/{:?}/{}/{}",
            platform, market_id, version_key
        )
        .to_lowercase();

        // Use get_object but skip the 24-hour expiry check for historical versions
        let result = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&full_key)
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

                info!("Retrieved version {} for {}/{}", version_key, platform, market_id);
                Ok(Some(job))
            }
            Err(e) => {
                let error_str = e.to_string();
                if error_str.contains("NoSuchKey") || error_str.contains("NotFound") {
                    info!("Version not found: {}", full_key);
                    Ok(None)
                } else {
                    warn!("S3 error for version {}: {}", full_key, error_str);
                    Err(TerminalError::internal(format!("S3 error: {}", e)))
                }
            }
        }
    }

    /// Get the cache key for a platform and market ID
    pub fn cache_key(platform: Platform, market_id: &str) -> String {
        format!("research/{:?}/{}", platform, market_id).to_lowercase()
    }

    // ========================================================================
    // Chat Storage Methods
    // ========================================================================

    /// Get chat history for a platform and market ID
    ///
    /// Returns empty ChatHistory if no chat exists yet.
    #[instrument(skip(self))]
    pub async fn get_chat(
        &self,
        platform: Platform,
        market_id: &str,
    ) -> Result<ChatHistory, TerminalError> {
        let key = format!(
            "research/{:?}/{}/chat.json",
            platform, market_id
        )
        .to_lowercase();

        let result = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(&key)
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

                let history: ChatHistory = serde_json::from_slice(&bytes).map_err(|e| {
                    TerminalError::parse(format!("Failed to parse chat history: {}", e))
                })?;

                info!("Loaded {} chat messages for {}/{}", history.messages.len(), platform, market_id);
                Ok(history)
            }
            Err(e) => {
                let error_str = e.to_string();
                // NoSuchKey means no chat exists yet, return empty
                if error_str.contains("NoSuchKey") || error_str.contains("NotFound") {
                    info!("No chat history for {}/{}, returning empty", platform, market_id);
                    Ok(ChatHistory::new())
                } else {
                    warn!("S3 error for chat {}: {}", key, error_str);
                    Err(TerminalError::internal(format!("S3 error: {}", e)))
                }
            }
        }
    }

    /// Save chat history for a platform and market ID
    #[instrument(skip(self, history))]
    pub async fn save_chat(
        &self,
        platform: Platform,
        market_id: &str,
        history: &ChatHistory,
    ) -> Result<(), TerminalError> {
        let key = format!(
            "research/{:?}/{}/chat.json",
            platform, market_id
        )
        .to_lowercase();

        let body = serde_json::to_vec(history)
            .map_err(|e| TerminalError::internal(format!("Failed to serialize chat: {}", e)))?;

        self.put_object(&key, body).await?;
        info!("Saved {} chat messages to {}", history.messages.len(), key);
        Ok(())
    }

    /// Append a message to the chat history
    ///
    /// This is a convenience method that:
    /// 1. Gets the existing chat history
    /// 2. Appends the new message
    /// 3. Saves the updated history
    #[instrument(skip(self, message))]
    pub async fn append_message(
        &self,
        platform: Platform,
        market_id: &str,
        message: ChatMessage,
    ) -> Result<ChatHistory, TerminalError> {
        let mut history = self.get_chat(platform, market_id).await?;
        history.append(message);
        self.save_chat(platform, market_id, &history).await?;
        Ok(history)
    }
}
