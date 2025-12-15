//! EIP-712 typed data signing for Polymarket orders
//!
//! Polymarket uses EIP-712 for:
//! 1. L1 Authentication (deriving/creating API keys)
//! 2. Order signing

use alloy::primitives::{keccak256, Address, B256, U256};
use alloy::sol;
use alloy::sol_types::{eip712_domain, SolStruct};

use crate::types::{Order, Result, CTF_EXCHANGE_ADDRESS, NEG_RISK_ADAPTER_ADDRESS, POLYGON_CHAIN_ID};
use crate::wallet::TradingWallet;

// ============================================================================
// L1 Auth Constants
// ============================================================================

/// The fixed message for CLOB auth
const CLOB_AUTH_MESSAGE: &str = "This message attests that I control the given wallet";

/// EIP-712 type hash for ClobAuth
/// keccak256("ClobAuth(address address,string timestamp,uint256 nonce,string message)")
/// IMPORTANT: Field name must be "address" not "address_" to match Polymarket's expectation
fn clob_auth_type_hash() -> B256 {
    keccak256("ClobAuth(address address,string timestamp,uint256 nonce,string message)")
}

/// EIP-712 domain type hash (without verifyingContract)
/// keccak256("EIP712Domain(string name,string version,uint256 chainId)")
fn domain_type_hash() -> B256 {
    keccak256("EIP712Domain(string name,string version,uint256 chainId)")
}

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

// NOTE: ClobAuth is NOT defined in sol! macro because we need the field name
// to be "address" not "address_" (which sol! forces due to keyword conflict).
// We manually implement EIP-712 for ClobAuth below.

/// Compute the EIP-712 domain separator for CLOB Auth
/// Domain: { name: "ClobAuthDomain", version: "1", chainId: 137 }
fn compute_clob_auth_domain_separator() -> B256 {
    // EIP-712 domain separator = keccak256(abi.encode(
    //     DOMAIN_TYPE_HASH,
    //     keccak256("ClobAuthDomain"),
    //     keccak256("1"),
    //     chainId
    // ))
    let mut encoded = Vec::with_capacity(128);

    // Domain type hash
    encoded.extend_from_slice(domain_type_hash().as_slice());

    // keccak256("ClobAuthDomain")
    encoded.extend_from_slice(keccak256("ClobAuthDomain").as_slice());

    // keccak256("1")
    encoded.extend_from_slice(keccak256("1").as_slice());

    // chainId (137) as uint256
    encoded.extend_from_slice(U256::from(POLYGON_CHAIN_ID).to_be_bytes::<32>().as_slice());

    keccak256(&encoded)
}

/// Compute the EIP-712 struct hash for ClobAuth
/// This manually encodes with field name "address" (not "address_")
fn compute_clob_auth_struct_hash(
    address: Address,
    timestamp: &str,
    nonce: u64,
    message: &str,
) -> B256 {
    // Struct hash = keccak256(abi.encode(
    //     TYPE_HASH,
    //     address,           // address is encoded directly (padded to 32 bytes)
    //     keccak256(timestamp), // string is encoded as hash
    //     nonce,             // uint256
    //     keccak256(message) // string is encoded as hash
    // ))
    let mut encoded = Vec::with_capacity(160);

    // Type hash
    encoded.extend_from_slice(clob_auth_type_hash().as_slice());

    // Address (padded to 32 bytes - 12 zero bytes + 20 byte address)
    encoded.extend_from_slice(&[0u8; 12]);
    encoded.extend_from_slice(address.as_slice());

    // keccak256(timestamp)
    encoded.extend_from_slice(keccak256(timestamp).as_slice());

    // nonce as uint256
    encoded.extend_from_slice(U256::from(nonce).to_be_bytes::<32>().as_slice());

    // keccak256(message)
    encoded.extend_from_slice(keccak256(message).as_slice());

    keccak256(&encoded)
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

    /// Sign an L1 authentication message using EIP-712 typed data
    ///
    /// Used for creating or deriving API keys.
    /// This manually computes the EIP-712 hash to ensure the field name is "address"
    /// (not "address_" which the sol! macro would force).
    pub async fn sign_l1_auth(&self, timestamp: u64, nonce: u64) -> Result<String> {
        let address = self.address();
        let timestamp_str = timestamp.to_string();

        tracing::info!("L1 auth EIP-712 signing (manual implementation):");
        tracing::info!("  Address: {}", address);
        tracing::info!("  Timestamp: {}", timestamp_str);
        tracing::info!("  Nonce: {}", nonce);
        tracing::info!("  Message: {}", CLOB_AUTH_MESSAGE);

        // 1. Compute domain separator
        let domain_separator = compute_clob_auth_domain_separator();
        tracing::info!("  Domain separator: 0x{}", hex::encode(domain_separator));

        // 2. Compute struct hash with correct field name "address"
        let struct_hash = compute_clob_auth_struct_hash(
            address,
            &timestamp_str,
            nonce,
            CLOB_AUTH_MESSAGE,
        );
        tracing::info!("  Struct hash: 0x{}", hex::encode(struct_hash));

        // 3. Compute final EIP-712 hash: keccak256("\x19\x01" + domain_separator + struct_hash)
        let mut data = Vec::with_capacity(66);
        data.extend_from_slice(&[0x19, 0x01]);
        data.extend_from_slice(domain_separator.as_slice());
        data.extend_from_slice(struct_hash.as_slice());
        let signing_hash = keccak256(&data);

        tracing::info!("  Final EIP-712 hash: 0x{}", hex::encode(signing_hash));

        // 4. Sign the hash
        let signature = self.sign_hash(signing_hash.into()).await?;

        let sig_hex = format!("0x{}", hex::encode(signature.as_bytes()));
        tracing::info!("  Signature: {}", sig_hex);

        Ok(sig_hex)
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
    fn test_generate_salt() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        // Salts should be different
        assert_ne!(salt1, salt2);
    }

    #[test]
    fn test_clob_auth_type_hash() {
        // Verify the type hash is computed correctly
        // This is the hash of "ClobAuth(address address,string timestamp,uint256 nonce,string message)"
        let type_hash = clob_auth_type_hash();
        // Just verify it's not empty and is 32 bytes
        assert_eq!(type_hash.len(), 32);
        println!("ClobAuth type hash: 0x{}", hex::encode(type_hash));
    }

    #[test]
    fn test_domain_separator() {
        // Verify domain separator computation
        let domain_sep = compute_clob_auth_domain_separator();
        assert_eq!(domain_sep.len(), 32);
        println!("Domain separator: 0x{}", hex::encode(domain_sep));
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
