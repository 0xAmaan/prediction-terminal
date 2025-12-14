//! News-related API endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use tracing::{error, info};

use crate::AppState;

/// Query parameters for listing news
#[derive(Debug, Deserialize)]
pub struct NewsQuery {
    /// Search query
    pub query: Option<String>,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Time range (e.g., "24h", "7d", "30d")
    pub time_range: Option<String>,
    /// Skip expensive embedding generation for faster responses
    #[serde(default)]
    pub skip_embeddings: bool,
}

/// Query parameters for article content
#[derive(Debug, Deserialize)]
pub struct ArticleQuery {
    /// Article URL to scrape
    pub url: String,
}

/// Create news routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/news", get(get_global_news))
        .route("/news/search", get(search_news))
        .route("/news/article", get(get_article_content))
        .route("/markets/{platform}/{id}/news", get(get_market_news))
}

/// GET /api/news - Get latest global prediction market news from RSS feeds
/// Returns cached data instantly, refreshes in background if needed
async fn get_global_news(
    State(state): State<AppState>,
    Query(params): Query<NewsQuery>,
) -> impl IntoResponse {
    let news_service = match &state.news_service {
        Some(service) => service,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "News service not configured"
                })),
            )
                .into_response();
        }
    };

    let limit = params.limit.unwrap_or(20);

    // Try to get cached news first (instant response)
    if let Ok(cached_feed) = state.news_cache.get_cached_global_news(limit) {
        if !cached_feed.items.is_empty() {
            // Return cached data immediately
            // Background refresh task will update cache if needed
            return (StatusCode::OK, Json(cached_feed)).into_response();
        }
    }

    // No cache available - fetch directly (only happens on first request)
    let search_params = terminal_core::NewsSearchParams {
        query: None,
        limit,
        time_range: params.time_range.or_else(|| Some("24h".to_string())),
        market_id: None,
        skip_embeddings: params.skip_embeddings,
    };

    match news_service.search_global_news(&search_params).await {
        Ok(feed) => {
            // Store in cache for next time
            if let Err(e) = state.news_cache.store_news_items("global", &feed.items) {
                error!("Failed to cache news items: {}", e);
            } else {
                state.news_cache.mark_refreshed().await;
            }
            (StatusCode::OK, Json(feed)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch global news: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to fetch news: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// GET /api/news/search - Search news with custom query
/// Note: RSS feeds don't support search, so this just returns latest news
async fn search_news(
    State(state): State<AppState>,
    Query(params): Query<NewsQuery>,
) -> impl IntoResponse {
    let news_service = match &state.news_service {
        Some(service) => service,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "News service not configured. Please set EXA_API_KEY."
                })),
            )
                .into_response();
        }
    };

    let query = match params.query {
        Some(q) if !q.is_empty() => q,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Query parameter is required"
                })),
            )
                .into_response();
        }
    };

    let search_params = terminal_core::NewsSearchParams {
        query: Some(query),
        limit: params.limit.unwrap_or(20),
        time_range: params.time_range,
        market_id: None,
        skip_embeddings: params.skip_embeddings,
    };

    match news_service.search_global_news(&search_params).await {
        Ok(feed) => (StatusCode::OK, Json(feed)).into_response(),
        Err(e) => {
            error!("Failed to search news: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to search news: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// GET /api/news/article?url=... - Get full article content
async fn get_article_content(
    State(state): State<AppState>,
    Query(params): Query<ArticleQuery>,
) -> impl IntoResponse {
    let news_service = match &state.news_service {
        Some(service) => service,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "News service not configured"
                })),
            )
                .into_response();
        }
    };

    match news_service.get_article_content(&params.url).await {
        Ok(content) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "content": content.markdown,
                "title": content.title,
                "description": content.description,
                "image_url": content.image_url,
                "site_name": content.site_name
            })),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to fetch article content: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to fetch article: {}", e)
                })),
            )
                .into_response()
        }
    }
}

/// GET /api/markets/:platform/:id/news - Get contextual news for a market
async fn get_market_news(
    State(state): State<AppState>,
    Path((platform, id)): Path<(String, String)>,
    Query(params): Query<NewsQuery>,
) -> impl IntoResponse {
    let news_service = match &state.news_service {
        Some(service) => service,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "News service not configured. Please set EXA_API_KEY."
                })),
            )
                .into_response();
        }
    };

    // Parse platform
    let platform_enum = match platform.to_lowercase().as_str() {
        "kalshi" => terminal_core::Platform::Kalshi,
        "polymarket" => terminal_core::Platform::Polymarket,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid platform. Use 'kalshi' or 'polymarket'"
                })),
            )
                .into_response();
        }
    };

    // Fetch the market to get its title and outcomes
    let market = match state.market_service.get_market(platform_enum, &id).await {
        Ok(m) => m,
        Err(terminal_core::TerminalError::NotFound(_)) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Market not found"
                })),
            )
                .into_response();
        }
        Err(e) => {
            error!("Failed to fetch market: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to fetch market: {}", e)
                })),
            )
                .into_response();
        }
    };

    // Log raw options_json for debugging
    if let Some(ref json) = market.options_json {
        info!(
            "Market {} raw options_json (first 1000 chars): {}",
            id,
            &json[..json.len().min(1000)]
        );
    }

    // Extract outcome titles from options_json for multi-outcome markets
    // Try multiple field names since Kalshi and Polymarket may differ
    let outcome_titles: Option<Vec<String>> = market.options_json.as_ref().and_then(|json| {
        serde_json::from_str::<Vec<serde_json::Value>>(json)
            .ok()
            .map(|options| {
                options
                    .iter()
                    .filter_map(|opt| {
                        // Try different field names
                        opt.get("title")
                            .or_else(|| opt.get("name"))
                            .or_else(|| opt.get("outcome"))
                            .or_else(|| opt.get("ticker"))
                            .and_then(|n| n.as_str())
                            .map(String::from)
                    })
                    .collect()
            })
    });

    // Log the extracted outcomes for debugging
    if let Some(ref outcomes) = outcome_titles {
        info!(
            "Market {} extracted {} outcomes: {:?}",
            id,
            outcomes.len(),
            outcomes.iter().take(5).collect::<Vec<_>>()
        );
    } else {
        info!("Market {} has no outcomes", id);
    }

    let limit = params.limit.unwrap_or(10);

    // Use keyword-only matching for market-specific news (faster, more precise)
    match news_service
        .get_market_news(&market.title, &id, limit, outcome_titles)
        .await
    {
        Ok(feed) => {
            info!(
                "Fetched {} news items for market: {}",
                feed.items.len(),
                market.title
            );
            (StatusCode::OK, Json(feed)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch market news: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Failed to fetch news: {}", e)
                })),
            )
                .into_response()
        }
    }
}
