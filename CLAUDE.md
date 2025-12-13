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
- `terminal-news/` - News aggregation (RSS feeds, Google News, Exa.ai, Firecrawl)
- `terminal-services/` - Business logic layer:
  - `MarketService` - Unified market data access
  - `MarketDataAggregator` - WebSocket connections to exchanges, broadcasts to frontend
  - `TradeCollector` - Background trade fetching and storage
  - `TradeStorage` - SQLite persistence for trades
  - `CandleService` - Price history/candlestick generation
  - `WebSocketState` - Frontend client subscription management
  - `NewsService` - News aggregation with caching and relevance filtering
- `terminal-api/` - Axum HTTP server + WebSocket endpoint

### Data Flow

1. **REST API**: Frontend → `terminal-api` → `MarketService` → platform clients → exchange APIs
2. **WebSocket Real-time**:
   - Exchange WebSockets → `MarketDataAggregator` → `WebSocketState` → Frontend clients
   - Frontend subscribes via `/ws` endpoint with JSON messages
3. **News Aggregation**:
   - **Global news**: RSS feeds (25+ sources) with round-robin diversification
   - **Market-specific news**: Google News RSS (primary) → Exa.ai (fallback) → RSS entity matching
   - Caching: 60s for RSS, 5 min for good results, 1 min for empty results
   - Relevance filtering: Geography-based + multi-term matching + title length validation

### Frontend Structure (Next.js 15 + React 19)

```
frontend/src/
├── app/              # Next.js App Router pages
│   ├── page.tsx      # Markets list
│   ├── market/       # Market detail view
│   └── news/         # Global news feed
├── components/
│   ├── ui/           # shadcn/ui primitives
│   ├── market/       # Market-specific components (chart, orderbook, trades)
│   ├── markets-grid/ # Market listing components
│   └── news/         # News components (market-news-section, news-card)
├── hooks/
│   ├── use-websocket.ts    # WebSocket connection management
│   ├── use-market-stream.ts # Real-time market data hook
│   └── use-news-stream.ts  # Real-time news updates
├── providers/
│   └── websocket-provider.tsx # Global WebSocket context
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
- `GET /api/markets/:platform/:id/news` - Market-specific news (Google News + filtering)
- `GET /api/news` - Global news feed (RSS with round-robin diversification)
- `WS /ws` - Real-time subscriptions (orderbook, trades, market updates, news)

### WebSocket Protocol

Frontend sends subscription messages:
```json
{"type": "Subscribe", "channel": {"type": "Orderbook", "platform": "kalshi", "market_id": "..."}}
{"type": "Subscribe", "channel": {"type": "News", "platform": "kalshi", "market_id": "..."}}
```

Server broadcasts updates:
```json
{"type": "OrderbookUpdate", "update_type": "Snapshot", "orderbook": {...}}
{"type": "NewsUpdate", "feed": {...}}
```

## Code Conventions

- **Rust**: Standard formatting (`rustfmt.toml` present), workspace dependencies in root `Cargo.toml`
- **TypeScript**: Arrow functions preferred, Tailwind CSS for styling, shadcn/ui components
- **Types**: `terminal-core` types are serialized to JSON and mirrored in `frontend/src/lib/types.ts`

## News Aggregation Architecture

### Global News Feed (RSS)
- **Source**: 25+ curated feeds (AP, BBC, Reuters, CNN, ESPN, etc.)
- **Strategy**: Round-robin source diversification to prevent single-source domination
- **Caching**: 60 seconds
- **Implementation**: `terminal-news/src/rss_client.rs` lines 233-324
- **Thumbnails**: Extracted from article's og:image meta tag, video embeds, or img tags

### Market-Specific News (Google News + Fallback Chain)
- **Primary**: Google News RSS with dynamic queries
  - Free, unlimited, no API key needed
  - Fresh news (minutes old)
  - Query building: Extract key terms + context terms (climate→emissions, election→polls)
  - Expand abbreviations: "US" → "United States"
  - Implementation: `terminal-news/src/google_news.rs`
- **Fallback**: Exa.ai (optional, requires API key) → RSS entity matching
- **Relevance Filtering**:
  - Title length filter: reject < 15 chars (catches parsing errors)
  - Geography-specific: US markets require "United States"/"U.S."/"America" in text
  - Multi-term matching: require at least 2 key terms (or 1 if geography verified)
  - Implementation: `terminal-services/src/news_service.rs` lines 337-446
- **Caching**: 5 minutes for good results, 1 minute for empty results
- **Critical Fix**: Important terms ("US", "UK", "EU") checked BEFORE length validation to prevent filtering out country codes

### Key Files for News
- `terminal-news/src/google_news.rs` - Google News client with query building
- `terminal-news/src/rss_client.rs` - RSS aggregation with diversification
- `terminal-services/src/news_service.rs` - News service with caching and filtering
- `frontend/src/components/news/` - News UI components
- thumbnail extraction