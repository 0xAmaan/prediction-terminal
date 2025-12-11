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
  // Tags for categorization (e.g., "Politics", "Crypto", "AI")
  tags: string[];
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
  outcome_name?: string; // For multi-outcome event trades - which sub-market this trade belongs to
  transaction_hash?: string; // For on-chain trades (Polymarket on Polygon)
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

// ============================================================================
// Market Stats Types (for enhanced table view)
// ============================================================================

/** Available timeframes for market stats */
export type Timeframe = "1h" | "24h" | "7d" | "30d";

/** Market statistics for a specific timeframe */
export interface MarketStats {
  market_id: string;
  platform: Platform;
  yes_price: string;
  no_price: string;
  /** Absolute price change (e.g., 0.0081 for +0.81 cents) */
  price_change: string;
  /** Percentage price change (e.g., 0.97 for +0.97%) */
  price_change_percent: string;
  /** Trading volume in the timeframe */
  volume: string;
  /** Number of YES trades in the timeframe */
  yes_txn_count: number;
  /** Number of NO trades in the timeframe */
  no_txn_count: number;
  /** The timeframe these stats cover */
  timeframe: Timeframe;
}

/** Response from /api/markets/stats endpoint */
export interface MarketStatsResponse {
  stats: MarketStats[];
  /** Sparkline price history for each market (market_id -> price points) */
  sparklines: Record<string, PriceHistoryPoint[]>;
  /** The timeframe used */
  timeframe: string;
  /** Number of markets */
  count: number;
}

/** Query params for fetching market stats */
export interface MarketStatsParams {
  timeframe?: Timeframe;
  platform?: Platform;
  limit?: number;
}

// ============================================================================
// News Types
// ============================================================================

export interface NewsSource {
  name: string;
  url: string;
  favicon_url: string | null;
}

export interface NewsItem {
  id: string;
  title: string;
  url: string;
  published_at: string;
  source: NewsSource;
  summary: string;
  content: string | null;
  image_url: string | null;
  relevance_score: number;
  related_market_ids: string[];
  search_query: string | null;
}

export interface NewsFeed {
  items: NewsItem[];
  total_count: number;
  next_cursor: string | null;
}

export interface ArticleContent {
  content: string;
  title: string | null;
  description: string | null;
  image_url: string | null;
  site_name: string | null;
}

export interface NewsSearchParams {
  query?: string;
  limit?: number;
  time_range?: string;
}

// ============================================================================
// Research Types
// ============================================================================

export interface ResearchJob {
  id: string;
  platform: string;
  market_id: string;
  market_title: string;
  status: ResearchStatus;
  progress: ResearchProgress;
  report?: SynthesizedReport;
  error?: string;
  created_at: string;
  updated_at: string;
  cached: boolean;
}

export type ResearchStatus =
  | "pending"
  | "decomposing"
  | "searching"
  | "analyzing"
  | "synthesizing"
  | "completed"
  | "failed";

export interface ResearchProgress {
  current_step: string;
  total_steps: number;
  completed_steps: number;
  current_query?: string;
  searches_completed: number;
  searches_total: number;
}

export interface SynthesizedReport {
  title: string;
  executive_summary: string;
  sections: ReportSection[];
  key_factors: KeyFactor[];
  confidence_assessment: string;
  sources: string[];
}

export interface ReportSection {
  heading: string;
  content: string;
}

export interface KeyFactor {
  factor: string;
  impact: "bullish" | "bearish" | "neutral";
  confidence: "high" | "medium" | "low";
}

export interface ResearchUpdate {
  type: "status_changed" | "progress_update" | "completed" | "failed";
  job_id: string;
  status?: ResearchStatus;
  progress?: ResearchProgress;
  report?: SynthesizedReport;
  error?: string;
}
