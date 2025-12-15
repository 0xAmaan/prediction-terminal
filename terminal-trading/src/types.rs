//! Shared types for Polymarket trading

use alloy::primitives::{Address, U256};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// ============================================================================
// Custom serialization for U256 fields (Polymarket requirement)
// ============================================================================

/// Serialize U256 as a decimal string (e.g., "1000000" not "0xf4240")
/// Used for: tokenId, makerAmount, takerAmount, expiration, nonce, feeRateBps
fn serialize_u256_as_decimal<S>(value: &U256, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

/// Deserialize U256 from a decimal string
fn deserialize_u256_from_decimal<'de, D>(deserializer: D) -> std::result::Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    U256::from_str_radix(&s, 10).map_err(serde::de::Error::custom)
}

/// Serialize salt as a plain u64 number (not a string)
/// Polymarket expects salt as an integer, not a string
fn serialize_salt_as_u64<S>(value: &U256, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Convert U256 to u64 - salt should fit in u64
    let salt_u64: u64 = value.try_into().map_err(|_| {
        serde::ser::Error::custom("Salt value too large for u64")
    })?;
    serializer.serialize_u64(salt_u64)
}

/// Deserialize salt from a u64 number
fn deserialize_salt_from_u64<'de, D>(deserializer: D) -> std::result::Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let n: u64 = Deserialize::deserialize(deserializer)?;
    Ok(U256::from(n))
}

/// Serialize Address as checksum format (e.g., "0xeFa7Cd2E9BFa38F04Af95df90da90B194e4ed191")
/// Polymarket expects checksum addresses, not lowercase
fn serialize_address_checksum<S>(value: &Address, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_checksum(None))
}

/// Deserialize Address from any format (checksum or lowercase)
fn deserialize_address<'de, D>(deserializer: D) -> std::result::Result<Address, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

// ============================================================================
// Contract Addresses (Polygon Mainnet)
// ============================================================================

/// Polymarket CTF Exchange contract address (for simple binary markets)
pub const CTF_EXCHANGE_ADDRESS: &str = "0x4bfb41d5b3570defd03c39a9a4d8de6bd8b8982e";

/// Polymarket Neg Risk CTF Exchange address (for multi-outcome markets - most markets use this)
pub const NEG_RISK_CTF_EXCHANGE_ADDRESS: &str = "0xC5d563A36AE78145C45a50134d48A1215220f80a";

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

    pub fn as_str(&self) -> &'static str {
        match self {
            Side::Buy => "BUY",
            Side::Sell => "SELL",
        }
    }
}

/// Serialize side as BUY/SELL string for API
fn serialize_side_as_string<S>(value: &u8, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let side_str = match value {
        0 => "BUY",
        1 => "SELL",
        _ => return Err(serde::ser::Error::custom("Invalid side value")),
    };
    serializer.serialize_str(side_str)
}

/// Deserialize side from BUY/SELL string
fn deserialize_side_from_string<'de, D>(deserializer: D) -> std::result::Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    match s.as_str() {
        "BUY" => Ok(0),
        "SELL" => Ok(1),
        _ => Err(serde::de::Error::custom(format!("Invalid side: {}", s))),
    }
}

/// Polymarket order structure for signing
/// Salt is serialized as u64 integer, other numeric fields as decimal strings
/// Addresses are serialized in checksum format (mixed case)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// Random salt for uniqueness - serialized as integer (not string)
    #[serde(serialize_with = "serialize_salt_as_u64", deserialize_with = "deserialize_salt_from_u64")]
    pub salt: U256,
    /// Maker address (our wallet) - checksum format
    #[serde(serialize_with = "serialize_address_checksum", deserialize_with = "deserialize_address")]
    pub maker: Address,
    /// Signer address (same as maker for EOA) - checksum format
    #[serde(serialize_with = "serialize_address_checksum", deserialize_with = "deserialize_address")]
    pub signer: Address,
    /// Taker address (zero for public orders) - checksum format
    #[serde(serialize_with = "serialize_address_checksum", deserialize_with = "deserialize_address")]
    pub taker: Address,
    /// Token ID (CLOB token ID for the outcome)
    #[serde(serialize_with = "serialize_u256_as_decimal", deserialize_with = "deserialize_u256_from_decimal")]
    pub token_id: U256,
    /// Amount maker is offering (in smallest units)
    #[serde(serialize_with = "serialize_u256_as_decimal", deserialize_with = "deserialize_u256_from_decimal")]
    pub maker_amount: U256,
    /// Amount maker wants to receive (in smallest units)
    #[serde(serialize_with = "serialize_u256_as_decimal", deserialize_with = "deserialize_u256_from_decimal")]
    pub taker_amount: U256,
    /// Expiration timestamp (0 for no expiry)
    #[serde(serialize_with = "serialize_u256_as_decimal", deserialize_with = "deserialize_u256_from_decimal")]
    pub expiration: U256,
    /// Nonce for order uniqueness
    #[serde(serialize_with = "serialize_u256_as_decimal", deserialize_with = "deserialize_u256_from_decimal")]
    pub nonce: U256,
    /// Fee rate in basis points
    #[serde(serialize_with = "serialize_u256_as_decimal", deserialize_with = "deserialize_u256_from_decimal")]
    pub fee_rate_bps: U256,
    /// Order side (0 = Buy, 1 = Sell) - serialized as "BUY" or "SELL" string for API
    #[serde(serialize_with = "serialize_side_as_string", deserialize_with = "deserialize_side_from_string")]
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

#[cfg(test)]
mod serialization_tests {
    use super::*;
    use alloy::primitives::{Address, U256};
    use std::str::FromStr;

    #[test]
    fn test_order_json_format() {
        let order = Order {
            salt: U256::from(12345u64),
            maker: Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap(),
            signer: Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266").unwrap(),
            taker: Address::ZERO,
            token_id: U256::from(123456789u64),
            maker_amount: U256::from(1000000u64),
            taker_amount: U256::from(500000u64),
            expiration: U256::ZERO,
            nonce: U256::ZERO,
            fee_rate_bps: U256::ZERO,
            side: 0,
            signature_type: 0,
        };

        let json = serde_json::to_string_pretty(&order).unwrap();
        println!("Order JSON:\n{}", json);
        
        // Check that the JSON is what Polymarket expects
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        
        // Polymarket expects string values for amounts, not numbers
        println!("\nmakerAmount type: {:?}", parsed["makerAmount"]);
        println!("tokenId type: {:?}", parsed["tokenId"]);
    }
}
