"use client";

import Link from "next/link";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";
import type { PredictionMarket, Platform } from "@/lib/types";

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

  return (
    <Link href={`/market/${market.platform}/${market.id}`}>
      <div className="p-3 rounded-lg border border-border/30 hover:bg-secondary/30 transition-colors cursor-pointer">
        <div className="flex items-start justify-between gap-2">
          <div className="flex-1 min-w-0">
            <p className="text-sm font-medium truncate">{market.title}</p>
            {market.leading_outcome && (
              <p className="text-xs text-muted-foreground mt-0.5 truncate">
                Leading: {market.leading_outcome}
              </p>
            )}
          </div>
          <div
            className={cn(
              "text-sm font-mono font-bold whitespace-nowrap",
              isHigh ? "text-green-500" : isLow ? "text-red-500" : "text-foreground"
            )}
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
      <div key={i} className="p-3 rounded-lg border border-border/30">
        <div className="flex items-start justify-between gap-2">
          <div className="flex-1">
            <div className="h-4 w-3/4 animate-pulse rounded bg-muted" />
            <div className="h-3 w-1/2 animate-pulse rounded bg-muted mt-1" />
          </div>
          <div className="h-4 w-10 animate-pulse rounded bg-muted" />
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
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-sm font-medium">Related Markets</CardTitle>
        </CardHeader>
        <CardContent>
          <RelatedMarketsSkeleton />
        </CardContent>
      </Card>
    );
  }

  if (filteredMarkets.length === 0) {
    return null; // Don't render if no related markets
  }

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-medium">Related Markets</CardTitle>
          <Badge variant="secondary" className="text-xs">
            {filteredMarkets.length}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        {filteredMarkets.map((market) => (
          <MarketCard key={market.id} market={market} />
        ))}
      </CardContent>
    </Card>
  );
};
