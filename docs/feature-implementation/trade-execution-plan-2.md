 Polymarket Trade Execution Integration Plan

 Overview

 Enable end-to-end trade execution on Polymarket through the terminal. The backend is fully implemented; we need to
 wire up the frontend and test carefully with real money.

 Approach: Single demo wallet (configured via TRADING_PRIVATE_KEY), step-by-step verification at each phase.

 ---
 Current State

 Backend: COMPLETE

 - terminal-trading/ - Wallet, CLOB client, order signing, balance queries
 - terminal-api/src/routes/trading.rs - All API endpoints implemented:
   - POST /api/trade/order - Submit order
   - DELETE /api/trade/order/{id} - Cancel order
   - GET /api/trade/orders - Open orders
   - GET /api/trade/balance - USDC balance + allowance
   - GET /api/trade/deposit - Deposit address
   - GET /api/trade/positions - User positions

 Frontend: UI BUILT BUT DISABLED

 - TradeExecution component exists with buy/sell form
 - Submit button disabled with "Trading not yet available"
 - API client methods fully defined in api.ts (lines 527-667)
 - Missing: tokenId prop, form submission wiring, balance display

 ---
 Phase 1: Backend Verification (No Code Changes)

 Goal: Verify all backend endpoints work before touching frontend.

 Step 1.1: Environment Setup

 1. Generate a new wallet private key OR use an existing one
 2. Add to .env.local:
 TRADING_PRIVATE_KEY=0x...
 3. Restart backend: cargo watch -w terminal-api -w terminal-trading -x 'run -p terminal-api'

 Step 1.2: Test Deposit Address

 curl http://localhost:3001/api/trade/deposit
 Expected: {"address":"0x...","network":"Polygon","token":"USDC.e"}

 Step 1.3: Test Balance (Before Funding)

 curl http://localhost:3001/api/trade/balance
 Expected: {"usdcBalance":"0.00","usdcAllowance":"0.00","walletAddress":"0x..."}

 Step 1.4: Fund the Wallet

 1. Copy deposit address from Step 1.2
 2. Send $5-10 USDC.e on Polygon from external wallet
 3. Re-test balance endpoint to confirm receipt

 Step 1.5: USDC Allowance (CRITICAL)

 If usdcAllowance is "0.00", orders will fail.

 We'll add a backend endpoint for approval:
 curl -X POST http://localhost:3001/api/trade/approve

 This will send an on-chain transaction to approve USDC spending for the CTF Exchange.

 Backend changes needed:
 - Add approve_usdc() function to terminal-trading/src/balance.rs
 - Add POST /api/trade/approve endpoint to terminal-api/src/routes/trading.rs

 Re-test balance to confirm allowance is now non-zero.

 Step 1.6: Test Order Submission (Small Test)

 Find a liquid market's token ID and submit a small limit order:
 curl -X POST http://localhost:3001/api/trade/order \
   -H "Content-Type: application/json" \
   -d '{
     "tokenId": "<CLOB_TOKEN_ID>",
     "side": "buy",
     "price": 0.05,
     "size": 1.0,
     "orderType": "GTC"
   }'

 Phase 1 Checkpoint

 - Deposit address endpoint works
 - Balance endpoint shows funded wallet
 - USDC allowance is set
 - Test order submission succeeds (or returns meaningful error)

 ---
 Phase 2: Frontend Trading Enablement

 Goal: Wire up the trade form to submit orders.

 Step 2.1: Update TradeExecution Component

 File: frontend/src/components/market/trade-execution.tsx

 Changes:
 1. Add props: tokenId?: string, marketTitle?: string, onOrderSubmitted?: () => void
 2. Add state: isSubmitting, submitError, submitSuccess
 3. Enable submit button when tokenId is provided
 4. Wire up form submission to api.submitOrder()
 5. Add loading spinner during submission
 6. Show success/error feedback

 Step 2.2: Add Order Confirmation Modal

 New file: frontend/src/components/market/order-confirmation-modal.tsx

 Features:
 - Display order details (side, amount, price, estimated cost)
 - "Real money" warning message
 - Confirm/Cancel buttons
 - Shows estimated cost in USD

 Step 2.3: Pass tokenId from Parent Components

 File: frontend/src/components/market/views/trading-view.tsx

 Change line 116-121:
 <TradeExecution
   yesPrice={currentYesPrice}
   noPrice={currentNoPrice}
   trades={trades}
   tokenId={market.clob_token_id}  // ADD THIS
   marketTitle={market.title}       // ADD THIS
   className="h-full"
 />

 File: frontend/src/components/market/views/multi-outcome-trading-view.tsx

 Similar change for multi-outcome markets using selectedOutcome.clob_token_id

 Phase 2 Checkpoint

 - TradeExecution accepts and uses tokenId
 - Submit button is enabled when tokenId exists
 - Confirmation modal appears before submission
 - Order submission calls API
 - Success/error feedback displayed

 ---
 Phase 3: Balance, Deposit & Positions UI

 Goal: Full trading dashboard with account info.

 Step 3.1: Create useTradingBalance Hook

 New file: frontend/src/hooks/use-trading-balance.ts

 // Polls balance every 30s
 // Returns: { balance, allowance, address, isLoading, error, refetch }

 Step 3.2: Add Balance Display to TradeExecution

 Modify TradeExecution to:
 1. Show available USDC balance above amount input
 2. Disable submit if amount > balance
 3. Show "Insufficient balance" warning
 4. Show allowance warning if 0

 Step 3.3: Create Deposit Panel Component

 New file: frontend/src/components/trading/deposit-panel.tsx

 Features:
 - Display deposit address (truncated with copy button)
 - QR code for address
 - Network: Polygon, Token: USDC.e info
 - Link to Polygon bridge for users without USDC.e

 Step 3.4: Create Open Orders Panel

 New file: frontend/src/components/trading/open-orders-panel.tsx

 Features:
 - List of open orders (id, market, side, price, size, status)
 - Cancel button per order
 - "Cancel All" button
 - Auto-refresh every 10s

 Step 3.5: Create Positions Panel

 New file: frontend/src/components/trading/positions-panel.tsx

 Features:
 - List positions (market, shares, avg price, current price, P&L)
 - Click to navigate to market
 - Total portfolio value

 Step 3.6: Add Trading Info to Market Page

 Option A: Add collapsible panel below TradeExecution showing:
 - Balance / Deposit
 - Open Orders (for this market)
 - Position (for this market)

 Phase 3 Checkpoint

 - Balance displayed in trade form
 - Deposit address panel with copy/QR
 - Open orders panel with cancel functionality
 - Positions panel showing P&L
 - All panels integrated into market view

 ---
 Phase 4: Error Handling & Safety

 Goal: Robust error handling for real money trading.

 Step 4.1: Error Message Mapping

 Create user-friendly error messages:
 - "Trading not enabled" → "Trading is not available. Check server configuration."
 - "Invalid side" → "Please select Buy or Sell"
 - "insufficient balance" → "Insufficient USDC balance. Please deposit more."
 - "insufficient allowance" → "USDC spending not approved. Please approve first."

 Step 4.2: Client-side Validation

 Before submission:
 - Price between 0.01 and 0.99
 - Size > 0
 - Amount <= available balance
 - tokenId is valid (not empty)

 Step 4.3: Large Order Warnings

 For orders above thresholds:
 - Orders > $50: Show additional confirmation
 - Orders > $100: Require typing "CONFIRM"

 Step 4.4: Transaction Logging

 Log all trading actions to localStorage:
 - Order submissions (params, timestamp, result)
 - Cancellations
 - Errors

 Display in developer console and optionally in UI.

 Phase 4 Checkpoint

 - User-friendly error messages
 - Client-side validation prevents invalid orders
 - Large order warnings implemented
 - Transaction logging in place

 ---
 Critical Files to Modify

 Backend

 - terminal-trading/src/balance.rs - Add approve_usdc() function for on-chain approval
 - terminal-api/src/routes/trading.rs - Add POST /api/trade/approve endpoint

 Frontend Changes

 1. frontend/src/components/market/trade-execution.tsx - Enable trading, add props
 2. frontend/src/components/market/views/trading-view.tsx - Pass tokenId
 3. frontend/src/components/market/views/multi-outcome-trading-view.tsx - Pass tokenId
 4. NEW frontend/src/components/market/order-confirmation-modal.tsx
 5. NEW frontend/src/hooks/use-trading-balance.ts
 6. NEW frontend/src/components/trading/deposit-panel.tsx
 7. NEW frontend/src/components/trading/open-orders-panel.tsx
 8. NEW frontend/src/components/trading/positions-panel.tsx

 ---
 Known Gotchas

 1. USDC Allowance: Must be approved before any trade works. We'll add POST /api/trade/approve endpoint to handle
 this. Requires small amount of MATIC for gas.
 2. Token ID: For binary markets, use market.clob_token_id. For multi-outcome, use selectedOutcome.clob_token_id.
 3. API Key Derivation: First order triggers L1 auth to derive API key. This may take a few seconds and could fail if
  Polymarket API is down.
 4. Neg Risk Markets: Most Polymarket markets use Neg Risk adapter. The OrderBuilder defaults to is_neg_risk: true.
 5. Polygon Gas: The wallet needs a tiny amount of MATIC for the approval transaction. Consider adding MATIC check to
  balance display.

 ---
 Testing Strategy

 1. Phase 1: Manual curl testing - verify all endpoints
 2. Phase 2: Submit tiny orders ($0.10) on liquid markets
 3. Phase 3: Verify balance updates after trades
 4. Phase 4: Test error scenarios (insufficient balance, invalid price, etc.)

 Each phase should be verified working before proceeding to the next.