"use client";

import { useMemo } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";

// ============================================================================
// Types
// ============================================================================

interface Trade {
  id: string;
  market_id: string;
  platform: string;
  timestamp: string;
  price: string;
  quantity: string;
  outcome: string; // "Yes" | "No" | "yes" | "no"
  side: string | null; // "Buy" | "Sell" | "buy" | "sell"
}

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
    <div className="grid grid-cols-4 gap-2 px-2 py-1.5 text-xs hover:bg-muted/50">
      {/* Price */}
      <div
        className={cn(
          "font-medium",
          isBuy ? "text-green-500" : "text-red-500",
        )}
      >
        {formatPrice(trade.price)}
      </div>

      {/* Quantity */}
      <div className="text-right">{formatQuantity(trade.quantity)}</div>

      {/* Outcome */}
      <div className="text-right">
        <span
          className={cn(
            "inline-flex items-center rounded-full px-1.5 py-0.5 text-[10px] font-medium",
            isYes
              ? "bg-green-500/10 text-green-500"
              : "bg-red-500/10 text-red-500",
          )}
        >
          {isYes ? "Yes" : "No"}
        </span>
      </div>

      {/* Time */}
      <div className="text-right text-muted-foreground">
        {formatTime(trade.timestamp)}
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
      <div key={i} className="grid grid-cols-4 gap-2 px-2 py-1.5">
        <div className="h-4 animate-pulse rounded bg-muted" />
        <div className="h-4 animate-pulse rounded bg-muted" />
        <div className="h-4 animate-pulse rounded bg-muted" />
        <div className="h-4 animate-pulse rounded bg-muted" />
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
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-medium">Recent Trades</CardTitle>
        </CardHeader>
        <CardContent>
          <TradeHistorySkeleton />
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-medium">Recent Trades</CardTitle>
          {trades.length > 0 && (
            <span className="text-xs text-muted-foreground">
              {trades.length} trades
            </span>
          )}
        </div>
      </CardHeader>
      <CardContent>
        {/* Header */}
        <div className="grid grid-cols-4 gap-2 border-b px-2 pb-2 text-xs font-medium text-muted-foreground">
          <div>Price</div>
          <div className="text-right">Qty</div>
          <div className="text-right">Side</div>
          <div className="text-right">Time</div>
        </div>

        {/* Trades */}
        <div className="max-h-[400px] overflow-y-auto">
          {displayTrades.length > 0 ? (
            displayTrades.map((trade, index) => (
              <TradeRow key={`${trade.id}-${index}`} trade={trade} />
            ))
          ) : (
            <div className="py-8 text-center text-sm text-muted-foreground">
              No trades yet
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
};
