//! Order creation and signing for Polymarket CLOB

use alloy::primitives::{Address, U256};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::eip712::generate_salt;
use crate::types::{Order, Result, Side, SignatureType, SignedOrder, TradingError};
use crate::wallet::TradingWallet;

// ============================================================================
// Order Types
// ============================================================================

/// Order type for submission
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderType {
    /// Good-Til-Cancelled - rests in book until filled or cancelled
    Gtc,
    /// Good-Til-Date - expires at specified timestamp
    Gtd,
    /// Fill-Or-Kill - must fill entirely or cancel immediately
    Fok,
    /// Fill-And-Kill - fill what you can, cancel the rest
    Fak,
}

impl OrderType {
    pub fn as_str(&self) -> &str {
        match self {
            OrderType::Gtc => "GTC",
            OrderType::Gtd => "GTD",
            OrderType::Fok => "FOK",
            OrderType::Fak => "FAK",
        }
    }
}

/// Order side (re-export for convenience)
pub use crate::types::Side as OrderSide;

// ============================================================================
// Order Builder
// ============================================================================

/// Builder for creating Polymarket orders
#[derive(Debug, Clone)]
pub struct OrderBuilder {
    /// Token ID (CLOB token ID for the outcome)
    token_id: String,
    /// Price (0.01 to 0.99)
    price: f64,
    /// Size (number of shares)
    size: f64,
    /// Side (Buy or Sell)
    side: Side,
    /// Expiration timestamp (0 for no expiry)
    expiration: u64,
    /// Fee rate in basis points (default 0)
    fee_rate_bps: u64,
    /// Whether this is a neg risk market
    is_neg_risk: bool,
}

impl OrderBuilder {
    /// Create a new order builder
    pub fn new(token_id: impl Into<String>, price: f64, size: f64, side: Side) -> Self {
        Self {
            token_id: token_id.into(),
            price,
            size,
            side,
            expiration: 0, // No expiry by default
            fee_rate_bps: 0,
            is_neg_risk: false, // Default to false (binary market) - pass true for multi-outcome markets
        }
    }

    /// Set expiration timestamp
    pub fn with_expiration(mut self, expiration: u64) -> Self {
        self.expiration = expiration;
        self
    }

    /// Set fee rate in basis points
    pub fn with_fee_rate(mut self, fee_rate_bps: u64) -> Self {
        self.fee_rate_bps = fee_rate_bps;
        self
    }

    /// Set whether this is a neg risk market
    pub fn with_neg_risk(mut self, is_neg_risk: bool) -> Self {
        self.is_neg_risk = is_neg_risk;
        self
    }

    /// Validate order parameters
    fn validate(&self) -> Result<()> {
        if self.price < 0.01 || self.price > 0.99 {
            return Err(TradingError::InvalidOrder(format!(
                "Price must be between 0.01 and 0.99, got {}",
                self.price
            )));
        }

        if self.size <= 0.0 {
            return Err(TradingError::InvalidOrder(format!(
                "Size must be positive, got {}",
                self.size
            )));
        }

        // Note: Polymarket has a minimum order size (typically 5 shares)
        // but it may vary by market, so we let the API validate this
        // and return a clear error message if rejected

        if self.token_id.is_empty() {
            return Err(TradingError::InvalidOrder(
                "Token ID cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    /// Build the order struct (unsigned)
    pub fn build(&self, wallet: &TradingWallet) -> Result<Order> {
        self.validate()?;

        let maker = wallet.address();
        let signer = wallet.address();
        let taker = Address::ZERO; // Open order

        // Parse token ID as U256
        let token_id = U256::from_str(&self.token_id)
            .map_err(|e| TradingError::InvalidOrder(format!("Invalid token ID: {}", e)))?;

        // Calculate amounts based on side and price
        // For Polymarket:
        // - makerAmount is what we're giving
        // - takerAmount is what we're receiving
        //
        // For BUY side (buying YES tokens with USDC):
        //   makerAmount = size * price (USDC we're paying)
        //   takerAmount = size (YES tokens we're receiving)
        //
        // For SELL side (selling YES tokens for USDC):
        //   makerAmount = size (YES tokens we're giving)
        //   takerAmount = size * price (USDC we're receiving)
        //
        // Polymarket uses 6 decimals for USDC (10^6 = 1 USDC)
        // And "1 share" = 10^6 units

        let scale = 1_000_000u64; // 10^6

        // Polymarket precision requirements:
        // - Token amounts: max 2 decimal places (e.g., 45.00 shares)
        // - USDC amounts: max 5 decimal places (e.g., 2.25000 USDC)
        // Round to these precisions to avoid "invalid amounts" errors

        let (maker_amount, taker_amount) = match self.side {
            Side::Buy => {
                // Buying: we give USDC (5 decimals), receive tokens (2 decimals)
                let usdc_raw = self.size * self.price;
                let usdc_rounded = (usdc_raw * 100000.0).round() / 100000.0;
                let usdc_amount = (usdc_rounded * scale as f64).round() as u64;

                let token_rounded = (self.size * 100.0).round() / 100.0;
                let token_amount = (token_rounded * scale as f64).round() as u64;

                (U256::from(usdc_amount), U256::from(token_amount))
            }
            Side::Sell => {
                // Selling: we give tokens (2 decimals), receive USDC (5 decimals)
                let token_rounded = (self.size * 100.0).round() / 100.0;
                let token_amount = (token_rounded * scale as f64).round() as u64;

                let usdc_raw = self.size * self.price;
                let usdc_rounded = (usdc_raw * 100000.0).round() / 100000.0;
                let usdc_amount = (usdc_rounded * scale as f64).round() as u64;

                (U256::from(token_amount), U256::from(usdc_amount))
            }
        };

        let salt = generate_salt();
        let nonce = U256::from(0); // Nonce is typically 0 for new orders

        Ok(Order {
            salt,
            maker,
            signer,
            taker,
            token_id,
            maker_amount,
            taker_amount,
            expiration: U256::from(self.expiration),
            nonce,
            fee_rate_bps: U256::from(self.fee_rate_bps),
            side: self.side.as_u8(),
            signature_type: SignatureType::Eoa as u8,
        })
    }

    /// Build and sign the order
    pub async fn build_and_sign(&self, wallet: &TradingWallet) -> Result<SignedOrder> {
        let order = self.build(wallet)?;
        let signature = wallet.sign_order(&order, self.is_neg_risk).await?;

        Ok(SignedOrder { order, signature })
    }

    /// Check if this is a neg risk market
    pub fn is_neg_risk(&self) -> bool {
        self.is_neg_risk
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Calculate the cost of an order in USDC
pub fn calculate_order_cost(price: f64, size: f64, side: Side) -> f64 {
    match side {
        Side::Buy => price * size,
        Side::Sell => 0.0, // Selling doesn't cost USDC upfront
    }
}

/// Calculate potential profit from an order
pub fn calculate_potential_profit(price: f64, size: f64, side: Side) -> f64 {
    match side {
        Side::Buy => (1.0 - price) * size, // If YES wins, we get $1 per share
        Side::Sell => price * size,         // We receive price * size immediately
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_builder_validation() {
        let wallet = TradingWallet::generate();

        // Valid order
        let builder = OrderBuilder::new("123456", 0.50, 100.0, Side::Buy);
        assert!(builder.build(&wallet).is_ok());

        // Invalid price (too low)
        let builder = OrderBuilder::new("123456", 0.001, 100.0, Side::Buy);
        assert!(builder.build(&wallet).is_err());

        // Invalid price (too high)
        let builder = OrderBuilder::new("123456", 0.999, 100.0, Side::Buy);
        assert!(builder.build(&wallet).is_err());

        // Invalid size
        let builder = OrderBuilder::new("123456", 0.50, -10.0, Side::Buy);
        assert!(builder.build(&wallet).is_err());

        // Empty token ID
        let builder = OrderBuilder::new("", 0.50, 100.0, Side::Buy);
        assert!(builder.build(&wallet).is_err());
    }

    #[test]
    fn test_order_amounts() {
        let wallet = TradingWallet::generate();

        // Buy 100 shares at $0.50
        let builder = OrderBuilder::new("123456", 0.50, 100.0, Side::Buy);
        let order = builder.build(&wallet).unwrap();

        // maker_amount should be 50 USDC (100 * 0.50) = 50_000_000 units
        // taker_amount should be 100 shares = 100_000_000 units
        assert_eq!(order.maker_amount, U256::from(50_000_000u64));
        assert_eq!(order.taker_amount, U256::from(100_000_000u64));
    }

    #[tokio::test]
    async fn test_build_and_sign() {
        let test_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let wallet = TradingWallet::from_private_key(test_key).unwrap();

        let builder = OrderBuilder::new(
            "71321045679252212594626385532706912750332728571942532289631379312455583992563",
            0.50,
            10.0,
            Side::Buy,
        );

        let signed = builder.build_and_sign(&wallet).await.unwrap();
        assert!(signed.signature.starts_with("0x"));
        assert_eq!(signed.signature.len(), 132); // 65 bytes = 130 hex + "0x"
    }
}
