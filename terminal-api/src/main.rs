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
    AggregatorConfig, CandleService, MarketDataAggregator, MarketService, NewsAggregator,
    NewsAggregatorConfig, NewsService, NewsServiceConfig, TradeCollector, TradeCollectorConfig,
    TradeStorage, WebSocketState,
};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub market_service: Arc<MarketService>,
    pub ws_state: Arc<WebSocketState>,
    pub trade_storage: Arc<TradeStorage>,
    pub candle_service: Arc<CandleService>,
    pub trade_collector: Arc<TradeCollector>,
    pub aggregator: Arc<MarketDataAggregator>,
    pub news_service: Option<Arc<NewsService>>,
    pub news_aggregator: Option<Arc<NewsAggregator>>,
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

    // Initialize services
    let market_service = Arc::new(MarketService::new(kalshi_client, polymarket_client));

    // Create subscription event channel for aggregator integration
    let (subscription_tx, subscription_rx) = WebSocketState::create_subscription_event_channel();

    // Create WebSocket state with subscription event sender
    let mut ws_state = WebSocketState::new((*market_service).clone());
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

    // Initialize trade collector
    let trade_collector = Arc::new(TradeCollector::new(
        market_service.clone(),
        trade_storage.clone(),
        Some(ws_state.clone()),
        TradeCollectorConfig::default(),
    ));

    // Start trade collector in background
    let collector_handle = trade_collector.clone();
    tokio::spawn(async move {
        collector_handle.start().await;
    });

    // Initialize news services (Exa semantic search + RSS fallback)
    let (news_service, news_aggregator) = {
        let exa_api_key = std::env::var("EXA_API_KEY").ok();
        let firecrawl_api_key = std::env::var("FIRECRAWL_API_KEY").ok();

        info!(
            "Initializing news service (Exa: {}, Firecrawl: {})",
            exa_api_key.is_some(),
            firecrawl_api_key.is_some()
        );

        let mut news_service = NewsService::new(
            exa_api_key,
            firecrawl_api_key,
            NewsServiceConfig::default(),
        );

        // Connect to market service for relevance scoring
        news_service.set_market_service(market_service.clone());

        let news_service = Arc::new(news_service);

        let news_aggregator = Arc::new(NewsAggregator::new(
            news_service.clone(),
            ws_state.clone(),
            NewsAggregatorConfig::default(),
        ));

        // Start news aggregator in background
        let aggregator_handle = news_aggregator.clone();
        tokio::spawn(async move {
            aggregator_handle.start().await;
        });

        (Some(news_service), Some(news_aggregator))
    };

    // Initialize and start market data aggregator
    let aggregator_config = AggregatorConfig::default();
    let mut aggregator = MarketDataAggregator::new(
        aggregator_config,
        ws_state.clone(),
        (*market_service).clone(),
    );

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

    // Create app state
    let state = AppState {
        market_service,
        ws_state,
        trade_storage,
        candle_service,
        trade_collector,
        aggregator,
        news_service,
        news_aggregator,
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
