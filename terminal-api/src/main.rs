//! Prediction Market Terminal API Server
//!
//! HTTP API server that aggregates data from Kalshi and Polymarket.

mod routes;

use axum::{
    http::{header, Method},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use terminal_kalshi::KalshiClient;
use terminal_polymarket::PolymarketClient;
use terminal_services::{
    AggregatorConfig, CandleService, DiscordAggregator, MarketCache, MarketDataAggregator,
    MarketService, MarketStatsService, NewsCache, ResearchService, TradeCollector, TradeCollectorConfig,
    TradeStorage, WebSocketState,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub market_cache: Arc<MarketCache>,
    pub market_service: Arc<MarketService>,
    pub ws_state: Arc<WebSocketState>,
    pub trade_storage: Arc<TradeStorage>,
    pub candle_service: Arc<CandleService>,
    pub trade_collector: Arc<TradeCollector>,
    pub aggregator: Arc<MarketDataAggregator>,
    pub market_stats_service: Arc<MarketStatsService>,
    pub news_service: Option<Arc<terminal_services::NewsService>>,
    pub news_cache: Arc<NewsCache>,
    /// Research service (optional - requires EXA_API_KEY and OPENAI_API_KEY)
    pub research_service: Option<Arc<ResearchService>>,
    /// Trading state (optional - requires TRADING_PRIVATE_KEY)
    pub trading_state: Option<routes::SharedTradingState>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env.local file
    if let Err(e) = dotenvy::from_filename(".env.local") {
        // Not an error if the file doesn't exist
        if !matches!(e, dotenvy::Error::Io(_)) {
            eprintln!("Warning: Failed to load .env.local: {}", e);
        }
    }

    // Initialize logging
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,terminal_api=debug")),
        )
        .init();

    info!("Starting Prediction Market Terminal API");

    // Log if Polymarket credentials are available
    if std::env::var("POLY_API_KEY").is_ok() {
        info!("Polymarket API credentials found in environment");
    } else {
        info!("No Polymarket API credentials found - trades endpoint will be unavailable");
    }

    // Initialize clients
    let kalshi_client = KalshiClient::new(false); // Use production API
    let polymarket_client = PolymarketClient::new();

    // Initialize market service
    let market_service = MarketService::new(kalshi_client, polymarket_client);
    let market_service_arc = Arc::new(market_service.clone());

    // Initialize market cache (in-memory + SQLite for instant lookups)
    let cache_db_path = std::env::var("CACHE_DB_PATH").unwrap_or_else(|_| "data/cache.db".to_string());
    info!("Initializing market cache at: {}", cache_db_path);
    let market_cache = match MarketCache::new(&cache_db_path, market_service.clone()).await {
        Ok(cache) => Arc::new(cache),
        Err(e) => {
            tracing::error!("Failed to initialize market cache at '{}': {}", cache_db_path, e);
            tracing::error!("Please ensure:");
            tracing::error!("  1. The 'data' directory exists and is writable");
            tracing::error!("  2. You have sufficient disk space");
            tracing::error!("  3. SQLite is properly installed");
            return Err(e.into());
        }
    };

    // Refresh markets in background on startup
    let cache_for_refresh = Arc::clone(&market_cache);
    tokio::spawn(async move {
        info!("Starting initial market cache refresh...");
        if let Err(e) = cache_for_refresh.refresh_all().await {
            tracing::error!("Failed to refresh market cache on startup: {}", e);
        } else {
            let stats = cache_for_refresh.stats();
            info!(
                "Market cache refreshed: {} total ({} Kalshi, {} Polymarket)",
                stats.total, stats.kalshi_count, stats.polymarket_count
            );
        }
    });

    // Create subscription event channel for aggregator integration
    let (subscription_tx, subscription_rx) = WebSocketState::create_subscription_event_channel();

    // Create WebSocket state with subscription event sender
    let mut ws_state = WebSocketState::new(market_service.clone());
    ws_state.set_subscription_event_sender(subscription_tx);
    let ws_state = Arc::new(ws_state);

    // Initialize trade storage (SQLite database)
    let db_path = std::env::var("TRADES_DB_PATH").unwrap_or_else(|_| "data/trades.db".to_string());
    info!("Initializing trade storage at: {}", db_path);
    let trade_storage = match TradeStorage::new(&db_path) {
        Ok(storage) => Arc::new(storage),
        Err(e) => {
            tracing::error!("Failed to initialize trade storage at '{}': {}", db_path, e);
            tracing::error!("Please ensure:");
            tracing::error!("  1. The 'data' directory exists and is writable");
            tracing::error!("  2. You have sufficient disk space");
            tracing::error!("  3. SQLite is properly installed");
            return Err(e.into());
        }
    };

    // Initialize candle service
    let candle_service = Arc::new(CandleService::new(trade_storage.clone()));

    // Initialize market stats service
    let market_stats_service = Arc::new(MarketStatsService::new(trade_storage.clone()));

    // Initialize trade collector
    // KALSHI_DISABLED: Disable Kalshi trade collection while focusing on Polymarket
    let trade_collector_config = TradeCollectorConfig {
        collect_kalshi: false,
        collect_polymarket: true,
        ..TradeCollectorConfig::default()
    };
    let trade_collector = Arc::new(TradeCollector::new(
        market_service_arc.clone(),
        trade_storage.clone(),
        Some(ws_state.clone()),
        trade_collector_config,
    ));

    // Start trade collector in background
    let collector_handle = trade_collector.clone();
    tokio::spawn(async move {
        collector_handle.start().await;
    });

    // Auto-track top markets for trade collection (after cache populates)
    // This ensures we have transaction count data for the most popular markets
    let auto_track_collector = trade_collector.clone();
    let auto_track_cache = market_cache.clone();
    tokio::spawn(async move {
        // Wait for cache to populate (check every 2 seconds, up to 30 seconds)
        for _ in 0..15 {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let markets = auto_track_cache.get_markets(Some(terminal_core::Platform::Polymarket));
            if !markets.is_empty() {
                // Track top 50 markets by volume for trade collection
                let top_n = 50;
                info!("Auto-tracking top {} Polymarket markets for trade collection", top_n);
                for market in markets.into_iter().take(top_n) {
                    auto_track_collector
                        .track_market(terminal_core::Platform::Polymarket, market.id)
                        .await;
                }
                break;
            }
        }
    });

    // Initialize and start market data aggregator
    // KALSHI_DISABLED: Disable Kalshi WebSocket while focusing on Polymarket
    let aggregator_config = AggregatorConfig {
        kalshi_enabled: false,
        polymarket_enabled: true,
    };
    let mut aggregator = MarketDataAggregator::new(
        aggregator_config,
        ws_state.clone(),
        market_service.clone(),
    );

    // Set trade storage for orderbook snapshot persistence
    aggregator.set_trade_storage(trade_storage.clone());

    // Start aggregator (connects to exchange WebSockets)
    if let Err(e) = aggregator.start().await {
        tracing::error!("Failed to start market data aggregator: {}", e);
    } else {
        info!("Market data aggregator started successfully");
    }

    // Spawn a task to process subscription events from frontend clients
    let aggregator = Arc::new(aggregator);
    let aggregator_for_events = Arc::clone(&aggregator);
    tokio::spawn(async move {
        aggregator_for_events.process_subscription_events(subscription_rx).await;
    });

    // Initialize news service
    // EXA_API_KEY and FIRECRAWL_API_KEY are optional - RSS feeds work without them
    let exa_api_key = std::env::var("EXA_API_KEY").ok();
    let firecrawl_api_key = std::env::var("FIRECRAWL_API_KEY").ok();
    let news_config = terminal_services::news_service::NewsServiceConfig::default();
    let mut news_service_instance = terminal_services::NewsService::new(exa_api_key.clone(), firecrawl_api_key, news_config);

    // Set market service for news service
    news_service_instance.set_market_service(market_service_arc.clone());

    let news_service = Some(Arc::new(news_service_instance));
    info!("News service initialized (RSS feeds + Google News)");

    // Initialize news cache for persistent storage
    let news_db_path = std::env::var("NEWS_DB_PATH").unwrap_or_else(|_| "data/news.db".to_string());
    info!("Initializing news cache at: {}", news_db_path);
    let news_cache = match NewsCache::new(&news_db_path) {
        Ok(cache) => Arc::new(cache),
        Err(e) => {
            tracing::error!("Failed to initialize news cache at '{}': {}", news_db_path, e);
            tracing::error!("Please ensure the 'data' directory exists and is writable");
            return Err(e.into());
        }
    };

    // Auto-generate market embeddings on startup if needed
    if let Some(news_svc) = &news_service {
        let news_svc_for_embeddings = Arc::clone(news_svc);
        tokio::spawn(async move {
            // Small delay to let server start
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            info!("Checking if market embeddings need to be generated...");
            match news_svc_for_embeddings.generate_market_embeddings().await {
                Ok(count) => {
                    info!("✅ Successfully generated {} market embeddings", count);
                }
                Err(e) => {
                    // This is expected if OPENAI_API_KEY is not set
                    info!("Market embeddings not generated: {}", e);
                    info!("Set OPENAI_API_KEY to enable semantic news matching");
                }
            }
        });

        // Background task to refresh news cache periodically
        let news_svc_for_cache = Arc::clone(news_svc);
        let news_cache_for_refresh = Arc::clone(&news_cache);
        tokio::spawn(async move {
            use terminal_core::NewsSearchParams;

            // Initial delay before first refresh (much shorter - just let server start)
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            loop {
                // Check if refresh is needed
                if !news_cache_for_refresh.needs_refresh().await {
                    // Wait 1 minute before checking again
                    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                    continue;
                }

                info!("Refreshing news cache...");
                let search_params = NewsSearchParams {
                    query: None,
                    limit: 100,
                    time_range: Some("24h".to_string()),
                    market_id: None,
                    skip_embeddings: false, // Generate embeddings for related markets
                };
                match news_svc_for_cache.search_global_news(&search_params).await {
                    Ok(feed) => {
                        // Store in cache
                        if let Err(e) = news_cache_for_refresh.store_news_items("global", &feed.items) {
                            tracing::error!("Failed to store news in cache: {}", e);
                        } else {
                            news_cache_for_refresh.mark_refreshed().await;
                            info!("✅ News cache refreshed with {} items (with embeddings)", feed.items.len());

                            // Cleanup old items
                            if let Err(e) = news_cache_for_refresh.cleanup_old_items() {
                                tracing::warn!("Failed to cleanup old news items: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to refresh news cache: {}", e);
                    }
                }

                // Wait 5 minutes before next refresh
                tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
            }
        });
    }

    // Initialize Discord integration (optional)
    // Requires DISCORD_BOT_TOKEN and DISCORD_SERVERS environment variables
    match terminal_news::discord::DiscordConfig::from_env() {
        Ok(Some(discord_config)) => {
            info!(
                "Discord integration enabled with {} server(s)",
                discord_config.servers.len()
            );
            let discord_aggregator = Arc::new(DiscordAggregator::new(
                discord_config,
                ws_state.clone(),
            ));
            tokio::spawn(async move {
                discord_aggregator.start().await;
            });
        }
        Ok(None) => {
            info!("Discord integration not configured (DISCORD_BOT_TOKEN not set)");
        }
        Err(e) => {
            tracing::warn!("Failed to load Discord configuration: {}. Discord integration disabled.", e);
        }
    }

    // Initialize research service (optional - may fail if API keys not set)
    let research_service = match ResearchService::new(market_service_arc.clone()).await {
        Ok(service) => {
            info!("Research service initialized successfully");
            let service = Arc::new(service);

            // Spawn task to forward research updates to WebSocket
            let ws_state_for_research = ws_state.clone();
            let mut research_rx = service.subscribe();
            tokio::spawn(async move {
                while let Ok(update) = research_rx.recv().await {
                    // Serialize the update to JSON Value
                    if let Ok(json_value) = serde_json::to_value(&update) {
                        ws_state_for_research.broadcast_research_update(json_value);
                    }
                }
            });

            Some(service)
        }
        Err(e) => {
            info!(
                "Research service not available: {}. Set EXA_API_KEY and OPENAI_API_KEY to enable.",
                e
            );
            None
        }
    };

    // Initialize trading state (optional - requires TRADING_PRIVATE_KEY)
    let trading_state = if std::env::var("TRADING_PRIVATE_KEY").is_ok() {
        info!("Trading private key found - trading endpoints will be available");
        Some(routes::create_trading_state())
    } else {
        info!("No TRADING_PRIVATE_KEY found - trading endpoints disabled");
        None
    };

    // Create app state
    let state = AppState {
        market_cache,
        market_service: market_service_arc,
        ws_state,
        trade_storage,
        candle_service,
        trade_collector,
        aggregator,
        market_stats_service,
        news_service,
        news_cache,
        research_service,
        trading_state,
    };

    // Configure CORS for frontend
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    // Build router
    let app = Router::new()
        .nest("/api", routes::api_routes())
        .merge(routes::ws_routes())
        .layer(cors)
        .with_state(state);

    // Start server
    let port = std::env::var("SERVER_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
