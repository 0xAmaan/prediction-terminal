"use client";

import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { cn } from "@/lib/utils";
import { api } from "@/lib/api";
import { useEventTradesStream } from "@/hooks/use-event-trades-stream";
import { ConnectionIndicator } from "./connection-indicator";
import type { Trade, MarketOption, Platform } from "@/lib/types";
import { Activity, ExternalLink } from "lucide-react";

// Fey color tokens
const fey = {
  bg300: "#131419",
  bg400: "#16181C",
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  teal: "#4DBE95",
  tealMuted: "rgba(77, 190, 149, 0.1)",
  red: "#D84F68",
  redMuted: "rgba(216, 79, 104, 0.1)",
  skyBlue: "#54BBF7",
  border: "rgba(255, 255, 255, 0.06)",
};

// ============================================================================
// Types
// ============================================================================

interface EventTradeHistoryProps {
  platform: Platform;
  eventId: string;
  options: MarketOption[];
  maxTrades?: number;
}

// ============================================================================
// Helpers
// ============================================================================

const formatPrice = (price: string | number): string => {
  const p = typeof price === "string" ? parseFloat(price) : price;
  return (p * 100).toFixed(1) + "¢";
};

const formatQuantity = (quantity: string | number): string => {
  const q = typeof quantity === "string" ? parseFloat(quantity) : quantity;
  if (q >= 1000000) {
    return (q / 1000000).toFixed(1) + "M";
  }
  if (q >= 1000) {
    return (q / 1000).toFixed(1) + "K";
  }
  return q.toFixed(0);
};

const formatTime = (timestamp: string): string => {
  const date = new Date(timestamp);
  const now = new Date();
  const diff = now.getTime() - date.getTime();

  // Less than 1 minute ago
  if (diff < 60000) {
    return "Just now";
  }

  // Less than 1 hour ago
  if (diff < 3600000) {
    const mins = Math.floor(diff / 60000);
    return `${mins}m ago`;
  }

  // Less than 24 hours ago
  if (diff < 86400000) {
    const hours = Math.floor(diff / 3600000);
    return `${hours}h ago`;
  }

  // Format as date
  return date.toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
};

// Truncate outcome name for display
const truncateOutcome = (name: string, maxLength: number = 20): string => {
  if (name.length <= maxLength) return name;
  return name.substring(0, maxLength - 1) + "…";
};

// Get Polygonscan URL for a transaction hash
const getPolygonscanUrl = (txHash: string): string => {
  return `https://polygonscan.com/tx/${txHash}`;
};

// ============================================================================
// Event Trade Row Component
// ============================================================================

interface EventTradeRowProps {
  trade: Trade;
}

const EventTradeRow = ({ trade }: EventTradeRowProps) => {
  const isBuy = trade.side?.toLowerCase() === "buy";

  return (
    <div
      className="grid grid-cols-[1fr_1fr_auto_1.5fr_1fr_auto] gap-2 px-2 py-1.5 text-xs transition-colors"
      onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = fey.bg400)}
      onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = "transparent")}
    >
      {/* Price */}
      <div
        className="font-medium"
        style={{ color: isBuy ? fey.teal : fey.red }}
      >
        {formatPrice(trade.price)}
      </div>

      {/* Quantity */}
      <div className="text-right font-mono" style={{ color: fey.grey100 }}>
        {formatQuantity(trade.quantity)}
      </div>

      {/* Side */}
      <div className="text-center">
        <span
          className="inline-flex items-center rounded-full px-1.5 py-0.5 text-[10px] font-medium"
          style={{
            backgroundColor: isBuy ? fey.tealMuted : fey.redMuted,
            color: isBuy ? fey.teal : fey.red,
          }}
        >
          {isBuy ? "Buy" : "Sell"}
        </span>
      </div>

      {/* Outcome Name */}
      <div
        className="truncate"
        style={{ color: fey.grey500 }}
        title={trade.outcome_name || "Unknown"}
      >
        {truncateOutcome(trade.outcome_name || "Unknown")}
      </div>

      {/* Time */}
      <div className="text-right" style={{ color: fey.grey500 }}>
        {formatTime(trade.timestamp)}
      </div>

      {/* Tx Link */}
      <div className="text-center">
        {trade.transaction_hash ? (
          <a
            href={getPolygonscanUrl(trade.transaction_hash)}
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center justify-center transition-colors"
            style={{ color: fey.grey500 }}
            onMouseEnter={(e) => (e.currentTarget.style.color = fey.skyBlue)}
            onMouseLeave={(e) => (e.currentTarget.style.color = fey.grey500)}
            title={`View on Polygonscan: ${trade.transaction_hash.slice(0, 10)}...`}
          >
            <ExternalLink className="h-3 w-3" />
          </a>
        ) : (
          <span style={{ color: "rgba(125, 139, 150, 0.3)" }}>—</span>
        )}
      </div>
    </div>
  );
};

// ============================================================================
// Loading Skeleton
// ============================================================================

const EventTradeHistorySkeleton = () => (
  <div className="space-y-1">
    {Array.from({ length: 8 }).map((_, i) => (
      <div key={i} className="grid grid-cols-[1fr_1fr_auto_1.5fr_1fr_auto] gap-2 px-2 py-1.5">
        <div className="h-4 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
        <div className="h-4 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
        <div className="h-4 w-8 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
        <div className="h-4 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
        <div className="h-4 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
        <div className="h-4 w-4 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
      </div>
    ))}
  </div>
);

// ============================================================================
// Main Event Trade History Component
// ============================================================================

export const EventTradeHistory = ({
  platform,
  eventId,
  options,
  maxTrades = 50,
}: EventTradeHistoryProps) => {
  // WebSocket streaming for live trades
  const {
    connectionState,
    trades: wsTrades,
    latency,
    subscriptionCount,
  } = useEventTradesStream({
    platform,
    eventId,
    options,
    maxTrades,
  });

  // Fetch initial trades via REST API for each outcome
  const { data: restTrades, isLoading } = useQuery({
    queryKey: ["event-trades", platform, eventId],
    queryFn: async () => {
      // Fetch trades for each outcome in parallel
      const tradePromises = options
        .filter((opt) => opt.condition_id || opt.market_id)
        .map(async (opt) => {
          try {
            const response = await api.getOutcomeTrades(
              platform,
              eventId,
              opt.market_id,
              20 // Limit per outcome
            );
            // Enrich with outcome name
            return (response?.trades || []).map((t) => ({
              ...t,
              outcome_name: opt.name,
            }));
          } catch {
            return [];
          }
        });

      const results = await Promise.all(tradePromises);
      const allTrades = results.flat();

      // Sort by timestamp descending and dedupe
      const seen = new Set<string>();
      return allTrades
        .filter((t) => {
          if (seen.has(t.id)) return false;
          seen.add(t.id);
          return true;
        })
        .sort(
          (a, b) =>
            new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
        )
        .slice(0, maxTrades);
    },
    enabled: options.length > 0,
    staleTime: 5 * 60 * 1000, // Real-time updates via WebSocket
  });

  // Merge WebSocket trades with REST trades (WS takes priority)
  const mergedTrades = useMemo(() => {
    const wsTradeIds = new Set(wsTrades.map((t) => t.id));
    const uniqueRestTrades = (restTrades || []).filter(
      (t) => !wsTradeIds.has(t.id)
    );

    // Combine and sort
    const allTrades = [...wsTrades, ...uniqueRestTrades];
    allTrades.sort(
      (a, b) =>
        new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
    );

    return allTrades.slice(0, maxTrades);
  }, [wsTrades, restTrades, maxTrades]);

  const showLoading = isLoading && wsTrades.length === 0;

  return (
    <div
      className="rounded-lg"
      style={{
        backgroundColor: fey.bg300,
        border: `1px solid ${fey.border}`,
      }}
    >
      <div className="p-5 pb-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div
              className="p-1.5 rounded"
              style={{ backgroundColor: "rgba(84, 187, 247, 0.1)" }}
            >
              <Activity className="h-4 w-4" style={{ color: fey.skyBlue }} />
            </div>
            <span
              className="text-base font-semibold"
              style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
            >
              Live Trades
            </span>
            {subscriptionCount > 0 && (
              <span
                className="text-[10px] px-1.5 py-0.5 rounded font-medium"
                style={{ backgroundColor: fey.bg400, color: fey.grey500 }}
              >
                {subscriptionCount} streams
              </span>
            )}
          </div>
          <div className="flex items-center gap-2">
            {mergedTrades.length > 0 && (
              <span className="text-xs" style={{ color: fey.grey500 }}>
                {mergedTrades.length} trades
              </span>
            )}
            <ConnectionIndicator
              state={connectionState}
              latency={latency}
              showLabel={false}
            />
          </div>
        </div>
      </div>
      <div className="px-5 pb-5">
        {showLoading ? (
          <EventTradeHistorySkeleton />
        ) : (
          <>
            {/* Header */}
            <div
              className="grid grid-cols-[1fr_1fr_auto_1.5fr_1fr_auto] gap-2 px-2 pb-2 text-xs font-medium"
              style={{ color: fey.grey500, borderBottom: `1px solid ${fey.border}` }}
            >
              <div>Price</div>
              <div className="text-right">Qty</div>
              <div className="text-center">Side</div>
              <div>Outcome</div>
              <div className="text-right">Time</div>
              <div className="text-center">Tx</div>
            </div>

            {/* Trades */}
            <div className="max-h-[400px] overflow-y-auto">
              {mergedTrades.length > 0 ? (
                mergedTrades.map((trade, index) => (
                  <EventTradeRow key={`${trade.id}-${index}`} trade={trade} />
                ))
              ) : (
                <div className="py-8 text-center text-sm" style={{ color: fey.grey500 }}>
                  No trades yet for this event
                </div>
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
};
