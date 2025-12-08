# Demo Talking Points (5 minutes)

These are topics and things to show during your demo video - not a full script.

---

## 1. Introduction (30 seconds)

**What to say:**
- "I built a Prediction Market Terminal that aggregates Kalshi and Polymarket into one interface"
- "Forked barter-rs, a Rust algorithmic trading framework"
- "Learned Rust from scratch this week"

**What to show:**
- The running frontend (markets grid)

---

## 2. What I Started With (1 minute)

**What to say:**
- "Barter-rs is a Rust trading framework for crypto exchanges"
- "It already had connectors for Binance, Coinbase, Kraken, etc."
- "But no support for prediction markets - that's what I added"

**What to show:**
- `README.md` at root - show the original barter-rs description
- `barter-data/src/exchange/` folder - show the 8 exchange directories
- Maybe open `barter-data/src/exchange/coinbase/mod.rs` briefly to show the Connector pattern

**Key point:** Each crypto exchange had dedicated code. I built the same thing for prediction markets.

---

## 3. Frontend Demo (2 minutes)

**Markets Grid:**
- Show the main page with markets from both platforms
- Point out the platform badges (green = Kalshi, blue = Polymarket)
- Use the platform filter to toggle between them
- Note the real-time connection indicator in the header

**Market Detail Page:**
- Click into an interesting market
- Show the **price chart** with timeframe toggles (1H, 24H, 7D, etc.)
- Show the **order book** - explain YES side vs NO side, bids vs asks
- Show the **trade history** updating
- Point out the market metadata (volume, close time, category)

**If available - Multi-outcome market:**
- Show a market with multiple outcomes (like Bitcoin price brackets)
- Explain how this required special handling

---

## 4. Technical Highlight (1 minute)

**What to say:**
- "Let me show you some of the Rust code I wrote"
- "About 9,400 lines across 5 new crates"

**What to show:**
- `terminal-kalshi/src/client.rs` (line 1-60) - "This is the Kalshi API client"
- `terminal-services/src/aggregator.rs` (line 1-50) - "This is the fan-out architecture for real-time data"

**Key technical points to mention:**
- RSA-PSS authentication for Kalshi WebSocket
- HMAC-SHA256 for Polymarket
- SQLite for storing trades
- WebSocket fan-out pattern (one connection per client, filtered broadcasts)

---

## 5. Reflection (30 seconds)

**What to say:**
- "Learned Rust from scratch - the ownership model was the biggest hurdle"
- "AI helped me understand the barter-rs architecture and debug async code"
- "The brownfield approach forced me to understand existing patterns before extending"
- "Multi-outcome markets and data unification were the hardest technical challenge"

---

## Things to Prepare Before Recording

1. **Backend running**: `cargo run -p terminal-api`
2. **Frontend running**: `cd frontend && bun run dev`
3. **Good markets visible**: Make sure there are interesting markets showing
4. **Code editor open**: Have the key files ready to show
5. **Quiet environment**: No notifications

---

## Backup Topics (if time permits)

- Show the `terminal-core/src/market.rs` types
- Explain the WebSocket subscription protocol
- Show the candle service generating price history
- Mention future plans (trading, alerts, portfolio)
