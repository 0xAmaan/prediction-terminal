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
  /** Buy volume during the period (taker buys) */
  buy_volume: string;
  /** Sell volume during the period (taker sells) */
  sell_volume: string;
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
  skip_embeddings?: boolean;
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

/** Rich source information with metadata for inline citations */
export interface SourceInfo {
  /** 1-indexed ID for citation references in content */
  id: number;
  /** The source URL */
  url: string;
  /** Page title */
  title: string | null;
  /** Site/publisher name */
  site_name: string | null;
  /** Favicon URL */
  favicon_url: string | null;
}

export interface SynthesizedReport {
  title: string;
  executive_summary: string;
  sections: ReportSection[];
  key_factors: KeyFactor[];
  confidence_assessment: string;
  /** Rich source info for inline citations */
  sources: SourceInfo[];
  /** Sources that aren't cited inline but are still relevant */
  general_sources?: string[];
  trading_analysis?: TradingAnalysis;
}

// ============================================================================
// Trading Analysis Types
// ============================================================================

export interface TradingAnalysis {
  fair_value_low: number;
  fair_value_high: number;
  current_price: number;
  implied_edge: number;
  estimate_confidence: EstimateConfidence;
  fair_value_reasoning: string;
  catalysts: Catalyst[];
  resolution_analysis: ResolutionAnalysis;
  contrarian_case: ContrarianAnalysis;
}

export type EstimateConfidence = "high" | "medium" | "low";

export interface Catalyst {
  date: string | null;
  event: string;
  expected_impact: CatalystImpact;
  direction_if_positive: Direction | null;
}

export type CatalystImpact = "high" | "medium" | "low";
export type Direction = "bullish" | "bearish";

export interface ResolutionAnalysis {
  resolution_summary: string;
  resolution_source: string | null;
  ambiguity_flags: string[];
  historical_edge_cases: string[];
}

export interface ContrarianAnalysis {
  consensus_view: string;
  contrarian_case: string;
  mispricing_reasons: string[];
  contrarian_triggers: string[];
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
  type:
    | "status_changed"
    | "progress_update"
    | "completed"
    | "failed"
    | "followup_started"
    | "document_editing"
    | "followup_completed";
  job_id: string;
  status?: ResearchStatus;
  progress?: ResearchProgress;
  report?: SynthesizedReport;
  error?: string;
  content_chunk?: string; // For document_editing streaming updates
}

// ============================================================================
// Version History Types
// ============================================================================

export interface ResearchVersion {
  key: string;
  created_at: string;
  version_number: number;
}

export interface ResearchVersionList {
  versions: ResearchVersion[];
}

// ============================================================================
// Chat Types
// ============================================================================

export type ChatRole = "user" | "assistant";

export interface ChatMessage {
  id: string;
  role: ChatRole;
  content: string;
  created_at: string;
  research_triggered: boolean;
}

export interface ChatHistory {
  messages: ChatMessage[];
}
