# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a **Prediction Market Terminal** - a full-stack application that aggregates prediction market data from Kalshi and Polymarket into a unified interface. The project is built on top of the Barter trading ecosystem (Rust libraries for algorithmic trading).

## Development Commands

### Frontend (Next.js)
```bash
cd frontend
bun install              # Install dependencies
bun run dev --turbopack  # Start dev server (usually already running)
bun run dev:backend      # Run Rust backend with hot reload (from frontend dir)
bun run build            # Production build
bun run lint             # ESLint
```

### Backend (Rust)
```bash
cargo build                    # Build all crates
cargo build -p terminal-api    # Build just the API server
cargo run -p terminal-api      # Run the API server
cargo watch -w terminal-api -w terminal-core -w terminal-services -w terminal-kalshi -w terminal-polymarket -x 'run -p terminal-api'  # Dev with hot reload

# Testing
cargo test                     # Run all tests
cargo test -p terminal-core    # Test a single crate

# Formatting
cargo fmt                      # Format all code (uses rustfmt.toml: edition 2024, crate-level imports)
```

### Environment
- Backend expects `.env.local` at repository root with API credentials
- `TRADES_DB_PATH` defaults to `data/trades.db` (SQLite)
- `SERVER_PORT` defaults to 3001
- Frontend uses `NEXT_PUBLIC_API_URL` (defaults to `http://localhost:3001`)

## Architecture

### Rust Workspace Structure

The workspace contains two ecosystems:

**Barter Ecosystem** (upstream trading libraries):
- `barter/` - Core trading engine
- `barter-data/` - Market data streaming
- `barter-execution/` - Order execution
- `barter-instrument/` - Exchange/instrument definitions
- `barter-integration/` - REST/WebSocket framework

**Terminal Ecosystem** (prediction market app):
- `terminal-core/` - Shared types (Platform, PredictionMarket, OrderBook, Trade, WebSocket messages)
- `terminal-kalshi/` - Kalshi API client (REST + WebSocket, RSA-PSS auth)
- `terminal-polymarket/` - Polymarket API client (REST + WebSocket, HMAC auth)
- `terminal-services/` - Business logic layer:
  - `MarketService` - Unified market data access
  - `MarketDataAggregator` - WebSocket connections to exchanges, broadcasts to frontend
  - `TradeCollector` - Background trade fetching and storage
  - `TradeStorage` - SQLite persistence for trades
  - `CandleService` - Price history/candlestick generation
  - `WebSocketState` - Frontend client subscription management
- `terminal-api/` - Axum HTTP server + WebSocket endpoint

### Data Flow

1. **REST API**: Frontend → `terminal-api` → `MarketService` → platform clients → exchange APIs
2. **WebSocket Real-time**:
   - Exchange WebSockets → `MarketDataAggregator` → `WebSocketState` → Frontend clients
   - Frontend subscribes via `/ws` endpoint with JSON messages

### Frontend Structure (Next.js 15 + React 19)

```
frontend/src/
├── app/              # Next.js App Router pages
│   ├── page.tsx      # Markets list
│   └── market/       # Market detail view
├── components/
│   ├── ui/           # shadcn/ui primitives
│   ├── market/       # Market-specific components (chart, orderbook, trades)
│   └── markets-grid/ # Market listing components
├── hooks/
│   ├── use-websocket.ts    # WebSocket connection management
│   └── use-market-stream.ts # Real-time market data hook
└── lib/
    ├── api.ts        # REST API client
    └── types.ts      # TypeScript types (mirrors terminal-core)
```

### Key API Endpoints

- `GET /api/markets` - List markets (filters: platform, search, limit)
- `GET /api/markets/:platform/:id` - Single market
- `GET /api/markets/:platform/:id/orderbook` - Order book
- `GET /api/markets/:platform/:id/trades` - Recent trades
- `GET /api/markets/:platform/:id/history` - Price candles
- `WS /ws` - Real-time subscriptions (orderbook, trades, market updates)

### WebSocket Protocol

Frontend sends subscription messages:
```json
{"type": "Subscribe", "channel": {"type": "Orderbook", "platform": "kalshi", "market_id": "..."}}
```

Server broadcasts updates:
```json
{"type": "OrderbookUpdate", "update_type": "Snapshot", "orderbook": {...}}
```

## Code Conventions

- **Rust**: Standard formatting (`rustfmt.toml` present), workspace dependencies in root `Cargo.toml`
- **TypeScript**: Arrow functions preferred, Tailwind CSS for styling, shadcn/ui components
- **Types**: `terminal-core` types are serialized to JSON and mirrored in `frontend/src/lib/types.ts`
