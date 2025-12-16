//! Authenticated CLOB API client for Polymarket

use hmac::{Hmac, Mac};
use reqwest::header::{HeaderMap, HeaderValue};
use sha2::Sha256;
use tracing::{debug, error, info, warn};

use crate::eip712::{current_timestamp, generate_nonce};
use crate::order::{OrderBuilder, OrderType};
use crate::types::{
    ApiCredentials, ApiKeyResponse, OpenOrder, OrderResponse, PostOrderRequest, Result,
    SignedOrder, TradingError, UserTrade,
};
use crate::wallet::TradingWallet;

// ============================================================================
// Constants
// ============================================================================

const CLOB_BASE_URL: &str = "https://clob.polymarket.com";

// Header names
const HEADER_ADDRESS: &str = "POLY_ADDRESS";
const HEADER_SIGNATURE: &str = "POLY_SIGNATURE";
const HEADER_TIMESTAMP: &str = "POLY_TIMESTAMP";
const HEADER_NONCE: &str = "POLY_NONCE";
const HEADER_API_KEY: &str = "POLY_API_KEY";
const HEADER_PASSPHRASE: &str = "POLY_PASSPHRASE";

type HmacSha256 = Hmac<Sha256>;

// ============================================================================
// CLOB Client
// ============================================================================

/// Authenticated client for Polymarket CLOB API
pub struct ClobClient {
    wallet: TradingWallet,
    http_client: reqwest::Client,
    base_url: String,
}

impl ClobClient {
    /// Create a new CLOB client with the given wallet
    pub fn new(wallet: TradingWallet) -> Self {
        // Build HTTP client with proper headers to avoid Cloudflare blocks
        let http_client = reqwest::Client::builder()
            .user_agent("polymarket-terminal/1.0")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            wallet,
            http_client,
            base_url: CLOB_BASE_URL.to_string(),
        }
    }

    /// Create a new CLOB client from environment
    pub fn from_env() -> Result<Self> {
        let wallet = TradingWallet::from_env()?;
        Ok(Self::new(wallet))
    }

    /// Get the wallet address
    pub fn address(&self) -> String {
        self.wallet.address_string()
    }

    /// Get a reference to the wallet
    pub fn wallet(&self) -> &TradingWallet {
        &self.wallet
    }

    /// Get a mutable reference to the wallet
    pub fn wallet_mut(&mut self) -> &mut TradingWallet {
        &mut self.wallet
    }

    // ========================================================================
    // L1 Authentication (EIP-712 signing for API key management)
    // ========================================================================

    /// Build L1 authentication headers
    async fn build_l1_headers(&self) -> Result<HeaderMap> {
        let timestamp = current_timestamp();
        let nonce = generate_nonce();

        let address = self.wallet.address_string();

        info!("Building L1 auth headers:");
        info!("  Address: {}", address);
        info!("  Timestamp: {}", timestamp);
        info!("  Nonce: {}", nonce);

        let signature = self.wallet.sign_l1_auth(timestamp, nonce).await?;

        info!("  Signature: {}", signature);
        info!("  Signature length: {} chars", signature.len());

        let mut headers = HeaderMap::new();
        headers.insert(
            HEADER_ADDRESS,
            HeaderValue::from_str(&address)
                .map_err(|e| TradingError::Api(format!("Invalid header value: {}", e)))?,
        );
        headers.insert(
            HEADER_SIGNATURE,
            HeaderValue::from_str(&signature)
                .map_err(|e| TradingError::Api(format!("Invalid header value: {}", e)))?,
        );
        headers.insert(
            HEADER_TIMESTAMP,
            HeaderValue::from_str(&timestamp.to_string())
                .map_err(|e| TradingError::Api(format!("Invalid header value: {}", e)))?,
        );
        headers.insert(
            HEADER_NONCE,
            HeaderValue::from_str(&nonce.to_string())
                .map_err(|e| TradingError::Api(format!("Invalid header value: {}", e)))?,
        );

        info!("L1 headers built successfully");
        Ok(headers)
    }

    /// Create new API credentials (L1 auth)
    pub async fn create_api_key(&mut self) -> Result<ApiCredentials> {
        info!("Creating new API key for wallet {}", self.wallet.address_string());

        let headers = self.build_l1_headers().await?;
        let url = format!("{}/auth/api-key", self.base_url);

        let response = self
            .http_client
            .post(&url)
            .headers(headers)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Failed to create API key: {} - {}", status, body);
            return Err(TradingError::Api(format!(
                "Failed to create API key: {} - {}",
                status, body
            )));
        }

        let api_key_response: ApiKeyResponse = response.json().await?;

        let credentials = ApiCredentials {
            api_key: api_key_response.api_key,
            secret: api_key_response.secret,
            passphrase: api_key_response.passphrase,
        };

        self.wallet.set_api_credentials(credentials.clone());
        info!("API key created successfully");
        info!("  API Key: {}", credentials.api_key);
        info!("  Secret: {}", credentials.secret);
        info!("  Passphrase: {}", credentials.passphrase);

        // Test the new credentials immediately with a simple L2 call
        info!("Testing new credentials with GET /data/orders...");
        match self.test_l2_auth().await {
            Ok(_) => info!("L2 auth test PASSED!"),
            Err(e) => error!("L2 auth test FAILED: {}", e),
        }

        Ok(credentials)
    }

    /// Test L2 authentication by calling a simple endpoint
    async fn test_l2_auth(&self) -> Result<()> {
        let path = "/data/orders";
        let headers = self.build_l2_headers("GET", path, "")?;
        let url = format!("{}{}", self.base_url, path);

        let response = self.http_client.get(&url).headers(headers).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(TradingError::Api(format!("{} - {}", status, body)))
        }
    }

    /// Derive existing API credentials (L1 auth)
    pub async fn derive_api_key(&mut self) -> Result<ApiCredentials> {
        info!(
            "Deriving API key for wallet {}",
            self.wallet.address_string()
        );

        let headers = self.build_l1_headers().await?;
        let url = format!("{}/auth/derive-api-key", self.base_url);

        let response = self.http_client.get(&url).headers(headers).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            warn!("Failed to derive API key: {} - {}", status, body);

            // If derivation fails (no existing key), try creating a new one
            // Polymarket returns different errors: 404, "not found", or "Could not derive"
            if status == 404
                || body.contains("not found")
                || body.contains("Could not derive")
            {
                info!("No existing API key found, creating new one");
                return self.create_api_key().await;
            }

            return Err(TradingError::Api(format!(
                "Failed to derive API key: {} - {}",
                status, body
            )));
        }

        let api_key_response: ApiKeyResponse = response.json().await?;

        let credentials = ApiCredentials {
            api_key: api_key_response.api_key,
            secret: api_key_response.secret,
            passphrase: api_key_response.passphrase,
        };

        self.wallet.set_api_credentials(credentials.clone());
        info!("API key derived successfully");

        Ok(credentials)
    }

    /// Ensure we have API credentials, deriving them if necessary
    pub async fn ensure_api_key(&mut self) -> Result<()> {
        if self.wallet.has_api_credentials() {
            return Ok(());
        }

        // Try to derive first, then create if that fails
        self.derive_api_key().await?;
        Ok(())
    }

    // ========================================================================
    // L2 Authentication (HMAC signing for trading operations)
    // ========================================================================

    /// Build HMAC signature for L2 auth
    fn build_hmac_signature(
        &self,
        secret: &str,
        timestamp: &str,
        method: &str,
        path: &str,
        body: &str,
    ) -> Result<String> {
        let message = format!("{}{}{}{}", timestamp, method, path, body);

        info!("========== HMAC SIGNATURE DEBUG ==========");
        info!("Secret (base64): {}", secret);
        info!("Secret length: {} chars", secret.len());

        // Polymarket uses URL-safe base64 for the secret
        // Try multiple decoders to handle different padding scenarios
        let secret_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            secret,
        )
        .or_else(|e1| {
            info!("URL_SAFE_NO_PAD decode failed: {}", e1);
            base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE, secret)
        })
        .or_else(|e2| {
            info!("URL_SAFE decode failed: {}", e2);
            // Try adding padding if missing
            let padded = match secret.len() % 4 {
                2 => format!("{}==", secret),
                3 => format!("{}=", secret),
                _ => secret.to_string(),
            };
            info!("Trying with padding: {}", padded);
            base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE, &padded)
        })
        .map_err(|e| TradingError::Signing(format!("Invalid secret encoding: {}. Secret preview: {}...", e, &secret.chars().take(8).collect::<String>())))?;

        info!("Secret decoded: {} bytes", secret_bytes.len());
        info!("Secret bytes (hex): {}", hex::encode(&secret_bytes));

        info!("HMAC message: {}", message);
        info!("HMAC message bytes (hex): {}", hex::encode(message.as_bytes()));

        let mut mac = HmacSha256::new_from_slice(&secret_bytes)
            .map_err(|e| TradingError::Signing(format!("Failed to create HMAC: {}", e)))?;

        mac.update(message.as_bytes());
        let result = mac.finalize();
        let result_bytes = result.into_bytes();

        info!("HMAC result bytes (hex): {}", hex::encode(&result_bytes));

        // Output signature in URL-safe base64 WITH padding (matching Python client)
        // IMPORTANT: Polymarket requires padding (= suffix), so use URL_SAFE not URL_SAFE_NO_PAD
        let signature = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE,
            &result_bytes,
        );

        info!("Final signature (base64): {}", signature);
        info!("===========================================");

        Ok(signature)
    }

    /// Build L2 authentication headers
    fn build_l2_headers(
        &self,
        method: &str,
        path: &str,
        body: &str,
    ) -> Result<HeaderMap> {
        let credentials = self
            .wallet
            .api_credentials()
            .ok_or_else(|| TradingError::MissingCredentials("API credentials not set".to_string()))?;

        let timestamp = current_timestamp().to_string();

        // Log HMAC inputs
        info!("========== L2 AUTH (HMAC) ==========");
        info!("Timestamp: {}", timestamp);
        info!("Method: {}", method);
        info!("Path: {}", path);
        info!("Body length: {} chars", body.len());
        info!("HMAC message = timestamp + method + path + body");
        info!("Message preview: {}{}{}{}...", timestamp, method, path, &body[..50.min(body.len())]);

        let signature = self.build_hmac_signature(
            &credentials.secret,
            &timestamp,
            method,
            path,
            body,
        )?;

        info!("HMAC signature: {}", signature);
        info!("====================================");

        let address = self.wallet.address_string();

        info!("========== L2 HEADERS ==========");
        info!("POLY_ADDRESS: {}", address);
        info!("POLY_SIGNATURE: {}", signature);
        info!("POLY_TIMESTAMP: {}", timestamp);
        info!("POLY_API_KEY: {}", credentials.api_key);
        info!("POLY_PASSPHRASE: {}", credentials.passphrase);
        info!("================================");

        let mut headers = HeaderMap::new();
        headers.insert(
            HEADER_ADDRESS,
            HeaderValue::from_str(&address)
                .map_err(|e| TradingError::Api(format!("Invalid header value: {}", e)))?,
        );
        headers.insert(
            HEADER_SIGNATURE,
            HeaderValue::from_str(&signature)
                .map_err(|e| TradingError::Api(format!("Invalid header value: {}", e)))?,
        );
        headers.insert(
            HEADER_TIMESTAMP,
            HeaderValue::from_str(&timestamp)
                .map_err(|e| TradingError::Api(format!("Invalid header value: {}", e)))?,
        );
        headers.insert(
            HEADER_API_KEY,
            HeaderValue::from_str(&credentials.api_key)
                .map_err(|e| TradingError::Api(format!("Invalid header value: {}", e)))?,
        );
        headers.insert(
            HEADER_PASSPHRASE,
            HeaderValue::from_str(&credentials.passphrase)
                .map_err(|e| TradingError::Api(format!("Invalid header value: {}", e)))?,
        );

        Ok(headers)
    }

    // ========================================================================
    // Trading Operations (L2 authenticated)
    // ========================================================================

    /// Submit a signed order to the CLOB
    pub async fn post_order(
        &self,
        signed_order: SignedOrder,
        order_type: OrderType,
    ) -> Result<OrderResponse> {
        debug!("Submitting order: {:?}", signed_order);

        // Get API key - required for the owner field (NOT the wallet address!)
        // See: https://github.com/Polymarket/py-clob-client/blob/main/py_clob_client/client.py
        // body = order_to_json(order, self.creds.api_key, orderType)
        let credentials = self
            .wallet
            .api_credentials()
            .ok_or_else(|| TradingError::MissingCredentials("API credentials required for order submission".to_string()))?;

        let path = "/order";
        let request = PostOrderRequest {
            order: signed_order,
            owner: credentials.api_key.clone(),  // API key, not wallet address!
            order_type: order_type.as_str().to_string(),
        };

        let body = serde_json::to_string(&request)?;

        // Log EVERYTHING so we can debug
        info!("========== POST /order REQUEST ==========");
        info!("Owner (API key): {}", &credentials.api_key);
        info!("Order type: {}", order_type.as_str());
        info!("FULL REQUEST BODY:\n{}", serde_json::to_string_pretty(&request).unwrap_or_default());
        info!("==========================================");

        let headers = self.build_l2_headers("POST", path, &body)?;

        let url = format!("{}{}", self.base_url, path);
        info!("Sending request to: {}", url);

        let response = self
            .http_client
            .post(&url)
            .headers(headers)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Order submission failed: {} - {}", status, body);
            return Err(TradingError::OrderRejected(format!(
                "{} - {}",
                status, body
            )));
        }

        let order_response: OrderResponse = response.json().await?;

        if !order_response.success {
            return Err(TradingError::OrderRejected(
                order_response.error_msg.unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        info!(
            "Order submitted successfully: {:?}",
            order_response.order_id
        );
        Ok(order_response)
    }

    /// Create and submit an order in one call
    pub async fn submit_order(
        &self,
        token_id: &str,
        price: f64,
        size: f64,
        side: crate::types::Side,
        order_type: OrderType,
    ) -> Result<OrderResponse> {
        let builder = OrderBuilder::new(token_id, price, size, side);
        let signed_order = builder.build_and_sign(&self.wallet).await?;
        self.post_order(signed_order, order_type).await
    }

    /// Cancel an order by ID
    pub async fn cancel_order(&self, order_id: &str) -> Result<()> {
        info!("Cancelling order: {}", order_id);

        let path = "/order";
        let body = serde_json::json!({ "orderID": order_id }).to_string();
        let headers = self.build_l2_headers("DELETE", path, &body)?;

        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http_client
            .delete(&url)
            .headers(headers)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Cancel order failed: {} - {}", status, body);
            return Err(TradingError::Api(format!(
                "Failed to cancel order: {} - {}",
                status, body
            )));
        }

        info!("Order cancelled successfully: {}", order_id);
        Ok(())
    }

    /// Cancel all orders
    pub async fn cancel_all_orders(&self) -> Result<()> {
        info!("Cancelling all orders");

        let path = "/cancel-all";
        let headers = self.build_l2_headers("DELETE", path, "")?;

        let url = format!("{}{}", self.base_url, path);
        let response = self
            .http_client
            .delete(&url)
            .headers(headers)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Cancel all orders failed: {} - {}", status, body);
            return Err(TradingError::Api(format!(
                "Failed to cancel all orders: {} - {}",
                status, body
            )));
        }

        info!("All orders cancelled successfully");
        Ok(())
    }

    /// Get open orders
    pub async fn get_open_orders(&self) -> Result<Vec<OpenOrder>> {
        debug!("Fetching open orders");

        let path = "/data/orders";
        let headers = self.build_l2_headers("GET", path, "")?;

        let url = format!("{}{}", self.base_url, path);
        let response = self.http_client.get(&url).headers(headers).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TradingError::Api(format!(
                "Failed to get orders: {} - {}",
                status, body
            )));
        }

        // API returns paginated response: {"data": [...], "next_cursor": "...", ...}
        #[derive(serde::Deserialize)]
        struct PaginatedResponse {
            data: Vec<OpenOrder>,
        }
        let response: PaginatedResponse = response.json().await?;
        Ok(response.data)
    }

    /// Get user's trades
    pub async fn get_trades(&self) -> Result<Vec<UserTrade>> {
        debug!("Fetching trades");

        let path = "/data/trades";
        let headers = self.build_l2_headers("GET", path, "")?;

        let url = format!("{}{}", self.base_url, path);
        let response = self.http_client.get(&url).headers(headers).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TradingError::Api(format!(
                "Failed to get trades: {} - {}",
                status, body
            )));
        }

        // API returns paginated response: {"data": [...], "next_cursor": "...", ...}
        #[derive(serde::Deserialize)]
        struct PaginatedResponse {
            data: Vec<UserTrade>,
        }
        let response: PaginatedResponse = response.json().await?;
        Ok(response.data)
    }

    /// Get USDC balance and allowance for the wallet
    pub async fn get_balance(&self) -> Result<crate::types::Balance> {
        let address = self.wallet.address_string();

        let usdc_balance = crate::balance::get_usdc_balance(&address).await?;
        let usdc_allowance = crate::balance::get_usdc_allowance(&address).await?;

        Ok(crate::types::Balance {
            usdc_balance,
            usdc_allowance,
        })
    }

    /// Get current positions from Polymarket Data API
    /// This is more reliable than calculating from trades as it queries the actual on-chain state
    pub async fn get_positions(&self) -> Result<Vec<crate::types::Position>> {
        let address = self.wallet.address_string();
        debug!("Fetching positions for wallet: {}", address);

        // Query the Polymarket Data API (no auth required)
        let url = format!(
            "https://data-api.polymarket.com/positions?user={}",
            address
        );

        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TradingError::Api(format!(
                "Failed to get positions from Data API: {} - {}",
                status, body
            )));
        }

        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct DataApiPosition {
            asset: String,
            #[serde(default)]
            condition_id: String,
            size: f64,
            avg_price: f64,
            cur_price: f64,
            #[serde(default)]
            title: String,
            #[serde(default)]
            outcome: String,
            #[serde(default)]
            cash_pnl: f64,
            #[serde(default)]
            negative_risk: bool,
        }

        let data_positions: Vec<DataApiPosition> = response.json().await?;
        debug!("Fetched {} positions from Data API", data_positions.len());

        let positions = data_positions
            .into_iter()
            .filter(|p| p.size > 0.0) // Only include positions with shares
            .map(|p| crate::types::Position {
                market_id: p.condition_id,
                token_id: p.asset,
                outcome: p.outcome,
                shares: format!("{:.6}", p.size),
                avg_price: format!("{:.4}", p.avg_price),
                current_price: format!("{:.4}", p.cur_price),
                pnl: format!("{:.2}", p.cash_pnl),
                title: p.title,
                neg_risk: p.negative_risk,
            })
            .collect();

        Ok(positions)
    }
}

impl std::fmt::Debug for ClobClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClobClient")
            .field("wallet", &self.wallet)
            .field("base_url", &self.base_url)
            .finish()
    }
}
