//! Research API endpoints for AI-powered market analysis

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use terminal_core::Platform;
use terminal_research::{ResearchJob, ResearchStatus};
use tracing::{error, info};

use crate::AppState;

/// Create research routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/research/:platform/:market_id", post(start_research))
        .route("/research/:platform/:market_id", get(get_research))
        .route("/research/job/:job_id", get(get_job))
        .route("/research/jobs", get(list_jobs))
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
    State(_state): State<AppState>,
    Path((platform_str, market_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Getting cached research for {} on {}",
        market_id, platform_str
    );

    // TODO: Look up cached research by platform/market_id from S3
    // For now, return 404
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "No cached research found. Start a new research job.".to_string(),
        }),
    )
        .into_response()
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

/// List all research jobs
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

/// Helper to parse platform string
fn parse_platform(s: &str) -> Option<Platform> {
    match s.to_lowercase().as_str() {
        "kalshi" | "k" => Some(Platform::Kalshi),
        "polymarket" | "poly" | "p" => Some(Platform::Polymarket),
        _ => None,
    }
}
