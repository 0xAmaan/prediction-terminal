export type Platform = "kalshi" | "polymarket";

export type MarketStatus = "open" | "closed" | "settled";

export interface PredictionMarket {
  id: string;
  platform: Platform;
  ticker: string | null;
  title: string;
  description: string | null;
  category: string | null;
  yes_price: string; // Decimal as string from Rust
  no_price: string;
  volume: string;
  liquidity: string | null;
  close_time: string | null; // ISO datetime
  created_at: string | null; // ISO datetime
  status: MarketStatus;
  image_url: string | null;
  url: string | null;
  // Multi-outcome event fields
  outcome_count: number | null;
  leading_outcome: string | null;
  is_multi_outcome: boolean;
  options_json: string | null; // JSON array of MarketOption
  resolution_source: string | null; // How the market will be resolved
  // Sports-specific fields
  is_sports: boolean;
  is_live: boolean;
  score: string | null; // e.g., "13 - 6"
  game_period: string | null; // e.g., "Q3", "2nd Half", "Map 2"
  home_team: string | null;
  away_team: string | null;
  home_odds: string | null;
  away_odds: string | null;
  spread_line: string | null;
  total_line: string | null;
}

// Option data for multi-outcome events
export interface MarketOption {
  name: string;
  yes_price: string;
  market_id: string;
  clob_token_id?: string; // YES token for orderbook/price history
  condition_id?: string; // For trades filtering
}

export interface MarketsResponse {
  markets: PredictionMarket[];
  count: number;
}

export interface UnifiedMarket {
  id: string;
  title: string;
  category: string | null;
  kalshi: PredictionMarket | null;
  polymarket: PredictionMarket | null;
  spread: string | null;
}

// Query params
export interface ListMarketsParams {
  platform?: "kalshi" | "polymarket" | "all";
  search?: string;
  limit?: number;
}

// ============================================================================
// Order Book Types
// ============================================================================

export interface OrderBookLevel {
  price: string;
  quantity: string;
  order_count: number | null;
}

export interface OrderBook {
  market_id: string;
  platform: Platform;
  timestamp: string;
  yes_bids: OrderBookLevel[];
  yes_asks: OrderBookLevel[];
  no_bids: OrderBookLevel[];
  no_asks: OrderBookLevel[];
  sequence: number | null;
}

// ============================================================================
// Trade Types
// ============================================================================

// API returns lowercase, but we accept both for flexibility
export type TradeSide = "Buy" | "Sell" | "buy" | "sell";
export type TradeOutcome = "Yes" | "No" | "yes" | "no";

export interface Trade {
  id: string;
  market_id: string;
  platform: Platform;
  timestamp: string;
  price: string;
  quantity: string;
  outcome: string; // TradeOutcome - accepts any case
  side: string | null; // TradeSide - accepts any case
}

export interface TradeHistory {
  market_id: string;
  platform: Platform;
  trades: Trade[];
  next_cursor: string | null;
}

// ============================================================================
// Price History Types (Candlestick data)
// ============================================================================

export type PriceInterval = "1m" | "5m" | "15m" | "1h" | "4h" | "1d";

export interface PriceCandle {
  timestamp: string;
  open: string;
  high: string;
  low: string;
  close: string;
  volume: string;
}

export interface PriceHistory {
  market_id: string;
  platform: Platform;
  interval: PriceInterval;
  candles: PriceCandle[];
}

// ============================================================================
// Multi-Outcome Price History Types (from Polymarket CLOB API)
// ============================================================================

/** A single price point from Polymarket's prices-history API */
export interface PriceHistoryPoint {
  t: number; // Unix timestamp in seconds
  p: number; // Price (0.0 - 1.0)
}

/** Price history for a single outcome (used in multi-line charts) */
export interface OutcomePriceHistory {
  name: string;
  market_id: string;
  color: string;
  history: PriceHistoryPoint[];
}
