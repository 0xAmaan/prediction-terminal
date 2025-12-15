//! Trading wallet management - generation, loading, and signing

use alloy::primitives::{Address, B256};
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::Signer;
use std::str::FromStr;
use tracing::{debug, info};

use crate::types::{ApiCredentials, Result, TradingError};

/// Trading wallet for Polymarket
#[derive(Clone)]
pub struct TradingWallet {
    signer: PrivateKeySigner,
    address: Address,
    api_credentials: Option<ApiCredentials>,
}

impl TradingWallet {
    /// Create a new wallet from a private key hex string
    pub fn from_private_key(private_key: &str) -> Result<Self> {
        let key = private_key.strip_prefix("0x").unwrap_or(private_key);

        let key_bytes = B256::from_str(key).map_err(|e| {
            TradingError::Wallet(format!("Invalid private key format: {}", e))
        })?;

        let signer = PrivateKeySigner::from_bytes(&key_bytes)
            .map_err(|e| TradingError::Wallet(format!("Failed to create signer: {}", e)))?;

        let address = signer.address();

        info!("Loaded trading wallet: {}", address);

        Ok(Self {
            signer,
            address,
            api_credentials: None,
        })
    }

    /// Load wallet from environment variable TRADING_PRIVATE_KEY
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let private_key = std::env::var("TRADING_PRIVATE_KEY").map_err(|_| {
            TradingError::MissingCredentials(
                "TRADING_PRIVATE_KEY environment variable not set".to_string(),
            )
        })?;

        Self::from_private_key(&private_key)
    }

    /// Generate a new random wallet
    pub fn generate() -> Self {
        let signer = PrivateKeySigner::random();
        let address = signer.address();

        info!("Generated new trading wallet: {}", address);
        info!(
            "Private key (save this!): 0x{}",
            hex::encode(signer.credential().to_bytes())
        );

        Self {
            signer,
            address,
            api_credentials: None,
        }
    }

    /// Get the wallet address
    pub fn address(&self) -> Address {
        self.address
    }

    /// Get the wallet address as a checksummed string
    pub fn address_string(&self) -> String {
        self.address.to_checksum(None)
    }

    /// Get the underlying signer for EIP-712 signing
    pub fn signer(&self) -> &PrivateKeySigner {
        &self.signer
    }

    /// Sign a message hash directly
    pub async fn sign_hash(&self, hash: B256) -> Result<alloy::signers::Signature> {
        self.signer
            .sign_hash(&hash)
            .await
            .map_err(|e| TradingError::Signing(format!("Failed to sign hash: {}", e)))
    }

    /// Sign arbitrary message bytes (for L1 auth)
    pub async fn sign_message(&self, message: &[u8]) -> Result<alloy::signers::Signature> {
        self.signer
            .sign_message(message)
            .await
            .map_err(|e| TradingError::Signing(format!("Failed to sign message: {}", e)))
    }

    /// Set API credentials after derivation
    pub fn set_api_credentials(&mut self, credentials: ApiCredentials) {
        debug!("Setting API credentials for wallet {}", self.address);
        self.api_credentials = Some(credentials);
    }

    /// Get API credentials if available
    pub fn api_credentials(&self) -> Option<&ApiCredentials> {
        self.api_credentials.as_ref()
    }

    /// Check if API credentials are set
    pub fn has_api_credentials(&self) -> bool {
        self.api_credentials.is_some()
    }

    /// Clear API credentials (for re-authentication)
    pub fn clear_api_credentials(&mut self) {
        debug!("Clearing API credentials for wallet {}", self.address);
        self.api_credentials = None;
    }

    /// Get private key as hex string (be careful with this!)
    pub fn private_key_hex(&self) -> String {
        format!("0x{}", hex::encode(self.signer.credential().to_bytes()))
    }
}

impl std::fmt::Debug for TradingWallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TradingWallet")
            .field("address", &self.address)
            .field("has_credentials", &self.api_credentials.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_generation() {
        let wallet = TradingWallet::generate();
        assert!(!wallet.address_string().is_empty());
        assert!(wallet.address_string().starts_with("0x"));
    }

    #[test]
    fn test_wallet_from_private_key() {
        // Known test private key (DO NOT use in production!)
        let test_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let wallet = TradingWallet::from_private_key(test_key).unwrap();

        // This should be the address derived from the test key
        assert_eq!(
            wallet.address_string().to_lowercase(),
            "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
        );
    }

    #[tokio::test]
    async fn test_sign_message() {
        let test_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let wallet = TradingWallet::from_private_key(test_key).unwrap();

        let message = b"test message";
        let signature = wallet.sign_message(message).await.unwrap();

        // Signature should be 65 bytes (r: 32, s: 32, v: 1)
        let sig_bytes = signature.as_bytes();
        assert_eq!(sig_bytes.len(), 65);
    }
}
