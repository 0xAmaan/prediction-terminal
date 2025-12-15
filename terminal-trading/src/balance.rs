//! Balance queries and USDC approval via Polygon JSON-RPC
//!
//! Query USDC.e balance and allowance for trading wallet using
//! direct eth_call to the ERC20 contract. Also provides USDC approval
//! functionality for the CTF Exchange.

use crate::types::{Result, TradingError, CTF_EXCHANGE_ADDRESS, NEG_RISK_ADAPTER_ADDRESS, NEG_RISK_CTF_EXCHANGE_ADDRESS, USDC_ADDRESS};
use crate::wallet::TradingWallet;
use alloy::network::EthereumWallet;
use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::{debug, info};

/// Polygon RPC endpoint
const POLYGON_RPC_URL: &str = "https://polygon-rpc.com";

/// ERC20 function selectors
const BALANCE_OF_SELECTOR: &str = "70a08231"; // balanceOf(address)
const ALLOWANCE_SELECTOR: &str = "dd62ed3e"; // allowance(address,address)
const APPROVE_SELECTOR: &str = "095ea7b3"; // approve(address,uint256)

/// USDC has 6 decimals
const USDC_DECIMALS: u32 = 6;

/// JSON-RPC request
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    method: &'static str,
    params: Vec<serde_json::Value>,
    id: u64,
}

/// JSON-RPC response
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    result: Option<String>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    message: String,
}

/// Query USDC.e balance for an address
///
/// Returns balance as a human-readable string with proper decimal places
pub async fn get_usdc_balance(address: &str) -> Result<String> {
    let padded_address = pad_address(address)?;
    let data = format!("0x{}{}", BALANCE_OF_SELECTOR, padded_address);

    let result = eth_call(USDC_ADDRESS, &data).await?;
    let balance = parse_uint256(&result)?;

    Ok(format_usdc(balance))
}

/// Query USDC.e allowance for CTF Exchange
///
/// Returns allowance as a human-readable string with proper decimal places
pub async fn get_usdc_allowance(owner: &str) -> Result<String> {
    get_usdc_allowance_for(owner, CTF_EXCHANGE_ADDRESS).await
}

/// Query USDC.e allowance for a specific spender
pub async fn get_usdc_allowance_for(owner: &str, spender: &str) -> Result<String> {
    let padded_owner = pad_address(owner)?;
    let padded_spender = pad_address(spender)?;
    let data = format!("0x{}{}{}", ALLOWANCE_SELECTOR, padded_owner, padded_spender);

    let result = eth_call(USDC_ADDRESS, &data).await?;
    let allowance = parse_uint256(&result)?;

    Ok(format_usdc(allowance))
}

/// Make an eth_call to the given contract
async fn eth_call(to: &str, data: &str) -> Result<String> {
    let client = reqwest::Client::new();

    let request = JsonRpcRequest {
        jsonrpc: "2.0",
        method: "eth_call",
        params: vec![
            serde_json::json!({
                "to": to,
                "data": data
            }),
            serde_json::json!("latest"),
        ],
        id: 1,
    };

    debug!("Making eth_call to {}: {}", to, data);

    let response = client
        .post(POLYGON_RPC_URL)
        .json(&request)
        .send()
        .await
        .map_err(|e| TradingError::Api(format!("RPC request failed: {}", e)))?;

    let rpc_response: JsonRpcResponse = response
        .json()
        .await
        .map_err(|e| TradingError::Api(format!("Failed to parse RPC response: {}", e)))?;

    if let Some(error) = rpc_response.error {
        return Err(TradingError::Api(format!("RPC error: {}", error.message)));
    }

    rpc_response
        .result
        .ok_or_else(|| TradingError::Api("No result in RPC response".to_string()))
}

/// Pad an address to 32 bytes (64 hex chars)
fn pad_address(address: &str) -> Result<String> {
    // Remove 0x prefix if present
    let address = address.strip_prefix("0x").unwrap_or(address);

    if address.len() != 40 {
        return Err(TradingError::Api(format!(
            "Invalid address length: {}",
            address.len()
        )));
    }

    // Pad to 32 bytes (64 hex chars) with leading zeros
    Ok(format!("{:0>64}", address))
}

/// Parse a hex string as a u128 (sufficient for USDC amounts)
fn parse_uint256(hex: &str) -> Result<u128> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);

    // Handle empty or zero result
    if hex.is_empty() || hex.chars().all(|c| c == '0') {
        return Ok(0);
    }

    // Take last 32 chars (128 bits) - USDC balances won't exceed this
    let hex = if hex.len() > 32 {
        &hex[hex.len() - 32..]
    } else {
        hex
    };

    u128::from_str_radix(hex, 16)
        .map_err(|e| TradingError::Api(format!("Failed to parse uint256: {}", e)))
}

/// Format USDC amount with proper decimals
fn format_usdc(raw_amount: u128) -> String {
    let divisor = 10u128.pow(USDC_DECIMALS);
    let whole = raw_amount / divisor;
    let fraction = raw_amount % divisor;

    if fraction == 0 {
        format!("{}.00", whole)
    } else {
        format!("{}.{:0>6}", whole, fraction)
    }
}

// ============================================================================
// USDC Approval Functions
// ============================================================================

/// Approval response containing transaction hash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalResponse {
    pub success: bool,
    pub transaction_hash: Option<String>,
    pub error: Option<String>,
}

/// Approve USDC spending for the CTF Exchange contract
///
/// This sends an on-chain transaction to approve USDC spending
/// for the Polymarket CTF Exchange. Requires MATIC for gas.
/// Note: USDC.e doesn't allow max uint256 approval, so we use a large amount (1B USDC)
pub async fn approve_usdc_for_ctf_exchange(wallet: &TradingWallet) -> Result<ApprovalResponse> {
    let spender = Address::from_str(CTF_EXCHANGE_ADDRESS)
        .map_err(|e| TradingError::Api(format!("Invalid CTF Exchange address: {}", e)))?;

    // 1 billion USDC (1e9 * 1e6 = 1e15) - USDC.e doesn't allow max uint256
    let amount = U256::from(1_000_000_000_000_000u64);

    approve_usdc(wallet, spender, amount).await
}

/// Approve USDC spending for ALL required Polymarket contracts
///
/// This approves USDC for:
/// 1. CTF Exchange (binary markets)
/// 2. Neg Risk CTF Exchange (multi-outcome markets)
/// 3. Neg Risk Adapter (required for multi-outcome trading)
pub async fn approve_usdc_for_all_exchanges(wallet: &TradingWallet) -> Result<Vec<ApprovalResponse>> {
    let amount = U256::from(1_000_000_000_000_000u64); // 1 billion USDC
    let mut results = Vec::new();

    let contracts = [
        (CTF_EXCHANGE_ADDRESS, "CTF Exchange"),
        (NEG_RISK_CTF_EXCHANGE_ADDRESS, "Neg Risk CTF Exchange"),
        (NEG_RISK_ADAPTER_ADDRESS, "Neg Risk Adapter"),
    ];

    for (address, name) in contracts {
        info!("Approving USDC for {}: {}", name, address);
        let spender = Address::from_str(address)
            .map_err(|e| TradingError::Api(format!("Invalid {} address: {}", name, e)))?;

        match approve_usdc(wallet, spender, amount).await {
            Ok(response) => {
                info!("{} approval successful: {:?}", name, response.transaction_hash);
                results.push(response);
            }
            Err(e) => {
                // If already approved, we might get an error - continue anyway
                info!("{} approval result: {}", name, e);
                results.push(ApprovalResponse {
                    success: false,
                    transaction_hash: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    Ok(results)
}

/// Approve USDC spending for a specific spender using alloy Provider
pub async fn approve_usdc(
    wallet: &TradingWallet,
    spender: Address,
    amount: U256,
) -> Result<ApprovalResponse> {
    let usdc_address = Address::from_str(USDC_ADDRESS)
        .map_err(|e| TradingError::Api(format!("Invalid USDC address: {}", e)))?;

    info!(
        "Approving USDC spending: from={}, spender={}, amount={}",
        wallet.address(),
        spender,
        amount
    );

    // Build approve calldata: approve(address spender, uint256 amount)
    let calldata = build_approve_calldata(spender, amount);

    // Create wallet for provider
    let eth_wallet = EthereumWallet::from(wallet.signer().clone());

    // Build provider with wallet
    let provider = ProviderBuilder::new()
        .wallet(eth_wallet)
        .connect_http(POLYGON_RPC_URL.parse().unwrap());

    // Build transaction request
    let tx = TransactionRequest::default()
        .to(usdc_address)
        .input(calldata.into());

    // Send transaction and wait for receipt
    let pending_tx = provider
        .send_transaction(tx)
        .await
        .map_err(|e| TradingError::Api(format!("Failed to send transaction: {}", e)))?;

    let tx_hash = format!("{:?}", pending_tx.tx_hash());
    info!("USDC approval transaction sent: {}", tx_hash);

    // Wait for receipt to confirm success
    let receipt = pending_tx
        .get_receipt()
        .await
        .map_err(|e| TradingError::Api(format!("Failed to get receipt: {}", e)))?;

    // Check if transaction succeeded
    if receipt.status() {
        info!("USDC approval confirmed in block {:?}", receipt.block_number);
        Ok(ApprovalResponse {
            success: true,
            transaction_hash: Some(tx_hash),
            error: None,
        })
    } else {
        Err(TradingError::Api(format!(
            "Approval transaction reverted: {}",
            tx_hash
        )))
    }
}

/// Build ERC20 approve calldata
fn build_approve_calldata(spender: Address, amount: U256) -> Vec<u8> {
    let mut calldata = Vec::with_capacity(68);

    // Function selector: approve(address,uint256) = 0x095ea7b3
    calldata.extend_from_slice(&hex::decode(APPROVE_SELECTOR).unwrap());

    // Pad spender address to 32 bytes
    calldata.extend_from_slice(&[0u8; 12]); // 12 zero bytes
    calldata.extend_from_slice(spender.as_slice()); // 20 bytes address

    // Amount as 32 bytes
    calldata.extend_from_slice(&amount.to_be_bytes::<32>());

    calldata
}

/// Check MATIC balance for gas
pub async fn get_matic_balance(address: &str) -> Result<String> {
    let client = reqwest::Client::new();

    let request = JsonRpcRequest {
        jsonrpc: "2.0",
        method: "eth_getBalance",
        params: vec![serde_json::json!(address), serde_json::json!("latest")],
        id: 1,
    };

    let response = client
        .post(POLYGON_RPC_URL)
        .json(&request)
        .send()
        .await
        .map_err(|e| TradingError::Api(format!("RPC request failed: {}", e)))?;

    let rpc_response: JsonRpcResponse = response
        .json()
        .await
        .map_err(|e| TradingError::Api(format!("Failed to parse RPC response: {}", e)))?;

    if let Some(error) = rpc_response.error {
        return Err(TradingError::Api(format!("RPC error: {}", error.message)));
    }

    let hex_balance = rpc_response
        .result
        .ok_or_else(|| TradingError::Api("No result in RPC response".to_string()))?;

    let balance =
        u128::from_str_radix(hex_balance.strip_prefix("0x").unwrap_or(&hex_balance), 16)
            .unwrap_or(0);

    // Format as MATIC (18 decimals)
    let divisor = 10u128.pow(18);
    let whole = balance / divisor;
    let fraction = (balance % divisor) / 10u128.pow(14); // 4 decimal places

    Ok(format!("{}.{:04}", whole, fraction))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pad_address() {
        let address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bE00";
        let padded = pad_address(address).unwrap();
        assert_eq!(padded.len(), 64);
        assert!(padded.starts_with("000000000000000000000000"));
    }

    #[test]
    fn test_parse_uint256() {
        assert_eq!(parse_uint256("0x0").unwrap(), 0);
        assert_eq!(parse_uint256("0x1").unwrap(), 1);
        assert_eq!(parse_uint256("0x0f4240").unwrap(), 1_000_000); // 1 USDC
    }

    #[test]
    fn test_format_usdc() {
        assert_eq!(format_usdc(0), "0.00");
        assert_eq!(format_usdc(1_000_000), "1.00"); // 1 USDC (no fraction)
        assert_eq!(format_usdc(1_500_000), "1.500000"); // 1.5 USDC
        assert_eq!(format_usdc(100_000_000), "100.00"); // 100 USDC (no fraction)
    }
}
