"use client";

import { useCallback, useEffect, useState } from "react";
import { useWebSocketContext } from "@/providers/websocket-provider";
import type {
  Platform,
  OrderBookUpdate,
  PriceUpdate,
  TradeUpdate,
  ServerMessage,
  ConnectionState,
  OrderBookLevel,
  Trade,
} from "./use-websocket";

// ============================================================================
// Market Stream Types
// ============================================================================

export interface MarketPrices {
  yesPrice: string;
  noPrice: string;
  timestamp: string;
}

export interface MarketOrderBook {
  yesBids: OrderBookLevel[];
  yesAsks: OrderBookLevel[];
  noBids: OrderBookLevel[];
  noAsks: OrderBookLevel[];
  timestamp: string;
}

// ============================================================================
// Market Stream Hook
// ============================================================================

interface UseMarketStreamOptions {
  /** Platform (kalshi or polymarket) */
  platform: Platform;
  /** Market ID */
  marketId: string;
  /** Subscribe to price updates */
  subscribePrices?: boolean;
  /** Subscribe to order book updates */
  subscribeOrderBook?: boolean;
  /** Subscribe to trade updates */
  subscribeTrades?: boolean;
  /** Maximum number of trades to keep */
  maxTrades?: number;
}

interface UseMarketStreamReturn {
  /** Current connection state */
  connectionState: ConnectionState;
  /** Latest prices */
  prices: MarketPrices | null;
  /** Latest order book */
  orderBook: MarketOrderBook | null;
  /** Recent trades */
  trades: Trade[];
  /** Error message */
  error: string | null;
  /** Latency to server (ms) */
  latency: number | null;
}

export const useMarketStream = (
  options: UseMarketStreamOptions,
): UseMarketStreamReturn => {
  const {
    platform,
    marketId,
    subscribePrices = true,
    subscribeOrderBook = true,
    subscribeTrades = true,
    maxTrades = 50,
  } = options;

  // Use the singleton WebSocket context instead of creating a new connection
  const { connectionState, error, subscribe, unsubscribe, onMessage, latency } =
    useWebSocketContext();

  const [prices, setPrices] = useState<MarketPrices | null>(null);
  const [orderBook, setOrderBook] = useState<MarketOrderBook | null>(null);
  const [trades, setTrades] = useState<Trade[]>([]);

  // Handle incoming messages
  const handleMessage = useCallback(
    (message: ServerMessage) => {
      // Check if message is for our market
      if ("market_id" in message && message.market_id !== marketId) {
        return;
      }
      if ("platform" in message && message.platform !== platform) {
        return;
      }

      switch (message.type) {
        case "price_update": {
          const priceMsg = message as PriceUpdate;
          setPrices({
            yesPrice: priceMsg.yes_price,
            noPrice: priceMsg.no_price,
            timestamp: priceMsg.timestamp,
          });
          break;
        }
        case "order_book_update": {
          const obMsg = message as OrderBookUpdate;
          setOrderBook({
            yesBids: obMsg.yes_bids,
            yesAsks: obMsg.yes_asks,
            noBids: obMsg.no_bids,
            noAsks: obMsg.no_asks,
            timestamp: obMsg.timestamp,
          });
          break;
        }
        case "trade_update": {
          const tradeMsg = message as TradeUpdate;
          setTrades((prev) => {
            const newTrades = [tradeMsg.trade, ...prev];
            return newTrades.slice(0, maxTrades);
          });
          break;
        }
      }
    },
    [marketId, platform, maxTrades],
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
    if (subscribePrices) {
      subscribe({ type: "price", platform, market_id: marketId });
    }
    if (subscribeOrderBook) {
      subscribe({ type: "order_book", platform, market_id: marketId });
    }
    if (subscribeTrades) {
      subscribe({ type: "trades", platform, market_id: marketId });
    }

    // Cleanup: unsubscribe when unmounting or options change
    return () => {
      if (subscribePrices) {
        unsubscribe({ type: "price", platform, market_id: marketId });
      }
      if (subscribeOrderBook) {
        unsubscribe({ type: "order_book", platform, market_id: marketId });
      }
      if (subscribeTrades) {
        unsubscribe({ type: "trades", platform, market_id: marketId });
      }
    };
  }, [
    connectionState,
    platform,
    marketId,
    subscribePrices,
    subscribeOrderBook,
    subscribeTrades,
    subscribe,
    unsubscribe,
  ]);

  return {
    connectionState,
    prices,
    orderBook,
    trades,
    error,
    latency,
  };
};
