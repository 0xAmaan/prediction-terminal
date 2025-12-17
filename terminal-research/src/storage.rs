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
use aws_sdk_s3::{operation::get_object::GetObjectError, primitives::ByteStream, Client};
use chrono::DateTime;
use futures::stream::{self, StreamExt};
use terminal_core::{Platform, TerminalError};
use tracing::{info, instrument, warn};

use crate::types::{ChatHistory, ChatMessage, EdgeIndex, MarketEdgeEntry, ResearchVersion};
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

                // Check if cache is still valid (using adaptive TTL)
                let age = chrono::Utc::now() - job.updated_at;
                let ttl_hours = job.cache_ttl_hours;
                if age.num_hours() > ttl_hours {
                    info!(
                        "Cache expired for key: {} (age: {} hours, TTL: {} hours)",
                        key,
                        age.num_hours(),
                        ttl_hours
                    );
                    return Ok(None);
                }

                info!("Cache hit for key: {}", key);
                Ok(Some(job))
            }
            Err(e) => {
                // Check for NoSuchKey using proper SDK error matching
                if let Some(GetObjectError::NoSuchKey(_)) = e.as_service_error() {
                    info!("Cache miss for key: {}", key);
                    return Ok(None);
                }

                // Fallback: also check error string for backwards compatibility
                let error_str = e.to_string();
                if error_str.contains("NoSuchKey") || error_str.contains("NotFound") || error_str.contains("404") {
                    info!("Cache miss for key: {}", key);
                    Ok(None)
                } else {
                    // Treat all other S3 errors as cache misses rather than fatal errors
                    // This includes: access denied, bucket not found, network errors, etc.
                    // The research service will simply proceed without cached data
                    warn!("S3 error for key {} (treating as cache miss): {}", key, error_str);
                    Ok(None)
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
                // Check for NoSuchKey using proper SDK error matching
                if let Some(GetObjectError::NoSuchKey(_)) = e.as_service_error() {
                    info!("Version not found: {}", full_key);
                    return Ok(None);
                }

                // Fallback: also check error string for backwards compatibility
                let error_str = e.to_string();
                if error_str.contains("NoSuchKey") || error_str.contains("NotFound") || error_str.contains("404") {
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

    /// List all saved research reports from S3
    ///
    /// Returns all completed research jobs stored in S3.
    /// This is used to populate the reports page with persisted research.
    #[instrument(skip(self))]
    pub async fn list_all_reports(&self) -> Result<Vec<ResearchJob>, TerminalError> {
        let prefix = "research/";

        // List all objects with the research prefix
        let mut continuation_token: Option<String> = None;
        let mut all_keys: Vec<String> = Vec::new();

        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(prefix);

            if let Some(token) = continuation_token {
                request = request.continuation_token(token);
            }

            let result = request
                .send()
                .await
                .map_err(|e| TerminalError::internal(format!("S3 list error: {}", e)))?;

            // Collect keys for current.json files only
            for obj in result.contents() {
                if let Some(key) = obj.key() {
                    if key.ends_with("/current.json") {
                        all_keys.push(key.to_string());
                    }
                }
            }

            // Check if there are more results
            if result.is_truncated() == Some(true) {
                continuation_token = result.next_continuation_token().map(|s| s.to_string());
            } else {
                break;
            }
        }

        info!("Found {} research reports in S3", all_keys.len());

        // Fetch reports in parallel with limited concurrency (10 concurrent requests)
        let client = self.client.clone();
        let bucket = self.bucket.clone();

        let mut reports: Vec<ResearchJob> = stream::iter(all_keys)
            .map(|key| {
                let client = client.clone();
                let bucket = bucket.clone();
                async move {
                    let result = client
                        .get_object()
                        .bucket(&bucket)
                        .key(&key)
                        .send()
                        .await;

                    match result {
                        Ok(output) => {
                            let bytes = match output.body.collect().await {
                                Ok(b) => b.into_bytes(),
                                Err(e) => {
                                    warn!("Failed to read S3 body for {}: {}", key, e);
                                    return None;
                                }
                            };

                            match serde_json::from_slice::<ResearchJob>(&bytes) {
                                Ok(job) => {
                                    // Check if cache is still valid
                                    let age = chrono::Utc::now() - job.updated_at;
                                    if age.num_hours() > job.cache_ttl_hours {
                                        info!("Cache expired for key: {}", key);
                                        return None;
                                    }
                                    // Only include completed reports
                                    if job.status == crate::types::ResearchStatus::Completed {
                                        Some(job)
                                    } else {
                                        None
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to parse report {}: {}", key, e);
                                    None
                                }
                            }
                        }
                        Err(e) => {
                            // Check for NoSuchKey
                            if let Some(GetObjectError::NoSuchKey(_)) = e.as_service_error() {
                                return None;
                            }
                            let error_str = e.to_string();
                            if !error_str.contains("NoSuchKey")
                                && !error_str.contains("NotFound")
                                && !error_str.contains("404")
                            {
                                warn!("S3 error for {}: {}", key, error_str);
                            }
                            None
                        }
                    }
                }
            })
            .buffer_unordered(10) // 10 concurrent S3 requests
            .filter_map(|opt| async { opt })
            .collect()
            .await;

        // Sort by updated_at (newest first)
        reports.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        info!("Returning {} completed research reports", reports.len());
        Ok(reports)
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
                // Check for NoSuchKey using proper SDK error matching
                if let Some(GetObjectError::NoSuchKey(_)) = e.as_service_error() {
                    info!("No chat history for {}/{}, returning empty", platform, market_id);
                    return Ok(ChatHistory::new());
                }

                // Fallback: also check error string for backwards compatibility
                let error_str = e.to_string();
                if error_str.contains("NoSuchKey") || error_str.contains("NotFound") || error_str.contains("404") {
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

    // ========================================================================
    // Edge Index Methods (for filtering mispriced markets)
    // ========================================================================

    const EDGE_INDEX_KEY: &'static str = "research/edge-index.json";

    /// Get the edge index containing all markets with research edge data
    #[instrument(skip(self))]
    pub async fn get_edge_index(&self) -> Result<EdgeIndex, TerminalError> {
        let result = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(Self::EDGE_INDEX_KEY)
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

                let index: EdgeIndex = serde_json::from_slice(&bytes).map_err(|e| {
                    TerminalError::parse(format!("Failed to parse edge index: {}", e))
                })?;

                info!("Loaded edge index with {} entries", index.entries.len());
                Ok(index)
            }
            Err(e) => {
                // Check for NoSuchKey
                if let Some(GetObjectError::NoSuchKey(_)) = e.as_service_error() {
                    info!("No edge index found, returning empty");
                    return Ok(EdgeIndex::new());
                }

                let error_str = e.to_string();
                if error_str.contains("NoSuchKey") || error_str.contains("NotFound") || error_str.contains("404") {
                    info!("No edge index found, returning empty");
                    Ok(EdgeIndex::new())
                } else {
                    warn!("S3 error for edge index: {}", error_str);
                    // Return empty index on error rather than failing
                    Ok(EdgeIndex::new())
                }
            }
        }
    }

    /// Save the edge index
    #[instrument(skip(self, index))]
    pub async fn save_edge_index(&self, index: &EdgeIndex) -> Result<(), TerminalError> {
        let body = serde_json::to_vec(index)
            .map_err(|e| TerminalError::internal(format!("Failed to serialize edge index: {}", e)))?;

        self.put_object(Self::EDGE_INDEX_KEY, body).await?;
        info!("Saved edge index with {} entries", index.entries.len());
        Ok(())
    }

    /// Update a single entry in the edge index
    ///
    /// This is a convenience method that:
    /// 1. Gets the existing index
    /// 2. Updates or inserts the entry
    /// 3. Saves the updated index
    #[instrument(skip(self, entry))]
    pub async fn update_edge_entry(&self, entry: MarketEdgeEntry) -> Result<(), TerminalError> {
        let mut index = self.get_edge_index().await?;
        index.upsert(entry);
        self.save_edge_index(&index).await?;
        Ok(())
    }
}
