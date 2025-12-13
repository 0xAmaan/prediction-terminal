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
  ChatHistory,
  ChatMessage,
  ResearchVersionList,
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
    if (params.limit) {
      searchParams.set("limit", params.limit.toString());
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

    const url = `${API_BASE}/api/news${searchParams.toString() ? `?${searchParams}` : ""}`;
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(`Failed to fetch news: ${response.statusText}`);
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
  ): Promise<NewsFeed> {
    const searchParams = new URLSearchParams();
    if (limit) {
      searchParams.set("limit", limit.toString());
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
};
