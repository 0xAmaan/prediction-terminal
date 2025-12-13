//! Shared types for Polymarket trading

use alloy::primitives::{Address, U256};
use serde::{Deserialize, Serialize};

// ============================================================================
// Contract Addresses (Polygon Mainnet)
// ============================================================================

/// Polymarket CTF Exchange contract address (Neg Risk CTF Exchange)
pub const CTF_EXCHANGE_ADDRESS: &str = "0xC5d563A36AE78145C45a50134d48A1215220f80a";

/// Polymarket Neg Risk Adapter address
pub const NEG_RISK_ADAPTER_ADDRESS: &str = "0xd91E80cF2E7be2e162c6513ceD06f1dD0dA35296";

/// USDC.e on Polygon
pub const USDC_ADDRESS: &str = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";

/// Conditional Token Framework on Polygon
pub const CTF_ADDRESS: &str = "0x4D97DCd97eC945f40cF65F87097ACe5EA0476045";

/// Polygon Chain ID
pub const POLYGON_CHAIN_ID: u64 = 137;

// ============================================================================
// API Credentials
// ============================================================================

/// API credentials for L2 (HMAC) authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCredentials {
    pub api_key: String,
    pub secret: String,
    pub passphrase: String,
}

// ============================================================================
// Order Types
// ============================================================================

/// Signature type for orders
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SignatureType {
    /// Direct EOA signature (what we use for backend-managed wallet)
    Eoa = 0,
    /// Polymarket proxy wallet (Magic/email users)
    PolyProxy = 1,
    /// Gnosis Safe (MetaMask users on Polymarket)
    PolyGnosisSafe = 2,
}

/// Order side
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn as_u8(&self) -> u8 {
        match self {
            Side::Buy => 0,
            Side::Sell => 1,
        }
    }
}

/// Polymarket order structure for signing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// Random salt for uniqueness
    pub salt: U256,
    /// Maker address (our wallet)
    pub maker: Address,
    /// Signer address (same as maker for EOA)
    pub signer: Address,
    /// Taker address (zero for public orders)
    pub taker: Address,
    /// Token ID (CLOB token ID for the outcome)
    pub token_id: U256,
    /// Amount maker is offering (in smallest units)
    pub maker_amount: U256,
    /// Amount maker wants to receive (in smallest units)
    pub taker_amount: U256,
    /// Expiration timestamp (0 for no expiry)
    pub expiration: U256,
    /// Nonce for order uniqueness
    pub nonce: U256,
    /// Fee rate in basis points
    pub fee_rate_bps: U256,
    /// Order side (0 = Buy, 1 = Sell)
    pub side: u8,
    /// Signature type
    pub signature_type: u8,
}

/// Signed order ready for submission
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignedOrder {
    #[serde(flatten)]
    pub order: Order,
    /// EIP-712 signature
    pub signature: String,
}

// ============================================================================
// API Request/Response Types
// ============================================================================

/// Request to create a new API key (L1 auth)
#[derive(Debug, Serialize)]
pub struct CreateApiKeyRequest {
    pub nonce: u64,
}

/// Response from API key creation
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyResponse {
    pub api_key: String,
    pub secret: String,
    pub passphrase: String,
}

/// Request to derive existing API key (L1 auth)
#[derive(Debug, Serialize)]
pub struct DeriveApiKeyRequest {
    pub nonce: u64,
}

/// Order submission request
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostOrderRequest {
    pub order: SignedOrder,
    pub owner: String,
    pub order_type: String,
}

/// Response from order submission
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    pub success: bool,
    #[serde(default)]
    pub error_msg: Option<String>,
    #[serde(default)]
    pub order_id: Option<String>,
    #[serde(default)]
    pub transaction_hashes: Vec<String>,
    #[serde(default)]
    pub status: Option<String>,
}

/// Open order from API
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenOrder {
    pub id: String,
    pub market: String,
    pub asset_id: String,
    pub side: String,
    pub original_size: String,
    pub size_matched: String,
    pub price: String,
    pub status: String,
    pub created_at: String,
    #[serde(default)]
    pub expiration: Option<String>,
    #[serde(default)]
    pub order_type: Option<String>,
}

/// Trade from API
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserTrade {
    pub id: String,
    pub market: String,
    pub asset_id: String,
    pub side: String,
    pub size: String,
    pub price: String,
    pub status: String,
    pub created_at: String,
    #[serde(default)]
    pub match_time: Option<String>,
    #[serde(default)]
    pub transaction_hash: Option<String>,
}

/// Balance info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub usdc_balance: String,
    pub usdc_allowance: String,
}

/// Position info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub market_id: String,
    pub token_id: String,
    pub outcome: String,
    pub shares: String,
    pub avg_price: String,
    pub current_price: String,
    pub pnl: String,
}

// ============================================================================
// Error Types
// ============================================================================

/// Trading errors
#[derive(Debug, thiserror::Error)]
pub enum TradingError {
    #[error("Wallet error: {0}")]
    Wallet(String),

    #[error("Signing error: {0}")]
    Signing(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Missing credentials: {0}")]
    MissingCredentials(String),

    #[error("Invalid order: {0}")]
    InvalidOrder(String),

    #[error("Order rejected: {0}")]
    OrderRejected(String),
}

pub type Result<T> = std::result::Result<T, TradingError>;
