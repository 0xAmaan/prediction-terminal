//! Health check endpoints

use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use serde::Serialize;

use crate::AppState;

/// Health check response
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    aggregator: terminal_services::AggregatorHealth,
}

/// Health check handler
async fn health_check(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    let aggregator_health = state.aggregator.get_health().await;

    let status = if aggregator_health.healthy {
        "healthy"
    } else {
        "degraded"
    };

    let response = HealthResponse {
        status: status.to_string(),
        aggregator: aggregator_health,
    };

    let code = if status == "healthy" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (code, Json(response))
}

/// Simple liveness check (always returns OK if server is running)
async fn liveness() -> &'static str {
    "OK"
}

/// Create health routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/health/live", get(liveness))
}
