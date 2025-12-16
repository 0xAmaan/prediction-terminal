//! Convert Discord messages to NewsItem format

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use twilight_model::channel::Message;
use twilight_model::id::{Id, marker::GuildMarker};

use terminal_core::{NewsItem, NewsSource};

use super::engagement::EngagementMetrics;

/// Convert a Discord message to a NewsItem
///
/// # Arguments
/// * `message` - The Discord message
/// * `guild_name` - Name of the Discord server/guild
/// * `channel_name` - Name of the channel
/// * `guild_icon_url` - Optional URL to the guild's icon
/// * `_engagement` - Engagement metrics for the message (unused, for future use)
/// * `relevance_score` - Pre-calculated relevance score
pub fn discord_message_to_news_item(
    message: &Message,
    guild_name: &str,
    channel_name: &str,
    guild_icon_url: Option<String>,
    _engagement: &EngagementMetrics,
    relevance_score: f64,
) -> NewsItem {
    let message_url = build_message_url(message);

    NewsItem {
        // Stable ID: Hash of Discord message URL
        id: generate_stable_id(&message_url),

        // Title: Truncated message content
        title: truncate_title(&message.content, 100),

        // URL: Direct link to Discord message
        url: message_url,

        // Published date: Discord message timestamp
        published_at: discord_timestamp_to_datetime(message),

        // Source: Discord server + channel
        source: NewsSource {
            name: format!("{} • #{}", guild_name, channel_name),
            url: build_guild_url(message.guild_id),
            favicon_url: guild_icon_url,
        },

        // Summary: Full message content (Discord messages are typically short)
        summary: message.content.clone(),

        // Content: None (Discord messages don't have separate long-form content)
        content: None,

        // Image: First image attachment or embed thumbnail
        image_url: extract_image_url(message),

        // Relevance: Pre-calculated score from engagement + keyword matching
        relevance_score,

        // Related markets: Empty initially (can be populated by NewsService)
        related_market_ids: vec![],

        // Search query: Discord server name for context
        search_query: Some(guild_name.to_string()),

        // AI-enriched fields (set later by NewsAnalyzer)
        matched_market: None,
        price_signal: None,
        suggested_action: None,
        signal_reasoning: None,
    }
}

/// Build a Discord message URL
fn build_message_url(message: &Message) -> String {
    format!(
        "https://discord.com/channels/{}/{}/{}",
        message.guild_id.map(|id| id.get()).unwrap_or(0),
        message.channel_id.get(),
        message.id.get()
    )
}

/// Build a Discord guild/server URL
fn build_guild_url(guild_id: Option<Id<GuildMarker>>) -> String {
    match guild_id {
        Some(id) => format!("https://discord.com/channels/{}", id.get()),
        None => "https://discord.com".to_string(),
    }
}

/// Generate a stable ID by hashing the message URL
fn generate_stable_id(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let hash = hasher.finalize();
    format!("discord_{}", hex::encode(&hash[..8])) // Use first 8 bytes for shorter ID
}

/// Truncate a title to a maximum length
fn truncate_title(content: &str, max_len: usize) -> String {
    if content.is_empty() {
        return "[Discord Message]".to_string();
    }

    if content.len() <= max_len {
        content.to_string()
    } else {
        // Try to truncate at a word boundary
        let truncated = &content[..max_len];
        match truncated.rfind(' ') {
            Some(pos) if pos > max_len / 2 => {
                format!("{}...", &truncated[..pos])
            }
            _ => format!("{}...", truncated),
        }
    }
}

/// Extract image URL from message attachments or embeds
fn extract_image_url(message: &Message) -> Option<String> {
    // First, check attachments for images
    for attachment in &message.attachments {
        if let Some(ref content_type) = attachment.content_type {
            if content_type.starts_with("image/") {
                return Some(attachment.url.clone());
            }
        }
    }

    // Then check embeds for thumbnails or images
    for embed in &message.embeds {
        if let Some(ref thumbnail) = embed.thumbnail {
            return Some(thumbnail.url.clone());
        }
        if let Some(ref image) = embed.image {
            return Some(image.url.clone());
        }
    }

    None
}

/// Convert Discord timestamp to chrono DateTime
fn discord_timestamp_to_datetime(message: &Message) -> DateTime<Utc> {
    DateTime::from_timestamp(message.timestamp.as_secs() as i64, 0)
        .unwrap_or_else(Utc::now)
}

#[cfg(test)]
mod tests {
    use super::*;
    use twilight_model::util::Timestamp;

    #[test]
    fn test_truncate_title_short() {
        let title = truncate_title("Short message", 100);
        assert_eq!(title, "Short message");
    }

    #[test]
    fn test_truncate_title_long() {
        let long_text = "This is a very long message that exceeds the maximum length and should be truncated at a reasonable point";
        let title = truncate_title(long_text, 50);
        assert!(title.len() <= 53); // 50 + "..."
        assert!(title.ends_with("..."));
    }

    #[test]
    fn test_truncate_title_empty() {
        let title = truncate_title("", 100);
        assert_eq!(title, "[Discord Message]");
    }

    #[test]
    fn test_generate_stable_id() {
        let url1 = "https://discord.com/channels/123/456/789";
        let url2 = "https://discord.com/channels/123/456/789";
        let url3 = "https://discord.com/channels/123/456/790";

        let id1 = generate_stable_id(url1);
        let id2 = generate_stable_id(url2);
        let id3 = generate_stable_id(url3);

        // Same URL should produce same ID
        assert_eq!(id1, id2);

        // Different URL should produce different ID
        assert_ne!(id1, id3);

        // Should start with "discord_"
        assert!(id1.starts_with("discord_"));
    }

    #[test]
    fn test_build_message_url() {
        use twilight_model::id::Id;

        let mut message = Message {
            id: Id::new(789),
            channel_id: Id::new(456),
            guild_id: Some(Id::new(123)),
            ..Default::default()
        };

        let url = build_message_url(&message);
        assert_eq!(url, "https://discord.com/channels/123/456/789");
    }
}
