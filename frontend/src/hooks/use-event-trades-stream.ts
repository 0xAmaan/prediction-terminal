"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useWebSocketContext } from "@/providers/websocket-provider";
import type { MarketOption, Trade, Platform } from "@/lib/types";
import type {
  ConnectionState,
  ServerMessage,
  TradeUpdate,
} from "./use-websocket";

// ============================================================================
// Event Trades Stream Types
// ============================================================================

interface UseEventTradesStreamOptions {
  /** Platform (kalshi or polymarket) */
  platform: Platform;
  /** Event ID */
  eventId: string;
  /** Market options from the event (each has clob_token_id for subscription) */
  options: MarketOption[];
  /** Maximum number of trades to keep */
  maxTrades?: number;
}

interface UseEventTradesStreamReturn {
  /** Current connection state */
  connectionState: ConnectionState;
  /** Recent trades from all outcomes (sorted by timestamp, newest first) */
  trades: Trade[];
  /** Error message */
  error: string | null;
  /** Latency to server (ms) */
  latency: number | null;
  /** Number of active subscriptions */
  subscriptionCount: number;
}

// ============================================================================
// Event Trades Stream Hook
// ============================================================================

/**
 * Hook for aggregating trades from multiple outcomes within a multi-outcome event.
 * Subscribes to trades for each outcome's clob_token_id and aggregates them into
 * a single sorted list with outcome names populated.
 */
export const useEventTradesStream = (
  options: UseEventTradesStreamOptions,
): UseEventTradesStreamReturn => {
  const {
    platform,
    eventId,
    options: marketOptions,
    maxTrades = 100,
  } = options;

  // Use the singleton WebSocket context
  const { connectionState, error, subscribe, unsubscribe, onMessage, latency } =
    useWebSocketContext();

  const [trades, setTrades] = useState<Trade[]>([]);
  const subscribedIdsRef = useRef<Set<string>>(new Set());

  // Build a lookup map: market_id/clob_token_id -> MarketOption
  // This allows us to quickly find the outcome name for incoming trades
  const optionLookup = useMemo(() => {
    const map = new Map<string, MarketOption>();
    for (const option of marketOptions) {
      // Map by market_id (primary key)
      if (option.market_id) {
        map.set(option.market_id, option);
      }
      // Also map by clob_token_id (the ID used in WebSocket subscriptions)
      if (option.clob_token_id) {
        map.set(option.clob_token_id, option);
      }
      // And by condition_id (sometimes trades come with this)
      if (option.condition_id) {
        map.set(option.condition_id, option);
      }
    }
    return map;
  }, [marketOptions]);

  // Get list of IDs to subscribe to (clob_token_id for each option)
  const subscriptionIds = useMemo(() => {
    return marketOptions
      .filter((opt) => opt.clob_token_id)
      .map((opt) => opt.clob_token_id as string);
  }, [marketOptions]);

  // Handle incoming trade messages
  const handleMessage = useCallback(
    (message: ServerMessage) => {
      // Only process trade updates
      if (message.type !== "trade_update") {
        return;
      }

      const tradeMsg = message as TradeUpdate;

      // Check if this trade is for one of our subscribed markets
      // The market_id in the trade should match one of our clob_token_ids
      const matchingOption = optionLookup.get(tradeMsg.market_id);

      // Also check by trade.market_id if different from message.market_id
      const tradeOption =
        matchingOption || optionLookup.get(tradeMsg.trade.market_id);

      // If we can't find a matching option, this trade isn't for our event
      if (!tradeOption && !subscribedIdsRef.current.has(tradeMsg.market_id)) {
        return;
      }

      // Enrich the trade with outcome_name
      const enrichedTrade: Trade = {
        ...tradeMsg.trade,
        outcome_name: tradeOption?.name || "Unknown",
      };

      // Add to trades list (sorted by timestamp, newest first, deduped)
      setTrades((prev) => {
        // Check for duplicate by trade ID
        if (prev.some((t) => t.id === enrichedTrade.id)) {
          return prev;
        }

        // Insert in sorted order (newest first)
        const newTrades = [enrichedTrade, ...prev];

        // Sort by timestamp descending (newest first)
        newTrades.sort(
          (a, b) =>
            new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime(),
        );

        // Cap at maxTrades
        return newTrades.slice(0, maxTrades);
      });
    },
    [optionLookup, maxTrades],
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

    // Subscribe to trades for each outcome's clob_token_id
    const idsToSubscribe = subscriptionIds.filter(
      (id) => !subscribedIdsRef.current.has(id),
    );

    for (const tokenId of idsToSubscribe) {
      subscribe({ type: "trades", platform, market_id: tokenId });
      subscribedIdsRef.current.add(tokenId);
    }

    // Cleanup: unsubscribe when unmounting
    return () => {
      for (const tokenId of subscribedIdsRef.current) {
        unsubscribe({ type: "trades", platform, market_id: tokenId });
      }
      subscribedIdsRef.current.clear();
    };
  }, [connectionState, platform, subscriptionIds, subscribe, unsubscribe]);

  // Clear trades when event changes
  useEffect(() => {
    setTrades([]);
    subscribedIdsRef.current.clear();
  }, [eventId]);

  return {
    connectionState,
    trades,
    error,
    latency,
    subscriptionCount: subscribedIdsRef.current.size,
  };
};
