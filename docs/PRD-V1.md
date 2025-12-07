# Prediction Market Trading Terminal

## Product Requirements Document

**Project Name:** Prediction Terminal (working title)
**Author:** Amaan
**Duration:** 1 week (Gauntlet Challenge)
**Base Repository:** Fork of [barter-rs](https://github.com/barter-rs/barter-rs)

---

## 1. Executive Summary

### What We're Building

A unified trading terminal that aggregates prediction markets from Kalshi and Polymarket, allowing users to:

- View markets across both platforms in one interface
- Compare prices for equivalent events
- Execute trades on either platform
- Track their portfolio across platforms
- Stay informed with relevant news

### Why It Matters

Currently, traders must juggle two separate platforms with different UIs, different data formats, and no cross-platform visibility. This terminal solves that friction.

### Success Criteria

- [ ] Users can connect their Kalshi + Polymarket accounts
- [ ] Markets from both platforms displayed in unified view
- [ ] Price comparison for matched markets works
- [ ] Users can execute trades via the terminal
- [ ] Portfolio aggregates positions from both platforms
- [ ] News feed shows relevant market context
- [ ] Deployed or trivially runnable

---

## 2. User Stories

### Core User Stories (MVP)

```
US-1: As a trader, I want to see all prediction markets in one place
      so I don't have to switch between platforms.

US-2: As a trader, I want to compare prices for the same event across platforms
      so I can find the best price.

US-3: As a trader, I want to execute trades from the terminal
      so I don't have to leave the app.

US-4: As a trader, I want to see my positions across both platforms
      so I know my total exposure.

US-5: As a trader, I want to see relevant news for markets I'm watching
      so I can make informed decisions.
```

### Extended User Stories (Post-MVP)

```
US-6: As a trader, I want price alerts when spreads exceed thresholds.

US-7: As a trader, I want to see historical price charts.

US-8: As a trader, I want to save my credentials securely between sessions.

US-9: As a trader, I want to filter markets by category (politics, crypto, sports).
```

---

## 3. Technical Architecture

### 3.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              FRONTEND                                       │
│                         (Future: Next.js)                                   │
│                                                                             │
│   Market Browser │ Portfolio │ Trade Execution │ News Feed │ Settings      │
└─────────────────────────────────┬───────────────────────────────────────────┘
                                  │
                                  │ HTTP REST + WebSocket
                                  │
┌─────────────────────────────────▼───────────────────────────────────────────┐
│                           RUST BACKEND                                      │
│                        (barter-rs fork)                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                         API LAYER (Axum)                             │  │
│  │                                                                      │  │
│  │  REST Endpoints:                    WebSocket:                       │  │
│  │  - GET  /api/markets                - /ws/prices (live updates)     │  │
│  │  - GET  /api/markets/:id            - /ws/portfolio (position updates)│ │
│  │  - GET  /api/portfolio                                               │  │
│  │  - POST /api/orders                                                  │  │
│  │  - DELETE /api/orders/:id                                            │  │
│  │  - GET  /api/news                                                    │  │
│  │  - POST /api/auth/connect                                            │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                      │                                      │
│  ┌───────────────────────────────────▼──────────────────────────────────┐  │
│  │                        SERVICE LAYER                                 │  │
│  │                                                                      │  │
│  │  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────────────┐ │  │
│  │  │ MarketService   │ │ TradingService  │ │ PortfolioService        │ │  │
│  │  │                 │ │                 │ │                         │ │  │
│  │  │ - list_markets  │ │ - place_order   │ │ - get_positions         │ │  │
│  │  │ - get_market    │ │ - cancel_order  │ │ - get_balances          │ │  │
│  │  │ - match_markets │ │ - get_orders    │ │ - calculate_pnl         │ │  │
│  │  │ - get_spreads   │ │                 │ │                         │ │  │
│  │  └─────────────────┘ └─────────────────┘ └─────────────────────────┘ │  │
│  │                                                                      │  │
│  │  ┌─────────────────┐ ┌─────────────────┐                            │  │
│  │  │ NewsService     │ │ SessionManager  │                            │  │
│  │  │                 │ │                 │                            │  │
│  │  │ - fetch_news    │ │ - store_creds   │                            │  │
│  │  │ - match_to_mkt  │ │ - get_client    │                            │  │
│  │  └─────────────────┘ └─────────────────┘                            │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                      │                                      │
│  ┌───────────────────────────────────▼──────────────────────────────────┐  │
│  │                      PLATFORM ADAPTERS                               │  │
│  │                                                                      │  │
│  │  ┌─────────────────────────┐    ┌─────────────────────────┐         │  │
│  │  │    KalshiAdapter        │    │   PolymarketAdapter     │         │  │
│  │  │                         │    │                         │         │  │
│  │  │  impl PlatformClient    │    │  impl PlatformClient    │         │  │
│  │  │                         │    │                         │         │  │
│  │  │  Wraps: kalshi-rs       │    │  Wraps: polyfill-rs     │         │  │
│  │  │                         │    │                         │         │  │
│  │  │  - list_markets()       │    │  - list_markets()       │         │  │
│  │  │  - get_orderbook()      │    │  - get_orderbook()      │         │  │
│  │  │  - place_order()        │    │  - place_order()        │         │  │
│  │  │  - cancel_order()       │    │  - cancel_order()       │         │  │
│  │  │  - get_positions()      │    │  - get_positions()      │         │  │
│  │  │  - get_balance()        │    │  - get_balance()        │         │  │
│  │  └─────────────────────────┘    └─────────────────────────┘         │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                    │                               │
                    ▼                               ▼
           ┌───────────────┐               ┌───────────────┐
           │  Kalshi API   │               │ Polymarket    │
           │               │               │ CLOB API      │
           │ REST + WS     │               │ REST + WS     │
           └───────────────┘               └───────────────┘
```

### 3.2 Project Structure

```
prediction-terminal/
├── Cargo.toml                    # Workspace root
├── README.md
├── config/
│   └── market_mappings.json      # Manual market matching file
│
├── crates/
│   ├── terminal-core/            # Core domain types
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── market.rs         # UnifiedMarket, MarketPair
│   │       ├── order.rs          # Order, OrderRequest, OrderResult
│   │       ├── position.rs       # Position, Portfolio
│   │       ├── platform.rs       # Platform enum, PlatformClient trait
│   │       └── news.rs           # NewsItem
│   │
│   ├── terminal-kalshi/          # Kalshi adapter
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── client.rs         # KalshiAdapter impl
│   │       ├── models.rs         # Kalshi-specific types
│   │       └── convert.rs        # Kalshi -> Unified conversions
│   │
│   ├── terminal-polymarket/      # Polymarket adapter
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── client.rs         # PolymarketAdapter impl
│   │       ├── models.rs         # Polymarket-specific types
│   │       └── convert.rs        # Polymarket -> Unified conversions
│   │
│   ├── terminal-services/        # Business logic
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── market_service.rs
│   │       ├── trading_service.rs
│   │       ├── portfolio_service.rs
│   │       ├── news_service.rs
│   │       ├── market_matcher.rs
│   │       └── session.rs
│   │
│   ├── terminal-api/             # HTTP API layer
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── server.rs
│   │       ├── routes/
│   │       │   ├── mod.rs
│   │       │   ├── markets.rs
│   │       │   ├── orders.rs
│   │       │   ├── portfolio.rs
│   │       │   ├── news.rs
│   │       │   └── auth.rs
│   │       ├── websocket.rs
│   │       └── errors.rs
│   │
│   └── terminal-cli/             # CLI interface
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           └── commands/
│               ├── mod.rs
│               ├── markets.rs
│               ├── trade.rs
│               ├── portfolio.rs
│               ├── watch.rs
│               └── config.rs
│
└── tests/
    └── integration/
```

### 3.3 Core Data Models

```rust
// ============== terminal-core/src/platform.rs ==============

use async_trait::async_trait;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    Kalshi,
    Polymarket,
}

#[async_trait]
pub trait PlatformClient: Send + Sync {
    // Market data
    async fn list_markets(&self) -> Result<Vec<PlatformMarket>, PlatformError>;
    async fn get_market(&self, id: &str) -> Result<PlatformMarket, PlatformError>;
    async fn get_orderbook(&self, id: &str) -> Result<OrderBook, PlatformError>;
    
    // Trading
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResult, PlatformError>;
    async fn cancel_order(&self, order_id: &str) -> Result<(), PlatformError>;
    async fn get_open_orders(&self) -> Result<Vec<Order>, PlatformError>;
    
    // Portfolio
    async fn get_positions(&self) -> Result<Vec<Position>, PlatformError>;
    async fn get_balance(&self) -> Result<Balance, PlatformError>;
}

// ============== terminal-core/src/market.rs ==============

use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

/// A market from a specific platform (raw)
#[derive(Debug, Clone)]
pub struct PlatformMarket {
    pub platform: Platform,
    pub id: String,              // Platform-specific ID
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub status: MarketStatus,
    pub yes_price: Decimal,      // 0.00 - 1.00
    pub no_price: Decimal,
    pub yes_bid: Decimal,
    pub yes_ask: Decimal,
    pub volume: Decimal,
    pub close_time: Option<DateTime<Utc>>,
}

/// A unified market that may exist on multiple platforms
#[derive(Debug, Clone)]
pub struct UnifiedMarket {
    pub id: String,              // Our internal ID
    pub title: String,
    pub category: Option<String>,
    pub kalshi: Option<PlatformMarket>,
    pub polymarket: Option<PlatformMarket>,
    pub spread: Option<Decimal>, // Price difference if on both
}

#[derive(Debug, Clone)]
pub struct MarketMapping {
    pub unified_id: String,
    pub title: String,
    pub kalshi_id: Option<String>,
    pub polymarket_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketStatus {
    Open,
    Closed,
    Settled,
}

#[derive(Debug, Clone)]
pub struct OrderBook {
    pub market_id: String,
    pub platform: Platform,
    pub bids: Vec<PriceLevel>,  // Buy orders
    pub asks: Vec<PriceLevel>,  // Sell orders
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: Decimal,
    pub quantity: Decimal,
}

// ============== terminal-core/src/order.rs ==============

#[derive(Debug, Clone)]
pub struct OrderRequest {
    pub market_id: String,       // Platform-specific market ID
    pub platform: Platform,
    pub side: Side,
    pub outcome: Outcome,        // YES or NO
    pub quantity: Decimal,
    pub price: Option<Decimal>,  // None = market order
    pub order_type: OrderType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Yes,
    No,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Market,
    Limit,
    GoodTilCancelled,
    FillOrKill,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
    pub market_id: String,
    pub platform: Platform,
    pub side: Side,
    pub outcome: Outcome,
    pub quantity: Decimal,
    pub filled_quantity: Decimal,
    pub price: Decimal,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

#[derive(Debug, Clone)]
pub struct OrderResult {
    pub order: Order,
    pub message: Option<String>,
}

// ============== terminal-core/src/position.rs ==============

#[derive(Debug, Clone)]
pub struct Position {
    pub market_id: String,
    pub market_title: String,
    pub platform: Platform,
    pub outcome: Outcome,
    pub quantity: Decimal,
    pub avg_price: Decimal,
    pub current_price: Decimal,
    pub unrealized_pnl: Decimal,
}

#[derive(Debug, Clone)]
pub struct Balance {
    pub platform: Platform,
    pub available: Decimal,
    pub locked: Decimal,       // In open orders
    pub total: Decimal,
}

#[derive(Debug, Clone)]
pub struct Portfolio {
    pub positions: Vec<Position>,
    pub balances: Vec<Balance>,
    pub total_value: Decimal,
    pub total_pnl: Decimal,
}

// ============== terminal-core/src/news.rs ==============

#[derive(Debug, Clone)]
pub struct NewsItem {
    pub id: String,
    pub title: String,
    pub summary: Option<String>,
    pub url: String,
    pub source: String,
    pub published_at: DateTime<Utc>,
    pub related_markets: Vec<String>,  // Unified market IDs
}
```

### 3.4 API Specification

#### REST Endpoints

```yaml
# Authentication / Session
POST /api/auth/connect
  Request:
    {
      "kalshi_api_key": "string (optional)",
      "kalshi_private_key_path": "string (optional)",
      "polymarket_private_key": "string (optional)",
      "polymarket_funder_address": "string (optional)"
    }
  Response:
    {
      "session_id": "string",
      "connected_platforms": ["kalshi", "polymarket"]
    }

# Markets
GET /api/markets
  Query params:
    - platform: "kalshi" | "polymarket" | "all" (default: "all")
    - category: string (optional)
    - search: string (optional)
    - matched_only: boolean (default: false)
  Response:
    {
      "markets": [UnifiedMarket],
      "count": number
    }

GET /api/markets/:id
  Response: UnifiedMarket

GET /api/markets/:id/orderbook
  Query params:
    - platform: "kalshi" | "polymarket"
  Response: OrderBook

GET /api/spreads
  Query params:
    - min_spread: number (optional, filter by minimum spread %)
  Response:
    {
      "spreads": [
        {
          "market": UnifiedMarket,
          "spread_pct": number,
          "kalshi_yes": number,
          "polymarket_yes": number
        }
      ]
    }

# Trading
POST /api/orders
  Request: OrderRequest
  Response: OrderResult

GET /api/orders
  Query params:
    - platform: "kalshi" | "polymarket" | "all"
    - status: "open" | "filled" | "all"
  Response:
    {
      "orders": [Order]
    }

DELETE /api/orders/:id
  Query params:
    - platform: "kalshi" | "polymarket"
  Response:
    { "success": boolean }

# Portfolio
GET /api/portfolio
  Response: Portfolio

GET /api/portfolio/positions
  Query params:
    - platform: "kalshi" | "polymarket" | "all"
  Response:
    { "positions": [Position] }

GET /api/portfolio/balances
  Response:
    { "balances": [Balance] }

# News
GET /api/news
  Query params:
    - market_id: string (optional, filter by market)
    - limit: number (default: 20)
  Response:
    { "items": [NewsItem] }
```

#### WebSocket Channels

```yaml
# Connect
WS /ws

# Subscribe to price updates
-> { "type": "subscribe", "channel": "prices", "market_ids": ["id1", "id2"] }
<- { "type": "price_update", "market_id": "id1", "platform": "kalshi", "yes_price": 0.45, "no_price": 0.55 }

# Subscribe to portfolio updates
-> { "type": "subscribe", "channel": "portfolio" }
<- { "type": "position_update", "position": Position }
<- { "type": "balance_update", "balance": Balance }

# Subscribe to news
-> { "type": "subscribe", "channel": "news", "market_ids": ["id1"] }
<- { "type": "news", "item": NewsItem }
```

---

## 4. Implementation Plan

### 4.1 Day-by-Day Schedule

#### Day 1: Foundation

**Morning:**
- [ ] Install Rust toolchain (`rustup`)
- [ ] Fork barter-rs repository
- [ ] Set up project structure (workspace, crates)
- [ ] Rust basics: ownership, borrowing, Result/Option, async

**Afternoon:**
- [ ] Create `terminal-core` crate with data models
- [ ] Define `PlatformClient` trait
- [ ] Get basic project compiling

**Evening:**
- [ ] Read kalshi-rs documentation
- [ ] Read polyfill-rs documentation
- [ ] Test Kalshi demo API (curl)

**Deliverable:** Project compiles, core types defined

---

#### Day 2: Kalshi Integration

**Morning:**
- [ ] Create `terminal-kalshi` crate
- [ ] Add kalshi-rs dependency
- [ ] Implement `KalshiAdapter::new()` with credentials
- [ ] Implement `list_markets()`

**Afternoon:**
- [ ] Implement `get_market()`
- [ ] Implement `get_orderbook()`
- [ ] Convert Kalshi types to unified types

**Evening:**
- [ ] Implement `get_positions()`
- [ ] Implement `get_balance()`
- [ ] Test with demo environment

**Deliverable:** Can read all Kalshi data

---

#### Day 3: Kalshi Trading + CLI Start

**Morning:**
- [ ] Implement `place_order()`
- [ ] Implement `cancel_order()`
- [ ] Implement `get_open_orders()`
- [ ] Test trading on demo environment

**Afternoon:**
- [ ] Create `terminal-cli` crate
- [ ] Set up clap for argument parsing
- [ ] Implement `markets list` command
- [ ] Implement `markets get <id>` command

**Evening:**
- [ ] Implement `trade` command
- [ ] Implement `portfolio` command
- [ ] Test full Kalshi flow via CLI

**Deliverable:** Full Kalshi support, CLI working

---

#### Day 4: Polymarket Integration

**Morning:**
- [ ] Create `terminal-polymarket` crate
- [ ] Add polyfill-rs dependency
- [ ] Implement `PolymarketAdapter::new()` with wallet
- [ ] Implement `list_markets()`

**Afternoon:**
- [ ] Implement `get_market()`
- [ ] Implement `get_orderbook()`
- [ ] Convert Polymarket types to unified types

**Evening:**
- [ ] Implement trading methods
- [ ] Implement portfolio methods
- [ ] Test with real Polymarket (small amounts)

**Deliverable:** Both platforms working via CLI

---

#### Day 5: Services + Market Matching

**Morning:**
- [ ] Create `terminal-services` crate
- [ ] Implement `MarketMatcher` with JSON config
- [ ] Create initial market mappings file (10-20 markets)

**Afternoon:**
- [ ] Implement `MarketService` (aggregates both platforms)
- [ ] Implement `TradingService` (routes to correct platform)
- [ ] Implement `PortfolioService` (aggregates positions)

**Evening:**
- [ ] Implement spread detection
- [ ] Update CLI to use services
- [ ] Test unified market view

**Deliverable:** Unified market view, spread detection

---

#### Day 6: HTTP API + News

**Morning:**
- [ ] Create `terminal-api` crate
- [ ] Set up Axum server
- [ ] Implement market endpoints
- [ ] Implement trading endpoints

**Afternoon:**
- [ ] Implement portfolio endpoints
- [ ] Implement auth/session endpoint
- [ ] Add basic error handling

**Evening:**
- [ ] Implement `NewsService` (RSS + NewsAPI)
- [ ] Implement news endpoint
- [ ] Match news to markets (keyword-based)

**Deliverable:** HTTP API functional, news feed working

---

#### Day 7: Polish + Documentation

**Morning:**
- [ ] Bug fixes from testing
- [ ] Add input validation
- [ ] Improve error messages

**Afternoon:**
- [ ] Write README with setup instructions
- [ ] Document architecture
- [ ] Create example commands / API calls

**Evening:**
- [ ] Record demo video
- [ ] Final testing
- [ ] Clean up code

**Deliverable:** Shippable product

---

### 4.2 Milestones & Checkpoints

| Milestone | Test | Day |
|-----------|------|-----|
| M1: Project compiles | `cargo build` succeeds | 1 |
| M2: Kalshi markets | `cargo run -- markets list --platform kalshi` shows markets | 2 |
| M3: Kalshi trading | `cargo run -- trade kalshi <market> buy yes 1 0.50` executes | 3 |
| M4: Polymarket markets | `cargo run -- markets list --platform polymarket` shows markets | 4 |
| M5: Unified view | `cargo run -- markets list` shows matched markets with spread | 5 |
| M6: HTTP API | `curl localhost:3000/api/markets` returns JSON | 6 |
| M7: News feed | `curl localhost:3000/api/news` returns news items | 6 |
| M8: Demo ready | Can show full flow in video | 7 |

---

## 5. External Dependencies

### 5.1 Rust Crates

```toml
[workspace.dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }

# HTTP server
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# HTTP client
reqwest = { version = "0.11", features = ["json"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# CLI
clap = { version = "4", features = ["derive"] }

# Decimal handling
rust_decimal = "1"

# Date/time
chrono = { version = "0.4", features = ["serde"] }

# Async trait
async-trait = "0.1"

# Error handling
thiserror = "1"
anyhow = "1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Platform SDKs
kalshi = "0.9"          # kalshi-rs crate
polyfill-rs = "0.1"     # or polyte, evaluate both

# News
rss = "2"               # RSS parsing
```

### 5.2 External APIs

| API | Purpose | Auth Required | Rate Limits |
|-----|---------|---------------|-------------|
| Kalshi REST | Market data, trading | API key + RSA signature | 100 req/sec |
| Kalshi WebSocket | Live prices | Token | - |
| Polymarket CLOB | Market data, trading | Wallet signature | Varies |
| NewsAPI.org | News feed | API key | 100 req/day (free) |
| RSS feeds | News feed | None | Varies |

### 5.3 Account Requirements

**Kalshi:**
- [ ] Create account at kalshi.com
- [ ] Complete KYC verification
- [ ] Generate API key in settings
- [ ] Note demo environment: `demo-api.kalshi.co`

**Polymarket:**
- [ ] Ethereum wallet with private key
- [ ] USDC on Polygon network
- [ ] Deposit to Polymarket proxy address
- [ ] Export private key for API use

---

## 6. Configuration

### 6.1 Environment Variables

```bash
# Kalshi
KALSHI_API_KEY_ID=your_key_id
KALSHI_PRIVATE_KEY_PATH=/path/to/private.pem
KALSHI_ENVIRONMENT=demo  # or "live"

# Polymarket
POLYMARKET_PRIVATE_KEY=0x...
POLYMARKET_FUNDER_ADDRESS=0x...
POLYMARKET_CHAIN_ID=137

# News
NEWS_API_KEY=your_newsapi_key

# Server
SERVER_HOST=127.0.0.1
SERVER_PORT=3000

# Logging
RUST_LOG=info,terminal=debug
```

### 6.2 Market Mappings File

```json
// config/market_mappings.json
{
  "mappings": [
    {
      "unified_id": "pres-2028-dem",
      "title": "Will a Democrat win the 2028 US Presidential Election?",
      "kalshi_ticker": "PRES-2028-DEM",
      "polymarket_condition_id": "0xabc123..."
    },
    {
      "unified_id": "btc-150k-2026",
      "title": "Will Bitcoin reach $150,000 by end of 2026?",
      "kalshi_ticker": "BTC-150K-2026",
      "polymarket_condition_id": "0xdef456..."
    }
  ]
}
```

---

## 7. CLI Command Reference

```bash
# Configuration
terminal config set kalshi-key <key_id>
terminal config set kalshi-key-path <path>
terminal config set polymarket-key <private_key>
terminal config show

# Markets
terminal markets list [--platform <kalshi|polymarket|all>] [--category <cat>]
terminal markets get <market_id>
terminal markets orderbook <market_id> --platform <platform>
terminal markets spreads [--min-spread <pct>]

# Trading
terminal trade <platform> <market_id> <buy|sell> <yes|no> <quantity> [price]
terminal orders list [--platform <platform>] [--status <open|filled|all>]
terminal orders cancel <order_id> --platform <platform>

# Portfolio
terminal portfolio [--platform <platform>]
terminal portfolio positions
terminal portfolio balances

# Monitoring
terminal watch <market_id>           # Live price updates
terminal watch spreads [--threshold <pct>]  # Alert on spread opportunities

# Server
terminal server start [--port <port>]
```

---

## 8. Risk & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Rust learning curve | High | Medium | Focus on key concepts, lean on AI |
| Kalshi verification delayed | Medium | Medium | Use demo environment |
| Polymarket wallet issues | Medium | Medium | Start with read-only, add trading later |
| API rate limits | Low | Low | Add caching, respect limits |
| Market matching complexity | Medium | Medium | Start with manual mappings |
| Time overrun | Medium | High | Cut scope (drop news first) |

---

## 9. Future Enhancements (Post-Week)

1. **WebSocket price streaming** - Real-time updates
2. **Persistent sessions** - Encrypted credential storage
3. **Frontend UI** - Next.js web interface
4. **Automated alerts** - Discord/Telegram notifications
5. **Historical data** - Price charts, analytics
6. **Fuzzy market matching** - LLM-based or similarity matching
7. **Paper trading mode** - Simulate trades without real money
8. **Multi-user support** - Proper auth, user accounts

---

## 10. Definition of Done

### MVP Complete When:

- [ ] User can connect Kalshi credentials via CLI
- [ ] User can connect Polymarket wallet via CLI
- [ ] User can list markets from both platforms
- [ ] User can see unified view of matched markets
- [ ] User can see price spreads between platforms
- [ ] User can place orders on Kalshi via CLI
- [ ] User can place orders on Polymarket via CLI
- [ ] User can view aggregated portfolio
- [ ] HTTP API serves all data
- [ ] News feed shows relevant items
- [ ] README documents setup and usage
- [ ] Demo video shows complete flow

---

## Appendix A: Kalshi API Quick Reference

```
Base URL: https://api.elections.kalshi.com/trade-api/v2
Demo URL: https://demo-api.kalshi.co/trade-api/v2

Endpoints:
  GET  /markets                 - List markets
  GET  /markets/:ticker         - Get market
  GET  /markets/:ticker/orderbook - Get orderbook
  GET  /events                  - List events
  GET  /events/:ticker          - Get event
  POST /portfolio/orders        - Place order
  GET  /portfolio/orders        - List orders
  DELETE /portfolio/orders/:id  - Cancel order
  GET  /portfolio/positions     - Get positions
  GET  /portfolio/balance       - Get balance
```

## Appendix B: Polymarket API Quick Reference

```
CLOB URL: https://clob.polymarket.com
Data URL: https://data-api.polymarket.com
WebSocket: wss://ws-subscriptions-clob.polymarket.com/ws

Endpoints:
  GET  /markets                 - List markets
  GET  /book                    - Get orderbook
  POST /order                   - Place order
  DELETE /order/:id             - Cancel order
  GET  /orders                  - List orders
  GET  /positions               - Get positions
```

## Appendix C: Resources

- [barter-rs Repository](https://github.com/barter-rs/barter-rs)
- [Kalshi API Docs](https://docs.kalshi.com)
- [Polymarket API Docs](https://docs.polymarket.com)
- [kalshi-rs Crate](https://crates.io/crates/kalshi)
- [polyfill-rs Repository](https://github.com/floor-licker/polyfill-rs)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
