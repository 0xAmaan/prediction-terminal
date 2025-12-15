"use client";

import { useQuery } from "@tanstack/react-query";
import { useState, useMemo } from "react";  // useMemo for live market categorization
import { Zap } from "lucide-react";
import { api } from "@/lib/api";
import type { PredictionMarket, Timeframe, MarketFilter } from "@/lib/types";
import { colors } from "./colors";
import { BinaryCard } from "./binary-card";
import { MultiOutcomeCard } from "./multi-outcome-card";
import { SportsCard } from "./sports-card";
import { MarketsTable, TableLoadingSkeleton, TimeframeSelector } from "./markets-table";
import { PlatformFilter, type FilterOption } from "../platform-filter";
import { ViewToggle } from "../view-toggle";

interface MarketsGridProps {
  search?: string;
}

// Card dispatcher - determines which card type to render
const MarketCard = ({
  market,
  index,
  href,
}: {
  market: PredictionMarket;
  index: number;
  href: string;
}) => {
  // Sports cards require team data to render properly
  if (market.is_sports && market.home_team && market.away_team) {
    return <SportsCard market={market} index={index} href={href} />;
  }
  if (market.is_multi_outcome) {
    return <MultiOutcomeCard market={market} index={index} href={href} />;
  }
  return <BinaryCard market={market} index={index} href={href} />;
};

// Loading skeleton - Fey style
const LoadingSkeleton = () => (
  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
    {Array.from({ length: 9 }).map((_, i) => (
      <div
        key={i}
        className="h-[310px] rounded-lg animate-pulse"
        style={{ background: colors.cardBg, border: `1px solid ${colors.border}` }}
      >
        <div className="p-5 h-full flex flex-col">
          <div className="flex justify-between mb-3">
            <div className="h-3 w-16 rounded" style={{ background: colors.elevated }} />
            <div className="h-3 w-10 rounded" style={{ background: colors.elevated }} />
          </div>
          <div className="h-5 w-full rounded mb-2" style={{ background: colors.elevated }} />
          <div className="h-5 w-3/4 rounded mb-auto" style={{ background: colors.elevated }} />
          <div className="grid grid-cols-2 gap-3 mt-4">
            <div className="h-10 rounded-lg" style={{ background: colors.elevated }} />
            <div className="h-10 rounded-lg" style={{ background: colors.elevated }} />
          </div>
          <div className="flex justify-between mt-4">
            <div className="h-3 w-20 rounded" style={{ background: colors.elevated }} />
            <div className="h-5 w-16 rounded" style={{ background: colors.elevated }} />
          </div>
        </div>
      </div>
    ))}
  </div>
);

export const MarketsGrid = ({ search = "" }: MarketsGridProps) => {
  // Default to "trending" to show high-volume markets first
  const [filter, setFilter] = useState<FilterOption>("trending");
  const [viewMode, setViewMode] = useState<"grid" | "table">("table");
  const [timeframe, setTimeframe] = useState<Timeframe>("24h");

  // Server-side filtering - filter is now passed to the API
  const { data, isLoading, error } = useQuery({
    queryKey: ["markets", search, filter],
    queryFn: () =>
      api.listMarkets({
        platform: "polymarket",
        search: search || undefined,
        filter: filter as MarketFilter,
        limit: 100,
      }),
  });

  // Categorize markets for display (filtering is done server-side)
  const { markets, liveMarkets } = useMemo(() => {
    if (!data?.markets) {
      return { markets: [], liveMarkets: [] };
    }

    const allMarkets = data.markets;
    const sportsMarkets = allMarkets.filter(
      (m) => m.is_sports && m.home_team && m.away_team
    );
    const live = sportsMarkets.filter((m) => m.is_live);

    return { markets: allMarkets, liveMarkets: live };
  }, [data?.markets]);

  const getMarketHref = (market: PredictionMarket) =>
    `/market/${market.platform}/${market.id}`;

  return (
    <div className="h-full flex flex-col">
      {/* Filters row */}
      <div className="shrink-0 flex items-center justify-between mb-6">
        <PlatformFilter value={filter} onChange={setFilter} />
        <div className="flex items-center gap-2">
          {viewMode === "table" && (
            <TimeframeSelector value={timeframe} onChange={setTimeframe} />
          )}
          <ViewToggle value={viewMode} onChange={setViewMode} />
        </div>
      </div>

      {/* Error state - Fey style */}
      {error && (
        <div
          className="shrink-0 rounded-lg p-4 text-sm mb-4"
          style={{
            border: `1px solid ${colors.redBorder}`,
            background: colors.redMuted,
            color: colors.red,
          }}
        >
          Failed to load markets. Is the API server running?
        </div>
      )}

      {/* Loading state */}
      {isLoading && (viewMode === "grid" ? <LoadingSkeleton /> : <TableLoadingSkeleton />)}

      {/* Content */}
      {!isLoading && !error && (
        <div className="flex-1 overflow-y-auto">
          {viewMode === "table" ? (
            /* Table View - Combined, with live markets sorted to top */
            <>
              <MarketsTable markets={markets} timeframe={timeframe} />
              {markets.length === 0 && (
                <div
                  className="text-center py-12 text-sm"
                  style={{ color: colors.textMuted }}
                >
                  No markets found
                </div>
              )}
            </>
          ) : (
            /* Grid View - Separate Live and All sections */
            <>
              {/* Live Sports Section */}
              {liveMarkets.length > 0 && (
                <section className="mb-10">
                  <div className="flex items-center gap-3 mb-5">
                    <div className="flex items-center gap-2">
                      <Zap className="h-4 w-4" style={{ color: colors.skyBlue }} />
                      <h2
                        className="text-lg font-semibold"
                        style={{ color: colors.textPrimary, letterSpacing: "-0.02em" }}
                      >
                        Live Now
                      </h2>
                    </div>
                    <div
                      className="flex-1 h-px"
                      style={{ background: colors.border }}
                    />
                  </div>

                  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
                    {liveMarkets.map((market, i) => (
                      <MarketCard
                        key={market.id}
                        market={market}
                        index={i}
                        href={getMarketHref(market)}
                      />
                    ))}
                  </div>
                </section>
              )}

              {/* All Markets Section */}
              <section>
                {liveMarkets.length > 0 && (
                  <div className="flex items-center gap-3 mb-5">
                    <h2
                      className="text-lg font-semibold"
                      style={{ color: colors.textPrimary, letterSpacing: "-0.02em" }}
                    >
                      All Markets
                    </h2>
                    <div className="flex-1 h-px" style={{ background: colors.border }} />
                  </div>
                )}

                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
                  {markets
                    .filter((m) => !m.is_live)
                    .map((market, i) => (
                      <MarketCard
                        key={market.id}
                        market={market}
                        index={i}
                        href={getMarketHref(market)}
                      />
                    ))}
                </div>

                {markets.length === 0 && (
                  <div
                    className="text-center py-12 text-sm"
                    style={{ color: colors.textMuted }}
                  >
                    No markets found
                  </div>
                )}
              </section>
            </>
          )}
        </div>
      )}
    </div>
  );
};
