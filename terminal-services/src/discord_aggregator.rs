//! Discord Aggregator Service
//!
//! Background service that connects to Discord Gateway, tracks message engagement,
//! and broadcasts high-quality messages as news items.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use terminal_news::discord::{
    calculate_relevance_score, discord_message_to_news_item, DiscordClient, DiscordConfig,
    EngagementTracker,
};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{
    Event, Intents, Shard, ShardId,
};
use twilight_model::channel::Message;
use twilight_model::gateway::payload::incoming::{MessageCreate, ReactionAdd, ReactionRemove};
use twilight_model::id::Id;

use crate::websocket::WebSocketState;

/// Discord aggregator configuration
pub struct DiscordAggregator {
    config: DiscordConfig,
    ws_state: Arc<WebSocketState>,
    engagement_tracker: Arc<EngagementTracker>,
    discord_client: Arc<DiscordClient>,
    cache: Arc<InMemoryCache>,
    /// Cache of channel_id -> channel_name
    channel_names: Arc<RwLock<HashMap<u64, String>>>,
    /// Cache of guild_id -> guild_name
    guild_names: Arc<RwLock<HashMap<u64, String>>>,
    /// Cache of guild_id -> guild_icon_url
    guild_icons: Arc<RwLock<HashMap<u64, Option<String>>>>,
}

impl DiscordAggregator {
    /// Create a new Discord aggregator
    pub fn new(config: DiscordConfig, ws_state: Arc<WebSocketState>) -> Self {
        let discord_client = Arc::new(DiscordClient::new(config.bot_token.clone()));

        // Create cache for Discord entities (guilds, channels, etc.)
        let cache = Arc::new(
            InMemoryCache::builder()
                .resource_types(ResourceType::CHANNEL | ResourceType::GUILD)
                .build(),
        );

        Self {
            config,
            ws_state,
            engagement_tracker: Arc::new(EngagementTracker::new()),
            discord_client,
            cache,
            channel_names: Arc::new(RwLock::new(HashMap::new())),
            guild_names: Arc::new(RwLock::new(HashMap::new())),
            guild_icons: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start the Discord aggregator
    ///
    /// This runs indefinitely, maintaining a connection to Discord Gateway
    /// and processing events.
    pub async fn start(self: Arc<Self>) {
        info!("Starting Discord aggregator");

        // Spawn periodic engagement check task
        let self_cleanup = Arc::clone(&self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(
                self_cleanup.config.update_interval_secs,
            ));

            loop {
                interval.tick().await;
                // Clean up old engagement metrics (older than 7 days)
                self_cleanup.engagement_tracker.cleanup_old_metrics(24 * 7);
            }
        });

        // Main Gateway connection loop with automatic reconnection
        loop {
            match self.run_gateway_loop().await {
                Ok(_) => {
                    warn!("Discord Gateway closed normally, reconnecting in 5s...");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
                Err(e) => {
                    error!("Discord Gateway error: {}, reconnecting in 10s...", e);
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            }
        }
    }

    /// Run the Discord Gateway event loop
    async fn run_gateway_loop(&self) -> Result<(), DiscordAggregatorError> {
        info!("Connecting to Discord Gateway...");

        // Configure Gateway intents
        let intents = Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT | Intents::GUILD_MESSAGE_REACTIONS;

        // Create shard (for small bots, we only need 1 shard)
        let mut shard = Shard::new(ShardId::ONE, self.config.bot_token.clone(), intents);

        info!("Discord Gateway shard created, starting event loop...");

        // Event loop
        loop {
            let event = match shard.next_event().await {
                Ok(event) => event,
                Err(source) => {
                    error!("Error receiving event: {:?}", source);
                    return Err(DiscordAggregatorError::GatewayError(source.to_string()));
                }
            };

            // Update cache
            self.cache.update(&event);

            // Process event
            match event {
                Event::Ready(ready) => {
                    info!(
                        "Discord Gateway connected as {} (session: {})",
                        ready.user.name, ready.session_id
                    );

                    // Pre-cache guild and channel information
                    self.precache_guild_info().await;
                }
                Event::MessageCreate(msg) => {
                    self.handle_message_create(msg).await;
                }
                Event::ReactionAdd(reaction) => {
                    self.handle_reaction_add(reaction).await;
                }
                Event::ReactionRemove(reaction) => {
                    self.handle_reaction_remove(reaction).await;
                }
                Event::GatewayClose(_) => {
                    warn!("Discord Gateway closed by server");
                    return Ok(());
                }
                _ => {
                    // Ignore other events
                }
            }
        }
    }

    /// Pre-cache guild and channel information on startup
    async fn precache_guild_info(&self) {
        info!("Pre-caching guild and channel information...");

        for server_config in &self.config.servers {
            let guild_id = Id::new(server_config.server_id);

            // Cache guild name
            match self.discord_client.get_guild_name(guild_id).await {
                Ok(name) => {
                    self.guild_names.write().await.insert(server_config.server_id, name.clone());
                    debug!("Cached guild name: {} = {}", server_config.server_id, name);
                }
                Err(e) => {
                    warn!("Failed to fetch guild name for {}: {}", server_config.server_id, e);
                }
            }

            // Cache guild icon
            match self.discord_client.get_guild_icon_url(guild_id).await {
                Ok(icon_url) => {
                    self.guild_icons.write().await.insert(server_config.server_id, icon_url);
                }
                Err(e) => {
                    warn!("Failed to fetch guild icon for {}: {}", server_config.server_id, e);
                }
            }

            // Cache channel names
            for &channel_id in &server_config.channel_ids {
                match self.discord_client.get_channel_name(Id::new(channel_id)).await {
                    Ok(name) => {
                        self.channel_names.write().await.insert(channel_id, name.clone());
                        debug!("Cached channel name: {} = {}", channel_id, name);
                    }
                    Err(e) => {
                        warn!("Failed to fetch channel name for {}: {}", channel_id, e);
                    }
                }
            }
        }

        info!("Guild and channel information cached");
    }

    /// Handle a new message being created
    async fn handle_message_create(&self, msg: Box<MessageCreate>) {
        let message = &msg.0;

        // Stage 1: Fast channel filter
        let channel_id = message.channel_id.get();
        if !self.is_monitored_channel(channel_id) {
            return; // Not a monitored channel, skip
        }

        debug!(
            "Message received in monitored channel: guild={:?}, channel={}, author={}, content='{}'",
            message.guild_id.map(|id| id.get()),
            channel_id,
            message.author.name,
            &message.content[..message.content.len().min(50)]
        );

        // Stage 2: Process message
        self.process_message(message).await;
    }

    /// Handle a reaction being added to a message
    async fn handle_reaction_add(&self, reaction: Box<ReactionAdd>) {
        let message_id = reaction.message_id.get();
        let user_id = reaction.user_id.get();

        // Update engagement tracker
        self.engagement_tracker
            .on_reaction_add(message_id, user_id);

        debug!(
            "Reaction added: message={}, user={}, emoji={:?}",
            message_id,
            user_id,
            reaction.emoji
        );

        // Re-evaluate message if we have it cached
        // For now, we'll let the periodic update handle re-scoring
    }

    /// Handle a reaction being removed from a message
    async fn handle_reaction_remove(&self, reaction: Box<ReactionRemove>) {
        let message_id = reaction.message_id.get();
        let user_id = reaction.user_id.get();

        // Update engagement tracker
        self.engagement_tracker
            .on_reaction_remove(message_id, user_id);

        debug!(
            "Reaction removed: message={}, user={}",
            message_id, user_id
        );
    }

    /// Process a message and potentially broadcast it as a news item
    async fn process_message(&self, message: &Message) {
        // Skip bot messages
        if message.author.bot {
            return;
        }

        // Skip empty messages
        if message.content.is_empty() {
            return;
        }

        // Get engagement metrics
        let engagement = self
            .engagement_tracker
            .get_or_create_metrics(message.id.get());

        // Find server config
        let guild_id = match message.guild_id {
            Some(id) => id.get(),
            None => {
                warn!("Message has no guild_id, skipping");
                return;
            }
        };

        let server_config = match self.config.server_by_id(guild_id) {
            Some(config) => config,
            None => {
                warn!("No config found for guild {}", guild_id);
                return;
            }
        };

        // Calculate keyword match score (simplified - would integrate with NewsService)
        // For now, use 0.0 as placeholder
        let keyword_match_score = 0.0;

        // Calculate relevance score
        let relevance_score = calculate_relevance_score(
            &engagement,
            &server_config.engagement_threshold,
            keyword_match_score,
        );

        debug!(
            "Message relevance: score={:.2}, reactions={}, replies={}, threshold=(r:{}, rep:{})",
            relevance_score,
            engagement.reaction_count,
            engagement.reply_count,
            server_config.engagement_threshold.reactions,
            server_config.engagement_threshold.replies
        );

        // Check if relevance is high enough
        const MIN_RELEVANCE: f64 = 0.35;
        if relevance_score < MIN_RELEVANCE {
            debug!("Message relevance too low ({:.2} < {:.2}), skipping", relevance_score, MIN_RELEVANCE);
            return;
        }

        // Get guild and channel names
        let guild_name = self
            .guild_names
            .read()
            .await
            .get(&guild_id)
            .cloned()
            .unwrap_or_else(|| server_config.server_name.clone());

        let channel_name = self
            .channel_names
            .read()
            .await
            .get(&message.channel_id.get())
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let guild_icon_url = self
            .guild_icons
            .read()
            .await
            .get(&guild_id)
            .cloned()
            .flatten();

        // Convert to NewsItem
        let news_item = discord_message_to_news_item(
            message,
            &guild_name,
            &channel_name,
            guild_icon_url,
            &engagement,
            relevance_score,
        );

        info!(
            "Publishing Discord NewsItem: title='{}', relevance={:.2}, engagement=({} reactions, {} replies)",
            news_item.title,
            relevance_score,
            engagement.reaction_count,
            engagement.reply_count
        );

        // Check for dry-run mode
        if std::env::var("DISCORD_DRY_RUN").is_ok() {
            info!("DRY RUN: Would publish NewsItem: {:?}", news_item);
            return;
        }

        // Broadcast to WebSocket clients
        self.ws_state.broadcast_global_news(news_item);
    }

    /// Check if a channel is being monitored
    fn is_monitored_channel(&self, channel_id: u64) -> bool {
        self.config
            .servers
            .iter()
            .any(|server| server.channel_ids.contains(&channel_id))
    }
}

/// Errors that can occur in the Discord aggregator
#[derive(Debug, thiserror::Error)]
pub enum DiscordAggregatorError {
    #[error("Gateway error: {0}")]
    GatewayError(String),

    #[error("Discord API error: {0}")]
    ApiError(String),
}
