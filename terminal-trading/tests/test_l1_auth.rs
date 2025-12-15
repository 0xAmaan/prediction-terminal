//! Test L1/L2 authentication and order submission for Polymarket CLOB
//!
//! Run with: cargo test -p terminal-trading --test test_l1_auth -- --nocapture
//! Or just run the order submission test:
//!   cargo test -p terminal-trading --test test_l1_auth test_order_submission -- --nocapture

use alloy::primitives::{Address, U256};
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::SignerSync;
use alloy::sol;
use alloy::sol_types::{eip712_domain, SolStruct};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

const CLOB_BASE_URL: &str = "https://clob.polymarket.com";
const POLYGON_CHAIN_ID: u64 = 137;

// Exchange contract addresses
// CTF Exchange - for simple binary markets
const CTF_EXCHANGE: &str = "0x4bfb41d5b3570defd03c39a9a4d8de6bd8b8982e";
// Neg Risk CTF Exchange - for multi-outcome markets (most markets)
const NEG_RISK_CTF_EXCHANGE: &str = "0xC5d563A36AE78145C45a50134d48A1215220f80a";

// Define ClobAuth using sol! macro - exactly like polymarket-rs-client
sol! {
    struct ClobAuth {
        address address;
        string timestamp;
        uint256 nonce;
        string message;
    }
}

// Define Order struct for EIP-712 signing
// IMPORTANT: The struct MUST be named "Order" for correct EIP-712 type hash
sol! {
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

// Order struct for JSON serialization (different field naming conventions)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OrderJson {
    salt: u64,
    maker: String,
    signer: String,
    taker: String,
    token_id: String,
    maker_amount: String,
    taker_amount: String,
    expiration: String,
    nonce: String,
    fee_rate_bps: String,
    side: String,
    signature_type: u8,
    signature: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PostOrderRequest {
    order: OrderJson,
    owner: String,
    order_type: String,
}

#[derive(Debug, Deserialize)]
struct OrderResponse {
    success: bool,
    #[serde(default)]
    error_msg: Option<String>,
    #[serde(default, rename = "orderID")]
    order_id: Option<String>,
}

/// Generate a random salt for order uniqueness
fn generate_salt() -> u64 {
    use rand::Rng;
    let mut rng = rand::rng();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as f64;
    let random_factor: f64 = rng.random();
    (timestamp * random_factor) as u64
}

/// Get current unix timestamp in seconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Sign L1 auth message using Alloy's built-in EIP-712 signing (sync version)
fn sign_clob_auth_message(
    signer: &PrivateKeySigner,
    timestamp: u64,
    nonce: u64,
) -> String {
    let domain = eip712_domain! {
        name: "ClobAuthDomain",
        version: "1",
        chain_id: POLYGON_CHAIN_ID,
    };

    let clob_auth = ClobAuth {
        address: signer.address(),
        timestamp: timestamp.to_string(),
        nonce: U256::from(nonce),
        message: "This message attests that I control the given wallet".to_string(),
    };

    println!("ClobAuth struct:");
    println!("  address: {:?}", clob_auth.address);
    println!("  timestamp: {}", clob_auth.timestamp);
    println!("  nonce: {}", clob_auth.nonce);
    println!("  message: {}", clob_auth.message);
    println!("Domain: ClobAuthDomain v1 chainId={}", POLYGON_CHAIN_ID);

    // Compute the EIP-712 signing hash
    let signing_hash = clob_auth.eip712_signing_hash(&domain);
    println!("Signing hash: 0x{}", hex::encode(signing_hash));

    // Sign using sync method
    let signature = signer
        .sign_hash_sync(&signing_hash)
        .expect("Failed to sign hash");

    format!("0x{}", hex::encode(signature.as_bytes()))
}

/// Build L1 auth headers
fn build_l1_headers(signer: &PrivateKeySigner) -> HeaderMap {
    let timestamp = current_timestamp();
    let nonce = 0u64; // Python client uses 0 as default nonce
    let address = signer.address().to_checksum(None);

    println!("\n=== Building L1 Headers ===");
    println!("Timestamp: {}", timestamp);
    println!("Nonce: {}", nonce);
    println!("Address: {}", address);

    let signature = sign_clob_auth_message(signer, timestamp, nonce);
    println!("Signature: {}", signature);

    let mut headers = HeaderMap::new();
    headers.insert("POLY_ADDRESS", HeaderValue::from_str(&address).unwrap());
    headers.insert("POLY_SIGNATURE", HeaderValue::from_str(&signature).unwrap());
    headers.insert(
        "POLY_TIMESTAMP",
        HeaderValue::from_str(&timestamp.to_string()).unwrap(),
    );
    headers.insert(
        "POLY_NONCE",
        HeaderValue::from_str(&nonce.to_string()).unwrap(),
    );

    headers
}

/// Test deriving API key
async fn test_derive_api_key(signer: &PrivateKeySigner) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n\n========== TEST: DERIVE API KEY ==========");
    let headers = build_l1_headers(signer);

    let client = reqwest::Client::builder()
        .user_agent("polymarket-terminal/1.0")
        .build()?;

    let url = format!("{}/auth/derive-api-key", CLOB_BASE_URL);
    println!("\nSending GET to: {}", url);

    let response = client.get(&url).headers(headers).send().await?;

    let status = response.status();
    let body = response.text().await?;

    println!("\nResponse:");
    println!("  Status: {}", status);
    println!("  Body: {}", body);

    if status.is_success() {
        println!("\n✅ DERIVE API KEY SUCCESS!");

        // Parse and test the credentials
        let creds: serde_json::Value = serde_json::from_str(&body)?;
        if let (Some(api_key), Some(secret), Some(passphrase)) = (
            creds.get("apiKey").and_then(|v| v.as_str()),
            creds.get("secret").and_then(|v| v.as_str()),
            creds.get("passphrase").and_then(|v| v.as_str()),
        ) {
            println!("\n======= Testing L2 auth with derived credentials =======");
            test_l2_auth(signer, api_key, secret, passphrase).await?;
        }
    } else {
        println!("\n❌ DERIVE API KEY FAILED");
    }

    Ok(())
}

/// Test creating API key
async fn test_create_api_key(signer: &PrivateKeySigner) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n\n========== TEST: CREATE API KEY ==========");
    let headers = build_l1_headers(signer);

    let client = reqwest::Client::builder()
        .user_agent("polymarket-terminal/1.0")
        .build()?;

    let url = format!("{}/auth/api-key", CLOB_BASE_URL);
    println!("\nSending POST to: {}", url);

    let response = client.post(&url).headers(headers).send().await?;

    let status = response.status();
    let body = response.text().await?;

    println!("\nResponse:");
    println!("  Status: {}", status);
    println!("  Body: {}", body);

    if status.is_success() {
        println!("\n✅ CREATE API KEY SUCCESS!");

        // Parse and test the credentials
        let creds: serde_json::Value = serde_json::from_str(&body)?;
        if let (Some(api_key), Some(secret), Some(passphrase)) = (
            creds.get("apiKey").and_then(|v| v.as_str()),
            creds.get("secret").and_then(|v| v.as_str()),
            creds.get("passphrase").and_then(|v| v.as_str()),
        ) {
            println!("\nTesting L2 auth with new credentials...");
            test_l2_auth(signer, api_key, secret, passphrase).await?;
        }
    } else {
        println!("\n❌ CREATE API KEY FAILED");
    }

    Ok(())
}

/// Test L2 authentication
async fn test_l2_auth(
    signer: &PrivateKeySigner,
    api_key: &str,
    secret: &str,
    passphrase: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Testing L2 Auth ===");

    let timestamp = current_timestamp().to_string();
    let method = "GET";
    let path = "/data/orders";
    let body = "";

    // Build HMAC signature
    let message = format!("{}{}{}{}", timestamp, method, path, body);
    println!("HMAC message: {}", message);

    // Decode secret from base64
    let secret_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        secret,
    )
    .or_else(|_| {
        base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE, secret)
    })
    .or_else(|_| {
        let padded = match secret.len() % 4 {
            2 => format!("{}==", secret),
            3 => format!("{}=", secret),
            _ => secret.to_string(),
        };
        base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE, &padded)
    })?;

    println!("Secret decoded: {} bytes", secret_bytes.len());

    // Compute HMAC
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(&secret_bytes)?;
    mac.update(message.as_bytes());
    let result = mac.finalize();

    // URL-safe base64 WITH padding (keep the = suffix per Polymarket spec)
    let signature = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE,
        result.into_bytes(),
    );

    println!("HMAC signature: {}", signature);

    // Build headers
    let address = signer.address().to_checksum(None);
    let mut headers = HeaderMap::new();
    headers.insert("POLY_ADDRESS", HeaderValue::from_str(&address)?);
    headers.insert("POLY_SIGNATURE", HeaderValue::from_str(&signature)?);
    headers.insert("POLY_TIMESTAMP", HeaderValue::from_str(&timestamp)?);
    headers.insert("POLY_API_KEY", HeaderValue::from_str(api_key)?);
    headers.insert("POLY_PASSPHRASE", HeaderValue::from_str(passphrase)?);

    println!("\nL2 Headers:");
    println!("  POLY_ADDRESS: {}", address);
    println!("  POLY_SIGNATURE: {}", signature);
    println!("  POLY_TIMESTAMP: {}", timestamp);
    println!("  POLY_API_KEY: {}", api_key);
    println!("  POLY_PASSPHRASE: {}", passphrase);

    // Make request
    let client = reqwest::Client::builder()
        .user_agent("polymarket-terminal/1.0")
        .build()?;

    let url = format!("{}{}", CLOB_BASE_URL, path);
    println!("\nSending GET to: {}", url);

    let response = client.get(&url).headers(headers).send().await?;

    let status = response.status();
    let body = response.text().await?;

    println!("\nResponse:");
    println!("  Status: {}", status);
    println!("  Body: {}", body);

    if status.is_success() {
        println!("\n✅ L2 AUTH SUCCESS!");
    } else {
        println!("\n❌ L2 AUTH FAILED");
    }

    Ok(())
}

#[tokio::test]
async fn test_l1_auth_flow() {
    // Try multiple locations for .env file
    dotenvy::from_filename("../.env.local").ok();
    dotenvy::from_filename(".env.local").ok();
    dotenvy::dotenv().ok();

    let private_key = std::env::var("TRADING_PRIVATE_KEY")
        .expect("TRADING_PRIVATE_KEY env var required - set in .env.local or as env var");
    let key = private_key.strip_prefix("0x").unwrap_or(&private_key);

    let signer = key.parse::<PrivateKeySigner>()
        .expect("Invalid private key");

    println!("Wallet address: {}", signer.address().to_checksum(None));

    // First try to derive existing key
    if let Err(e) = test_derive_api_key(&signer).await {
        eprintln!("Error during derive: {}", e);
    }

    // Then try to create new key
    if let Err(e) = test_create_api_key(&signer).await {
        eprintln!("Error during create: {}", e);
    }
}

/// Sign an order using EIP-712
fn sign_order(signer: &PrivateKeySigner, order: &Order, is_neg_risk: bool) -> String {
    // Use correct exchange contract based on market type
    let verifying_contract: Address = if is_neg_risk {
        NEG_RISK_CTF_EXCHANGE.parse().unwrap()
    } else {
        CTF_EXCHANGE.parse().unwrap()
    };

    println!("Using verifying contract: {}", verifying_contract);

    let domain = eip712_domain! {
        name: "Polymarket CTF Exchange",
        version: "1",
        chain_id: POLYGON_CHAIN_ID,
        verifying_contract: verifying_contract,
    };

    // Compute the EIP-712 signing hash
    let signing_hash = order.eip712_signing_hash(&domain);
    println!("Order signing hash: 0x{}", hex::encode(signing_hash));

    // Sign using sync method
    let signature = signer
        .sign_hash_sync(&signing_hash)
        .expect("Failed to sign order");

    format!("0x{}", hex::encode(signature.as_bytes()))
}

/// Build L2 headers for POST request with body
fn build_l2_post_headers(
    address: &str,
    api_key: &str,
    secret: &str,
    passphrase: &str,
    path: &str,
    body: &str,
) -> Result<HeaderMap, Box<dyn std::error::Error>> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let timestamp = current_timestamp().to_string();
    let method = "POST";

    // Build HMAC message: timestamp + method + path + body
    let message = format!("{}{}{}{}", timestamp, method, path, body);
    println!("HMAC message: {}", message);

    // Decode secret from base64
    let secret_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
        secret,
    )
    .or_else(|_| {
        base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE, secret)
    })
    .or_else(|_| {
        let padded = match secret.len() % 4 {
            2 => format!("{}==", secret),
            3 => format!("{}=", secret),
            _ => secret.to_string(),
        };
        base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE, &padded)
    })?;

    // Compute HMAC
    let mut mac = HmacSha256::new_from_slice(&secret_bytes)?;
    mac.update(message.as_bytes());
    let result = mac.finalize();

    // URL-safe base64 WITH padding
    let signature = base64::Engine::encode(
        &base64::engine::general_purpose::URL_SAFE,
        result.into_bytes(),
    );

    println!("L2 HMAC signature: {}", signature);

    // Build headers
    let mut headers = HeaderMap::new();
    headers.insert("POLY_ADDRESS", HeaderValue::from_str(address)?);
    headers.insert("POLY_SIGNATURE", HeaderValue::from_str(&signature)?);
    headers.insert("POLY_TIMESTAMP", HeaderValue::from_str(&timestamp)?);
    headers.insert("POLY_API_KEY", HeaderValue::from_str(api_key)?);
    headers.insert("POLY_PASSPHRASE", HeaderValue::from_str(passphrase)?);
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    Ok(headers)
}

/// Test order submission
async fn test_submit_order(
    signer: &PrivateKeySigner,
    api_key: &str,
    secret: &str,
    passphrase: &str,
    token_id: &str,
    price: f64,
    size: f64,
    side: &str, // "BUY" or "SELL"
    is_neg_risk: bool, // true for multi-outcome markets, false for binary
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n\n========== TEST: SUBMIT ORDER ==========");

    let address = signer.address();
    let address_str = address.to_checksum(None);

    // Calculate amounts based on side
    // For BUY: makerAmount = price * size * 1e6 (USDC), takerAmount = size * 1e6 (shares)
    // For SELL: makerAmount = size * 1e6 (shares), takerAmount = price * size * 1e6 (USDC)
    let (maker_amount, taker_amount, side_u8) = if side == "BUY" {
        let usdc_amount = (price * size * 1_000_000.0) as u64;
        let share_amount = (size * 1_000_000.0) as u64;
        (usdc_amount, share_amount, 0u8)
    } else {
        let share_amount = (size * 1_000_000.0) as u64;
        let usdc_amount = (price * size * 1_000_000.0) as u64;
        (share_amount, usdc_amount, 1u8)
    };

    let salt = generate_salt();
    let token_id_u256 = U256::from_str_radix(token_id, 10)
        .unwrap_or_else(|_| token_id.parse().unwrap_or(U256::ZERO));

    println!("Order parameters:");
    println!("  Token ID: {}", token_id);
    println!("  Price: {}", price);
    println!("  Size: {}", size);
    println!("  Side: {}", side);
    println!("  Salt: {}", salt);
    println!("  Maker amount: {}", maker_amount);
    println!("  Taker amount: {}", taker_amount);

    // Create EIP-712 order struct
    let order = Order {
        salt: U256::from(salt),
        maker: address,
        signer: address,
        taker: Address::ZERO,
        tokenId: token_id_u256,
        makerAmount: U256::from(maker_amount),
        takerAmount: U256::from(taker_amount),
        expiration: U256::ZERO,
        nonce: U256::ZERO,
        feeRateBps: U256::ZERO,
        side: side_u8,
        signatureType: 0, // EOA
    };

    // Sign the order
    let signature = sign_order(signer, &order, is_neg_risk);
    println!("Order signature: {}", signature);

    // Create JSON order
    let order_json = OrderJson {
        salt,
        maker: address_str.clone(),
        signer: address_str.clone(),
        taker: Address::ZERO.to_checksum(None),
        token_id: token_id.to_string(),
        maker_amount: maker_amount.to_string(),
        taker_amount: taker_amount.to_string(),
        expiration: "0".to_string(),
        nonce: "0".to_string(),
        fee_rate_bps: "0".to_string(),
        side: side.to_string(),
        signature_type: 0,
        signature,
    };

    // Create request body
    let request = PostOrderRequest {
        order: order_json,
        owner: api_key.to_string(), // API key, not wallet address!
        order_type: "GTC".to_string(),
    };

    let body = serde_json::to_string(&request)?;
    println!("\nRequest body:\n{}", serde_json::to_string_pretty(&request)?);

    // Build L2 headers
    let path = "/order";
    let headers = build_l2_post_headers(&address_str, api_key, secret, passphrase, path, &body)?;

    // Send request
    let client = reqwest::Client::builder()
        .user_agent("polymarket-terminal/1.0")
        .build()?;

    let url = format!("{}{}", CLOB_BASE_URL, path);
    println!("\nSending POST to: {}", url);

    let response = client
        .post(&url)
        .headers(headers)
        .body(body)
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await?;

    println!("\nResponse:");
    println!("  Status: {}", status);
    println!("  Body: {}", body);

    if status.is_success() {
        println!("\n✅ ORDER SUBMISSION SUCCESS!");
    } else {
        println!("\n❌ ORDER SUBMISSION FAILED");
    }

    Ok(())
}

#[tokio::test]
async fn test_order_submission() {
    // Load env
    dotenvy::from_filename("../.env.local").ok();
    dotenvy::from_filename(".env.local").ok();
    dotenvy::dotenv().ok();

    let private_key = std::env::var("TRADING_PRIVATE_KEY")
        .expect("TRADING_PRIVATE_KEY env var required");
    let key = private_key.strip_prefix("0x").unwrap_or(&private_key);

    let signer = key.parse::<PrivateKeySigner>()
        .expect("Invalid private key");

    println!("Wallet address: {}", signer.address().to_checksum(None));

    // First derive API credentials
    let headers = build_l1_headers(&signer);
    let client = reqwest::Client::builder()
        .user_agent("polymarket-terminal/1.0")
        .build()
        .unwrap();

    let url = format!("{}/auth/derive-api-key", CLOB_BASE_URL);
    let response = client.get(&url).headers(headers).send().await.unwrap();

    if !response.status().is_success() {
        eprintln!("Failed to derive API key!");
        return;
    }

    let body = response.text().await.unwrap();
    let creds: serde_json::Value = serde_json::from_str(&body).unwrap();

    let api_key = creds.get("apiKey").and_then(|v| v.as_str()).unwrap();
    let secret = creds.get("secret").and_then(|v| v.as_str()).unwrap();
    let passphrase = creds.get("passphrase").and_then(|v| v.as_str()).unwrap();

    println!("API Key: {}", api_key);

    // Active market: "Will NVIDIA be the largest company in the world by market cap on December 31?"
    // This is a YES token from an active neg_risk market
    let token_id = "94850533403292240972948844256810904078895883844462287088135166537739765648754";

    // Submit a small test order (adjust price/size as needed)
    // Price: 0.85 = 85 cents
    // Size: 0.1 = $0.10 worth
    // Most Polymarket markets are neg_risk (multi-outcome), try that first
    if let Err(e) = test_submit_order(
        &signer,
        api_key,
        secret,
        passphrase,
        token_id,
        0.85,  // price
        0.1,   // size
        "BUY",
        true,  // is_neg_risk - try with Neg Risk CTF Exchange first
    ).await {
        eprintln!("Error during order submission with neg_risk=true: {}", e);

        // Try with CTF Exchange if neg_risk fails
        println!("\n\nRetrying with CTF Exchange (is_neg_risk=false)...\n");
        if let Err(e2) = test_submit_order(
            &signer,
            api_key,
            secret,
            passphrase,
            token_id,
            0.85,
            0.1,
            "BUY",
            false,  // is_neg_risk - try with standard CTF Exchange
        ).await {
            eprintln!("Error during order submission with neg_risk=false: {}", e2);
        }
    }
}
