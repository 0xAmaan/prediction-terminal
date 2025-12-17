//! Research API endpoints for AI-powered market analysis

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use terminal_core::Platform;
use terminal_research::{ChatMessage, ResearchJob, ResearchStatus, ResearchVersionList};
use tracing::{error, info};

use crate::AppState;

/// Create research routes
///
/// NOTE: Route ordering matters in Axum. More specific routes (with additional
/// path segments) must come BEFORE less specific routes with wildcards.
pub fn routes() -> Router<AppState> {
    Router::new()
        // Most specific routes first (with extra path segments after :market_id)
        .route(
            "/research/{platform}/{market_id}/versions/{version_key}",
            get(get_version),
        )
        .route(
            "/research/{platform}/{market_id}/versions",
            get(list_versions),
        )
        .route("/research/{platform}/{market_id}/chat", get(get_chat))
        .route("/research/{platform}/{market_id}/chat", post(send_chat))
        // Less specific routes last
        .route("/research/{platform}/{market_id}", post(start_research))
        .route("/research/{platform}/{market_id}", get(get_research))
        // Static routes (no wildcards in the middle)
        .route("/research/job/{job_id}", get(get_job))
        .route("/research/jobs", get(list_jobs))
        .route("/research/reports", get(list_reports))
        .route("/research/mispriced", get(get_mispriced_markets))
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug, Serialize)]
struct StartResearchResponse {
    job_id: String,
    status: ResearchStatus,
}

/// Start a new research job for a market
async fn start_research(
    State(state): State<AppState>,
    Path((platform_str, market_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Starting research for {} on {}", market_id, platform_str);

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

    // Check if research service is available
    let research_service = match &state.research_service {
        Some(service) => service,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "Research service not available. Check EXA_API_KEY and OPENAI_API_KEY environment variables.".to_string(),
                }),
            )
                .into_response();
        }
    };

    match research_service.start_research(platform, &market_id).await {
        Ok(job) => {
            let job_id = job.id.clone();

            // Spawn background task to execute research
            let research_service = research_service.clone();
            tokio::spawn(async move {
                if let Err(e) = research_service.execute_research(&job_id).await {
                    error!("Research execution failed: {}", e);
                }
            });

            (
                StatusCode::ACCEPTED,
                Json(StartResearchResponse {
                    job_id: job.id,
                    status: job.status,
                }),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to start research: {}", e);
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

/// Get cached research for a market (by platform/market_id)
async fn get_research(
    State(state): State<AppState>,
    Path((platform_str, market_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Getting cached research for {} on {}",
        market_id, platform_str
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

    // Check if research service is available
    let research_service = match &state.research_service {
        Some(service) => service,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "Research service not available.".to_string(),
                }),
            )
                .into_response();
        }
    };

    // Try to get cached research
    match research_service.get_cached_research(platform, &market_id).await {
        Ok(Some(job)) => (StatusCode::OK, Json(job)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "No cached research found. Start a new research job.".to_string(),
            }),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to check research cache: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to check cache: {}", e),
                }),
            )
                .into_response()
        }
    }
}

/// Get a research job by ID
async fn get_job(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting research job: {}", job_id);

    let research_service = match &state.research_service {
        Some(service) => service,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "Research service not available.".to_string(),
                }),
            )
                .into_response();
        }
    };

    match research_service.get_job(&job_id).await {
        Some(job) => (StatusCode::OK, Json(job)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Job not found: {}", job_id),
            }),
        )
            .into_response(),
    }
}

/// List all research jobs (in-memory, current session only)
async fn list_jobs(State(state): State<AppState>) -> impl IntoResponse {
    info!("Listing research jobs");

    let research_service = match &state.research_service {
        Some(service) => service,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "Research service not available.".to_string(),
                }),
            )
                .into_response();
        }
    };

    let jobs: Vec<ResearchJob> = research_service.list_jobs().await;
    (StatusCode::OK, Json(jobs)).into_response()
}

/// List all saved research reports from S3 (persisted)
async fn list_reports(State(state): State<AppState>) -> impl IntoResponse {
    info!("Listing all saved research reports");

    let research_service = match &state.research_service {
        Some(service) => service,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "Research service not available.".to_string(),
                }),
            )
                .into_response();
        }
    };

    match research_service.list_all_reports().await {
        Ok(reports) => (StatusCode::OK, Json(reports)).into_response(),
        Err(e) => {
            error!("Failed to list reports: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to list reports: {}", e),
                }),
            )
                .into_response()
        }
    }
}

/// List all versions of research for a market
async fn list_versions(
    State(state): State<AppState>,
    Path((platform_str, market_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Listing research versions for {} on {}",
        market_id, platform_str
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

    let research_service = match &state.research_service {
        Some(service) => service,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "Research service not available.".to_string(),
                }),
            )
                .into_response();
        }
    };

    match research_service.list_versions(platform, &market_id).await {
        Ok(versions) => (StatusCode::OK, Json(ResearchVersionList { versions })).into_response(),
        Err(e) => {
            error!("Failed to list versions: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to list versions: {}", e),
                }),
            )
                .into_response()
        }
    }
}

/// Get a specific version of research
async fn get_version(
    State(state): State<AppState>,
    Path((platform_str, market_id, version_key)): Path<(String, String, String)>,
) -> impl IntoResponse {
    info!(
        "Getting research version {} for {} on {}",
        version_key, market_id, platform_str
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

    let research_service = match &state.research_service {
        Some(service) => service,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "Research service not available.".to_string(),
                }),
            )
                .into_response();
        }
    };

    match research_service
        .get_version(platform, &market_id, &version_key)
        .await
    {
        Ok(Some(job)) => (StatusCode::OK, Json(job)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Version not found: {}", version_key),
            }),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to get version: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to get version: {}", e),
                }),
            )
                .into_response()
        }
    }
}

// ============================================================================
// Chat Endpoints
// ============================================================================

/// Request body for sending a chat message
#[derive(Debug, Deserialize)]
struct SendChatRequest {
    message: String,
}

/// Response for sending a chat message
#[derive(Debug, Serialize)]
struct SendChatResponse {
    message: ChatMessage,
}

/// Get chat history for a market's research
async fn get_chat(
    State(state): State<AppState>,
    Path((platform_str, market_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Getting chat history for {} on {}",
        market_id, platform_str
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

    let research_service = match &state.research_service {
        Some(service) => service,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "Research service not available.".to_string(),
                }),
            )
                .into_response();
        }
    };

    match research_service.get_chat(platform, &market_id).await {
        Ok(history) => (StatusCode::OK, Json(history)).into_response(),
        Err(e) => {
            error!("Failed to get chat history: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to get chat history: {}", e),
                }),
            )
                .into_response()
        }
    }
}

/// Send a chat message and get a response
async fn send_chat(
    State(state): State<AppState>,
    Path((platform_str, market_id)): Path<(String, String)>,
    Json(request): Json<SendChatRequest>,
) -> impl IntoResponse {
    info!(
        "Sending chat message for {} on {}: {}",
        market_id, platform_str, request.message
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

    let research_service = match &state.research_service {
        Some(service) => service,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "Research service not available.".to_string(),
                }),
            )
                .into_response();
        }
    };

    match research_service
        .send_chat_message(platform, &market_id, &request.message)
        .await
    {
        Ok(response_message) => (
            StatusCode::OK,
            Json(SendChatResponse {
                message: response_message,
            }),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to send chat message: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to send message: {}", e),
                }),
            )
                .into_response()
        }
    }
}

/// Get markets with research indicating mispricing (edge > 2%)
async fn get_mispriced_markets(State(state): State<AppState>) -> impl IntoResponse {
    info!("Getting mispriced markets from edge index");

    let research_service = match &state.research_service {
        Some(service) => service,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "Research service not available.".to_string(),
                }),
            )
                .into_response();
        }
    };

    match research_service.get_edge_index().await {
        Ok(index) => {
            // Filter to only markets with significant edge (>2%)
            let mispriced: Vec<_> = index.entries.into_iter().filter(|e| e.implied_edge.abs() > 0.02).collect();
            info!("Found {} mispriced markets", mispriced.len());
            (StatusCode::OK, Json(mispriced)).into_response()
        }
        Err(e) => {
            error!("Failed to get edge index: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to get mispriced markets: {}", e),
                }),
            )
                .into_response()
        }
    }
}

/// Helper to parse platform string
fn parse_platform(s: &str) -> Option<Platform> {
    match s.to_lowercase().as_str() {
        "kalshi" | "k" => Some(Platform::Kalshi),
        "polymarket" | "poly" | "p" => Some(Platform::Polymarket),
        _ => None,
    }
}
