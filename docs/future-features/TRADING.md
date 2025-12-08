# Trading & Order Placement

## Overview

This document outlines the high-level architecture for adding trading capabilities to the terminal. Trading is the most complex feature and is intentionally scoped as a future enhancement.

**Status**: Not started - no trading infrastructure exists currently.

## Current State

### What exists
- Both `KalshiClient` and `PolymarketClient` have authentication support
- API keys can be loaded from environment variables
- Auth is currently used only for read-only operations (balance, positions)

### What's missing
- Order placement API endpoints
- Order management (cancel, modify)
- Position tracking UI
- Balance management
- Trade confirmation flows
- Risk controls
- Order history

## Scope & Complexity

This is the largest feature in the roadmap. Full implementation includes:

| Component | Complexity | Notes |
|-----------|------------|-------|
| API key management | Medium | Secure storage, validation |
| Order placement backend | Medium | Call exchange APIs |
| Order placement frontend | Medium | Form, validation, confirmation |
| Position tracking | Medium | Real-time position updates |
| Order history | Low | Query and display past orders |
| Risk controls | High | Max position, daily limits, confirmations |
| Error handling | High | Exchange errors, timeouts, partial fills |
| Testing | High | Paper trading mode, integration tests |

Estimated: 40-80 hours of development depending on scope.

## Exchange APIs

### Kalshi Trading API

Documentation: https://trading-api.readme.io/

**Key endpoints**:
```
POST /trade-api/v2/portfolio/orders        - Place order
DELETE /trade-api/v2/portfolio/orders/:id  - Cancel order
GET /trade-api/v2/portfolio/orders         - List orders
GET /trade-api/v2/portfolio/positions      - Get positions
GET /trade-api/v2/portfolio/balance        - Get balance
```

**Order structure**:
```json
{
  "ticker": "KXBTC-100K-25DEC31",
  "side": "yes",           // "yes" or "no"
  "action": "buy",         // "buy" or "sell"
  "count": 100,            // Number of contracts
  "type": "limit",         // "limit" or "market"
  "yes_price": 45,         // Price in cents (for limit orders)
  "expiration_ts": null    // Optional expiration
}
```

**Authentication**: API key + secret in headers.

### Polymarket Trading API

Documentation: https://docs.polymarket.com/

**Key endpoints** (CLOB API):
```
POST /order                  - Place order
DELETE /order/:id            - Cancel order
GET /orders                  - List orders
GET /positions               - Get positions
```

**Order structure**:
```json
{
  "tokenID": "123...",       // CLOB token ID
  "side": "BUY",             // "BUY" or "SELL"
  "price": "0.45",           // Price as decimal string
  "size": "100",             // Size as string
  "orderType": "GTC",        // "GTC", "FOK", "GTD"
  "signature": "0x..."       // EIP-712 signature
}
```

**Authentication**: Wallet signature (more complex than Kalshi).

## Proposed Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────────────┐
│                         Frontend                                 │
├─────────────────────────────────────────────────────────────────┤
│  OrderEntry   │  Positions   │  OrderHistory  │  Balance        │
│  Component    │  Panel       │  Table         │  Display        │
└───────┬───────┴──────┬───────┴───────┬────────┴────────┬────────┘
        │              │               │                 │
        ▼              ▼               ▼                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                      API Layer                                   │
├─────────────────────────────────────────────────────────────────┤
│  POST /orders        │  GET /positions  │  GET /balance         │
│  DELETE /orders/:id  │  GET /orders     │                       │
└───────┬──────────────┴────────┬─────────┴───────────────────────┘
        │                       │
        ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Trading Service                               │
├─────────────────────────────────────────────────────────────────┤
│  - Order validation          │  - Position aggregation          │
│  - Risk checks               │  - Balance tracking              │
│  - Exchange routing          │  - Order state management        │
└───────┬──────────────────────┴──────────────────────────────────┘
        │
        ├────────────────┬────────────────┐
        ▼                ▼                ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ KalshiClient │  │ PolyClient   │  │ Credentials  │
│ (trading)    │  │ (trading)    │  │ Store        │
└──────────────┘  └──────────────┘  └──────────────┘
```

### Component Breakdown

#### 1. Credentials Store

Secure storage for API keys:

```rust
pub struct CredentialsStore {
    // Encrypted at rest, loaded into memory on startup
    kalshi_api_key: Option<String>,
    kalshi_api_secret: Option<String>,
    polymarket_private_key: Option<String>,  // For signing
}

impl CredentialsStore {
    pub fn load_from_env() -> Self;
    pub fn load_from_file(path: &str) -> Self;  // Encrypted JSON
    pub fn is_configured(&self, platform: Platform) -> bool;
}
```

#### 2. Trading Service

Core trading logic:

```rust
pub struct TradingService {
    kalshi: Arc<KalshiClient>,
    polymarket: Arc<PolymarketClient>,
    credentials: Arc<CredentialsStore>,
    risk_manager: Arc<RiskManager>,
}

impl TradingService {
    pub async fn place_order(&self, order: OrderRequest) -> Result<Order, TradingError>;
    pub async fn cancel_order(&self, platform: Platform, order_id: &str) -> Result<(), TradingError>;
    pub async fn get_positions(&self, platform: Option<Platform>) -> Result<Vec<Position>, TradingError>;
    pub async fn get_balance(&self, platform: Platform) -> Result<Balance, TradingError>;
    pub async fn get_orders(&self, platform: Option<Platform>, status: Option<OrderStatus>) -> Result<Vec<Order>, TradingError>;
}
```

#### 3. Risk Manager

Pre-trade checks:

```rust
pub struct RiskManager {
    config: RiskConfig,
}

#[derive(Debug, Clone)]
pub struct RiskConfig {
    pub max_position_size: u64,           // Max contracts per market
    pub max_order_size: u64,              // Max contracts per order
    pub max_daily_volume: Decimal,        // Max $ traded per day
    pub require_confirmation: bool,       // Require UI confirmation
    pub allowed_platforms: Vec<Platform>, // Which platforms enabled
}

impl RiskManager {
    pub fn validate_order(&self, order: &OrderRequest, current_position: Option<&Position>) -> Result<(), RiskError>;
}
```

#### 4. Order Entry Component

Frontend order form:

```tsx
interface OrderEntryProps {
  market: PredictionMarket;
  side: "yes" | "no";
  onSubmit: (order: OrderRequest) => void;
}

function OrderEntry({ market, side, onSubmit }: OrderEntryProps) {
  const [orderType, setOrderType] = useState<"limit" | "market">("limit");
  const [price, setPrice] = useState<number>(side === "yes" ? market.yes_price : market.no_price);
  const [quantity, setQuantity] = useState<number>(10);

  const estimatedCost = quantity * price;
  const maxProfit = quantity * (1 - price);

  return (
    <form onSubmit={handleSubmit}>
      {/* Order type selector */}
      {/* Price input (for limit) */}
      {/* Quantity input */}
      {/* Cost/profit display */}
      {/* Confirmation checkbox (if required) */}
      {/* Submit button */}
    </form>
  );
}
```

## Implementation Phases

### Phase 1: Read-Only Portfolio (Foundation)

- Add portfolio endpoints (positions, balance, orders)
- Display current positions in UI
- Display account balance
- No order placement yet

### Phase 2: Order Placement (Core)

- Implement order placement in backend
- Add order entry form in frontend
- Basic validation (price, quantity)
- Order confirmation dialog

### Phase 3: Order Management

- Cancel orders
- View order history
- Order status updates (via WebSocket)
- Partial fill handling

### Phase 4: Risk Controls

- Position size limits
- Daily volume limits
- Price sanity checks
- Require confirmation for large orders

### Phase 5: Advanced Features

- Paper trading mode (simulated orders)
- Quick order buttons (buy/sell at bid/ask)
- Keyboard shortcuts
- Sound alerts on fills

## Security Considerations

1. **API key storage**: Never store keys in plaintext; use encryption or OS keychain
2. **Key scope**: Use read+trade keys, not withdrawal-enabled keys
3. **Input validation**: Sanitize all user input
4. **Rate limiting**: Respect exchange rate limits
5. **Audit logging**: Log all trading activity
6. **Two-factor**: Consider requiring 2FA for trades above threshold

## Error Handling

Common failure modes:

| Error | Handling |
|-------|----------|
| Insufficient balance | Show error, suggest deposit |
| Invalid price | Reject order, show valid range |
| Market closed | Show market status, disable form |
| Rate limited | Queue order, retry with backoff |
| Partial fill | Update position, show fill status |
| Network timeout | Show warning, allow retry |

## Testing Strategy

1. **Unit tests**: Order validation, risk checks
2. **Integration tests**: Against exchange sandboxes (if available)
3. **Paper trading**: Simulated order execution for testing UI
4. **Manual testing**: Small real orders before full deployment

## Files to Create (Future)

| File | Purpose |
|------|---------|
| `terminal-services/src/trading_service.rs` | Core trading logic |
| `terminal-services/src/risk_manager.rs` | Pre-trade validation |
| `terminal-services/src/credentials.rs` | Secure key storage |
| `terminal-api/src/routes/trading.rs` | API endpoints |
| `frontend/src/components/trading/order-entry.tsx` | Order form |
| `frontend/src/components/trading/positions.tsx` | Position display |
| `frontend/src/components/trading/order-history.tsx` | Order history |

## Success Criteria

1. Can place limit orders on both platforms
2. Can cancel pending orders
3. Positions update in real-time after fills
4. Risk controls prevent oversized positions
5. Order history persisted and queryable
6. Error states handled gracefully

## Out of Scope (for initial version)

- Advanced order types (stop-loss, OCO)
- Automated trading / bots
- Multi-leg orders (arbitrage execution)
- Withdrawal management
- Tax reporting
- Mobile app support
