⏺ The Flow to Submit a Polymarket Order

  What needs to happen:

  Step 1: Get API Credentials (L1 Auth)
  - Sign an EIP-712 message with your private key to prove you own the wallet
  - Call POST /auth/api-key or GET /auth/derive-api-key with that signature
  - Polymarket returns: api_key, secret, passphrase
  - ✅ This works - we successfully create API keys (logs show "API key created successfully")

  Step 2: Build the Order
  - Create order struct with: maker, taker, tokenId, amounts, side, etc.
  - Sign the order with EIP-712 (proves you authorize this trade)
  - ✅ This works - order format now matches Python client exactly

  Step 3: Submit Order (L2 Auth)
  - Compute HMAC-SHA256 signature over: timestamp + "POST" + "/order" + JSON_body
  - Send POST to /order with headers:
    - POLY_ADDRESS: your wallet address
    - POLY_API_KEY: the api_key from Step 1
    - POLY_PASSPHRASE: the passphrase from Step 1
    - POLY_SIGNATURE: the HMAC signature
    - POLY_TIMESTAMP: current unix timestamp
  - ❌ This fails with 401 "Invalid api key"

  What's broken:

  The L2 authentication is failing. We create an API key successfully, but when we try to USE it, Polymarket says it's invalid.

  Possible causes:
  1. The API key we get back isn't actually valid/registered
  2. The HMAC signature is wrong (but we verified it matches Python...)
  3. Something about our headers is wrong
  4. The credentials aren't being stored/used correctly

  What I just added:

  A test that calls GET /data/orders (a simple L2 endpoint) immediately after creating the API key. This will tell us:
  - If L2 auth works AT ALL with the fresh credentials
  - Or if the API key is DOA from the moment we create it

  Run it and look for "L2 auth test PASSED" or "L2 auth test FAILED" - that will narrow down where exactly the problem is.



⏺ Polymarket Trade Execution - Full Summary

  1. Rust Trading Library (terminal-trading)

  A complete Rust crate for Polymarket trading with:

  Core Components

  - wallet.rs - Wallet management (load from env, EIP-712 signing)
  - clob_client.rs - Polymarket CLOB API client with L1/L2 authentication
  - order.rs - Order building, signing, and submission
  - eip712.rs - EIP-712 typed data signing for Polymarket orders
  - balance.rs - USDC balance, allowance checking, and approvals
  - positions.rs - Position tracking

  Key Environment Variables

  POLYGON_PRIVATE_KEY=0x...        # Your trading wallet private key
  POLYGON_RPC_URL=https://...      # Polygon RPC endpoint

  ---
  2. CLI Testing Script

  File: terminal-trading/examples/trade_cli.rs

  # Run the CLI
  cargo run -p terminal-trading --example trade_cli

  CLI Commands

  | Command                        | Description                               |
  |--------------------------------|-------------------------------------------|
  | balance                        | Check USDC balance and allowances         |
  | approve                        | Approve USDC for all 3 exchange contracts |
  | buy <token_id> <price> <size>  | Buy YES shares                            |
  | sell <token_id> <price> <size> | Sell YES shares                           |
  | positions                      | View current positions                    |
  | help                           | Show commands                             |
  | quit                           | Exit                                      |

  Example Session

  > balance
  USDC Balance: 2.85
  Allowances: CTF=✓ NegRiskCTF=✓ NegRiskAdapter=✓

  > buy 71321... 0.05 20
  Submitting BUY order: 20 shares @ $0.05
  Order placed! ID: abc123...

  ---
  3. Issues Fixed

  Issue 1: Invalid Signature (negRisk)

  Problem: Multi-outcome markets use a different exchange contract (Neg Risk CTF Exchange) than binary markets (CTF Exchange). The EIP-712 signature must specify the correct verifying contract.

  Fix: Added negRisk parameter throughout the stack:
  - Frontend passes negRisk={true} for multi-outcome markets
  - API forwards it to the trading library
  - Order builder uses correct exchange address for signing

  Issue 2: Missing USDC Approval

  Problem: Multi-outcome markets require approval for 3 contracts, not just 2:
  1. CTF Exchange (binary markets)
  2. Neg Risk CTF Exchange (multi-outcome)
  3. Neg Risk Adapter (required for multi-outcome fills)

  Fix: Created approve_usdc_for_all_exchanges() function:
  // terminal-trading/src/balance.rs
  pub async fn approve_usdc_for_all_exchanges(wallet: &TradingWallet) -> Result<Vec<ApprovalResponse>>

  Issue 3: Wrong Price in UI (Orderbook Sorting)

  Problem: The CLOB API returns asks sorted descending (highest first). The UI grabbed yes_asks[0] thinking it was the best ask, but got 99.9¢ instead of 5¢.

  Fix: Added sorting in terminal-polymarket/src/types.rs:
  // Sort bids descending (best/highest bid first)
  bids.sort_by(|a, b| b.price.cmp(&a.price));

  // Sort asks ascending (best/lowest ask first)  
  asks.sort_by(|a, b| a.price.cmp(&b.price));

  Issue 4: Market Orders Using Wrong Price

  Problem: Market orders used currentPrice (mid-market) instead of actual ask price, causing orders to not fill.

  Fix: Added bestAsk and bestBid props to TradeExecution:
  const marketPrice = isBuy
    ? (bestAsk ?? currentPrice)
    : (bestBid ?? currentPrice);

  ---
  4. API Integration

  Endpoint: POST /api/trading/order

  File: terminal-api/src/routes/trading.rs

  {
    "token_id": "71321...",
    "side": "buy",
    "price": 0.05,
    "size": 20,
    "order_type": "GTC",
    "neg_risk": true
  }

  Endpoint: POST /api/trading/approve

  Approves USDC for all 3 exchange contracts in one call.

  Endpoint: GET /api/trading/balance

  Returns USDC balance and allowance status.

  ---
  5. Frontend Integration

  TradeExecution Component

  File: frontend/src/components/market/trade-execution.tsx

  Props Added:
  interface TradeExecutionProps {
    // ... existing props
    negRisk?: boolean;      // Multi-outcome market flag
    bestAsk?: number;       // From orderbook for market buys
    bestBid?: number;       // From orderbook for market sells
  }

  Usage in Views

  Binary markets (trading-view.tsx):
  <TradeExecution
    negRisk={false}
    bestAsk={orderBook?.yes_asks?.[0] ? parseFloat(orderBook.yes_asks[0].price) : undefined}
    bestBid={orderBook?.yes_bids?.[0] ? parseFloat(orderBook.yes_bids[0].price) : undefined}
  />

  Multi-outcome markets (multi-outcome-trading-view.tsx):
  <TradeExecution
    negRisk={true}
    bestAsk={orderBook?.yes_asks?.[0] ? parseFloat(orderBook.yes_asks[0].price) : undefined}
    bestBid={orderBook?.yes_bids?.[0] ? parseFloat(orderBook.yes_bids[0].price) : undefined}
  />

  Toast Notifications

  Using sonner for trade feedback:
  - Loading: "Submitting order..."
  - Success: "Order placed! ID: abc123..."
  - Error: "Order failed: [error message]"

  ---
  6. Contract Addresses (Polygon)

  // USDC on Polygon
  const USDC_ADDRESS: &str = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";

  // Polymarket Exchange Contracts
  const CTF_EXCHANGE: &str = "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E";
  const NEG_RISK_CTF_EXCHANGE: &str = "0xC5d563A36AE78145C45a50134d48A1215220f80a";
  const NEG_RISK_ADAPTER: &str = "0xd91E80cF2E7be2e162c6513ceD06f1dD0dA35296";

  ---
  7. Your Successful Trades

  | Market                     | Shares | Price | Cost  | Status    |
  |----------------------------|--------|-------|-------|-----------|
  | Chloé Zhao (Best Director) | 20     | 5¢    | $1.00 | ✅ Filled |
  | US Recession 2025          | 10     | 0.7¢  | $0.07 | ✅ Filled |

  Total Position Value: $1.04

  ---
  8. Files Modified

  | File                                                                | Changes                                        |
  |---------------------------------------------------------------------|------------------------------------------------|
  | terminal-trading/src/balance.rs                                     | Added approve_usdc_for_all_exchanges()         |
  | terminal-trading/src/lib.rs                                         | Export new function                            |
  | terminal-polymarket/src/types.rs                                    | Fixed orderbook sorting                        |
  | terminal-api/src/routes/trading.rs                                  | Updated approve endpoint, added negRisk        |
  | frontend/src/components/market/trade-execution.tsx                  | Added negRisk, bestAsk, bestBid props + toasts |
  | frontend/src/lib/api.ts                                             | Added negRisk to submitOrder                   |
  | frontend/src/components/market/views/trading-view.tsx               | Pass negRisk=false, bestAsk, bestBid           |
  | frontend/src/components/market/views/multi-outcome-trading-view.tsx | Pass negRisk=true, bestAsk, bestBid            |

  ---
  Quick Reference

  # Test CLI trading
  cargo run -p terminal-trading --example trade_cli

  # Build backend
  cargo build -p terminal-api

  # Run backend with hot reload
  cargo watch -w terminal-api -w terminal-core -w terminal-services -w terminal-trading -w terminal-polymarket -x 'run -p terminal-api'