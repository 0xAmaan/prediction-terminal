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
    MarketService, MarketStatsService, ResearchService, TradeCollector, TradeCollectorConfig,
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
    /// Research service (optional - requires EXA_API_KEY and OPENAI_API_KEY)
    pub research_service: Option<Arc<ResearchService>>,
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
    let market_cache = MarketCache::new(&cache_db_path, market_service.clone())
        .await
        .expect("Failed to initialize market cache");
    let market_cache = Arc::new(market_cache);

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
    let trade_storage = Arc::new(
        TradeStorage::new(&db_path).expect("Failed to initialize trade storage"),
    );

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
    let news_service = terminal_services::NewsService::new(exa_api_key.clone(), firecrawl_api_key, news_config);
    let news_service = Some(Arc::new(news_service));
    info!("News service initialized (RSS feeds + Google News)");

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
        research_service,
    };

    // Configure CORS for frontend
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
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
