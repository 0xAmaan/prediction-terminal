"use client";

import { useCallback, useEffect, useState } from "react";
import { useWebSocketContext } from "@/providers/websocket-provider";
import type {
  Platform,
  ServerMessage,
  ConnectionState,
  NewsItem,
  NewsUpdate,
} from "./use-websocket";

// ============================================================================
// News Stream Hook
// ============================================================================

interface UseNewsStreamOptions {
  /** Subscribe to global news */
  subscribeGlobal?: boolean;
  /** Subscribe to specific market news */
  market?: {
    platform: Platform;
    marketId: string;
  };
  /** Maximum news items to keep */
  maxItems?: number;
}

interface UseNewsStreamReturn {
  /** Current connection state */
  connectionState: ConnectionState;
  /** News items (newest first) */
  news: NewsItem[];
  /** Error message */
  error: string | null;
}

export const useNewsStream = (
  options: UseNewsStreamOptions = {},
): UseNewsStreamReturn => {
  const { subscribeGlobal = true, market, maxItems = 50 } = options;

  const { connectionState, error, subscribe, unsubscribe, onMessage } =
    useWebSocketContext();

  const [news, setNews] = useState<NewsItem[]>([]);

  // Handle incoming messages
  const handleMessage = useCallback(
    (message: ServerMessage) => {
      if (message.type !== "news_update") {
        return;
      }

      const newsUpdate = message as NewsUpdate;

      // Filter based on subscription type
      if (market) {
        // If subscribed to market news, only accept messages for that market
        if (!newsUpdate.market_context) return;
        if (newsUpdate.market_context.platform !== market.platform) return;
        if (newsUpdate.market_context.market_id !== market.marketId) return;
      } else if (!subscribeGlobal) {
        // Not subscribed to anything
        return;
      }

      setNews((prev) => {
        // Deduplicate by ID
        if (prev.some((item) => item.id === newsUpdate.item.id)) {
          return prev;
        }
        return [newsUpdate.item, ...prev].slice(0, maxItems);
      });
    },
    [market, subscribeGlobal, maxItems],
  );

  // Register message handler
  useEffect(() => {
    const unregister = onMessage(handleMessage);
    return unregister;
  }, [onMessage, handleMessage]);

  // Manage subscriptions based on connection state
  useEffect(() => {
    if (connectionState !== "connected") {
      return;
    }

    // Subscribe to requested channels
    if (subscribeGlobal && !market) {
      subscribe({ type: "global_news" });
    }
    if (market) {
      subscribe({
        type: "market_news",
        platform: market.platform,
        market_id: market.marketId,
      });
    }

    // Cleanup: unsubscribe when unmounting or options change
    return () => {
      if (subscribeGlobal && !market) {
        unsubscribe({ type: "global_news" });
      }
      if (market) {
        unsubscribe({
          type: "market_news",
          platform: market.platform,
          market_id: market.marketId,
        });
      }
    };
  }, [connectionState, subscribeGlobal, market, subscribe, unsubscribe]);

  return {
    connectionState,
    news,
    error,
  };
};
