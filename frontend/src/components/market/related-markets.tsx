"use client";

import Link from "next/link";
import { cn } from "@/lib/utils";
import type { PredictionMarket, Platform } from "@/lib/types";

// Fey color tokens
const fey = {
  bg300: "#131419",
  bg400: "#16181C",
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  teal: "#4DBE95",
  red: "#D84F68",
  border: "rgba(255, 255, 255, 0.06)",
};

// ============================================================================
// Types
// ============================================================================

interface RelatedMarketsProps {
  markets: PredictionMarket[];
  currentMarketId: string;
  isLoading?: boolean;
  maxDisplay?: number;
}

// ============================================================================
// Helpers
// ============================================================================

const formatPrice = (price: string): string => {
  const num = parseFloat(price);
  if (isNaN(num)) return "—";
  return `${(num * 100).toFixed(0)}¢`;
};

// ============================================================================
// Market Card
// ============================================================================

interface MarketCardProps {
  market: PredictionMarket;
}

const MarketCard = ({ market }: MarketCardProps) => {
  const yesPrice = parseFloat(market.yes_price);
  const isHigh = yesPrice >= 0.7;
  const isLow = yesPrice <= 0.3;

  const getPriceColor = () => {
    if (isHigh) return fey.teal;
    if (isLow) return fey.red;
    return fey.grey100;
  };

  return (
    <Link href={`/market/${market.platform}/${market.id}`}>
      <div
        className="p-3 rounded-lg transition-colors cursor-pointer"
        style={{ border: `1px solid ${fey.border}` }}
        onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = fey.bg400)}
        onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = "transparent")}
      >
        <div className="flex items-start justify-between gap-2">
          <div className="flex-1 min-w-0">
            <p
              className="text-sm font-medium truncate"
              style={{ color: fey.grey100 }}
            >
              {market.title}
            </p>
            {market.leading_outcome && (
              <p
                className="text-xs mt-0.5 truncate"
                style={{ color: fey.grey500 }}
              >
                Leading: {market.leading_outcome}
              </p>
            )}
          </div>
          <div
            className="text-sm font-mono font-bold whitespace-nowrap"
            style={{ color: getPriceColor() }}
          >
            {formatPrice(market.yes_price)}
          </div>
        </div>
      </div>
    </Link>
  );
};

// ============================================================================
// Loading Skeleton
// ============================================================================

const RelatedMarketsSkeleton = () => (
  <div className="space-y-2">
    {Array.from({ length: 3 }).map((_, i) => (
      <div
        key={i}
        className="p-3 rounded-lg"
        style={{ border: `1px solid ${fey.border}` }}
      >
        <div className="flex items-start justify-between gap-2">
          <div className="flex-1">
            <div
              className="h-4 w-3/4 animate-pulse rounded"
              style={{ backgroundColor: fey.bg400 }}
            />
            <div
              className="h-3 w-1/2 animate-pulse rounded mt-1"
              style={{ backgroundColor: fey.bg400 }}
            />
          </div>
          <div
            className="h-4 w-10 animate-pulse rounded"
            style={{ backgroundColor: fey.bg400 }}
          />
        </div>
      </div>
    ))}
  </div>
);

// ============================================================================
// Main Component
// ============================================================================

export const RelatedMarkets = ({
  markets,
  currentMarketId,
  isLoading = false,
  maxDisplay = 5,
}: RelatedMarketsProps) => {
  // Filter out current market and limit display
  const filteredMarkets = markets
    .filter((m) => m.id !== currentMarketId)
    .slice(0, maxDisplay);

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
            Related Markets
          </span>
        </div>
        <div className="px-5 pb-5">
          <RelatedMarketsSkeleton />
        </div>
      </div>
    );
  }

  if (filteredMarkets.length === 0) {
    return null; // Don't render if no related markets
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
            Related Markets
          </span>
          <span
            className="text-xs px-2 py-0.5 rounded font-medium"
            style={{ backgroundColor: fey.bg400, color: fey.grey500 }}
          >
            {filteredMarkets.length}
          </span>
        </div>
      </div>
      <div className="px-5 pb-5 space-y-2">
        {filteredMarkets.map((market) => (
          <MarketCard key={market.id} market={market} />
        ))}
      </div>
    </div>
  );
};
