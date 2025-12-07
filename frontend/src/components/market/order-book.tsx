"use client";

import { useMemo } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { cn } from "@/lib/utils";

// ============================================================================
// Types
// ============================================================================

interface OrderBookLevel {
  price: string;
  quantity: string;
  order_count?: number | null;
}

interface OrderBookProps {
  yesBids: OrderBookLevel[];
  yesAsks: OrderBookLevel[];
  noBids?: OrderBookLevel[];
  noAsks?: OrderBookLevel[];
  isLoading?: boolean;
  maxLevels?: number;
  showNoSide?: boolean;
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

// ============================================================================
// Order Book Side Component
// ============================================================================

interface OrderBookSideProps {
  bids: OrderBookLevel[];
  asks: OrderBookLevel[];
  side: "yes" | "no";
  maxLevels: number;
}

const OrderBookSide = ({ bids, asks, side, maxLevels }: OrderBookSideProps) => {
  // Calculate max quantity for depth visualization
  const maxQuantity = useMemo(() => {
    const allLevels = [...bids, ...asks];
    return Math.max(...allLevels.map((l) => parseFloat(l.quantity) || 0), 1);
  }, [bids, asks]);

  // Sort and limit levels
  const sortedBids = useMemo(
    () =>
      [...bids]
        .sort((a, b) => parseFloat(b.price) - parseFloat(a.price))
        .slice(0, maxLevels),
    [bids, maxLevels],
  );

  const sortedAsks = useMemo(
    () =>
      [...asks]
        .sort((a, b) => parseFloat(a.price) - parseFloat(b.price))
        .slice(0, maxLevels),
    [asks, maxLevels],
  );

  // Calculate spread
  const spread = useMemo(() => {
    if (sortedBids.length === 0 || sortedAsks.length === 0) return null;
    const bestBid = parseFloat(sortedBids[0].price);
    const bestAsk = parseFloat(sortedAsks[0].price);
    return ((bestAsk - bestBid) * 100).toFixed(2);
  }, [sortedBids, sortedAsks]);

  const sideLabel = side === "yes" ? "YES" : "NO";
  const bidColor = side === "yes" ? "bg-green-500/20" : "bg-red-500/20";
  const askColor = side === "yes" ? "bg-red-500/20" : "bg-green-500/20";
  const bidTextColor = side === "yes" ? "text-green-500" : "text-red-500";
  const askTextColor = side === "yes" ? "text-red-500" : "text-green-500";

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between px-2">
        <span className="text-xs font-semibold text-muted-foreground">
          {sideLabel}
        </span>
        {spread && (
          <span className="text-xs text-muted-foreground">
            Spread: {spread}¢
          </span>
        )}
      </div>

      {/* Header */}
      <div className="grid grid-cols-3 gap-2 px-2 text-xs font-medium text-muted-foreground">
        <div>Price</div>
        <div className="text-right">Qty</div>
        <div className="text-right">Total</div>
      </div>

      {/* Asks (reversed so lowest is at bottom) */}
      <div className="space-y-0.5">
        {[...sortedAsks].reverse().map((level, i) => {
          const qty = parseFloat(level.quantity);
          const depthPercent = (qty / maxQuantity) * 100;
          return (
            <div
              key={`ask-${i}`}
              className="relative grid grid-cols-3 gap-2 px-2 py-0.5 text-xs"
            >
              <div
                className={cn("absolute inset-y-0 right-0", askColor)}
                style={{ width: `${depthPercent}%` }}
              />
              <div className={cn("relative z-10", askTextColor)}>
                {formatPrice(level.price)}
              </div>
              <div className="relative z-10 text-right">
                {formatQuantity(level.quantity)}
              </div>
              <div className="relative z-10 text-right text-muted-foreground">
                {level.order_count ?? "-"}
              </div>
            </div>
          );
        })}
      </div>

      {/* Spread indicator */}
      {spread && (
        <div className="flex items-center justify-center py-1">
          <div className="h-px flex-1 bg-border" />
          <span className="px-2 text-xs text-muted-foreground">
            {formatPrice(
              sortedBids[0] ? parseFloat(sortedBids[0].price) : 0,
            )}{" "}
            -{" "}
            {formatPrice(sortedAsks[0] ? parseFloat(sortedAsks[0].price) : 0)}
          </span>
          <div className="h-px flex-1 bg-border" />
        </div>
      )}

      {/* Bids */}
      <div className="space-y-0.5">
        {sortedBids.map((level, i) => {
          const qty = parseFloat(level.quantity);
          const depthPercent = (qty / maxQuantity) * 100;
          return (
            <div
              key={`bid-${i}`}
              className="relative grid grid-cols-3 gap-2 px-2 py-0.5 text-xs"
            >
              <div
                className={cn("absolute inset-y-0 right-0", bidColor)}
                style={{ width: `${depthPercent}%` }}
              />
              <div className={cn("relative z-10", bidTextColor)}>
                {formatPrice(level.price)}
              </div>
              <div className="relative z-10 text-right">
                {formatQuantity(level.quantity)}
              </div>
              <div className="relative z-10 text-right text-muted-foreground">
                {level.order_count ?? "-"}
              </div>
            </div>
          );
        })}
      </div>

      {/* Empty state */}
      {sortedBids.length === 0 && sortedAsks.length === 0 && (
        <div className="py-8 text-center text-sm text-muted-foreground">
          No orders available
        </div>
      )}
    </div>
  );
};

// ============================================================================
// Loading Skeleton
// ============================================================================

const OrderBookSkeleton = () => (
  <div className="space-y-2">
    <div className="h-4 w-16 animate-pulse rounded bg-muted" />
    <div className="grid grid-cols-3 gap-2 px-2">
      <div className="h-3 animate-pulse rounded bg-muted" />
      <div className="h-3 animate-pulse rounded bg-muted" />
      <div className="h-3 animate-pulse rounded bg-muted" />
    </div>
    {Array.from({ length: 5 }).map((_, i) => (
      <div key={i} className="grid grid-cols-3 gap-2 px-2">
        <div className="h-4 animate-pulse rounded bg-muted" />
        <div className="h-4 animate-pulse rounded bg-muted" />
        <div className="h-4 animate-pulse rounded bg-muted" />
      </div>
    ))}
  </div>
);

// ============================================================================
// Main Order Book Component
// ============================================================================

export const OrderBook = ({
  yesBids,
  yesAsks,
  noBids = [],
  noAsks = [],
  isLoading = false,
  maxLevels = 10,
  showNoSide = false,
}: OrderBookProps) => {
  if (isLoading) {
    return (
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-medium">Order Book</CardTitle>
        </CardHeader>
        <CardContent>
          <OrderBookSkeleton />
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="text-sm font-medium">Order Book</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <OrderBookSide
          bids={yesBids}
          asks={yesAsks}
          side="yes"
          maxLevels={maxLevels}
        />

        {showNoSide && (noBids.length > 0 || noAsks.length > 0) && (
          <>
            <div className="h-px bg-border" />
            <OrderBookSide
              bids={noBids}
              asks={noAsks}
              side="no"
              maxLevels={maxLevels}
            />
          </>
        )}
      </CardContent>
    </Card>
  );
};
