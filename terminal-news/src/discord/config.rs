//! Discord integration configuration

use serde::{Deserialize, Serialize};
use std::env;

/// Configuration for Discord integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    /// Discord bot token for authentication
    pub bot_token: String,
    /// List of Discord servers to monitor
    pub servers: Vec<ServerConfig>,
    /// How often to check engagement metrics (in seconds)
    #[serde(default = "default_update_interval")]
    pub update_interval_secs: u64,
}

/// Configuration for a single Discord server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Discord server/guild ID
    pub server_id: u64,
    /// Human-readable server name
    pub server_name: String,
    /// List of channel IDs to monitor
    pub channel_ids: Vec<u64>,
    /// Engagement threshold for this server
    pub engagement_threshold: EngagementThreshold,
    /// How many hours of message history to backfill on startup
    #[serde(default = "default_backfill_hours")]
    pub backfill_hours: u64,
}

/// Engagement thresholds for filtering messages
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EngagementThreshold {
    /// Minimum total reaction count across all emoji
    pub reactions: u32,
    /// Minimum number of replies in thread
    pub replies: u32,
}

impl DiscordConfig {
    /// Load Discord configuration from environment variables
    ///
    /// Expects:
    /// - DISCORD_BOT_TOKEN: Discord bot token
    /// - DISCORD_SERVERS: JSON array of server configurations
    pub fn from_env() -> Result<Option<Self>, ConfigError> {
        let bot_token = match env::var("DISCORD_BOT_TOKEN") {
            Ok(token) => token,
            Err(_) => return Ok(None), // Not configured, return None
        };

        let servers_json = match env::var("DISCORD_SERVERS") {
            Ok(json) => json,
            Err(_) => return Ok(None), // Not configured, return None
        };

        let servers: Vec<ServerConfig> = serde_json::from_str(&servers_json)
            .map_err(|e| ConfigError::InvalidJson {
                field: "DISCORD_SERVERS".to_string(),
                error: e.to_string(),
            })?;

        if servers.is_empty() {
            return Err(ConfigError::EmptyServerList);
        }

        Ok(Some(Self {
            bot_token,
            servers,
            update_interval_secs: default_update_interval(),
        }))
    }

    /// Get all monitored channel IDs across all servers
    pub fn all_channel_ids(&self) -> Vec<u64> {
        self.servers
            .iter()
            .flat_map(|s| s.channel_ids.iter().copied())
            .collect()
    }

    /// Find server config by guild ID
    pub fn server_by_id(&self, guild_id: u64) -> Option<&ServerConfig> {
        self.servers.iter().find(|s| s.server_id == guild_id)
    }
}

fn default_update_interval() -> u64 {
    60 // 60 seconds
}

fn default_backfill_hours() -> u64 {
    24 // 24 hours
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid JSON in {field}: {error}")]
    InvalidJson { field: String, error: String },

    #[error("DISCORD_SERVERS cannot be empty")]
    EmptyServerList,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_server_config() {
        let json = r#"
        {
            "server_id": 1234567890,
            "server_name": "Test Server",
            "channel_ids": [111111, 222222],
            "engagement_threshold": {
                "reactions": 5,
                "replies": 3
            },
            "backfill_hours": 12
        }
        "#;

        let config: ServerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.server_id, 1234567890);
        assert_eq!(config.server_name, "Test Server");
        assert_eq!(config.channel_ids.len(), 2);
        assert_eq!(config.engagement_threshold.reactions, 5);
        assert_eq!(config.engagement_threshold.replies, 3);
        assert_eq!(config.backfill_hours, 12);
    }

    #[test]
    fn test_parse_multiple_servers() {
        let json = r#"
        [
            {
                "server_id": 111,
                "server_name": "Server 1",
                "channel_ids": [1, 2],
                "engagement_threshold": {"reactions": 5, "replies": 3}
            },
            {
                "server_id": 222,
                "server_name": "Server 2",
                "channel_ids": [3, 4, 5],
                "engagement_threshold": {"reactions": 10, "replies": 5}
            }
        ]
        "#;

        let servers: Vec<ServerConfig> = serde_json::from_str(json).unwrap();
        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].server_name, "Server 1");
        assert_eq!(servers[1].server_name, "Server 2");
        assert_eq!(servers[1].channel_ids.len(), 3);
    }

    #[test]
    fn test_all_channel_ids() {
        let config = DiscordConfig {
            bot_token: "test_token".to_string(),
            servers: vec![
                ServerConfig {
                    server_id: 111,
                    server_name: "S1".to_string(),
                    channel_ids: vec![1, 2],
                    engagement_threshold: EngagementThreshold {
                        reactions: 5,
                        replies: 3,
                    },
                    backfill_hours: 24,
                },
                ServerConfig {
                    server_id: 222,
                    server_name: "S2".to_string(),
                    channel_ids: vec![3, 4, 5],
                    engagement_threshold: EngagementThreshold {
                        reactions: 10,
                        replies: 5,
                    },
                    backfill_hours: 24,
                },
            ],
            update_interval_secs: 60,
        };

        let all_channels = config.all_channel_ids();
        assert_eq!(all_channels.len(), 5);
        assert!(all_channels.contains(&1));
        assert!(all_channels.contains(&5));
    }
}
