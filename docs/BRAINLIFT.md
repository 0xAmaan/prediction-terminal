# Prediction Market Terminal - Brainlift

## Project Summary

Forked **barter-rs** (a Rust algorithmic trading framework for crypto exchanges) and built a **Prediction Market Terminal** that aggregates real-time data from Kalshi and Polymarket into a unified trading interface.

**Repository**: Fork of [barter-rs/barter-rs](https://github.com/barter-rs/barter-rs)

---

## What Was Barter-rs?

Barter is a Rust ecosystem for building high-performance algorithmic trading systems. Before my modifications, it:

- Supported **8 crypto exchanges**: Binance, Coinbase, Kraken, OKX, Bybit, Gateio, Bitfinex, BitMEX
- Provided WebSocket connectors for real-time market data (trades, orderbooks)
- Included a trading engine with pluggable Strategy/RiskManager components
- Was designed for live trading, paper trading, and backtesting

Each exchange had dedicated code in `barter-data/src/exchange/` implementing the `Connector` trait with exchange-specific WebSocket URLs, subscription formats, and data transformers.

**Key insight**: Barter was built for crypto - it had no support for prediction markets like Kalshi or Polymarket.

---

## What I Built

### Rust Backend (~9,400 lines across 5 new crates)

| Crate | Purpose |
|-------|---------|
| `terminal-core` | Shared types: PredictionMarket, OrderBook, Trade, WebSocket protocol |
| `terminal-kalshi` | Kalshi REST API + WebSocket client with RSA-PSS authentication |
| `terminal-polymarket` | Polymarket Gamma API + WebSocket client with HMAC-SHA256 auth |
| `terminal-services` | MarketService, MarketDataAggregator, TradeStorage (SQLite), CandleService |
| `terminal-api` | Axum HTTP server + WebSocket endpoint |

### Frontend (~7,500 lines of TypeScript/React)

- **Next.js 16 + React 19** application
- **Real-time WebSocket** integration for orderbooks, trades, prices
- **TradingView Lightweight Charts** for price history
- **Luxury terminal UI** with dark theme
- Market cards supporting binary, multi-outcome, and sports markets

---

## Technical Highlights

### 1. Platform Authentication
- **Kalshi**: RSA-PSS signing for WebSocket connections
- **Polymarket**: HMAC-SHA256 signature scheme for authenticated endpoints

### 2. Real-Time Fan-Out Architecture
```
Kalshi WebSocket ─┐
                  ├─→ MarketDataAggregator ─→ SubscriptionManager ─→ Client WebSockets
Polymarket WS ────┘
```
- Backend maintains persistent connections to both exchanges
- Normalizes data into unified format
- Single WebSocket connection per frontend client
- Subscription protocol routes updates to interested clients

### 3. Data Persistence
- SQLite storage for trade history
- CandleService generates OHLCV candles from stored trades
- Supports 6 timeframes: 1m, 5m, 15m, 1h, 4h, 1d

### 4. Multi-Outcome Market Support
- Kalshi markets can have multiple outcomes (not just Yes/No)
- Implemented market grouping to aggregate individual outcomes
- Frontend supports multi-outcome cards and charts

---

## Hardest Challenge

**Multi-outcome markets and data unification.** Kalshi has markets like "Bitcoin price on Dec 31" that are actually part of a larger event with multiple price brackets ($80K-$90K, $90K-$100K, etc.). Each bracket is technically a separate market.

Aggregating these into a unified view while maintaining individual outcome tracking required:
- Market grouping logic in the Kalshi client
- Extended data types to support outcome arrays
- Frontend components that adapt between binary and multi-outcome displays

---

## AI Usage

- **Understanding Rust**: Ownership, borrowing, lifetimes - the concepts that make Rust different
- **Debugging async code**: Tokio runtime issues, WebSocket reconnection logic
- **Architecture design**: Fan-out pattern, subscription management
- **API exploration**: Understanding Kalshi/Polymarket authentication schemes

---

## What I Learned

1. **Rust from scratch in a week** - The compiler is strict but helpful once you understand it
2. **Real-time data architecture** - Fan-out patterns, connection health monitoring, graceful degradation
3. **Prediction market APIs** - Each platform has quirks (Kalshi's RSA-PSS, Polymarket's event model)
4. **Brownfield development** - Understanding existing architecture before extending it

---

## Project Statistics

| Metric | Value |
|--------|-------|
| New Rust code | ~9,400 lines |
| Frontend code | ~7,500 lines |
| Original barter-rs | ~30,000 lines |
| New crates added | 5 |
| Platforms integrated | 2 (Kalshi, Polymarket) |
| API endpoints | 8 REST + 1 WebSocket |
