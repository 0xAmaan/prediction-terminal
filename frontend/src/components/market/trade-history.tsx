"use client";

import { useMemo } from "react";
import { cn } from "@/lib/utils";
import type { Trade } from "@/lib/types";
import { ExternalLink } from "lucide-react";

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

interface TradeHistoryProps {
  trades: Trade[];
  isLoading?: boolean;
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

// Get Polygonscan URL for a transaction hash
const getPolygonscanUrl = (txHash: string): string => {
  return `https://polygonscan.com/tx/${txHash}`;
};

// ============================================================================
// Trade Row Component
// ============================================================================

interface TradeRowProps {
  trade: Trade;
}

const TradeRow = ({ trade }: TradeRowProps) => {
  const isBuy = trade.side?.toLowerCase() === "buy";
  const isYes = trade.outcome?.toLowerCase() === "yes";

  return (
    <div
      className="grid grid-cols-[1fr_1fr_auto_1fr_auto] gap-2 px-2 py-1.5 text-xs transition-colors"
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
      <div className="text-right" style={{ color: fey.grey100 }}>
        {formatQuantity(trade.quantity)}
      </div>

      {/* Outcome */}
      <div className="text-center">
        <span
          className="inline-flex items-center rounded-full px-1.5 py-0.5 text-[10px] font-medium"
          style={{
            backgroundColor: isYes ? fey.tealMuted : fey.redMuted,
            color: isYes ? fey.teal : fey.red,
          }}
        >
          {isYes ? "Yes" : "No"}
        </span>
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

const TradeHistorySkeleton = () => (
  <div className="space-y-1">
    {Array.from({ length: 8 }).map((_, i) => (
      <div key={i} className="grid grid-cols-[1fr_1fr_auto_1fr_auto] gap-2 px-2 py-1.5">
        <div className="h-4 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
        <div className="h-4 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
        <div className="h-4 w-8 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
        <div className="h-4 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
        <div className="h-4 w-4 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
      </div>
    ))}
  </div>
);

// ============================================================================
// Main Trade History Component
// ============================================================================

export const TradeHistory = ({
  trades,
  isLoading = false,
  maxTrades = 50,
}: TradeHistoryProps) => {
  const displayTrades = useMemo(
    () => trades.slice(0, maxTrades),
    [trades, maxTrades],
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
            Recent Trades
          </span>
        </div>
        <div className="px-5 pb-5">
          <TradeHistorySkeleton />
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
          <span
            className="text-sm font-semibold"
            style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
          >
            Recent Trades
          </span>
          {trades.length > 0 && (
            <span className="text-xs" style={{ color: fey.grey500 }}>
              {trades.length} trades
            </span>
          )}
        </div>
      </div>
      <div className="px-5 pb-5">
        {/* Header */}
        <div
          className="grid grid-cols-[1fr_1fr_auto_1fr_auto] gap-2 px-2 pb-2 text-xs font-medium"
          style={{ color: fey.grey500, borderBottom: `1px solid ${fey.border}` }}
        >
          <div>Price</div>
          <div className="text-right">Qty</div>
          <div className="text-center">Side</div>
          <div className="text-right">Time</div>
          <div className="text-center">Tx</div>
        </div>

        {/* Trades */}
        <div className="max-h-[400px] overflow-y-auto">
          {displayTrades.length > 0 ? (
            displayTrades.map((trade, index) => (
              <TradeRow key={`${trade.id}-${index}`} trade={trade} />
            ))
          ) : (
            <div className="py-8 text-center text-sm" style={{ color: fey.grey500 }}>
              No trades yet
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
