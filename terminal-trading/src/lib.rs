//! Terminal Trading - Polymarket Trade Execution
//!
//! This crate provides:
//! - Wallet management (generation, loading from env, EIP-712 signing)
//! - Polymarket CLOB API client with authentication
//! - Order creation, signing, and submission
//! - Position and balance tracking

pub mod balance;
pub mod clob_client;
pub mod eip712;
pub mod order;
pub mod positions;
pub mod types;
pub mod wallet;

pub use balance::{
    approve_usdc_for_all_exchanges, approve_usdc_for_ctf_exchange, get_matic_balance,
    get_usdc_allowance, get_usdc_allowance_for, get_usdc_balance, ApprovalResponse,
};
pub use clob_client::ClobClient;
pub use order::{OrderBuilder, OrderSide, OrderType};
pub use positions::calculate_positions;
pub use types::*;
pub use wallet::TradingWallet;
