//! EIP-712 typed data signing for Polymarket orders
//!
//! Polymarket uses EIP-712 for:
//! 1. L1 Authentication (deriving/creating API keys)
//! 2. Order signing

use alloy::primitives::{Address, U256};
use alloy::sol;
use alloy::sol_types::{eip712_domain, SolStruct};

use crate::types::{Order, Result, CTF_EXCHANGE_ADDRESS, NEG_RISK_ADAPTER_ADDRESS, POLYGON_CHAIN_ID};
use crate::wallet::TradingWallet;

// ============================================================================
// EIP-712 Domain for Polymarket CTF Exchange
// ============================================================================

/// Get the EIP-712 domain for CTF Exchange (used for order signing)
pub fn ctf_exchange_domain() -> alloy::sol_types::Eip712Domain {
    let exchange_address: Address = CTF_EXCHANGE_ADDRESS.parse().unwrap();

    eip712_domain! {
        name: "Polymarket CTF Exchange",
        version: "1",
        chain_id: POLYGON_CHAIN_ID,
        verifying_contract: exchange_address,
    }
}

/// Get the EIP-712 domain for Neg Risk CTF Exchange
pub fn neg_risk_ctf_exchange_domain() -> alloy::sol_types::Eip712Domain {
    let adapter_address: Address = NEG_RISK_ADAPTER_ADDRESS.parse().unwrap();

    eip712_domain! {
        name: "Polymarket CTF Exchange",
        version: "1",
        chain_id: POLYGON_CHAIN_ID,
        verifying_contract: adapter_address,
    }
}

// ============================================================================
// Polymarket Order Type Definition (EIP-712)
// ============================================================================

// Define the Order struct for EIP-712 signing using alloy's sol! macro
sol! {
    #[derive(Debug)]
    struct PolymarketOrder {
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
// L1 Authentication Message
// ============================================================================

/// Message format for L1 authentication (API key creation/derivation)
pub fn build_l1_auth_message(timestamp: u64, nonce: u64) -> String {
    format!(
        "I am signing this message to generate my Polymarket API key.\n\ntimestamp: {}\nnonce: {}",
        timestamp, nonce
    )
}

// ============================================================================
// Signing Functions
// ============================================================================

impl TradingWallet {
    /// Sign an order using EIP-712
    ///
    /// This produces the signature needed for submitting orders to the CLOB.
    pub async fn sign_order(&self, order: &Order, is_neg_risk: bool) -> Result<String> {
        // Convert our Order to the EIP-712 struct
        let eip712_order = PolymarketOrder {
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
            neg_risk_ctf_exchange_domain()
        } else {
            ctf_exchange_domain()
        };

        // Calculate the EIP-712 signing hash
        let signing_hash = eip712_order.eip712_signing_hash(&domain);

        // Sign the hash
        let signature = self.sign_hash(signing_hash).await?;

        // Return as hex string
        Ok(format!("0x{}", hex::encode(signature.as_bytes())))
    }

    /// Sign an L1 authentication message
    ///
    /// Used for creating or deriving API keys.
    pub async fn sign_l1_auth(&self, timestamp: u64, nonce: u64) -> Result<String> {
        let message = build_l1_auth_message(timestamp, nonce);

        let signature = self.sign_message(message.as_bytes()).await?;

        Ok(format!("0x{}", hex::encode(signature.as_bytes())))
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Generate a random salt for order uniqueness
pub fn generate_salt() -> U256 {
    use rand::Rng;
    let mut rng = rand::rng();
    let bytes: [u8; 32] = rng.random();
    U256::from_be_bytes(bytes)
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
    fn test_l1_auth_message() {
        let message = build_l1_auth_message(1700000000, 12345);
        assert!(message.contains("1700000000"));
        assert!(message.contains("12345"));
        assert!(message.contains("Polymarket API key"));
    }

    #[test]
    fn test_generate_salt() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        // Salts should be different
        assert_ne!(salt1, salt2);
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
