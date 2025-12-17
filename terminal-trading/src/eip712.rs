//! EIP-712 typed data signing for Polymarket orders
//!
//! Polymarket uses EIP-712 for:
//! 1. L1 Authentication (deriving/creating API keys)
//! 2. Order signing

use alloy::primitives::{Address, U256};
use alloy::sol;
use alloy::sol_types::{eip712_domain, SolStruct};

use crate::types::{Result, POLYGON_CHAIN_ID};
use crate::types::{CTF_EXCHANGE_ADDRESS, NEG_RISK_CTF_EXCHANGE_ADDRESS};
use crate::wallet::TradingWallet;

// ============================================================================
// L1 Auth Constants
// ============================================================================

/// The fixed message for CLOB auth
const CLOB_AUTH_MESSAGE: &str = "This message attests that I control the given wallet";

// ============================================================================
// EIP-712 Struct Definitions using sol! macro
// ============================================================================

// Define ClobAuth using sol! macro
// IMPORTANT: Alloy's sol! macro allows "address address;" syntax which produces
// the correct EIP-712 type hash "ClobAuth(address address,string timestamp,uint256 nonce,string message)"
sol! {
    struct ClobAuth {
        address address;
        string timestamp;
        uint256 nonce;
        string message;
    }
}

// Define Order struct for EIP-712 signing
// IMPORTANT: The struct MUST be named "Order" (not "PolymarketOrder") for correct type hash
sol! {
    #[derive(Debug)]
    struct Order {
        uint256 salt;
        address maker;
        address signer;
        address taker;
        uint256 tokenId;
        uint256 makerAmount;
        uint256 takerAmount;
        uint256 expiration;
        uint256 nonce;
        uint256 feeRateBps;
        uint8 side;
        uint8 signatureType;
    }
}

// ============================================================================
// EIP-712 Domains
// ============================================================================

/// Get the EIP-712 domain for ClobAuth (L1 authentication)
fn clob_auth_domain() -> alloy::sol_types::Eip712Domain {
    eip712_domain! {
        name: "ClobAuthDomain",
        version: "1",
        chain_id: POLYGON_CHAIN_ID,
    }
}

/// Get the EIP-712 domain for CTF Exchange (binary markets)
pub fn ctf_exchange_domain() -> alloy::sol_types::Eip712Domain {
    let exchange_address: Address = CTF_EXCHANGE_ADDRESS.parse().unwrap();

    eip712_domain! {
        name: "Polymarket CTF Exchange",
        version: "1",
        chain_id: POLYGON_CHAIN_ID,
        verifying_contract: exchange_address,
    }
}

/// Get the EIP-712 domain for Neg Risk CTF Exchange (multi-outcome markets)
pub fn neg_risk_ctf_exchange_domain() -> alloy::sol_types::Eip712Domain {
    let exchange_address: Address = NEG_RISK_CTF_EXCHANGE_ADDRESS.parse().unwrap();

    eip712_domain! {
        name: "Polymarket CTF Exchange",
        version: "1",
        chain_id: POLYGON_CHAIN_ID,
        verifying_contract: exchange_address,
    }
}


// ============================================================================
// Signing Functions
// ============================================================================

impl TradingWallet {
    /// Sign an order using EIP-712
    ///
    /// This produces the signature needed for submitting orders to the CLOB.
    pub async fn sign_order(&self, order: &crate::types::Order, is_neg_risk: bool) -> Result<String> {
        // Convert our types::Order to the EIP-712 Order struct
        let eip712_order = Order {
            salt: order.salt,
            maker: order.maker,
            signer: order.signer,
            taker: order.taker,
            tokenId: order.token_id,
            makerAmount: order.maker_amount,
            takerAmount: order.taker_amount,
            expiration: order.expiration,
            nonce: order.nonce,
            feeRateBps: order.fee_rate_bps,
            side: order.side,
            signatureType: order.signature_type,
        };

        // Get the appropriate domain
        let domain = if is_neg_risk {
            tracing::info!("Using Neg Risk CTF Exchange domain for multi-outcome market");
            neg_risk_ctf_exchange_domain()
        } else {
            tracing::info!("Using CTF Exchange domain for binary market");
            ctf_exchange_domain()
        };

        // Log key order details for debugging
        tracing::info!("========== EIP-712 ORDER SIGNING ==========");
        tracing::info!("  negRisk: {}", is_neg_risk);
        tracing::info!("  salt: {}", order.salt);
        tracing::info!("  maker: {}", order.maker);
        tracing::info!("  tokenId: {}", order.token_id);
        tracing::info!("  makerAmount: {}", order.maker_amount);
        tracing::info!("  takerAmount: {}", order.taker_amount);
        tracing::info!("  side: {} ({})", order.side, if order.side == 0 { "BUY" } else { "SELL" });
        tracing::info!("  signatureType: {}", order.signature_type);
        tracing::info!("============================================");

        // Calculate the EIP-712 signing hash
        let signing_hash = eip712_order.eip712_signing_hash(&domain);

        tracing::debug!("Order signing hash: 0x{}", hex::encode(signing_hash));

        // Sign the hash
        let signature = self.sign_hash(signing_hash.into()).await?;

        let sig_hex = format!("0x{}", hex::encode(signature.as_bytes()));
        tracing::debug!("Order signature: {}", sig_hex);

        // Return as hex string
        Ok(sig_hex)
    }

    /// Sign an L1 authentication message using EIP-712 typed data
    ///
    /// Used for creating or deriving API keys.
    /// Uses the sol! macro approach for proper EIP-712 type hash.
    pub async fn sign_l1_auth(&self, timestamp: u64, nonce: u64) -> Result<String> {
        let address = self.address();
        let timestamp_str = timestamp.to_string();

        tracing::debug!("L1 auth EIP-712 signing:");
        tracing::debug!("  Address: {}", address);
        tracing::debug!("  Timestamp: {}", timestamp_str);
        tracing::debug!("  Nonce: {}", nonce);
        tracing::debug!("  Message: {}", CLOB_AUTH_MESSAGE);

        // Create the ClobAuth struct for EIP-712 signing
        let clob_auth = ClobAuth {
            address,
            timestamp: timestamp_str,
            nonce: U256::from(nonce),
            message: CLOB_AUTH_MESSAGE.to_string(),
        };

        // Get the domain
        let domain = clob_auth_domain();

        // Calculate the EIP-712 signing hash using Alloy's built-in functionality
        let signing_hash = clob_auth.eip712_signing_hash(&domain);

        tracing::debug!("  EIP-712 signing hash: 0x{}", hex::encode(signing_hash));

        // Sign the hash
        let signature = self.sign_hash(signing_hash.into()).await?;

        let sig_hex = format!("0x{}", hex::encode(signature.as_bytes()));
        tracing::debug!("  Signature: {}", sig_hex);

        Ok(sig_hex)
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Generate a random salt for order uniqueness
/// Salt is generated as a u64 value (not full 256-bit) per Polymarket convention
pub fn generate_salt() -> U256 {
    use rand::Rng;
    let mut rng = rand::rng();

    // Generate a large random salt based on timestamp + random bits
    // This ensures uniqueness and avoids collisions
    let timestamp_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    // Use timestamp as base and add random bits
    // This is similar to how the official Python client generates salts
    let random_bits: u32 = rng.random();
    let salt = timestamp_ms.wrapping_mul(1000).wrapping_add(random_bits as u64);

    tracing::debug!("Generated order salt: {}", salt);
    U256::from(salt)
}

/// Generate a nonce (typically timestamp-based)
pub fn generate_nonce() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Get current timestamp in seconds
pub fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_salt() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        // Salts should be different
        assert_ne!(salt1, salt2);
    }

    #[test]
    fn test_domains() {
        // Just verify domains can be created without panicking
        let _clob_domain = clob_auth_domain();
        let _ctf_domain = ctf_exchange_domain();
        let _neg_risk_domain = neg_risk_ctf_exchange_domain();
    }

    #[tokio::test]
    async fn test_sign_l1_auth() {
        let test_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let wallet = TradingWallet::from_private_key(test_key).unwrap();

        let signature = wallet.sign_l1_auth(1700000000, 12345).await.unwrap();
        assert!(signature.starts_with("0x"));
        // 65 bytes = 130 hex chars + "0x" prefix
        assert_eq!(signature.len(), 132);
    }
}
