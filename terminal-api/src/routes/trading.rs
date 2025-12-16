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

use terminal_trading::{
    approve_ctf_for_all_exchanges, approve_usdc_for_all_exchanges, check_ctf_approval,
    get_matic_balance, ClobClient, OrderBuilder, OrderType, Side,
};

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
    /// Whether this is a neg_risk market (multi-outcome). Default: false (binary market)
    #[serde(default)]
    pub neg_risk: bool,
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
    /// Whether CTF tokens are approved for selling (all required contracts)
    pub ctf_approved: bool,
    /// Whether CTF Exchange specifically is approved
    pub ctf_exchange_approved: bool,
    /// Whether Neg Risk CTF Exchange is approved
    pub neg_risk_ctf_approved: bool,
    /// Whether Neg Risk Adapter is approved (required for multi-outcome markets)
    pub neg_risk_adapter_approved: bool,
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
    pub title: String,
    pub neg_risk: bool,
}

/// Approval response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApproveResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matic_balance: Option<String>,
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

    // Ensure API credentials are available for trading
    {
        let mut state = trading_state.write().await;
        if let Some(client) = state.client_mut() {
            if !client.wallet().has_api_credentials() {
                info!("API credentials not set, attempting to derive...");
                if let Err(e) = client.ensure_api_key().await {
                    error!("Failed to derive API key: {}", e);
                    return (
                        StatusCode::SERVICE_UNAVAILABLE,
                        Json(SubmitOrderResponse {
                            success: false,
                            order_id: None,
                            error: Some(format!(
                                "Failed to authenticate with Polymarket: {}. Try restarting the backend.",
                                e
                            )),
                            transaction_hashes: vec![],
                        }),
                    );
                }
                info!("API credentials derived successfully");
            }
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
    // Use neg_risk from request (defaults to false for binary markets)
    let builder = OrderBuilder::new(&req.token_id, req.price, req.size, side)
        .with_neg_risk(req.neg_risk);
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
    match client.post_order(signed_order.clone(), order_type.clone()).await {
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
            let error_str = format!("{}", e);

            // Check if this is a 401 Unauthorized error - credentials may be stale
            if error_str.contains("401") || error_str.contains("Unauthorized") || error_str.contains("Invalid api key") {
                info!("Got 401 error, refreshing API credentials and retrying...");

                // Drop the read lock and get a write lock to refresh credentials
                drop(state);

                let mut state = trading_state.write().await;
                if let Some(client) = state.client_mut() {
                    // Clear stale credentials
                    client.wallet_mut().clear_api_credentials();

                    // Re-derive API key
                    if let Err(derive_err) = client.derive_api_key().await {
                        error!("Failed to refresh API key: {}", derive_err);
                        return (
                            StatusCode::UNAUTHORIZED,
                            Json(SubmitOrderResponse {
                                success: false,
                                order_id: None,
                                error: Some(format!("Authentication failed: {}. Original error: {}", derive_err, error_str)),
                                transaction_hashes: vec![],
                            }),
                        );
                    }

                    info!("API credentials refreshed, retrying order submission...");

                    // Retry the order submission
                    match client.post_order(signed_order, order_type).await {
                        Ok(response) => {
                            info!("Order submitted on retry: {:?}", response.order_id);
                            return (
                                StatusCode::OK,
                                Json(SubmitOrderResponse {
                                    success: response.success,
                                    order_id: response.order_id,
                                    error: response.error_msg,
                                    transaction_hashes: response.transaction_hashes,
                                }),
                            );
                        }
                        Err(retry_err) => {
                            error!("Order submission failed on retry: {}", retry_err);
                            return (
                                StatusCode::BAD_REQUEST,
                                Json(SubmitOrderResponse {
                                    success: false,
                                    order_id: None,
                                    error: Some(format!("{}", retry_err)),
                                    transaction_hashes: vec![],
                                }),
                            );
                        }
                    }
                }
            }

            // Check for "not enough balance / allowance" error - provide helpful guidance
            if error_str.contains("not enough balance") || error_str.contains("allowance") {
                // Check if this was a SELL order
                if req.side.to_lowercase() == "sell" {
                    error!("Sell order failed with balance/allowance error: {}", e);
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(SubmitOrderResponse {
                            success: false,
                            order_id: None,
                            error: Some(format!(
                                "Cannot sell: CTF tokens not approved for exchange. Call POST /api/trade/approve-ctf first, then retry. Original error: {}",
                                error_str
                            )),
                            transaction_hashes: vec![],
                        }),
                    );
                } else {
                    // BUY order - likely USDC balance/allowance issue
                    error!("Buy order failed with balance/allowance error: {}", e);
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(SubmitOrderResponse {
                            success: false,
                            order_id: None,
                            error: Some(format!(
                                "Cannot buy: Insufficient USDC balance or allowance. Check GET /api/trade/balance and call POST /api/trade/approve if needed. Original error: {}",
                                error_str
                            )),
                            transaction_hashes: vec![],
                        }),
                    );
                }
            }

            error!("Order submission failed: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(SubmitOrderResponse {
                    success: false,
                    order_id: None,
                    error: Some(error_str),
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
                    ctf_approved: false,
                    ctf_exchange_approved: false,
                    neg_risk_ctf_approved: false,
                    neg_risk_adapter_approved: false,
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
                    ctf_approved: false,
                    ctf_exchange_approved: false,
                    neg_risk_ctf_approved: false,
                    neg_risk_adapter_approved: false,
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
                    ctf_approved: false,
                    ctf_exchange_approved: false,
                    neg_risk_ctf_approved: false,
                    neg_risk_adapter_approved: false,
                }),
            );
        }
    };

    let address = client.address();

    // Query actual balance from Polygon
    let balance_result = client.get_balance().await;

    // Query CTF approval status (for selling)
    let ctf_status = check_ctf_approval(&address).await.unwrap_or_else(|e| {
        error!("Failed to check CTF approval: {}", e);
        terminal_trading::CtfApprovalStatus {
            ctf_exchange_approved: false,
            neg_risk_ctf_exchange_approved: false,
            neg_risk_adapter_approved: false,
            can_sell: false,
        }
    });

    match balance_result {
        Ok(balance) => (
            StatusCode::OK,
            Json(BalanceResponse {
                usdc_balance: balance.usdc_balance,
                usdc_allowance: balance.usdc_allowance,
                wallet_address: address,
                ctf_approved: ctf_status.can_sell,
                ctf_exchange_approved: ctf_status.ctf_exchange_approved,
                neg_risk_ctf_approved: ctf_status.neg_risk_ctf_exchange_approved,
                neg_risk_adapter_approved: ctf_status.neg_risk_adapter_approved,
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
                    ctf_approved: ctf_status.can_sell,
                    ctf_exchange_approved: ctf_status.ctf_exchange_approved,
                    neg_risk_ctf_approved: ctf_status.neg_risk_ctf_exchange_approved,
                    neg_risk_adapter_approved: ctf_status.neg_risk_adapter_approved,
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
                    title: p.title,
                    neg_risk: p.neg_risk,
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

/// Approve USDC spending for CTF Exchange
async fn approve_usdc(State(state): State<AppState>) -> impl IntoResponse {
    info!("Approving USDC for CTF Exchange");

    let trading_state = match state.trading_state.as_ref() {
        Some(ts) => ts,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApproveResponse {
                    success: false,
                    transaction_hash: None,
                    error: Some("Trading not enabled".to_string()),
                    matic_balance: None,
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
                Json(ApproveResponse {
                    success: false,
                    transaction_hash: None,
                    error: Some(format!("Failed to initialize: {}", e)),
                    matic_balance: None,
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
                Json(ApproveResponse {
                    success: false,
                    transaction_hash: None,
                    error: Some("Trading client not available".to_string()),
                    matic_balance: None,
                }),
            );
        }
    };

    // Check MATIC balance for gas first
    let address = client.address();
    let matic_balance = match get_matic_balance(&address).await {
        Ok(b) => Some(b),
        Err(e) => {
            error!("Failed to get MATIC balance: {}", e);
            None
        }
    };

    // Check if we have enough MATIC for gas (need at least ~0.01 MATIC)
    if let Some(ref balance) = matic_balance {
        let balance_f64: f64 = balance.parse().unwrap_or(0.0);
        if balance_f64 < 0.001 {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApproveResponse {
                    success: false,
                    transaction_hash: None,
                    error: Some(format!(
                        "Insufficient MATIC for gas. Balance: {} MATIC. Need at least 0.001 MATIC.",
                        balance
                    )),
                    matic_balance,
                }),
            );
        }
    }

    // Execute approvals for ALL required contracts (CTF Exchange, Neg Risk CTF, Neg Risk Adapter)
    match approve_usdc_for_all_exchanges(client.wallet()).await {
        Ok(approvals) => {
            // Collect all transaction hashes
            let tx_hashes: Vec<_> = approvals
                .iter()
                .filter_map(|a| a.transaction_hash.clone())
                .collect();
            let all_success = approvals.iter().all(|a| a.success);
            let errors: Vec<_> = approvals
                .iter()
                .filter_map(|a| a.error.clone())
                .collect();

            info!("USDC approvals completed: {} successes, {} transactions",
                  approvals.iter().filter(|a| a.success).count(),
                  tx_hashes.len());

            (
                StatusCode::OK,
                Json(ApproveResponse {
                    success: all_success || !tx_hashes.is_empty(),
                    transaction_hash: tx_hashes.first().cloned(),
                    error: if errors.is_empty() { None } else { Some(errors.join("; ")) },
                    matic_balance,
                }),
            )
        }
        Err(e) => {
            error!("USDC approval failed: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApproveResponse {
                    success: false,
                    transaction_hash: None,
                    error: Some(format!("{}", e)),
                    matic_balance,
                }),
            )
        }
    }
}

/// Approve CTF (outcome tokens) for selling
///
/// This approves the CTF Exchange contracts to transfer your outcome tokens.
/// Required for selling positions.
async fn approve_ctf(State(state): State<AppState>) -> impl IntoResponse {
    info!("Approving CTF tokens for exchanges (for selling)");

    let trading_state = match state.trading_state.as_ref() {
        Some(ts) => ts,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApproveResponse {
                    success: false,
                    transaction_hash: None,
                    error: Some("Trading not enabled".to_string()),
                    matic_balance: None,
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
                Json(ApproveResponse {
                    success: false,
                    transaction_hash: None,
                    error: Some(format!("Failed to initialize: {}", e)),
                    matic_balance: None,
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
                Json(ApproveResponse {
                    success: false,
                    transaction_hash: None,
                    error: Some("Trading client not available".to_string()),
                    matic_balance: None,
                }),
            );
        }
    };

    // Check MATIC balance for gas first
    let address = client.address();
    let matic_balance = match get_matic_balance(&address).await {
        Ok(b) => Some(b),
        Err(e) => {
            error!("Failed to get MATIC balance: {}", e);
            None
        }
    };

    // Check if we have enough MATIC for gas
    if let Some(ref balance) = matic_balance {
        let balance_f64: f64 = balance.parse().unwrap_or(0.0);
        if balance_f64 < 0.001 {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApproveResponse {
                    success: false,
                    transaction_hash: None,
                    error: Some(format!(
                        "Insufficient MATIC for gas. Balance: {} MATIC. Need at least 0.001 MATIC.",
                        balance
                    )),
                    matic_balance,
                }),
            );
        }
    }

    // Execute CTF approvals for exchange contracts
    match approve_ctf_for_all_exchanges(client.wallet()).await {
        Ok(approvals) => {
            let tx_hashes: Vec<_> = approvals
                .iter()
                .filter_map(|a| a.transaction_hash.clone())
                .collect();
            let all_success = approvals.iter().all(|a| a.success);
            let errors: Vec<_> = approvals
                .iter()
                .filter_map(|a| a.error.clone())
                .collect();

            info!("CTF approvals completed: {} successes, {} transactions",
                  approvals.iter().filter(|a| a.success).count(),
                  tx_hashes.len());

            (
                StatusCode::OK,
                Json(ApproveResponse {
                    success: all_success || !tx_hashes.is_empty(),
                    transaction_hash: tx_hashes.first().cloned(),
                    error: if errors.is_empty() { None } else { Some(errors.join("; ")) },
                    matic_balance,
                }),
            )
        }
        Err(e) => {
            error!("CTF approval failed: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApproveResponse {
                    success: false,
                    transaction_hash: None,
                    error: Some(format!("{}", e)),
                    matic_balance,
                }),
            )
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
        .route("/trade/approve", post(approve_usdc))
        .route("/trade/approve-ctf", post(approve_ctf))
}
