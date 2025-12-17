import type {
  MarketsResponse,
  ListMarketsParams,
  PredictionMarket,
  OrderBook,
  TradeHistory,
  PriceHistory,
  PriceInterval,
  OutcomePriceHistory,
  PriceHistoryPoint,
  MarketStatsResponse,
  MarketStatsParams,
  NewsFeed,
  NewsSearchParams,
  ArticleContent,
  ResearchJob,
  ResearchJobSummary,
  ChatHistory,
  ChatMessage,
  ResearchVersionList,
  MarketEdgeEntry,
} from "./types";

const API_BASE = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001";

export const api = {
  async listMarkets(params: ListMarketsParams = {}): Promise<MarketsResponse> {
    const searchParams = new URLSearchParams();

    if (params.platform && params.platform !== "all") {
      searchParams.set("platform", params.platform);
    }
    if (params.search) {
      searchParams.set("search", params.search);
    }
    if (params.filter && params.filter !== "all") {
      searchParams.set("filter", params.filter);
    }
    if (params.limit) {
      searchParams.set("limit", params.limit.toString());
    }
    if (params.sort) {
      searchParams.set("sort", params.sort);
    }

    const url = `${API_BASE}/api/markets${searchParams.toString() ? `?${searchParams}` : ""}`;
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(`Failed to fetch markets: ${response.statusText}`);
    }

    return response.json();
  },

  async getMarket(platform: string, id: string): Promise<PredictionMarket> {
    const response = await fetch(`${API_BASE}/api/markets/${platform}/${id}`);

    if (!response.ok) {
      throw new Error(`Failed to fetch market: ${response.statusText}`);
    }

    return response.json();
  },

  async healthCheck(): Promise<{ status: string; service: string }> {
    const response = await fetch(`${API_BASE}/api/health`);

    if (!response.ok) {
      throw new Error(`Health check failed: ${response.statusText}`);
    }

    return response.json();
  },

  // ========================================================================
  // Order Book & Trade Methods
  // ========================================================================

  async getOrderBook(
    platform: string,
    id: string,
    depth?: number,
  ): Promise<OrderBook> {
    const searchParams = new URLSearchParams();
    if (depth) {
      searchParams.set("depth", depth.toString());
    }

    const url = `${API_BASE}/api/markets/${platform}/${encodeURIComponent(id)}/orderbook${searchParams.toString() ? `?${searchParams}` : ""}`;
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(`Failed to fetch order book: ${response.statusText}`);
    }

    return response.json();
  },

  async getTrades(
    platform: string,
    id: string,
    limit?: number,
    cursor?: string,
  ): Promise<TradeHistory> {
    const searchParams = new URLSearchParams();
    if (limit) {
      searchParams.set("limit", limit.toString());
    }
    if (cursor) {
      searchParams.set("cursor", cursor);
    }

    const url = `${API_BASE}/api/markets/${platform}/${encodeURIComponent(id)}/trades${searchParams.toString() ? `?${searchParams}` : ""}`;
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(`Failed to fetch trades: ${response.statusText}`);
    }

    return response.json();
  },

  async getRelatedMarkets(
    platform: string,
    id: string,
    limit?: number,
  ): Promise<MarketsResponse> {
    const searchParams = new URLSearchParams();
    if (limit) {
      searchParams.set("limit", limit.toString());
    }

    const url = `${API_BASE}/api/markets/${platform}/${encodeURIComponent(id)}/related${searchParams.toString() ? `?${searchParams}` : ""}`;
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(
        `Failed to fetch related markets: ${response.statusText}`,
      );
    }

    return response.json();
  },

  // ========================================================================
  // Price History Methods
  // ========================================================================

  async getPriceHistory(
    platform: string,
    id: string,
    options?: { interval?: PriceInterval; timeframe?: string },
  ): Promise<PriceHistory> {
    const searchParams = new URLSearchParams();
    if (options?.interval) {
      searchParams.set("interval", options.interval);
    }
    if (options?.timeframe) {
      searchParams.set("timeframe", options.timeframe);
    }

    const url = `${API_BASE}/api/markets/${platform}/${encodeURIComponent(id)}/history${searchParams.toString() ? `?${searchParams}` : ""}`;
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(`Failed to fetch price history: ${response.statusText}`);
    }

    return response.json();
  },

  // ========================================================================
  // Multi-Outcome / Outcome-Specific Methods
  // ========================================================================

  /** Get price history for top N outcomes (multi-line chart data) */
  async getMultiOutcomePrices(
    platform: string,
    id: string,
    options?: { top?: number; interval?: string },
  ): Promise<OutcomePriceHistory[]> {
    const searchParams = new URLSearchParams();
    if (options?.top) {
      searchParams.set("top", options.top.toString());
    }
    if (options?.interval) {
      searchParams.set("interval", options.interval);
    }

    const url = `${API_BASE}/api/markets/${platform}/${encodeURIComponent(id)}/prices-history${searchParams.toString() ? `?${searchParams}` : ""}`;
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(
        `Failed to fetch multi-outcome prices: ${response.statusText}`,
      );
    }

    return response.json();
  },

  /** Get orderbook for a specific outcome within an event */
  async getOutcomeOrderBook(
    platform: string,
    eventId: string,
    outcomeId: string,
  ): Promise<OrderBook> {
    const url = `${API_BASE}/api/markets/${platform}/${encodeURIComponent(eventId)}/outcomes/${encodeURIComponent(outcomeId)}/orderbook`;
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(
        `Failed to fetch outcome orderbook: ${response.statusText}`,
      );
    }

    return response.json();
  },

  /** Get trades for a specific outcome within an event */
  async getOutcomeTrades(
    platform: string,
    eventId: string,
    outcomeId: string,
    limit?: number,
  ): Promise<TradeHistory> {
    const searchParams = new URLSearchParams();
    if (limit) {
      searchParams.set("limit", limit.toString());
    }

    const url = `${API_BASE}/api/markets/${platform}/${encodeURIComponent(eventId)}/outcomes/${encodeURIComponent(outcomeId)}/trades${searchParams.toString() ? `?${searchParams}` : ""}`;
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(`Failed to fetch outcome trades: ${response.statusText}`);
    }

    return response.json();
  },

  /** Get price history for a specific outcome */
  async getOutcomePriceHistory(
    platform: string,
    eventId: string,
    outcomeId: string,
    interval?: string,
  ): Promise<PriceHistoryPoint[]> {
    const searchParams = new URLSearchParams();
    if (interval) {
      searchParams.set("interval", interval);
    }

    const url = `${API_BASE}/api/markets/${platform}/${encodeURIComponent(eventId)}/outcomes/${encodeURIComponent(outcomeId)}/prices-history${searchParams.toString() ? `?${searchParams}` : ""}`;
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(
        `Failed to fetch outcome price history: ${response.statusText}`,
      );
    }

    return response.json();
  },

  // ========================================================================
  // Market Stats Methods
  // ========================================================================

  /** Get market stats with volume, txn counts, price changes for a timeframe */
  async getMarketStats(
    params: MarketStatsParams = {},
  ): Promise<MarketStatsResponse> {
    const searchParams = new URLSearchParams();
    if (params.timeframe) {
      searchParams.set("timeframe", params.timeframe);
    }
    if (params.platform) {
      searchParams.set("platform", params.platform);
    }
    if (params.limit) {
      searchParams.set("limit", params.limit.toString());
    }

    const url = `${API_BASE}/api/markets/stats${searchParams.toString() ? `?${searchParams}` : ""}`;
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(`Failed to fetch market stats: ${response.statusText}`);
    }

    return response.json();
  },

  // ========================================================================
  // News Methods
  // ========================================================================

  /** Get global prediction market news */
  async getGlobalNews(params: NewsSearchParams = {}): Promise<NewsFeed> {
    const searchParams = new URLSearchParams();
    if (params.query) {
      searchParams.set("query", params.query);
    }
    if (params.limit) {
      searchParams.set("limit", params.limit.toString());
    }
    if (params.time_range) {
      searchParams.set("time_range", params.time_range);
    }
    if (params.skip_embeddings) {
      searchParams.set("skip_embeddings", "true");
    }

    const url = `${API_BASE}/api/news${searchParams.toString() ? `?${searchParams}` : ""}`;
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(`Failed to fetch news: ${response.statusText}`);
    }

    return response.json();
  },

  /** Get AI-enriched news with market matching and buy/sell signals */
  async getEnrichedNews(): Promise<NewsFeed> {
    const url = `${API_BASE}/api/news/enriched`;
    const response = await fetch(url);

    if (!response.ok) {
      // Fall back to regular news if enriched endpoint fails
      console.warn("Enriched news not available, falling back to regular news");
      return this.getGlobalNews({ limit: 20, skip_embeddings: true });
    }

    return response.json();
  },

  /** Search news with custom query */
  async searchNews(
    query: string,
    params: Omit<NewsSearchParams, "query"> = {},
  ): Promise<NewsFeed> {
    const searchParams = new URLSearchParams();
    searchParams.set("query", query);
    if (params.limit) {
      searchParams.set("limit", params.limit.toString());
    }
    if (params.time_range) {
      searchParams.set("time_range", params.time_range);
    }

    const url = `${API_BASE}/api/news/search?${searchParams}`;
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(`Failed to search news: ${response.statusText}`);
    }

    return response.json();
  },

  /** Get full article content */
  async getArticleContent(articleUrl: string): Promise<ArticleContent> {
    const url = `${API_BASE}/api/news/article?url=${encodeURIComponent(articleUrl)}`;
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(`Failed to fetch article: ${response.statusText}`);
    }

    return response.json();
  },

  /** Get news for a specific market */
  async getMarketNews(
    platform: string,
    id: string,
    limit?: number,
    skipEmbeddings?: boolean,
  ): Promise<NewsFeed> {
    const searchParams = new URLSearchParams();
    if (limit) {
      searchParams.set("limit", limit.toString());
    }
    if (skipEmbeddings) {
      searchParams.set("skip_embeddings", "true");
    }

    const url = `${API_BASE}/api/markets/${platform}/${encodeURIComponent(id)}/news${searchParams.toString() ? `?${searchParams}` : ""}`;
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(`Failed to fetch market news: ${response.statusText}`);
    }

    return response.json();
  },

  // ========================================================================
  // Research Methods
  // ========================================================================

  async startResearch(
    platform: string,
    marketId: string,
  ): Promise<{ job_id: string; status: string }> {
    const response = await fetch(
      `${API_BASE}/api/research/${platform}/${encodeURIComponent(marketId)}`,
      { method: "POST" },
    );

    if (!response.ok) {
      throw new Error(`Failed to start research: ${response.statusText}`);
    }

    return response.json();
  },

  async getResearchJob(jobId: string): Promise<ResearchJob> {
    const response = await fetch(
      `${API_BASE}/api/research/job/${encodeURIComponent(jobId)}`,
    );

    if (!response.ok) {
      throw new Error(`Failed to get research job: ${response.statusText}`);
    }

    return response.json();
  },

  async listResearchJobs(): Promise<ResearchJob[]> {
    const response = await fetch(`${API_BASE}/api/research/jobs`);

    if (!response.ok) {
      throw new Error(`Failed to list research jobs: ${response.statusText}`);
    }

    return response.json();
  },

  /** List all saved research reports from S3 (persisted, summary only for faster loading) */
  async listSavedReports(): Promise<ResearchJobSummary[]> {
    const response = await fetch(
      `${API_BASE}/api/research/reports?summary_only=true`,
    );

    if (!response.ok) {
      throw new Error(`Failed to list saved reports: ${response.statusText}`);
    }

    return response.json();
  },

  async getResearchByMarket(
    platform: string,
    marketId: string,
  ): Promise<ResearchJob | null> {
    const response = await fetch(
      `${API_BASE}/api/research/${platform}/${encodeURIComponent(marketId)}`,
    );

    if (response.status === 404) {
      return null;
    }

    if (!response.ok) {
      throw new Error(`Failed to get research: ${response.statusText}`);
    }

    return response.json();
  },

  /** Get markets with research indicating mispricing (edge > 2%) */
  async getMispricedMarkets(): Promise<MarketEdgeEntry[]> {
    const response = await fetch(`${API_BASE}/api/research/mispriced`);

    if (!response.ok) {
      throw new Error(
        `Failed to get mispriced markets: ${response.statusText}`,
      );
    }

    return response.json();
  },

  // ========================================================================
  // Chat Methods
  // ========================================================================

  async getChatHistory(
    platform: string,
    marketId: string,
  ): Promise<ChatHistory> {
    const response = await fetch(
      `${API_BASE}/api/research/${platform}/${encodeURIComponent(marketId)}/chat`,
    );

    // 404 means no chat history exists yet - return empty history
    if (response.status === 404) {
      return { messages: [] };
    }

    if (!response.ok) {
      throw new Error(`Failed to get chat history: ${response.statusText}`);
    }

    return response.json();
  },

  async sendChatMessage(
    platform: string,
    marketId: string,
    message: string,
  ): Promise<ChatMessage> {
    const response = await fetch(
      `${API_BASE}/api/research/${platform}/${encodeURIComponent(marketId)}/chat`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ message }),
      },
    );

    if (!response.ok) {
      throw new Error(`Failed to send message: ${response.statusText}`);
    }

    const data = await response.json();
    return data.message;
  },

  // ========================================================================
  // Version History Methods
  // ========================================================================

  async getVersions(
    platform: string,
    marketId: string,
  ): Promise<ResearchVersionList> {
    const response = await fetch(
      `${API_BASE}/api/research/${platform}/${encodeURIComponent(marketId)}/versions`,
    );

    if (!response.ok) {
      throw new Error(`Failed to get versions: ${response.statusText}`);
    }

    return response.json();
  },

  async getVersion(
    platform: string,
    marketId: string,
    versionKey: string,
  ): Promise<ResearchJob> {
    const response = await fetch(
      `${API_BASE}/api/research/${platform}/${encodeURIComponent(marketId)}/versions/${encodeURIComponent(versionKey)}`,
    );

    if (!response.ok) {
      throw new Error(`Failed to get version: ${response.statusText}`);
    }

    return response.json();
  },

  // ========================================================================
  // Trading Methods
  // ========================================================================

  /** Submit a new order to Polymarket */
  async submitOrder(params: {
    tokenId: string;
    side: "buy" | "sell";
    price: number;
    size: number;
    orderType?: "GTC" | "GTD" | "FOK" | "FAK";
    /** Whether this is a neg_risk market (multi-outcome). Affects which exchange contract is used for signing. */
    negRisk?: boolean;
  }): Promise<{
    success: boolean;
    orderId?: string;
    error?: string;
    transactionHashes: string[];
  }> {
    const response = await fetch(`${API_BASE}/api/trade/order`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        tokenId: params.tokenId,
        side: params.side,
        price: params.price,
        size: params.size,
        orderType: params.orderType || "GTC",
        negRisk: params.negRisk ?? false,
      }),
    });

    if (!response.ok) {
      const data = await response.json().catch(() => ({}));
      throw new Error(
        data.error || `Failed to submit order: ${response.statusText}`,
      );
    }

    return response.json();
  },

  /** Cancel an order by ID */
  async cancelOrder(orderId: string): Promise<void> {
    const response = await fetch(
      `${API_BASE}/api/trade/order/${encodeURIComponent(orderId)}`,
      {
        method: "DELETE",
      },
    );

    if (!response.ok) {
      const data = await response.json().catch(() => ({}));
      throw new Error(
        data.error || `Failed to cancel order: ${response.statusText}`,
      );
    }
  },

  /** Cancel all orders */
  async cancelAllOrders(): Promise<void> {
    const response = await fetch(`${API_BASE}/api/trade/orders/cancel-all`, {
      method: "DELETE",
    });

    if (!response.ok) {
      const data = await response.json().catch(() => ({}));
      throw new Error(
        data.error || `Failed to cancel all orders: ${response.statusText}`,
      );
    }
  },

  /** Get open orders */
  async getOpenOrders(): Promise<
    Array<{
      id: string;
      market: string;
      assetId: string;
      side: string;
      originalSize: string;
      sizeMatched: string;
      price: string;
      status: string;
      createdAt: string;
    }>
  > {
    const response = await fetch(`${API_BASE}/api/trade/orders`);

    // 503 means trading is not configured - return empty array
    if (response.status === 503) {
      return [];
    }

    if (!response.ok) {
      throw new Error(`Failed to get open orders: ${response.statusText}`);
    }

    return response.json();
  },

  /** Get deposit address for funding the trading wallet */
  async getDepositAddress(): Promise<{
    address: string;
    network: string;
    token: string;
  }> {
    const response = await fetch(`${API_BASE}/api/trade/deposit`);

    // 503 means trading is not configured - return empty placeholder
    if (response.status === 503) {
      return { address: "", network: "Polygon", token: "USDC.e" };
    }

    if (!response.ok) {
      throw new Error(`Failed to get deposit address: ${response.statusText}`);
    }

    return response.json();
  },

  /** Get trading wallet balance */
  async getTradingBalance(): Promise<{
    usdcBalance: string;
    usdcAllowance: string;
    walletAddress: string;
    /** Whether CTF tokens are approved for selling (all required contracts) */
    ctfApproved: boolean;
    /** Whether CTF Exchange specifically is approved */
    ctfExchangeApproved: boolean;
    /** Whether Neg Risk CTF Exchange is approved */
    negRiskCtfApproved: boolean;
    /** Whether Neg Risk Adapter is approved (required for multi-outcome markets) */
    negRiskAdapterApproved: boolean;
  }> {
    const response = await fetch(`${API_BASE}/api/trade/balance`);

    // 503 means trading is not configured - return empty balance
    if (response.status === 503) {
      return {
        usdcBalance: "0",
        usdcAllowance: "0",
        walletAddress: "",
        ctfApproved: false,
        ctfExchangeApproved: false,
        negRiskCtfApproved: false,
        negRiskAdapterApproved: false,
      };
    }

    if (!response.ok) {
      throw new Error(`Failed to get balance: ${response.statusText}`);
    }

    return response.json();
  },

  /** Get current positions from Polymarket Data API */
  async getPositions(): Promise<
    Array<{
      marketId: string;
      tokenId: string;
      outcome: string;
      shares: string;
      avgPrice: string;
      currentPrice: string;
      pnl: string;
      title: string;
      negRisk: boolean;
    }>
  > {
    const response = await fetch(`${API_BASE}/api/trade/positions`);

    // 503 means trading is not configured - return empty array
    if (response.status === 503) {
      return [];
    }

    if (!response.ok) {
      throw new Error(`Failed to get positions: ${response.statusText}`);
    }

    return response.json();
  },

  /** Approve USDC spending for the CTF Exchange (required before buying) */
  async approveUsdc(): Promise<{
    success: boolean;
    transactionHash?: string;
    error?: string;
    maticBalance?: string;
  }> {
    const response = await fetch(`${API_BASE}/api/trade/approve`, {
      method: "POST",
    });

    if (!response.ok) {
      const data = await response.json().catch(() => ({}));
      throw new Error(
        data.error || `Failed to approve USDC: ${response.statusText}`,
      );
    }

    return response.json();
  },

  /** Approve CTF tokens for the exchange (required before selling) */
  async approveCtf(): Promise<{
    success: boolean;
    transactionHash?: string;
    error?: string;
    maticBalance?: string;
  }> {
    const response = await fetch(`${API_BASE}/api/trade/approve-ctf`, {
      method: "POST",
    });

    if (!response.ok) {
      const data = await response.json().catch(() => ({}));
      throw new Error(
        data.error || `Failed to approve CTF tokens: ${response.statusText}`,
      );
    }

    return response.json();
  },
};
