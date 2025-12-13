//! API route definitions

mod health;
mod markets;
mod news;
mod research;
pub mod trading;
pub mod ws;

use axum::Router;
use crate::AppState;

// Re-export trading state types
pub use trading::{create_trading_state, SharedTradingState};

/// Create all API routes
pub fn api_routes() -> Router<AppState> {
    Router::new()
        .merge(markets::routes())
        .merge(news::routes())
        .merge(health::routes())
        .merge(research::routes())
        .merge(trading::routes())
}

/// Create WebSocket routes (separate from API)
pub fn ws_routes() -> Router<AppState> {
    ws::routes()
}
