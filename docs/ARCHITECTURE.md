# Architecture Overview

This document explains the technical architecture of the Prediction Market Terminal, including the Rust backend, WebSocket real-time system, and frontend integration.

---

## System Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              FRONTEND                                        │
│                         Next.js 16 + React 19                                │
│                                                                              │
│   Markets Grid │ Market Detail │ Price Chart │ OrderBook │ Trade History    │
│                                                                              │
│   ┌──────────────────────────────────────────────────────────────────────┐  │
│   │  useMarketStream hook ←→ Single WebSocket Connection                 │  │
│   │  TanStack Query       ←→ REST API calls                              │  │
│   └──────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────┬───────────────────────────────────────┘
                                      │
                                      │ HTTP :3001 / WS :3001/ws
                                      │
┌─────────────────────────────────────▼───────────────────────────────────────┐
│                            RUST BACKEND                                      │
│                           terminal-api (Axum)                                │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                         REST ROUTES                                     │ │
│  │  GET  /api/markets                    - List markets (filters)          │ │
│  │  GET  /api/markets/:platform/:id      - Single market                   │ │
│  │  GET  /api/markets/:platform/:id/orderbook - Order book                 │ │
│  │  GET  /api/markets/:platform/:id/trades    - Trade history              │ │
│  │  GET  /api/markets/:platform/:id/history   - Price candles              │ │
│  │  GET  /ws                             - WebSocket upgrade               │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                       │
│  ┌───────────────────────────────────▼────────────────────────────────────┐ │
│  │                      SERVICE LAYER (terminal-services)                  │ │
│  │                                                                         │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────────────┐    │ │
│  │  │  MarketService  │  │ CandleService   │  │ TradeCollector       │    │ │
│  │  │                 │  │                 │  │                      │    │ │
│  │  │ - get_markets   │  │ - get_candles   │  │ - Background polling │    │ │
│  │  │ - get_market    │  │ - 6 timeframes  │  │ - Store to SQLite    │    │ │
│  │  └─────────────────┘  └─────────────────┘  └──────────────────────┘    │ │
│  │                                                                         │ │
│  │  ┌─────────────────────────────────────────────────────────────────┐   │ │
│  │  │                   MarketDataAggregator                           │   │ │
│  │  │                                                                  │   │ │
│  │  │  - Connects to Kalshi + Polymarket WebSockets                   │   │ │
│  │  │  - Normalizes incoming data to unified types                    │   │ │
│  │  │  - Tracks connection health (stale detection)                   │   │ │
│  │  │  - Routes updates via SubscriptionManager                       │   │ │
│  │  └─────────────────────────────────────────────────────────────────┘   │ │
│  │                                      │                                  │ │
│  │  ┌───────────────────────────────────▼──────────────────────────────┐  │ │
│  │  │                   WebSocketState + SubscriptionManager           │  │ │
│  │  │                                                                  │  │ │
│  │  │  - Manages client connections                                   │  │ │
│  │  │  - Tracks per-client subscriptions                              │  │ │
│  │  │  - Broadcasts filtered updates                                  │  │ │
│  │  └──────────────────────────────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                      │                                       │
│  ┌───────────────────────────────────▼────────────────────────────────────┐ │
│  │                     PLATFORM CLIENTS                                    │ │
│  │                                                                         │ │
│  │  ┌─────────────────────────────┐    ┌─────────────────────────────┐    │ │
│  │  │    terminal-kalshi          │    │   terminal-polymarket       │    │ │
│  │  │                             │    │                             │    │ │
│  │  │  KalshiClient (REST)        │    │  PolymarketClient (REST)    │    │ │
│  │  │  - list_markets()           │    │  - list_all_events()        │    │ │
│  │  │  - get_orderbook()          │    │  - get_orderbook()          │    │ │
│  │  │  - get_trades()             │    │  - get_trades()             │    │ │
│  │  │                             │    │  - get_price_history()      │    │ │
│  │  │  KalshiWebSocket            │    │                             │    │ │
│  │  │  - RSA-PSS authentication   │    │  PolymarketWebSocket        │    │ │
│  │  │  - orderbook_delta channel  │    │  - HMAC-SHA256 auth         │    │ │
│  │  │  - Auto-reconnect           │    │  - market channel           │    │ │
│  │  └─────────────────────────────┘    └─────────────────────────────┘    │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────┘
                    │                               │
                    ▼                               ▼
           ┌───────────────┐               ┌───────────────┐
           │  Kalshi API   │               │ Polymarket    │
           │               │               │ Gamma API     │
           │ REST + WS     │               │ REST + WS     │
           └───────────────┘               └───────────────┘
```

---

## Rust Crate Structure

### terminal-core
**Purpose**: Shared types used across all crates

```
terminal-core/src/
├── lib.rs          # Re-exports
├── platform.rs     # Platform enum (Kalshi, Polymarket)
├── market.rs       # PredictionMarket, OrderBook, Trade, PriceCandle
├── websocket.rs    # ClientMessage, ServerMessage, SubscriptionKey
├── position.rs     # Portfolio, Balance (future use)
└── error.rs        # TerminalError type
```

Key types:
- `PredictionMarket`: Full market record with prices, volume, status
- `OrderBook`: Bid/ask levels for YES and NO outcomes
- `Trade`: Individual trade with side/outcome
- `ClientMessage`/`ServerMessage`: WebSocket protocol

### terminal-kalshi
**Purpose**: Kalshi API integration

```
terminal-kalshi/src/
├── lib.rs
├── client.rs       # KalshiClient - REST API
├── websocket.rs    # KalshiWebSocket - Real-time updates
└── types.rs        # Kalshi-specific response types
```

**Authentication**: RSA-PSS signing for WebSocket connections
- Signs timestamp with private key
- Includes signature in WebSocket headers

### terminal-polymarket
**Purpose**: Polymarket API integration

```
terminal-polymarket/src/
├── lib.rs
├── client.rs       # PolymarketClient - REST API
├── websocket.rs    # PolymarketWebSocket - Real-time updates
└── types.rs        # Polymarket-specific response types
```

**Authentication**: HMAC-SHA256 for authenticated endpoints
- Signs: `timestamp + method + path + body`
- Uses base64-encoded secret

### terminal-services
**Purpose**: Business logic layer

```
terminal-services/src/
├── lib.rs
├── market_service.rs     # Unified market data access
├── aggregator.rs         # MarketDataAggregator - Real-time coordinator
├── trade_storage.rs      # SQLite persistence
├── trade_collector.rs    # Background trade fetching
├── candle_service.rs     # OHLCV generation
└── websocket/
    ├── mod.rs
    ├── handler.rs        # WebSocketState, connection handling
    └── subscription.rs   # SubscriptionManager
```

### terminal-api
**Purpose**: HTTP/WebSocket server

```
terminal-api/src/
├── main.rs           # Server initialization, AppState
└── routes/
    ├── mod.rs
    ├── markets.rs    # Market endpoints
    ├── ws.rs         # WebSocket upgrade handler
    └── health.rs     # Health check
```

---

## Real-Time Data Flow

### 1. Exchange Connection
The `MarketDataAggregator` maintains WebSocket connections to both platforms:

```rust
// terminal-services/src/aggregator.rs
pub struct MarketDataAggregator {
    kalshi_ws: Option<KalshiWebSocket>,
    polymarket_ws: Option<PolymarketWebSocket>,
    ws_state: Arc<WebSocketState>,
    // ...
}
```

### 2. Message Normalization
Incoming messages from each platform are normalized to unified types:

- Kalshi orderbook delta → `OrderBook`
- Polymarket price change → `PriceUpdate`
- Both → `ServerMessage` enum

### 3. Client Subscription Protocol
Frontend subscribes to specific data:

```json
// Client → Server
{"type": "Subscribe", "channel": {"type": "Orderbook", "platform": "kalshi", "market_id": "KXBTC-100K"}}

// Server → Client
{"type": "OrderbookUpdate", "update_type": "Snapshot", "orderbook": {...}}
```

### 4. Filtered Broadcast
The `SubscriptionManager` tracks which clients want which data:

```rust
// terminal-services/src/websocket/subscription.rs
pub struct SubscriptionManager {
    subscriptions: RwLock<HashMap<ClientId, HashSet<SubscriptionKey>>>,
    broadcast_tx: broadcast::Sender<BroadcastMessage>,
}
```

When a message arrives, it's only sent to clients subscribed to that market/channel.

---

## Frontend Architecture

### Data Fetching Strategy
- **REST API** (TanStack Query): Initial data, market lists, historical data
- **WebSocket**: Real-time updates (prices, orderbook, trades)

### Key Hook: useMarketStream

```typescript
// frontend/src/hooks/use-market-stream.ts
export function useMarketStream(platform: string, marketId: string) {
  // Subscribes to WebSocket channels
  // Merges REST data with WebSocket updates
  // Returns: { prices, orderBook, trades, connectionState }
}
```

### Single Connection Pattern
The frontend maintains ONE WebSocket connection via `WebSocketProvider`:

```typescript
// frontend/src/providers/websocket-provider.tsx
// - Single persistent connection to ws://localhost:3001/ws
// - Ping/pong for latency measurement
// - Automatic reconnection with exponential backoff
```

---

## Data Persistence

### Trade Storage (SQLite)

```sql
CREATE TABLE trades (
    id TEXT PRIMARY KEY,
    platform TEXT NOT NULL,
    market_id TEXT NOT NULL,
    price REAL NOT NULL,
    quantity REAL NOT NULL,
    side TEXT NOT NULL,
    timestamp INTEGER NOT NULL
);

CREATE INDEX idx_trades_market ON trades(platform, market_id, timestamp);
```

### Candle Generation

The `CandleService` generates OHLCV candles from stored trades:

```rust
// terminal-services/src/candle_service.rs
pub async fn get_candles(
    &self,
    platform: Platform,
    market_id: &str,
    interval: CandleInterval,  // 1m, 5m, 15m, 1h, 4h, 1d
    start_time: Option<DateTime<Utc>>,
) -> Result<Vec<PriceCandle>, TerminalError>
```

---

## Key Files Reference

| Component | File | Purpose |
|-----------|------|---------|
| Core types | `terminal-core/src/market.rs` | PredictionMarket, OrderBook, Trade |
| WebSocket protocol | `terminal-core/src/websocket.rs` | ClientMessage, ServerMessage |
| Kalshi client | `terminal-kalshi/src/client.rs` | REST API methods |
| Kalshi WebSocket | `terminal-kalshi/src/websocket.rs` | RSA-PSS auth, real-time |
| Polymarket client | `terminal-polymarket/src/client.rs` | REST API with HMAC |
| Aggregator | `terminal-services/src/aggregator.rs` | Fan-out coordinator |
| Subscription manager | `terminal-services/src/websocket/handler.rs` | Client connection handling |
| Server init | `terminal-api/src/main.rs` | AppState, service wiring |
| Frontend WebSocket | `frontend/src/providers/websocket-provider.tsx` | Single connection |
| Market stream hook | `frontend/src/hooks/use-market-stream.ts` | Subscribe + merge data |

---

## Comparison with Original Barter-rs

| Aspect | Original Barter | Terminal Extension |
|--------|-----------------|-------------------|
| Target | Crypto exchanges | Prediction markets |
| Exchanges | 8 (Binance, Coinbase, etc.) | 2 (Kalshi, Polymarket) |
| Pattern | Connector trait | Custom client classes |
| Data focus | Trades, L1/L2 orderbooks | Markets, YES/NO orderbooks |
| Trading | Full execution engine | Read-only (future) |
| Frontend | None | Full Next.js app |
