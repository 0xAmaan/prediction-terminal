//! Trading API routes for Polymarket order execution

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use terminal_trading::{ClobClient, OrderBuilder, OrderType, Side};

use crate::AppState;

// ============================================================================
// Types
// ============================================================================

/// Request to submit a new order
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitOrderRequest {
    /// CLOB token ID for the outcome
    pub token_id: String,
    /// Order side: "buy" or "sell"
    pub side: String,
    /// Limit price (0.01 to 0.99)
    pub price: f64,
    /// Number of shares
    pub size: f64,
    /// Order type: "GTC", "GTD", or "FOK"
    #[serde(default = "default_order_type")]
    pub order_type: String,
}

fn default_order_type() -> String {
    "GTC".to_string()
}

/// Response from order submission
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitOrderResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub transaction_hashes: Vec<String>,
}

/// Balance response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResponse {
    pub usdc_balance: String,
    pub usdc_allowance: String,
    pub wallet_address: String,
}

/// Deposit info response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositInfoResponse {
    pub address: String,
    pub network: String,
    pub token: String,
}

/// Open order response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenOrderResponse {
    pub id: String,
    pub market: String,
    pub asset_id: String,
    pub side: String,
    pub original_size: String,
    pub size_matched: String,
    pub price: String,
    pub status: String,
    pub created_at: String,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Position response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionResponse {
    pub market_id: String,
    pub token_id: String,
    pub outcome: String,
    pub shares: String,
    pub avg_price: String,
    pub current_price: String,
    pub pnl: String,
}

// ============================================================================
// Trading State
// ============================================================================

/// Shared state for trading operations
pub struct TradingState {
    clob_client: Option<ClobClient>,
    initialized: bool,
}

impl TradingState {
    pub fn new() -> Self {
        Self {
            clob_client: None,
            initialized: false,
        }
    }

    /// Initialize the CLOB client from environment
    pub async fn initialize(&mut self) -> Result<(), String> {
        if self.initialized {
            return Ok(());
        }

        match ClobClient::from_env() {
            Ok(mut client) => {
                // Try to derive API key
                if let Err(e) = client.ensure_api_key().await {
                    error!("Failed to ensure API key: {}", e);
                    // Don't fail - we can still return deposit address
                }
                self.clob_client = Some(client);
                self.initialized = true;
                info!("Trading client initialized");
                Ok(())
            }
            Err(e) => {
                error!("Failed to initialize trading client: {}", e);
                Err(format!("Failed to initialize trading: {}", e))
            }
        }
    }

    pub fn client(&self) -> Option<&ClobClient> {
        self.clob_client.as_ref()
    }

    pub fn client_mut(&mut self) -> Option<&mut ClobClient> {
        self.clob_client.as_mut()
    }
}

/// Type alias for shared trading state
pub type SharedTradingState = Arc<RwLock<TradingState>>;

/// Create a new shared trading state
pub fn create_trading_state() -> SharedTradingState {
    Arc::new(RwLock::new(TradingState::new()))
}

// ============================================================================
// Route Handlers
// ============================================================================

/// Submit a new order
async fn submit_order(
    State(state): State<AppState>,
    Json(req): Json<SubmitOrderRequest>,
) -> impl IntoResponse {
    info!("Submitting order: {:?}", req);

    // Get trading state
    let trading_state = match state.trading_state.as_ref() {
        Some(ts) => ts,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(SubmitOrderResponse {
                    success: false,
                    order_id: None,
                    error: Some("Trading not enabled".to_string()),
                    transaction_hashes: vec![],
                }),
            );
        }
    };

    // Ensure initialized
    {
        let mut state = trading_state.write().await;
        if let Err(e) = state.initialize().await {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(SubmitOrderResponse {
                    success: false,
                    order_id: None,
                    error: Some(e),
                    transaction_hashes: vec![],
                }),
            );
        }
    }

    // Parse side
    let side = match req.side.to_lowercase().as_str() {
        "buy" => Side::Buy,
        "sell" => Side::Sell,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(SubmitOrderResponse {
                    success: false,
                    order_id: None,
                    error: Some(format!("Invalid side: {}", req.side)),
                    transaction_hashes: vec![],
                }),
            );
        }
    };

    // Parse order type
    let order_type = match req.order_type.to_uppercase().as_str() {
        "GTC" => OrderType::Gtc,
        "GTD" => OrderType::Gtd,
        "FOK" => OrderType::Fok,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(SubmitOrderResponse {
                    success: false,
                    order_id: None,
                    error: Some(format!("Invalid order type: {}", req.order_type)),
                    transaction_hashes: vec![],
                }),
            );
        }
    };

    // Get client and submit order
    let state = trading_state.read().await;
    let client = match state.client() {
        Some(c) => c,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(SubmitOrderResponse {
                    success: false,
                    order_id: None,
                    error: Some("Trading client not available".to_string()),
                    transaction_hashes: vec![],
                }),
            );
        }
    };

    // Build and sign order
    let builder = OrderBuilder::new(&req.token_id, req.price, req.size, side);
    let signed_order = match builder.build_and_sign(client.wallet()).await {
        Ok(o) => o,
        Err(e) => {
            error!("Failed to build/sign order: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(SubmitOrderResponse {
                    success: false,
                    order_id: None,
                    error: Some(format!("Failed to build order: {}", e)),
                    transaction_hashes: vec![],
                }),
            );
        }
    };

    // Submit to CLOB
    match client.post_order(signed_order, order_type).await {
        Ok(response) => {
            info!("Order submitted: {:?}", response.order_id);
            (
                StatusCode::OK,
                Json(SubmitOrderResponse {
                    success: response.success,
                    order_id: response.order_id,
                    error: response.error_msg,
                    transaction_hashes: response.transaction_hashes,
                }),
            )
        }
        Err(e) => {
            error!("Order submission failed: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(SubmitOrderResponse {
                    success: false,
                    order_id: None,
                    error: Some(format!("{}", e)),
                    transaction_hashes: vec![],
                }),
            )
        }
    }
}

/// Cancel an order by ID
async fn cancel_order(
    State(state): State<AppState>,
    Path(order_id): Path<String>,
) -> impl IntoResponse {
    info!("Cancelling order: {}", order_id);

    let trading_state = match state.trading_state.as_ref() {
        Some(ts) => ts,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "Trading not enabled".to_string(),
                }),
            );
        }
    };

    let state = trading_state.read().await;
    let client = match state.client() {
        Some(c) => c,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "Trading client not available".to_string(),
                }),
            );
        }
    };

    match client.cancel_order(&order_id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ErrorResponse {
                error: String::new(),
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("{}", e),
            }),
        ),
    }
}

/// Cancel all orders
async fn cancel_all_orders(State(state): State<AppState>) -> impl IntoResponse {
    info!("Cancelling all orders");

    let trading_state = match state.trading_state.as_ref() {
        Some(ts) => ts,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "Trading not enabled".to_string(),
                }),
            );
        }
    };

    let state = trading_state.read().await;
    let client = match state.client() {
        Some(c) => c,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "Trading client not available".to_string(),
                }),
            );
        }
    };

    match client.cancel_all_orders().await {
        Ok(_) => (
            StatusCode::OK,
            Json(ErrorResponse {
                error: String::new(),
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("{}", e),
            }),
        ),
    }
}

/// Get open orders
async fn get_open_orders(State(state): State<AppState>) -> impl IntoResponse {
    let trading_state = match state.trading_state.as_ref() {
        Some(ts) => ts,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, Json(vec![]));
        }
    };

    let state = trading_state.read().await;
    let client = match state.client() {
        Some(c) => c,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, Json(vec![]));
        }
    };

    match client.get_open_orders().await {
        Ok(orders) => {
            let response: Vec<OpenOrderResponse> = orders
                .into_iter()
                .map(|o| OpenOrderResponse {
                    id: o.id,
                    market: o.market,
                    asset_id: o.asset_id,
                    side: o.side,
                    original_size: o.original_size,
                    size_matched: o.size_matched,
                    price: o.price,
                    status: o.status,
                    created_at: o.created_at,
                })
                .collect();
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            error!("Failed to get open orders: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]))
        }
    }
}

/// Get deposit address
async fn get_deposit_address(State(state): State<AppState>) -> impl IntoResponse {
    let trading_state = match state.trading_state.as_ref() {
        Some(ts) => ts,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(DepositInfoResponse {
                    address: String::new(),
                    network: "Polygon".to_string(),
                    token: "USDC.e".to_string(),
                }),
            );
        }
    };

    // Ensure initialized
    {
        let mut state = trading_state.write().await;
        if let Err(e) = state.initialize().await {
            error!("Failed to initialize trading: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(DepositInfoResponse {
                    address: String::new(),
                    network: "Polygon".to_string(),
                    token: "USDC.e".to_string(),
                }),
            );
        }
    }

    let state = trading_state.read().await;
    let address = state
        .client()
        .map(|c| c.address())
        .unwrap_or_default();

    (
        StatusCode::OK,
        Json(DepositInfoResponse {
            address,
            network: "Polygon".to_string(),
            token: "USDC.e".to_string(),
        }),
    )
}

/// Get wallet balance via Polygon RPC
async fn get_balance(State(state): State<AppState>) -> impl IntoResponse {
    let trading_state = match state.trading_state.as_ref() {
        Some(ts) => ts,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(BalanceResponse {
                    usdc_balance: "0".to_string(),
                    usdc_allowance: "0".to_string(),
                    wallet_address: String::new(),
                }),
            );
        }
    };

    // Ensure initialized
    {
        let mut ts = trading_state.write().await;
        if let Err(e) = ts.initialize().await {
            error!("Failed to initialize trading: {}", e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(BalanceResponse {
                    usdc_balance: "0".to_string(),
                    usdc_allowance: "0".to_string(),
                    wallet_address: String::new(),
                }),
            );
        }
    }

    let ts = trading_state.read().await;
    let client = match ts.client() {
        Some(c) => c,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(BalanceResponse {
                    usdc_balance: "0".to_string(),
                    usdc_allowance: "0".to_string(),
                    wallet_address: String::new(),
                }),
            );
        }
    };

    let address = client.address();

    // Query actual balance from Polygon
    match client.get_balance().await {
        Ok(balance) => (
            StatusCode::OK,
            Json(BalanceResponse {
                usdc_balance: balance.usdc_balance,
                usdc_allowance: balance.usdc_allowance,
                wallet_address: address,
            }),
        ),
        Err(e) => {
            error!("Failed to get balance: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(BalanceResponse {
                    usdc_balance: "0".to_string(),
                    usdc_allowance: "0".to_string(),
                    wallet_address: address,
                }),
            )
        }
    }
}

/// Get positions from trade history
async fn get_positions(State(state): State<AppState>) -> impl IntoResponse {
    let trading_state = match state.trading_state.as_ref() {
        Some(ts) => ts,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, Json(vec![]));
        }
    };

    // Ensure initialized
    {
        let mut ts = trading_state.write().await;
        if let Err(e) = ts.initialize().await {
            error!("Failed to initialize trading: {}", e);
            return (StatusCode::SERVICE_UNAVAILABLE, Json(vec![]));
        }
    }

    let ts = trading_state.read().await;
    let client = match ts.client() {
        Some(c) => c,
        None => {
            return (StatusCode::SERVICE_UNAVAILABLE, Json(vec![]));
        }
    };

    match client.get_positions().await {
        Ok(positions) => {
            let response: Vec<PositionResponse> = positions
                .into_iter()
                .map(|p| PositionResponse {
                    market_id: p.market_id,
                    token_id: p.token_id,
                    outcome: p.outcome,
                    shares: p.shares,
                    avg_price: p.avg_price,
                    current_price: p.current_price,
                    pnl: p.pnl,
                })
                .collect();
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            error!("Failed to get positions: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]))
        }
    }
}

// ============================================================================
// Router
// ============================================================================

/// Create trading routes
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/trade/order", post(submit_order))
        .route("/trade/order/{order_id}", delete(cancel_order))
        .route("/trade/orders", get(get_open_orders))
        .route("/trade/orders/cancel-all", delete(cancel_all_orders))
        .route("/trade/deposit", get(get_deposit_address))
        .route("/trade/balance", get(get_balance))
        .route("/trade/positions", get(get_positions))
}
