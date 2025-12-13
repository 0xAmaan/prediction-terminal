//! Balance queries via Polygon JSON-RPC
//!
//! Query USDC.e balance and allowance for trading wallet using
//! direct eth_call to the ERC20 contract.

use crate::types::{Result, TradingError, CTF_EXCHANGE_ADDRESS, USDC_ADDRESS};
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Polygon RPC endpoint
const POLYGON_RPC_URL: &str = "https://polygon-rpc.com";

/// ERC20 function selectors
const BALANCE_OF_SELECTOR: &str = "70a08231"; // balanceOf(address)
const ALLOWANCE_SELECTOR: &str = "dd62ed3e"; // allowance(address,address)

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
