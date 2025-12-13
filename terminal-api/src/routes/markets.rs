//! Market-related API endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use terminal_core::{Platform, PredictionMarket, PriceInterval};
use tracing::{error, info};

use crate::AppState;

/// Query parameters for listing markets
#[derive(Debug, Deserialize)]
pub struct ListMarketsQuery {
    /// Filter by platform (kalshi, polymarket, or all)
    pub platform: Option<String>,
    /// Search query
    pub search: Option<String>,
    /// Maximum number of results
    pub limit: Option<usize>,
}

/// Response for listing markets
#[derive(Debug, Serialize)]
pub struct MarketsResponse {
    pub markets: Vec<PredictionMarket>,
    pub count: usize,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Query parameters for order book
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct OrderBookQuery {
    /// Depth of order book levels (reserved for future use)
    pub depth: Option<usize>,
}

/// Query parameters for trades
#[derive(Debug, Deserialize)]
pub struct TradesQuery {
    /// Maximum number of trades
    pub limit: Option<u32>,
    /// Cursor for pagination
    pub cursor: Option<String>,
}

/// Query parameters for related markets
#[derive(Debug, Deserialize)]
pub struct RelatedMarketsQuery {
    /// Maximum number of related markets
    pub limit: Option<usize>,
}

/// Query parameters for price history
#[derive(Debug, Deserialize)]
pub struct PriceHistoryQuery {
    /// Candle interval (1m, 5m, 15m, 1h, 4h, 1d)
    pub interval: Option<String>,
    /// Timeframe preset (1H, 24H, 7D, 30D, ALL)
    pub timeframe: Option<String>,
}

/// Query parameters for multi-outcome prices
#[derive(Debug, Deserialize)]
pub struct MultiOutcomePricesQuery {
    /// Number of top outcomes to include (default 5)
    pub top: Option<usize>,
    /// Interval: "1h", "6h", "1d", "1w", "max"
    pub interval: Option<String>,
}

/// Query parameters for outcome-specific data
#[derive(Debug, Deserialize)]
pub struct OutcomeQuery {
    /// Maximum number of trades
    pub limit: Option<u32>,
}

/// Query parameters for outcome price history
#[derive(Debug, Deserialize)]
pub struct OutcomePriceHistoryQuery {
    /// Interval: "1h", "6h", "1d", "1w", "max"
    pub interval: Option<String>,
}

/// Create market routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/markets", get(list_markets))
        .route("/markets/:platform/:id", get(get_market))
        .route("/markets/:platform/:id/orderbook", get(get_orderbook))
        .route("/markets/:platform/:id/trades", get(get_trades))
        .route("/markets/:platform/:id/history", get(get_price_history))
        .route("/markets/:platform/:id/related", get(get_related_markets))
        // Multi-outcome / outcome-specific routes
        .route(
            "/markets/:platform/:id/prices-history",
            get(get_multi_outcome_prices),
        )
        .route(
            "/markets/:platform/:id/outcomes/:outcome_id/orderbook",
            get(get_outcome_orderbook),
        )
        .route(
            "/markets/:platform/:id/outcomes/:outcome_id/trades",
            get(get_outcome_trades),
        )
        .route(
            "/markets/:platform/:id/outcomes/:outcome_id/prices-history",
            get(get_outcome_prices),
        )
}

/// List markets with optional filtering
async fn list_markets(
    State(state): State<AppState>,
    Query(params): Query<ListMarketsQuery>,
) -> impl IntoResponse {
    info!("Listing markets with params: {:?}", params);

    // Parse platform filter
    let platform_filter: Option<Platform> =
        params
            .platform
            .as_ref()
            .and_then(|p| match p.to_lowercase().as_str() {
                "kalshi" | "k" => Some(Platform::Kalshi),
                "polymarket" | "poly" | "p" => Some(Platform::Polymarket),
                "all" | "" => None,
                _ => None,
            });

    // Fetch markets
    let result = if let Some(query) = &params.search {
        state
            .market_service
            .search_markets(query, platform_filter, params.limit)
            .await
    } else {
        match platform_filter {
            Some(platform) => {
                state
                    .market_service
                    .get_markets_by_platform(platform, params.limit)
                    .await
            }
            None => state.market_service.get_all_markets(params.limit).await,
        }
    };

    match result {
        Ok(markets) => {
            let count = markets.len();
            info!("Returning {} markets", count);
            (StatusCode::OK, Json(MarketsResponse { markets, count })).into_response()
        }
        Err(e) => {
            error!("Failed to fetch markets: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Get a single market by platform and ID
async fn get_market(
    State(state): State<AppState>,
    Path((platform_str, id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Getting market: {} on {}", id, platform_str);

    let platform = match platform_str.to_lowercase().as_str() {
        "kalshi" | "k" => Platform::Kalshi,
        "polymarket" | "poly" | "p" => Platform::Polymarket,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Unknown platform: {}", platform_str),
                }),
            )
                .into_response();
        }
    };

    match state.market_service.get_market(platform, &id).await {
        Ok(market) => (StatusCode::OK, Json(market)).into_response(),
        Err(terminal_core::TerminalError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Market not found: {}", id),
            }),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to fetch market: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

// ============================================================================
// Order Book, Trades, and Related Markets Endpoints
// ============================================================================

/// Helper to parse platform string
fn parse_platform(platform_str: &str) -> Option<Platform> {
    match platform_str.to_lowercase().as_str() {
        "kalshi" | "k" => Some(Platform::Kalshi),
        "polymarket" | "poly" | "p" => Some(Platform::Polymarket),
        _ => None,
    }
}

/// Get order book for a market
async fn get_orderbook(
    State(state): State<AppState>,
    Path((platform_str, id)): Path<(String, String)>,
    Query(_params): Query<OrderBookQuery>,
) -> impl IntoResponse {
    info!("Getting orderbook for {} on {}", id, platform_str);

    let platform = match parse_platform(&platform_str) {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Unknown platform: {}", platform_str),
                }),
            )
                .into_response();
        }
    };

    match state.market_service.get_orderbook(platform, &id).await {
        Ok(orderbook) => (StatusCode::OK, Json(orderbook)).into_response(),
        Err(terminal_core::TerminalError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Market not found: {}", id),
            }),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to fetch orderbook: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Get recent trades for a market
async fn get_trades(
    State(state): State<AppState>,
    Path((platform_str, id)): Path<(String, String)>,
    Query(params): Query<TradesQuery>,
) -> impl IntoResponse {
    info!("Getting trades for {} on {}", id, platform_str);

    let platform = match parse_platform(&platform_str) {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Unknown platform: {}", platform_str),
                }),
            )
                .into_response();
        }
    };

    // Start tracking this market for ongoing collection
    state
        .trade_collector
        .track_market(platform, id.clone())
        .await;

    match state
        .market_service
        .get_trades(platform, &id, params.limit, params.cursor.as_deref())
        .await
    {
        Ok(trade_history) => {
            // Store trades in SQLite for price history generation
            if !trade_history.trades.is_empty() {
                if let Err(e) = state.trade_storage.store_trades(&trade_history.trades) {
                    error!("Failed to store trades: {}", e);
                }
            }
            (StatusCode::OK, Json(trade_history)).into_response()
        }
        Err(terminal_core::TerminalError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Market not found: {}", id),
            }),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to fetch trades: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Get related markets
async fn get_related_markets(
    State(state): State<AppState>,
    Path((platform_str, id)): Path<(String, String)>,
    Query(params): Query<RelatedMarketsQuery>,
) -> impl IntoResponse {
    info!("Getting related markets for {} on {}", id, platform_str);

    let platform = match parse_platform(&platform_str) {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Unknown platform: {}", platform_str),
                }),
            )
                .into_response();
        }
    };

    match state
        .market_service
        .get_related_markets(platform, &id, params.limit)
        .await
    {
        Ok(markets) => {
            let count = markets.len();
            (StatusCode::OK, Json(MarketsResponse { markets, count })).into_response()
        }
        Err(terminal_core::TerminalError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Market not found: {}", id),
            }),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to fetch related markets: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Get price history (candles) for a market
async fn get_price_history(
    State(state): State<AppState>,
    Path((platform_str, id)): Path<(String, String)>,
    Query(params): Query<PriceHistoryQuery>,
) -> impl IntoResponse {
    info!("Getting price history for {} on {}", id, platform_str);

    let platform = match parse_platform(&platform_str) {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Unknown platform: {}", platform_str),
                }),
            )
                .into_response();
        }
    };

    // Check if we have any trades for this market
    let trade_count = state
        .trade_storage
        .get_trade_count(platform, &id)
        .unwrap_or(0);

    // If no trades, do an immediate backfill and start tracking
    if trade_count == 0 {
        info!(
            "No trades found for {:?}/{}, initiating backfill",
            platform, id
        );

        // Start tracking for future updates
        state
            .trade_collector
            .track_market(platform, id.clone())
            .await;

        // Do immediate backfill (fetch up to 5 pages of historical trades)
        if let Err(e) = state
            .trade_collector
            .backfill_market(platform, &id, 5)
            .await
        {
            error!("Failed to backfill trades: {}", e);
            // Continue anyway - we'll return empty data
        }
    }

    // If timeframe is provided, use the convenience method
    if let Some(timeframe) = params.timeframe {
        match state
            .candle_service
            .get_candles_for_timeframe(platform, &id, &timeframe)
        {
            Ok(mut history) => {
                state.candle_service.fill_gaps(&mut history);
                return (StatusCode::OK, Json(history)).into_response();
            }
            Err(e) => {
                error!("Failed to fetch price history: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: e.to_string(),
                    }),
                )
                    .into_response();
            }
        }
    }

    // Otherwise use interval with default time range
    let interval = params
        .interval
        .as_deref()
        .and_then(PriceInterval::from_str)
        .unwrap_or_default();

    let now = Utc::now();
    let from = now - Duration::hours(24); // Default to 24h

    match state
        .candle_service
        .build_candles(platform, &id, interval, from, now)
    {
        Ok(mut history) => {
            state.candle_service.fill_gaps(&mut history);
            (StatusCode::OK, Json(history)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch price history: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

// ============================================================================
// Multi-Outcome / Outcome-Specific Endpoints
// ============================================================================

/// Get price history for multiple outcomes (top N by price)
async fn get_multi_outcome_prices(
    State(state): State<AppState>,
    Path((platform_str, id)): Path<(String, String)>,
    Query(params): Query<MultiOutcomePricesQuery>,
) -> impl IntoResponse {
    info!(
        "Getting multi-outcome prices for {} on {}",
        id, platform_str
    );

    let platform = match parse_platform(&platform_str) {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Unknown platform: {}", platform_str),
                }),
            )
                .into_response();
        }
    };

    let top = params.top.unwrap_or(5);
    let interval = params.interval.as_deref().unwrap_or("1d");

    match state
        .market_service
        .get_multi_outcome_prices(platform, &id, top, interval)
        .await
    {
        Ok(outcomes) => (StatusCode::OK, Json(outcomes)).into_response(),
        Err(terminal_core::TerminalError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Market not found: {}", id),
            }),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to fetch multi-outcome prices: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Get orderbook for a specific outcome within a multi-outcome event
async fn get_outcome_orderbook(
    State(state): State<AppState>,
    Path((platform_str, event_id, outcome_id)): Path<(String, String, String)>,
) -> impl IntoResponse {
    info!(
        "Getting outcome orderbook for {} in {} on {}",
        outcome_id, event_id, platform_str
    );

    let platform = match parse_platform(&platform_str) {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Unknown platform: {}", platform_str),
                }),
            )
                .into_response();
        }
    };

    match state
        .market_service
        .get_outcome_orderbook(platform, &event_id, &outcome_id)
        .await
    {
        Ok(orderbook) => (StatusCode::OK, Json(orderbook)).into_response(),
        Err(terminal_core::TerminalError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Outcome not found: {}", outcome_id),
            }),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to fetch outcome orderbook: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Get trades for a specific outcome within a multi-outcome event
async fn get_outcome_trades(
    State(state): State<AppState>,
    Path((platform_str, event_id, outcome_id)): Path<(String, String, String)>,
    Query(params): Query<OutcomeQuery>,
) -> impl IntoResponse {
    info!(
        "Getting outcome trades for {} in {} on {}",
        outcome_id, event_id, platform_str
    );

    let platform = match parse_platform(&platform_str) {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Unknown platform: {}", platform_str),
                }),
            )
                .into_response();
        }
    };

    match state
        .market_service
        .get_outcome_trades(platform, &event_id, &outcome_id, params.limit)
        .await
    {
        Ok(trade_history) => (StatusCode::OK, Json(trade_history)).into_response(),
        Err(terminal_core::TerminalError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Outcome not found: {}", outcome_id),
            }),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to fetch outcome trades: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}

/// Get price history for a specific outcome
async fn get_outcome_prices(
    State(state): State<AppState>,
    Path((platform_str, event_id, outcome_id)): Path<(String, String, String)>,
    Query(params): Query<OutcomePriceHistoryQuery>,
) -> impl IntoResponse {
    info!(
        "Getting outcome price history for {} in {} on {}",
        outcome_id, event_id, platform_str
    );

    let platform = match parse_platform(&platform_str) {
        Some(p) => p,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: format!("Unknown platform: {}", platform_str),
                }),
            )
                .into_response();
        }
    };

    let interval = params.interval.as_deref().unwrap_or("1d");
    let _ = event_id; // We don't need event_id for this - outcome_id is the token_id

    match state
        .market_service
        .get_outcome_prices(platform, &outcome_id, interval)
        .await
    {
        Ok(history) => (StatusCode::OK, Json(history)).into_response(),
        Err(terminal_core::TerminalError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Outcome not found: {}", outcome_id),
            }),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to fetch outcome price history: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response()
        }
    }
}
