# Multi-Outcome Markets Implementation Plan

## Overview

Multi-outcome markets (like "Super Bowl Champion 2026" with 33 teams) currently show a simplified view with just the outcomes list. This document outlines how to add full support including outcome selection, order books, trades, and price charts.

## Current State

- Multi-outcome markets are detected via `is_multi_outcome: true` and `outcome_count > 2`
- Options are stored in `options_json` as a JSON array: `[{name, yes_price, market_id}, ...]`
- UI shows a simple list of outcomes with prices
- Order book, trades, and price chart are **hidden** for multi-outcome markets

## Polymarket Data Model

Each outcome in a multi-outcome event is essentially its own binary market:

```
Event (e.g., Super Bowl Champion 2026)
├── Market 1: "Los Angeles Rams" (condition_id: 0x..., clob_token_ids: [yes_token, no_token])
├── Market 2: "Philadelphia Eagles" (condition_id: 0x..., clob_token_ids: [yes_token, no_token])
├── Market 3: "Kansas City Chiefs" (condition_id: 0x..., clob_token_ids: [yes_token, no_token])
└── ... (33 total outcomes)
```

### Key IDs per Outcome
- `market_id`: Numeric ID for the outcome's market (e.g., "540224" for LA Rams)
- `condition_id`: Hex string used for trades API filtering (e.g., "0xabc123...")
- `clob_token_ids`: Array of two token IDs [yes_token, no_token] for order book

## API Endpoints Needed

### 1. Get Outcome Details
```
GET /api/markets/{platform}/{event_id}/outcomes/{outcome_id}
```
Returns detailed info for a specific outcome including:
- condition_id
- clob_token_ids (yes and no)
- current prices
- volume

### 2. Get Outcome Order Book
```
GET /api/markets/{platform}/{event_id}/outcomes/{outcome_id}/orderbook
```
Uses the outcome's clob_token_id to fetch order book from CLOB API.

### 3. Get Outcome Trades
```
GET /api/markets/{platform}/{event_id}/outcomes/{outcome_id}/trades
```
Uses the outcome's condition_id to filter trades from data API.

### 4. Get Multi-Outcome Price History
```
GET /api/markets/{platform}/{event_id}/history?outcomes=top5
```
Returns price history for top N outcomes (by volume or price).
Could also support: `?outcomes=540224,540231` for specific outcomes.

## Frontend Changes

### 1. Outcome Selector Component
```tsx
// components/market/outcome-selector.tsx
interface OutcomeSelectorProps {
  options: MarketOption[];
  selectedOutcome: MarketOption | null;
  onSelect: (outcome: MarketOption) => void;
}
```

Features:
- Dropdown or list to select an outcome
- Shows outcome name and current price
- Sorted by price (highest probability first) or alphabetically
- Search/filter for markets with many outcomes

### 2. Updated Market Page Flow

```
Multi-Outcome Market Page
├── Header (title, platform badge, status)
├── Outcome Selector (dropdown to pick outcome)
│   └── Default: highest probability outcome
├── Price Chart
│   └── Multi-line chart showing top 5 outcomes
│   └── Selected outcome highlighted
├── Selected Outcome Section
│   ├── Price Cards (Yes/No for selected outcome)
│   ├── Order Book (for selected outcome)
│   └── Trade History (for selected outcome)
└── All Outcomes List (expandable, sorted by probability)
```

### 3. Multi-Line Price Chart

Using lightweight-charts, overlay multiple series:
```tsx
const chart = createChart(container);

// Add a line series for each top outcome
topOutcomes.forEach((outcome, i) => {
  const series = chart.addLineSeries({
    color: OUTCOME_COLORS[i],
    title: outcome.name,
  });
  series.setData(outcome.priceHistory);
});
```

## Backend Changes (Rust)

### 1. Extend PolymarketClient

```rust
impl PolymarketClient {
    /// Get detailed info for a specific outcome within an event
    pub async fn get_outcome_details(
        &self,
        event_id: &str,
        outcome_market_id: &str,
    ) -> Result<OutcomeDetails, TerminalError>;

    /// Get order book for a specific outcome
    pub async fn get_outcome_orderbook(
        &self,
        event_id: &str,
        outcome_market_id: &str,
    ) -> Result<OrderBook, TerminalError>;

    /// Get trades for a specific outcome
    pub async fn get_outcome_trades(
        &self,
        event_id: &str,
        outcome_market_id: &str,
        limit: Option<u32>,
    ) -> Result<TradeHistory, TerminalError>;
}
```

### 2. New Types

```rust
/// Detailed outcome information
pub struct OutcomeDetails {
    pub market_id: String,
    pub name: String,
    pub condition_id: String,
    pub clob_token_ids: Vec<String>,
    pub yes_price: Decimal,
    pub no_price: Decimal,
    pub volume: Decimal,
}
```

### 3. Extend Market Service

```rust
impl MarketService {
    pub async fn get_outcome_orderbook(
        &self,
        platform: Platform,
        event_id: &str,
        outcome_id: &str,
    ) -> Result<OrderBook, TerminalError>;

    pub async fn get_outcome_trades(
        &self,
        platform: Platform,
        event_id: &str,
        outcome_id: &str,
        limit: Option<u32>,
    ) -> Result<TradeHistory, TerminalError>;
}
```

## Data Flow Example

User views "Super Bowl Champion 2026" and selects "Los Angeles Rams":

1. **Page Load**: Fetch event details, parse `options_json`
2. **Default Selection**: Auto-select highest probability outcome (LA Rams at 15%)
3. **Order Book**:
   - Look up LA Rams' `market_id` from options → "540224"
   - Fetch from gamma API: `GET /markets/540224` → get `clobTokenIds`
   - Fetch order book: `GET clob.polymarket.com/book?token_id={yes_token}`
4. **Trades**:
   - Look up LA Rams' `condition_id` from gamma API
   - Fetch trades: `GET data-api.polymarket.com/trades?market={condition_id}`
5. **Price Chart**:
   - Fetch price history for top 5 outcomes
   - Display as multi-line chart

## Kalshi Considerations

Kalshi also has multi-outcome events (e.g., election markets with multiple candidates).
Their structure is similar:
- Event contains multiple markets
- Each market has its own ticker for order book/trades

The same pattern applies - need to fetch outcome-specific data.

## Implementation Phases

### Phase 1: Outcome Selection UI
- Add outcome selector dropdown to market page
- When outcome selected, update URL: `/market/polymarket/23656?outcome=540224`
- Show selected outcome's basic info (price, volume)

### Phase 2: Outcome Order Book & Trades
- Add backend endpoints for outcome-specific data
- Wire up order book and trades for selected outcome
- Update frontend to fetch/display outcome-specific data

### Phase 3: Multi-Line Price Chart
- Add backend endpoint for multi-outcome price history
- Create multi-line chart component
- Highlight selected outcome on chart

### Phase 4: Polish
- Keyboard navigation for outcome selector
- URL persistence for selected outcome
- Mobile-friendly outcome selector
- Loading states and error handling

## API Response Examples

### Gamma API: Get Market Details
```bash
curl "https://gamma-api.polymarket.com/markets/540224"
```
```json
{
  "id": "540224",
  "question": "Los Angeles R",
  "conditionId": "0x...",
  "clobTokenIds": "[\"123...\", \"456...\"]",
  "outcomePrices": "[\"0.147\", \"0.853\"]",
  "volume": "2438106"
}
```

### Data API: Get Trades for Outcome
```bash
curl "https://data-api.polymarket.com/trades?market=0x{conditionId}&limit=50"
```
Returns trades filtered to this specific outcome.

### CLOB API: Get Order Book for Outcome
```bash
curl "https://clob.polymarket.com/book?token_id={clobTokenId}"
```
Returns order book for the outcome's yes token.

## Notes

- Multi-outcome markets can have 2-100+ outcomes
- Top outcomes by volume/probability are most relevant
- Consider pagination or virtualization for markets with many outcomes
- Price chart should limit to top 5-10 outcomes for readability
- Mobile UI may need different approach (tabs instead of dropdown?)
