//! Discord API client wrapper

use twilight_http::Client as HttpClient;
use twilight_model::id::{Id, marker::{ChannelMarker, GuildMarker}};
use std::sync::Arc;

/// Discord client for fetching guild and channel information
pub struct DiscordClient {
    http: Arc<HttpClient>,
}

impl DiscordClient {
    /// Create a new Discord client
    pub fn new(token: String) -> Self {
        Self {
            http: Arc::new(HttpClient::new(token)),
        }
    }

    /// Get guild (server) name
    pub async fn get_guild_name(&self, guild_id: Id<GuildMarker>) -> Result<String, DiscordClientError> {
        let guild = self.http
            .guild(guild_id)
            .await
            .map_err(|e| DiscordClientError::HttpError(e.to_string()))?
            .model()
            .await
            .map_err(|e| DiscordClientError::DeserializationError(e.to_string()))?;

        Ok(guild.name)
    }

    /// Get channel name
    pub async fn get_channel_name(&self, channel_id: Id<ChannelMarker>) -> Result<String, DiscordClientError> {
        let channel = self.http
            .channel(channel_id)
            .await
            .map_err(|e| DiscordClientError::HttpError(e.to_string()))?
            .model()
            .await
            .map_err(|e| DiscordClientError::DeserializationError(e.to_string()))?;

        Ok(channel.name.unwrap_or_else(|| "unknown".to_string()))
    }

    /// Get guild icon URL
    pub async fn get_guild_icon_url(&self, guild_id: Id<GuildMarker>) -> Result<Option<String>, DiscordClientError> {
        let guild = self.http
            .guild(guild_id)
            .await
            .map_err(|e| DiscordClientError::HttpError(e.to_string()))?
            .model()
            .await
            .map_err(|e| DiscordClientError::DeserializationError(e.to_string()))?;

        Ok(guild.icon.map(|hash| {
            format!(
                "https://cdn.discordapp.com/icons/{}/{}.png",
                guild_id.get(),
                hash
            )
        }))
    }
}

/// Errors that can occur when using the Discord client
#[derive(Debug, thiserror::Error)]
pub enum DiscordClientError {
    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),
}
