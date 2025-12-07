# Prediction Market Terminal — Project Brief & Next Steps

## Project Context

### What We're Building

A **unified trading terminal** that aggregates prediction markets from Kalshi and Polymarket into a single interface. Think Bloomberg Terminal for prediction markets — one place to see all markets, compare prices across platforms, and (eventually) execute trades.

### Why This Matters

**The Problem:**
- Traders must juggle two separate platforms with different UIs, auth flows, and data formats
- No way to see equivalent markets side-by-side to find best prices
- Price discrepancies exist between platforms (same event, different odds) — arbitrage opportunities are invisible
- Portfolio tracking requires logging into each platform separately

**The Opportunity:**
- Polymarket + Kalshi combined: ~$10B monthly volume (Nov 2025)
- No unified tooling exists for cross-platform comparison
- Professional traders need this — it's table stakes in traditional finance
- Demonstrates real systems integration skills (brownfield Rust, multiple APIs, real-time data)

### Value Proposition

| User | Value |
|------|-------|
| Active traders | Find best prices, spot arbitrage, unified portfolio view |
| Researchers/analysts | Single source for prediction market data across platforms |
| Casual users | Easier discovery — see all markets without platform hopping |

---

## Current State

### What's Built (MVP v0.1)

```
barter-rs/                          
├── crates/
│   ├── terminal-core/              ✅ Core types (Platform, PredictionMarket, etc.)
│   ├── terminal-kalshi/            ✅ Kalshi API client (REST, pagination)
│   ├── terminal-polymarket/        ✅ Polymarket Gamma API client
│   ├── terminal-services/          ✅ MarketService (aggregates both platforms)
│   └── terminal-api/               ✅ Axum HTTP server
│
└── frontend/                       ✅ Next.js + shadcn/ui
    └── Markets table with platform filter, search, basic styling
```

**Working:**
- Backend serves markets from both Kalshi and Polymarket
- Frontend displays unified table with platform filter
- Search by title works
- Basic Axiom-inspired dark theme

**Known Issues:**
1. Only ~200 markets showing (pagination incomplete)
2. Polymarket multi-outcome events show individual options as separate rows (clutters UI)
3. Prices not fully normalized (Kalshi has bid/ask, Polymarket has single price)
4. Volume units inconsistent (Kalshi in cents, Polymarket in USDC)
5. No market matching/spreads view
6. No live price updates (WebSocket not implemented)
7. No portfolio integration

---

## Architecture Decisions (Locked In)

### Data Sources

| Platform | Endpoint | Purpose |
|----------|----------|---------|
| Kalshi | `GET /trade-api/v2/markets` | Market list, prices, metadata |
| Kalshi | `WSS /trade-api/ws/v2` | Live orderbook updates (future) |
| Polymarket | `GET gamma-api.polymarket.com/events` | Events (not /markets — avoids multi-outcome clutter) |
| Polymarket | `WSS ws-subscriptions-clob.polymarket.com` | Live prices (future) |

### Unified Data Model

```rust
struct UnifiedMarket {
    // Normalized fields (always present)
    id: String,                    // "kalshi:{ticker}" or "poly:{slug}"
    platform: Platform,
    title: String,
    yes_price: Decimal,            // 0.00-1.00 probability
    no_price: Decimal,
    volume_usd: Decimal,           // Normalized to USD
    status: MarketStatus,
    close_time: Option<DateTime>,
    external_url: String,
    
    // Platform-specific (preserved for drill-down)
    kalshi_data: Option<KalshiSpecificData>,
    polymarket_data: Option<PolymarketSpecificData>,
}
```

**Key principle:** Unified fields for display/comparison, platform-specific structs preserve all original data.

### Market Matching Strategy

**Phase 1 (MVP):** Manual JSON mappings
```json
{
  "mappings": [
    {
      "id": "btc-100k-2025",
      "kalshi_ticker": "KXBTC-100K-25DEC31",
      "polymarket_slug": "will-bitcoin-reach-100000-in-2025"
    }
  ]
}
```

**Phase 2 (Post-MVP):** Embedding similarity to suggest matches, human review to confirm

**Why not NLP auto-matching?** Resolution rules differ between platforms. High textual similarity ≠ same market. A "Bitcoin $100k" market on each platform might resolve to different outcomes due to different sources/timing.

---

## Next Steps (Prioritized)

### Phase 1: Data Quality (Days 1-2)

**Goal:** Clean, accurate, complete data from both platforms

#### 1.1 Fix Polymarket to fetch Events, not Markets

**Problem:** Currently fetching `/markets` returns individual options ("Jurassic World Rebirth" at 5%) instead of events ("Highest Grossing Movie 2025?")

**Solution:**
```rust
// terminal-polymarket/src/client.rs

// CHANGE FROM:
pub async fn list_markets(&self) -> Result<Vec<PolymarketMarket>>

// TO:
pub async fn list_events(&self) -> Result<Vec<PolymarketEvent>>

// Fetch from: GET https://gamma-api.polymarket.com/events?active=true&closed=false&limit=100
```

**Conversion logic:**
- Binary event (1 market, 2 outcomes) → 1 UnifiedMarket with yes/no prices
- Multi-outcome event (N markets) → 1 UnifiedMarket with `is_multi_outcome: true`, show leading option

#### 1.2 Implement Full Pagination

**Problem:** Only fetching first page (~100-200 markets per platform)

**Kalshi pagination:**
```rust
// Kalshi uses cursor-based pagination
loop {
    let response = client.get("/markets")
        .query(&[("limit", "200"), ("cursor", &cursor)])
        .send().await?;
    
    markets.extend(response.markets);
    
    match response.cursor {
        Some(c) => cursor = c,
        None => break,
    }
}
```

**Polymarket pagination:**
```rust
// Polymarket uses offset-based
let mut offset = 0;
loop {
    let response = client.get("/events")
        .query(&[("limit", "100"), ("offset", &offset.to_string()), ("active", "true")])
        .send().await?;
    
    if response.is_empty() { break; }
    events.extend(response);
    offset += 100;
}
```

#### 1.3 Normalize Data Properly

**Price normalization:**
```rust
// Kalshi: has bid/ask/last — use midpoint or last_price
fn kalshi_yes_price(m: &KalshiMarket) -> Decimal {
    m.last_price.unwrap_or_else(|| (m.yes_bid + m.yes_ask) / dec!(2))
}

// Polymarket: parse JSON string
fn polymarket_yes_price(m: &PolymarketMarket) -> Decimal {
    let prices: Vec<String> = serde_json::from_str(&m.outcome_prices).unwrap();
    prices[0].parse().unwrap_or_default()
}
```

**Volume normalization:**
```rust
// Kalshi: cents → dollars
let volume_usd = Decimal::from(kalshi_market.volume) / dec!(100);

// Polymarket: already USDC string
let volume_usd: Decimal = polymarket_event.volume.parse().unwrap_or_default();
```

#### 1.4 Preserve Platform-Specific Data

Update `UnifiedMarket` struct to include:
```rust
pub struct KalshiSpecificData {
    pub ticker: String,
    pub event_ticker: String,
    pub yes_bid: Decimal,
    pub yes_ask: Decimal,
    pub no_bid: Decimal,
    pub no_ask: Decimal,
    pub last_price: Option<Decimal>,
    pub spread: Decimal,
    pub open_interest: u64,
    pub volume_24h_cents: Option<u64>,
    pub rules_primary: String,
}

pub struct PolymarketSpecificData {
    pub slug: String,
    pub condition_id: String,
    pub clob_token_ids: Vec<String>,
    pub liquidity_usd: Decimal,
    pub is_multi_outcome: bool,
    pub outcome_count: usize,
    pub leading_outcome: Option<String>,
    pub image_url: Option<String>,
}
```

---

### Phase 2: Market Matching & Spreads (Days 2-3)

**Goal:** Show equivalent markets side-by-side with price spreads

#### 2.1 Create Manual Mappings File

```json
// config/market_mappings.json
{
  "version": 1,
  "mappings": [
    {
      "unified_id": "btc-100k-2025",
      "title": "Bitcoin $100k by end of 2025",
      "kalshi_ticker": "KXBTC-100K-25DEC31",
      "polymarket_slug": "will-bitcoin-reach-100000-in-2025",
      "notes": "Kalshi uses CoinGecko, Polymarket uses consensus reporting"
    },
    {
      "unified_id": "fed-cuts-2025",
      "title": "Number of Fed rate cuts in 2025",
      "kalshi_ticker": "KXFED-25",
      "polymarket_slug": "how-many-fed-rate-cuts-in-2025"
    }
  ]
}
```

Start with 10-20 high-volume markets that exist on both platforms.

#### 2.2 Implement MatcherService

```rust
// terminal-services/src/matcher_service.rs

pub struct MatchedMarket {
    pub unified_id: String,
    pub title: String,
    pub kalshi: Option<UnifiedMarket>,
    pub polymarket: Option<UnifiedMarket>,
    pub spread: Option<Decimal>,        // |kalshi_yes - poly_yes|
    pub spread_pct: Option<Decimal>,    // spread as percentage
    pub better_price_platform: Option<Platform>,
}

impl MatcherService {
    pub fn match_markets(&self, markets: &[UnifiedMarket]) -> Vec<MatchedMarket> {
        let mappings = self.load_mappings();
        
        mappings.iter().map(|m| {
            let kalshi = markets.iter()
                .find(|mkt| mkt.kalshi_data.as_ref()
                    .map(|k| k.ticker == m.kalshi_ticker).unwrap_or(false));
            
            let polymarket = markets.iter()
                .find(|mkt| mkt.polymarket_data.as_ref()
                    .map(|p| p.slug == m.polymarket_slug).unwrap_or(false));
            
            let spread = match (kalshi, polymarket) {
                (Some(k), Some(p)) => Some((k.yes_price - p.yes_price).abs()),
                _ => None,
            };
            
            MatchedMarket {
                unified_id: m.unified_id.clone(),
                title: m.title.clone(),
                kalshi: kalshi.cloned(),
                polymarket: polymarket.cloned(),
                spread,
                spread_pct: spread.map(|s| s * dec!(100)),
                better_price_platform: determine_better_price(kalshi, polymarket),
            }
        }).collect()
    }
}
```

#### 2.3 Add Spreads API Endpoint

```rust
// terminal-api/src/routes/spreads.rs

// GET /api/spreads?min_spread=0.02
pub async fn get_spreads(
    Query(params): Query<SpreadsParams>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let markets = state.market_service.list_markets().await?;
    let matched = state.matcher_service.match_markets(&markets);
    
    let filtered: Vec<_> = matched.into_iter()
        .filter(|m| m.spread.map(|s| s >= params.min_spread).unwrap_or(false))
        .sorted_by(|a, b| b.spread.cmp(&a.spread))  // Largest spread first
        .collect();
    
    Json(filtered)
}
```

#### 2.4 Frontend Spreads View

New component showing side-by-side comparison:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ SPREADS                                              Min spread: [2%  ▼]    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│ Bitcoin $100k by 2025                                         Spread: 9%   │
│ ┌────────────────────────────┐  ┌────────────────────────────┐             │
│ │ KALSHI              43%    │  │ POLYMARKET          52%    │  ◀── Better │
│ │ Bid: 42¢  Ask: 44¢        │  │ $2.5M volume               │             │
│ │ $15k volume               │  │                            │             │
│ └────────────────────────────┘  └────────────────────────────┘             │
│                                                                             │
│ Fed Rate Cuts 2025                                            Spread: 3%   │
│ ┌────────────────────────────┐  ┌────────────────────────────┐             │
│ │ KALSHI              67%    │  │ POLYMARKET          64%    │             │
│ │ ...                        │  │ ...                        │             │
│ └────────────────────────────┘  └────────────────────────────┘             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### Phase 3: Caching & Performance (Day 3-4)

**Goal:** Don't hit external APIs on every request

#### 3.1 In-Memory Cache

```rust
// terminal-services/src/cache.rs

use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

pub struct MarketCache {
    markets: Arc<RwLock<CachedData<Vec<UnifiedMarket>>>>,
    ttl: Duration,
}

struct CachedData<T> {
    data: T,
    fetched_at: Instant,
}

impl MarketCache {
    pub async fn get_markets(&self) -> Vec<UnifiedMarket> {
        let cache = self.markets.read().await;
        if cache.fetched_at.elapsed() < self.ttl {
            return cache.data.clone();
        }
        drop(cache);
        
        // Refresh
        self.refresh().await
    }
    
    pub async fn refresh(&self) -> Vec<UnifiedMarket> {
        let kalshi = self.kalshi_client.list_markets().await.unwrap_or_default();
        let poly = self.poly_client.list_events().await.unwrap_or_default();
        
        let combined = combine_and_normalize(kalshi, poly);
        
        let mut cache = self.markets.write().await;
        cache.data = combined.clone();
        cache.fetched_at = Instant::now();
        
        combined
    }
}
```

#### 3.2 Background Refresh Job

```rust
// terminal-api/src/main.rs

// Spawn background task on startup
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 min
    loop {
        interval.tick().await;
        if let Err(e) = cache.refresh().await {
            tracing::error!("Cache refresh failed: {}", e);
        }
    }
});
```

---

### Phase 4: Polish & Documentation (Days 4-5)

#### 4.1 Frontend Improvements

- [ ] Loading skeletons while fetching
- [ ] Error states with retry button
- [ ] Sort by: volume, spread, close date, alphabetical
- [ ] Category filter pills (Politics, Crypto, Sports, Economics)
- [ ] Click row → side panel with full market details
- [ ] Platform-specific data shown in detail view
- [ ] External link buttons to original platform

#### 4.2 API Improvements

- [ ] Add `/api/markets/:platform/:id` for single market
- [ ] Add query params: `?category=`, `?status=`, `?sort_by=`
- [ ] Proper error responses with status codes
- [ ] Request logging with tracing

#### 4.3 Documentation

- [ ] README with setup instructions
- [ ] Environment variables documented
- [ ] API reference (endpoints, params, response shapes)
- [ ] Architecture diagram
- [ ] Manual mapping contribution guide

---

### Phase 5: Stretch Goals (If Time Permits)

#### WebSocket Live Prices

```rust
// terminal-api/src/websocket.rs

// Client subscribes to specific markets
// Backend maintains connections to Kalshi + Polymarket WSS
// Broadcasts price updates to subscribed clients

// Frontend:
// const ws = new WebSocket('ws://localhost:3001/ws');
// ws.send(JSON.stringify({ subscribe: ['kalshi:KXBTC-100K', 'poly:bitcoin-100k'] }));
// ws.onmessage = (e) => updatePrice(JSON.parse(e.data));
```

#### Portfolio Integration

- Kalshi: Requires API key + RSA private key for auth
- Polymarket: Requires wallet signature

```rust
// GET /api/portfolio
{
  "kalshi": {
    "balance_usd": 1500.00,
    "positions": [
      { "ticker": "KXBTC-100K", "side": "yes", "quantity": 100, "avg_price": 0.40 }
    ]
  },
  "polymarket": {
    "balance_usdc": 2000.00,
    "positions": [...]
  },
  "total_exposure_usd": 3500.00
}
```

#### Semantic Market Matching Suggestions

Use embeddings to suggest potential matches:
```rust
// Generate embeddings for all market titles
// Find pairs with cosine similarity > 0.9
// Present as "suggested matches" for human review
// Human confirms → adds to market_mappings.json
```

---

## Success Criteria

### MVP Complete (End of Week)

- [ ] **All markets visible** — Full pagination, 500+ markets from each platform
- [ ] **Clean data** — Events (not options), normalized prices/volume
- [ ] **Spreads view** — 10+ manually matched markets with spread calculation
- [ ] **Performant** — Cached data, <500ms response times
- [ ] **Documented** — README explains setup, architecture, usage
- [ ] **Demo-ready** — Can record compelling 5-min walkthrough

### Demo Script

1. Show unified markets view (both platforms)
2. Filter by platform, search by title
3. Switch to spreads view
4. Highlight a market with 5%+ spread
5. Click through to see platform-specific details
6. Explain architecture (Rust backend, data normalization, caching)
7. Discuss why market matching is hard (resolution differences)
8. Show manual mapping approach as pragmatic solution

---

## Technical Reminders

### API Endpoints Reference

```
Kalshi REST:
  Base: https://api.elections.kalshi.com/trade-api/v2
  Demo: https://demo-api.kalshi.co/trade-api/v2
  
  GET /markets?limit=200&cursor={cursor}
  GET /markets/{ticker}
  GET /events
  GET /events/{event_ticker}

Polymarket REST:
  Gamma: https://gamma-api.polymarket.com
  
  GET /events?active=true&closed=false&limit=100&offset={offset}
  GET /events/{id}
  GET /markets?active=true  (avoid — returns individual options)

Polymarket Subgraph (historical):
  https://api.goldsky.com/api/public/project_cl6mb8i9h0003e201j6li0diw/subgraphs/polymarket-trading/prod/gn
```

### Environment Variables

```bash
# Backend
KALSHI_ENVIRONMENT=demo              # or "prod"
KALSHI_API_KEY=                      # Optional for read-only
KALSHI_PRIVATE_KEY_PATH=             # Optional for read-only
SERVER_PORT=3001
RUST_LOG=info,terminal=debug

# Frontend
NEXT_PUBLIC_API_URL=http://localhost:3001
```

### Running Locally

```bash
# Terminal 1: Backend
cd barter-rs
cargo run -p terminal-api

# Terminal 2: Frontend
cd barter-rs/frontend
bun run dev

# Open http://localhost:3000
```

---

## Open Questions (Decisions Needed)

1. **Multi-outcome display:** Collapse to single row with "18 options" badge, or show top 3 options?

2. **Price display:** Show midpoint, last trade, or best bid/ask? (Currently inconsistent)

3. **Spread calculation:** Use yes_price difference, or account for bid/ask spread on Kalshi?

4. **Category taxonomy:** Use Kalshi's categories, Polymarket's tags, or create unified taxonomy?

5. **Resolution warnings:** Flag matched markets with different resolution sources?

---

## Reference: What The APIs Return

### Kalshi Market Object
```json
{
  "ticker": "KXBTC-25DEC31-T100000",
  "event_ticker": "KXBTC",
  "title": "Bitcoin above $100,000 on December 31?",
  "yes_bid": 0.42,
  "yes_ask": 0.44,
  "no_bid": 0.56,
  "no_ask": 0.58,
  "last_price": 0.43,
  "volume": 1523400,
  "open_interest": 892000,
  "status": "active",
  "close_time": "2025-12-31T23:59:00Z",
  "result": null
}
```

### Polymarket Event Object
```json
{
  "id": "12345",
  "slug": "will-bitcoin-reach-100000-in-2025",
  "title": "Will Bitcoin reach $100,000 in 2025?",
  "volume": "2500000.00",
  "liquidity": "150000.00",
  "endDate": "2025-12-31T23:59:59Z",
  "active": true,
  "closed": false,
  "markets": [
    {
      "id": "0xabc123...",
      "outcomePrices": "[\"0.52\", \"0.48\"]",
      "outcomes": "[\"Yes\", \"No\"]",
      "clobTokenIds": "[\"98765...\", \"98766...\"]"
    }
  ]
}
```
