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
}

// Option data for multi-outcome events
export interface MarketOption {
  name: string;
  yes_price: string;
  market_id: string;
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
