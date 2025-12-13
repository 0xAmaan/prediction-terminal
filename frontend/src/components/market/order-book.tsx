"use client";

import { useMemo } from "react";
import { cn } from "@/lib/utils";

// Fey color tokens
const fey = {
  bg300: "#131419",
  bg400: "#16181C",
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  teal: "#4DBE95",
  tealMuted: "rgba(77, 190, 149, 0.15)",
  red: "#D84F68",
  redMuted: "rgba(216, 79, 104, 0.15)",
  border: "rgba(255, 255, 255, 0.06)",
};

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
  // Fey colors for bids/asks
  const bidBgColor = side === "yes" ? fey.tealMuted : fey.redMuted;
  const askBgColor = side === "yes" ? fey.redMuted : fey.tealMuted;
  const bidTextColor = side === "yes" ? fey.teal : fey.red;
  const askTextColor = side === "yes" ? fey.red : fey.teal;

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between px-2">
        <span
          className="text-xs font-semibold"
          style={{ color: fey.grey500 }}
        >
          {sideLabel}
        </span>
        {spread && (
          <span className="text-xs" style={{ color: fey.grey500 }}>
            Spread: {spread}¢
          </span>
        )}
      </div>

      {/* Header */}
      <div
        className="grid grid-cols-3 gap-2 px-2 text-xs font-medium"
        style={{ color: fey.grey500 }}
      >
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
                className="absolute inset-y-0 right-0"
                style={{ width: `${depthPercent}%`, backgroundColor: askBgColor }}
              />
              <div className="relative z-10" style={{ color: askTextColor }}>
                {formatPrice(level.price)}
              </div>
              <div className="relative z-10 text-right" style={{ color: fey.grey100 }}>
                {formatQuantity(level.quantity)}
              </div>
              <div className="relative z-10 text-right" style={{ color: fey.grey500 }}>
                {level.order_count ?? "-"}
              </div>
            </div>
          );
        })}
      </div>

      {/* Spread indicator */}
      {spread && (
        <div className="flex items-center justify-center py-1">
          <div className="h-px flex-1" style={{ backgroundColor: fey.border }} />
          <span className="px-2 text-xs" style={{ color: fey.grey500 }}>
            {formatPrice(
              sortedBids[0] ? parseFloat(sortedBids[0].price) : 0,
            )}{" "}
            -{" "}
            {formatPrice(sortedAsks[0] ? parseFloat(sortedAsks[0].price) : 0)}
          </span>
          <div className="h-px flex-1" style={{ backgroundColor: fey.border }} />
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
                className="absolute inset-y-0 right-0"
                style={{ width: `${depthPercent}%`, backgroundColor: bidBgColor }}
              />
              <div className="relative z-10" style={{ color: bidTextColor }}>
                {formatPrice(level.price)}
              </div>
              <div className="relative z-10 text-right" style={{ color: fey.grey100 }}>
                {formatQuantity(level.quantity)}
              </div>
              <div className="relative z-10 text-right" style={{ color: fey.grey500 }}>
                {level.order_count ?? "-"}
              </div>
            </div>
          );
        })}
      </div>

      {/* Empty state */}
      {sortedBids.length === 0 && sortedAsks.length === 0 && (
        <div className="py-8 text-center text-sm" style={{ color: fey.grey500 }}>
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
    <div className="h-4 w-16 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
    <div className="grid grid-cols-3 gap-2 px-2">
      <div className="h-3 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
      <div className="h-3 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
      <div className="h-3 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
    </div>
    {Array.from({ length: 5 }).map((_, i) => (
      <div key={i} className="grid grid-cols-3 gap-2 px-2">
        <div className="h-4 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
        <div className="h-4 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
        <div className="h-4 animate-pulse rounded" style={{ backgroundColor: fey.bg400 }} />
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
            Order Book
          </span>
        </div>
        <div className="px-5 pb-5">
          <OrderBookSkeleton />
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
        <span
          className="text-sm font-semibold"
          style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
        >
          Order Book
        </span>
      </div>
      <div className="px-5 pb-5 space-y-4">
        <OrderBookSide
          bids={yesBids}
          asks={yesAsks}
          side="yes"
          maxLevels={maxLevels}
        />

        {showNoSide && (noBids.length > 0 || noAsks.length > 0) && (
          <>
            <div className="h-px" style={{ backgroundColor: fey.border }} />
            <OrderBookSide
              bids={noBids}
              asks={noAsks}
              side="no"
              maxLevels={maxLevels}
            />
          </>
        )}
      </div>
    </div>
  );
};
