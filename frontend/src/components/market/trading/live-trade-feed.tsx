"use client";

import { useMemo, useRef, useEffect, useState } from "react";
import { cn } from "@/lib/utils";
import type { Trade } from "@/lib/types";
import { ExternalLink, Radio } from "lucide-react";

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

interface LiveTradeFeedProps {
  trades: Trade[];
  isLoading?: boolean;
  maxTrades?: number;
  isConnected?: boolean;
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

  if (diff < 60000) return "Just now";
  if (diff < 3600000) {
    const mins = Math.floor(diff / 60000);
    return `${mins}m ago`;
  }
  if (diff < 86400000) {
    const hours = Math.floor(diff / 3600000);
    return `${hours}h ago`;
  }

  return date.toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
};

const getPolygonscanUrl = (txHash: string): string => {
  return `https://polygonscan.com/tx/${txHash}`;
};

// ============================================================================
// Trade Row Component with Animation
// ============================================================================

interface TradeRowProps {
  trade: Trade;
  isNew?: boolean;
}

const TradeRow = ({ trade, isNew = false }: TradeRowProps) => {
  const isBuy = trade.side?.toLowerCase() === "buy";
  const isYes = trade.outcome?.toLowerCase() === "yes";

  return (
    <div
      className={cn(
        "grid grid-cols-[70px_50px_44px_1fr_24px] gap-2 px-3 py-2 text-xs transition-colors",
        isNew && "animate-trade-enter",
        isNew && isBuy && "animate-trade-flash-green",
        isNew && !isBuy && "animate-trade-flash-red"
      )}
      style={{
        // Hover handled via CSS
      }}
      onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = fey.bg400)}
      onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = "transparent")}
    >
      {/* Price */}
      <div
        className="font-mono font-medium"
        style={{ color: isBuy ? fey.teal : fey.red }}
      >
        {formatPrice(trade.price)}
      </div>

      {/* Quantity */}
      <div className="font-mono text-right" style={{ color: fey.grey500 }}>
        {formatQuantity(trade.quantity)}
      </div>

      {/* Outcome Badge */}
      <div className="text-center">
        <span
          className="inline-flex items-center rounded px-1.5 py-0.5 text-[10px] font-medium"
          style={{
            backgroundColor: isYes ? fey.tealMuted : fey.redMuted,
            color: isYes ? fey.teal : fey.red,
          }}
        >
          {isYes ? "YES" : "NO"}
        </span>
      </div>

      {/* Time */}
      <div className="text-right truncate" style={{ color: fey.grey500 }}>
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
            title="View on Polygonscan"
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

const TradeFeedSkeleton = () => (
  <div className="space-y-0.5">
    {Array.from({ length: 10 }).map((_, i) => (
      <div
        key={i}
        className="grid grid-cols-[70px_50px_44px_1fr_24px] gap-2 px-3 py-2"
      >
        <div className="h-4 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
        <div className="h-4 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
        <div className="h-4 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
        <div className="h-4 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
        <div className="h-4 w-4 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
      </div>
    ))}
  </div>
);

// ============================================================================
// Main Live Trade Feed Component
// ============================================================================

export const LiveTradeFeed = ({
  trades,
  isLoading = false,
  maxTrades = 50,
  isConnected = false,
}: LiveTradeFeedProps) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [newTradeIds, setNewTradeIds] = useState<Set<string>>(new Set());
  const prevTradesRef = useRef<Trade[]>([]);

  // Track new trades for animation
  useEffect(() => {
    if (trades.length > 0 && prevTradesRef.current.length > 0) {
      const prevIds = new Set(prevTradesRef.current.map((t) => t.id));
      const newIds = trades
        .filter((t) => !prevIds.has(t.id))
        .map((t) => t.id);

      if (newIds.length > 0) {
        setNewTradeIds(new Set(newIds));
        // Clear new status after animation
        setTimeout(() => setNewTradeIds(new Set()), 1000);
      }
    }
    prevTradesRef.current = trades;
  }, [trades]);

  const displayTrades = useMemo(
    () => trades.slice(0, maxTrades),
    [trades, maxTrades]
  );

  if (isLoading) {
    return (
      <div
        className="rounded-lg"
        style={{
          backgroundColor: fey.bg300,
          border: `1px solid ${fey.border}`,
        }}
      >
        <div className="p-5 pb-3">
          <span
            className="text-sm font-semibold"
            style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
          >
            Live Trades
          </span>
        </div>
        <div className="px-0 pb-0">
          <TradeFeedSkeleton />
        </div>
      </div>
    );
  }

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
            <span
              className="text-sm font-semibold"
              style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
            >
              Live Trades
            </span>
            {isConnected && (
              <div className="flex items-center gap-1.5">
                <Radio className="h-3 w-3 animate-pulse" style={{ color: fey.skyBlue }} />
                <span className="text-[10px] font-medium" style={{ color: fey.skyBlue }}>
                  LIVE
                </span>
              </div>
            )}
          </div>
          {trades.length > 0 && (
            <span className="text-xs" style={{ color: fey.grey500 }}>
              {trades.length} trades
            </span>
          )}
        </div>
      </div>
      <div className="px-0 pb-0">
        {/* Header */}
        <div
          className="grid grid-cols-[70px_50px_44px_1fr_24px] gap-2 px-3 pb-2 text-[10px] font-medium uppercase tracking-wider"
          style={{ color: fey.grey500, borderBottom: `1px solid ${fey.border}` }}
        >
          <div>Price</div>
          <div className="text-right">Qty</div>
          <div className="text-center">Side</div>
          <div className="text-right">Time</div>
          <div className="text-center">Tx</div>
        </div>

        {/* Trades */}
        <div
          ref={containerRef}
          className="max-h-[400px] overflow-y-auto scrollbar-thin"
        >
          {displayTrades.length > 0 ? (
            displayTrades.map((trade) => (
              <TradeRow
                key={trade.id}
                trade={trade}
                isNew={newTradeIds.has(trade.id)}
              />
            ))
          ) : (
            <div className="py-12 text-center">
              <div className="text-sm" style={{ color: fey.grey500 }}>
                No trades yet
              </div>
              <div
                className="text-xs mt-1"
                style={{ color: "rgba(125, 139, 150, 0.6)" }}
              >
                Trades will appear here in real-time
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
