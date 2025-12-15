  Trading Architecture

  This is a "backend-managed wallet" design, NOT a connect-wallet flow.

  How it works:

  1. You (the operator) generate a private key - This lives on your server in an environment variable (TRADING_PRIVATE_KEY)
  2. The backend owns this wallet - All signing happens server-side using that private key
  3. Users deposit funds to that wallet - The /api/trade/deposit endpoint returns the wallet address. Users send USDC.e (on Polygon) to this address.
  4. Trading is instant - When you call submitOrder(), the backend signs the order with EIP-712 and submits it directly to Polymarket's CLOB. No user signature popups, no MetaMask, no round-trips.

  The flow:

  ┌─────────────────────────────────────────────────────────────┐
  │                        YOUR SERVER                          │
  │                                                             │
  │   TRADING_PRIVATE_KEY=0xabc123...                          │
  │                    ↓                                        │
  │   ┌─────────────────────────────────────┐                  │
  │   │      Backend Wallet (you control)    │                  │
  │   │      Address: 0x742d35Cc...          │                  │
  │   └─────────────────────────────────────┘                  │
  │                    ↓                                        │
  │   Signs orders with EIP-712 → Polymarket CLOB              │
  └─────────────────────────────────────────────────────────────┘

  ┌─────────────────────────────────────────────────────────────┐
  │                      TO FUND IT                             │
  │                                                             │
  │   1. Get wallet address from /api/trade/deposit             │
  │   2. Send USDC.e on Polygon to that address                 │
  │   3. Approve CTF Exchange contract (one-time)               │
  └─────────────────────────────────────────────────────────────┘

  Why this design?

  - Speed - No waiting for user signatures
  - Simplicity - No wallet connection UI needed
  - Control - You manage the funds directly
  - Personal use - This is for YOUR trading, not a public platform

  To get started:

  # 1. Generate a new wallet (or use an existing one)
  # You can use any tool - here's a quick way with node:
  node -e "console.log('0x' + require('crypto').randomBytes(32).toString('hex'))"

  # 2. Add to .env.local at repo root
  echo "TRADING_PRIVATE_KEY=0xyour_private_key_here" >> .env.local

  # 3. Start the server
  cargo run -p terminal-api

  # 4. Get your deposit address
  curl http://localhost:3001/api/trade/deposit
  # Returns: {"address":"0x...","network":"Polygon","token":"USDC.e"}

  # 5. Send USDC.e to that address on Polygon
  # (Use MetaMask, exchange withdrawal, etc.)

  # 6. Check balance
  curl http://localhost:3001/api/trade/balance

  Does this make sense? Want me to create a quick setup script that generates a wallet and shows the deposit address?