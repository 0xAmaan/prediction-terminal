Polymarket Trade Execution - Research & Implementation Plan

 Executive Summary

 This document covers the research findings and implementation plan for enabling live trade execution on Polymarket through your
 terminal.

 ---
 Part 1: Research Findings

 How Polymarket Trading Works

 Architecture Overview

 ┌─────────────────────────────────────────────────────────────────────────────┐
 │                           USER'S WALLET                                     │
 │  (Polygon Network - holds USDC.e + Conditional Tokens ERC1155)             │
 └─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     │ Signs orders (EIP-712)
                                     ▼
 ┌─────────────────────────────────────────────────────────────────────────────┐
 │                        POLYMARKET CLOB API                                  │
 │                   https://clob.polymarket.com                               │
 │  - Receives signed orders                                                   │
 │  - Matches orders off-chain                                                 │
 │  - Submits matched trades on-chain                                          │
 └─────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     │ Settlement
                                     ▼
 ┌─────────────────────────────────────────────────────────────────────────────┐
 │                    POLYGON BLOCKCHAIN                                       │
 │  - CTF Exchange Contract (order matching/settlement)                        │
 │  - Conditional Token Framework (ERC1155 outcome tokens)                     │
 │  - USDC.e Collateral                                                        │
 └─────────────────────────────────────────────────────────────────────────────┘

 Key Concepts

 1. CLOB (Central Limit Order Book): Polymarket runs an off-chain order book. Orders are signed client-side but matched by
 Polymarket's servers. Only matched trades settle on-chain.
 2. Conditional Tokens: Each market outcome (YES/NO) is an ERC1155 token on Polygon. When you buy YES, you're buying that specific
 token.
 3. USDC.e Collateral: All trading uses USDC.e (bridged USDC from Ethereum) on Polygon as collateral.
 4. Proxy Wallets: Polymarket users have proxy wallets (Gnosis Safe for MetaMask users, custom proxy for email/Magic users) that
 hold their funds.

 ---
 Authentication & Credentials

 Two-Level Authentication System

 L1 - Private Key Authentication (EIP-712 Signing)
 - Used for: Creating/deriving API keys, signing orders
 - Headers: POLY_ADDRESS, POLY_SIGNATURE, POLY_TIMESTAMP, POLY_NONCE
 - Signature: EIP-712 typed data signature from wallet

 L2 - API Key Authentication (HMAC)
 - Used for: All trading endpoints (post order, cancel, get orders)
 - Headers: POLY_ADDRESS, POLY_SIGNATURE (HMAC), POLY_TIMESTAMP, POLY_API_KEY, POLY_PASSPHRASE
 - Your codebase already has HMAC signing infrastructure in terminal-polymarket/src/client.rs

 Signature Types for Orders

 | Type             | Value | Use Case                                          |
 |------------------|-------|---------------------------------------------------|
 | EOA              | 0     | Direct trading from Externally Owned Account      |
 | POLY_PROXY       | 1     | Email/Magic login users (Polymarket proxy wallet) |
 | POLY_GNOSIS_SAFE | 2     | MetaMask/browser wallet users (Gnosis Safe)       |

 ---
 Order Flow

 1. User wants to buy 100 YES shares at $0.50

 2. Client creates order object:
    {
      tokenId: "71321045679252212594626385532706912750332728571942532289631379312455583992563",
      price: 0.50,
      size: 100,
      side: BUY
    }

 3. Client signs order (EIP-712) with private key

 4. Client POSTs to /order with:
    - Signed order
    - Order type (GTC, FOK, GTD)
    - L2 authentication headers

 5. CLOB matches order against resting orders

 6. If matched: Settlement happens on Polygon
    If not matched (limit): Order rests in book

 7. Response contains orderId and transaction hashes (if matched)

 ---
 Order Types

 | Type | Name               | Behavior                                 |
 |------|--------------------|------------------------------------------|
 | GTC  | Good-Til-Cancelled | Rests in book until filled or cancelled  |
 | GTD  | Good-Til-Date      | Expires at specified timestamp           |
 | FOK  | Fill-Or-Kill       | Must fill entirely or cancel immediately |
 | FAK  | Fill-And-Kill      | Fill what's available, cancel rest       |

 ---
 API Endpoints for Trading

 | Endpoint             | Method | Purpose                              |
 |----------------------|--------|--------------------------------------|
 | /auth/api-key        | POST   | Create new API credentials (L1 auth) |
 | /auth/derive-api-key | GET    | Derive existing API key (L1 auth)    |
 | /order               | POST   | Submit single order (L2 auth)        |
 | /orders              | POST   | Submit multiple orders (L2 auth)     |
 | /order               | DELETE | Cancel single order (L2 auth)        |
 | /orders              | DELETE | Cancel multiple orders (L2 auth)     |
 | /cancel-all          | DELETE | Cancel all orders (L2 auth)          |
 | /data/orders         | GET    | Get open orders (L2 auth)            |
 | /data/trades         | GET    | Get user's trades (L2 auth)          |

 ---
 Geo-Blocking & Legal Status

 Current Restrictions (as of 2024-2025)

 - US Users Blocked: Following a 2022 CFTC settlement, Polymarket geo-blocks US IP addresses
 - Enforcement: IP-based blocking + Cloudflare protection
 - API Access: The CLOB API also has geo-restrictions; users report 403 errors from US IPs when creating API keys
 - VPN Usage: Technically possible but violates Terms of Service
 - DOJ Investigation: In November 2024, DOJ raided Polymarket founder's home investigating whether US residents were trading

 Future US Access

 - Polymarket acquired QCEX (CFTC-licensed) in 2025
 - Planning regulated US access by end of 2025/early 2026
 - $2B investment from NYSE parent company (ICE)

 For Your Terminal

 If targeting non-US users: Can implement trading directly
 If targeting US users: Must wait for regulated US product OR accept legal risk

 ---
 Settlement & Blockchain Details

 Network: Polygon Mainnet (Chain ID: 137)

 Key Contract Addresses

 | Contract                          | Address                                    | Purpose                   |
 |-----------------------------------|--------------------------------------------|---------------------------|
 | USDC.e                            | 0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174 | Collateral token          |
 | CTF (Conditional Token Framework) | 0x4d97dcd97ec945f40cf65f87097ace5ea0476045 | Outcome tokens            |
 | CTF Exchange                      | (varies)                                   | Order matching/settlement |

 Token Allowances Required

 Before trading, users must approve:
 1. USDC.e spending by CTF Exchange
 2. Conditional token spending by CTF Exchange

 ---
 What Your Codebase Already Has

 From the exploration of terminal-polymarket/:

 ✅ HMAC signature generation (build_signature in client.rs)
 ✅ Credential loading (POLY_API_KEY, POLY_SECRET, POLY_PASSPHRASE from env)
 ✅ WebSocket infrastructure for real-time data
 ✅ Market data endpoints (orderbook, trades, prices)
 ✅ Type definitions for orders, trades, markets

 ❌ Missing for Trading:
 - EIP-712 order signing
 - Order submission endpoints
 - Order cancellation
 - Position/balance tracking
 - API key creation/derivation flow

 ---
 Part 2: Implementation Plan

 Architecture Decision

 Backend-Managed Wallet with Rust-Native Signing

 ┌─────────────────────────────────────────────────────────────────────────────┐
 │                         FRONTEND (Next.js)                                  │
 │  - Trade execution UI (Buy/Sell form)                                       │
 │  - Balance display                                                          │
 │  - Order history                                                            │
 │  - Deposit address display                                                  │
 └────────────────────────────────────────────────────────────────────────────┘
                                     │
                             REST API calls
                                     ▼
 ┌─────────────────────────────────────────────────────────────────────────────┐
 │                      TERMINAL-API (Axum)                                    │
 │  New endpoints:                                                              │
 │  - POST /api/trade/order      (submit order)                                │
 │  - DELETE /api/trade/order    (cancel order)                                │
 │  - GET /api/trade/orders      (open orders)                                 │
 │  - GET /api/trade/positions   (current positions)                           │
 │  - GET /api/trade/balance     (USDC balance)                                │
 │  - GET /api/trade/deposit     (deposit address)                             │
 └────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
 ┌─────────────────────────────────────────────────────────────────────────────┐
 │                    TERMINAL-TRADING (New Crate)                             │
 │  - Wallet management (generate, load from env)                              │
 │  - EIP-712 order signing                                                    │
 │  - API key derivation                                                       │
 │  - Order submission to CLOB                                                 │
 │  - Position tracking                                                        │
 └────────────────────────────────────────────────────────────────────────────┘
                                     │
                                     ▼
 ┌─────────────────────────────────────────────────────────────────────────────┐
 │                    POLYMARKET CLOB API                                      │
 │                 https://clob.polymarket.com                                 │
 └────────────────────────────────────────────────────────────────────────────┘

 Why Backend-Managed Wallet?

 1. Instant execution - No round-trip to user for signatures
 2. Simpler UX - User just deposits funds and clicks "Buy"
 3. Better for trading - Can implement stop-loss, auto-rebalancing later
 4. Custody model - Similar to centralized exchanges (Coinbase, Binance)

 ---
 Implementation Phases

 Phase 1: Core Trading Infrastructure (Backend)

 New Crate: terminal-trading/

 terminal-trading/
 ├── Cargo.toml
 └── src/
     ├── lib.rs
     ├── wallet.rs        # Wallet generation, loading, signing
     ├── eip712.rs        # EIP-712 typed data signing for orders
     ├── order.rs         # Order creation, signing, types
     ├── clob_client.rs   # CLOB API client (authenticated)
     ├── positions.rs     # Position/balance tracking
     └── types.rs         # Shared types

 Key Dependencies:
 [dependencies]
 ethers = { version = "2.0", features = ["rustls"] }  # Wallet, signing, EIP-712
 alloy-primitives = "0.7"                              # Address, U256 types
 serde = { version = "1.0", features = ["derive"] }
 serde_json = "1.0"
 reqwest = { version = "0.12", features = ["json"] }
 tokio = { version = "1", features = ["full"] }
 hmac = "0.12"
 sha2 = "0.10"
 base64 = "0.22"
 uuid = { version = "1.0", features = ["v4"] }

 wallet.rs - Core Functions:
 pub struct TradingWallet {
     wallet: LocalWallet,
     address: Address,
     api_key: Option<ApiCredentials>,
 }

 impl TradingWallet {
     // Load from private key in env
     pub fn from_env() -> Result<Self>;

     // Generate new wallet
     pub fn generate() -> Self;

     // Sign EIP-712 order
     pub async fn sign_order(&self, order: &Order) -> Result<Signature>;

     // Create/derive API credentials
     pub async fn derive_api_credentials(&mut self) -> Result<ApiCredentials>;
 }

 order.rs - Order Types:
 pub struct OrderArgs {
     pub token_id: String,      // CLOB token ID
     pub price: f64,            // 0.01 to 0.99
     pub size: f64,             // Number of shares
     pub side: Side,            // Buy or Sell
 }

 pub struct SignedOrder {
     pub salt: u128,
     pub maker: Address,
     pub signer: Address,
     pub taker: Address,
     pub token_id: String,
     pub maker_amount: U256,
     pub taker_amount: U256,
     pub expiration: u64,
     pub nonce: u64,
     pub fee_rate_bps: u64,
     pub side: u8,
     pub signature_type: u8,
     pub signature: String,
 }

 pub enum OrderType {
     GTC,  // Good-Til-Cancelled
     GTD,  // Good-Til-Date
     FOK,  // Fill-Or-Kill
     FAK,  // Fill-And-Kill
 }

 clob_client.rs - Authenticated CLOB Client:
 pub struct AuthenticatedClobClient {
     wallet: TradingWallet,
     http_client: reqwest::Client,
     base_url: String,
 }

 impl AuthenticatedClobClient {
     // Create/derive API key on first use
     pub async fn ensure_api_key(&mut self) -> Result<()>;

     // Submit order
     pub async fn post_order(&self, order: SignedOrder, order_type: OrderType) -> Result<OrderResponse>;

     // Cancel order
     pub async fn cancel_order(&self, order_id: &str) -> Result<()>;

     // Cancel all orders
     pub async fn cancel_all(&self) -> Result<()>;

     // Get open orders
     pub async fn get_open_orders(&self) -> Result<Vec<Order>>;

     // Get trades
     pub async fn get_trades(&self) -> Result<Vec<Trade>>;

     // Get balance/allowance
     pub async fn get_balance(&self) -> Result<Balance>;
 }

 ---
 Phase 2: API Endpoints (terminal-api)

 New Route File: terminal-api/src/routes/trading.rs

 // POST /api/trade/order
 pub async fn submit_order(
     State(state): State<AppState>,
     Json(req): Json<SubmitOrderRequest>,
 ) -> Result<Json<OrderResponse>, ApiError>;

 // DELETE /api/trade/order/:id
 pub async fn cancel_order(
     State(state): State<AppState>,
     Path(order_id): Path<String>,
 ) -> Result<Json<()>, ApiError>;

 // GET /api/trade/orders
 pub async fn get_open_orders(
     State(state): State<AppState>,
 ) -> Result<Json<Vec<Order>>, ApiError>;

 // GET /api/trade/positions
 pub async fn get_positions(
     State(state): State<AppState>,
 ) -> Result<Json<Vec<Position>>, ApiError>;

 // GET /api/trade/balance
 pub async fn get_balance(
     State(state): State<AppState>,
 ) -> Result<Json<Balance>, ApiError>;

 // GET /api/trade/deposit-address
 pub async fn get_deposit_address(
     State(state): State<AppState>,
 ) -> Result<Json<DepositInfo>, ApiError>;

 Request/Response Types:
 #[derive(Deserialize)]
 pub struct SubmitOrderRequest {
     pub token_id: String,      // Market's CLOB token ID
     pub side: String,          // "buy" or "sell"
     pub price: f64,            // Limit price (0.01-0.99)
     pub size: f64,             // Number of shares
     pub order_type: String,    // "GTC", "FOK", etc.
 }

 #[derive(Serialize)]
 pub struct OrderResponse {
     pub success: bool,
     pub order_id: Option<String>,
     pub error_msg: Option<String>,
     pub transaction_hashes: Vec<String>,
 }

 #[derive(Serialize)]
 pub struct Balance {
     pub usdc_balance: String,
     pub usdc_allowance: String,
 }

 #[derive(Serialize)]
 pub struct Position {
     pub market_id: String,
     pub token_id: String,
     pub outcome: String,       // "YES" or "NO"
     pub shares: String,
     pub avg_price: String,
     pub current_price: String,
     pub pnl: String,
 }

 ---
 Phase 3: Frontend Integration

 Update TradeExecution Component:

 The other agent is building frontend/src/components/market/trade-execution.tsx as a placeholder. We'll need to wire it up to
 actually submit orders:

 // API client additions (frontend/src/lib/api.ts)
 export const tradingApi = {
   submitOrder: (params: {
     tokenId: string;
     side: 'buy' | 'sell';
     price: number;
     size: number;
     orderType: 'GTC' | 'FOK';
   }) => fetch('/api/trade/order', { method: 'POST', body: JSON.stringify(params) }),

   cancelOrder: (orderId: string) =>
     fetch(`/api/trade/order/${orderId}`, { method: 'DELETE' }),

   getOpenOrders: () => fetch('/api/trade/orders'),

   getBalance: () => fetch('/api/trade/balance'),

   getPositions: () => fetch('/api/trade/positions'),

   getDepositAddress: () => fetch('/api/trade/deposit-address'),
 };

 TradeExecution Component Flow:
 1. User enters amount and price
 2. Click "Buy YES" or "Sell YES"
 3. Frontend calls POST /api/trade/order
 4. Backend signs order with managed wallet
 5. Backend submits to CLOB
 6. Response shown to user
 7. Balance/positions refresh

 ---
 Environment Variables

 Add to .env.local:
 # Trading Wallet (generate once, keep secret!)
 TRADING_PRIVATE_KEY=0x...      # 64 hex chars
 TRADING_WALLET_ADDRESS=0x...   # Derived from private key

 # Will be auto-derived on first run
 POLY_API_KEY=                  # Auto-populated
 POLY_SECRET=                   # Auto-populated
 POLY_PASSPHRASE=               # Auto-populated

 # Optional: Use VPN/proxy for US access
 # HTTP_PROXY=socks5://127.0.0.1:1080

 ---
 EIP-712 Order Signing Details

 Polymarket uses EIP-712 typed data signing. The domain and types:

 // Domain separator
 let domain = EIP712Domain {
     name: "Polymarket CTF Exchange",
     version: "1",
     chain_id: 137,  // Polygon
     verifying_contract: EXCHANGE_ADDRESS,
 };

 // Order type
 let order_type = vec![
     ("salt", "uint256"),
     ("maker", "address"),
     ("signer", "address"),
     ("taker", "address"),
     ("tokenId", "uint256"),
     ("makerAmount", "uint256"),
     ("takerAmount", "uint256"),
     ("expiration", "uint256"),
     ("nonce", "uint256"),
     ("feeRateBps", "uint256"),
     ("side", "uint8"),
     ("signatureType", "uint8"),
 ];

 ---
 Files to Create/Modify

 New Files:
 1. terminal-trading/Cargo.toml
 2. terminal-trading/src/lib.rs
 3. terminal-trading/src/wallet.rs
 4. terminal-trading/src/eip712.rs
 5. terminal-trading/src/order.rs
 6. terminal-trading/src/clob_client.rs
 7. terminal-trading/src/positions.rs
 8. terminal-trading/src/types.rs
 9. terminal-api/src/routes/trading.rs

 Modify:
 1. Cargo.toml (workspace - add terminal-trading)
 2. terminal-api/Cargo.toml (add terminal-trading dependency)
 3. terminal-api/src/main.rs (add trading routes)
 4. frontend/src/lib/api.ts (add trading API client)
 5. frontend/src/components/market/trade-execution.tsx (wire up to API)

 ---
 Testing Strategy

 1. Unit tests for EIP-712 signing (compare with known good signatures)
 2. Integration tests against CLOB testnet (if available) or mainnet with small amounts
 3. Manual testing with $1-5 orders first
 4. VPN required for US-based testing

 ---
 Security Considerations

 1. Private key storage: Keep in env var, never commit to git
 2. Rate limiting: Implement on API endpoints
 3. Order validation: Validate prices (0.01-0.99), sizes (positive)
 4. Error handling: Don't leak wallet info in error messages

 ---
 Sources

 - https://docs.polymarket.com/developers/CLOB/authentication
 - https://docs.polymarket.com/developers/CLOB/orders/create-order
 - https://docs.polymarket.com/developers/proxy-wallet
 - https://docs.polymarket.com/developers/CTF/overview
 - https://github.com/polymarket/py-clob-client
 - https://github.com/polymarket/clob-client
 - https://coindesk.com/policy/2024/11/14/polymarkets-probe-highlights-challenges-of-blocking-us-users-and-their-vpns/amp
 - https://www.theblock.co/linked/131383/following-cftc-settlement-prediction-platform-polymarket-geoblocks-trades-in-the-us