import type {
  MarketsResponse,
  ListMarketsParams,
  PredictionMarket,
  OrderBook,
  TradeHistory,
  PriceHistory,
  PriceInterval,
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
};
