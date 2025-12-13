//! API route definitions

mod health;
mod markets;
mod news;
pub mod ws;

use axum::Router;
use crate::AppState;

/// Create all API routes
pub fn api_routes() -> Router<AppState> {
    Router::new()
        .merge(markets::routes())
        .merge(news::routes())
        .merge(health::routes())
}

/// Create WebSocket routes (separate from API)
pub fn ws_routes() -> Router<AppState> {
    ws::routes()
}
