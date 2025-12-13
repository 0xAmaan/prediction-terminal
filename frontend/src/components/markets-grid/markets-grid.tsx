"use client";

import { useQuery } from "@tanstack/react-query";
import { useState, useMemo } from "react";
import { Zap } from "lucide-react";
import { api } from "@/lib/api";
import type { PredictionMarket, Timeframe } from "@/lib/types";
import { colors } from "./colors";
import { BinaryCard } from "./binary-card";
import { MultiOutcomeCard } from "./multi-outcome-card";
import { SportsCard } from "./sports-card";
import { MarketsTable, TableLoadingSkeleton, TimeframeSelector } from "./markets-table";
import { PlatformFilter, type FilterOption } from "../platform-filter";
import { ViewToggle } from "../view-toggle";

// Tag matching helpers
const CRYPTO_TAGS = ["crypto", "bitcoin", "ethereum", "btc", "eth", "solana", "defi", "nft", "stablecoin"];
const POLITICS_TAGS = ["politics", "u.s. politics", "trump", "election", "congress", "government", "geopolitics"];
const SPORTS_TAGS = ["sports", "nba", "nfl", "mlb", "nhl", "soccer", "football", "basketball", "baseball"];

const hasMatchingTag = (market: PredictionMarket, tagList: string[]): boolean => {
  const marketTags = (market.tags || []).map(t => t.toLowerCase());
  return tagList.some(tag => marketTags.some(mt => mt.includes(tag)));
};

const isExpiringSoon = (market: PredictionMarket): boolean => {
  if (!market.close_time) return false;
  const closeDate = new Date(market.close_time);
  const now = new Date();
  const sevenDays = 7 * 24 * 60 * 60 * 1000;
  return closeDate.getTime() - now.getTime() < sevenDays && closeDate > now;
};

const isNewMarket = (market: PredictionMarket): boolean => {
  if (!market.created_at) return false;
  const createdDate = new Date(market.created_at);
  const now = new Date();
  const sevenDays = 7 * 24 * 60 * 60 * 1000;
  return now.getTime() - createdDate.getTime() < sevenDays;
};

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
  const [filter, setFilter] = useState<FilterOption>("all");
  const [viewMode, setViewMode] = useState<"grid" | "table">("table");
  const [timeframe, setTimeframe] = useState<Timeframe>("24h");

  const { data, isLoading, error } = useQuery({
    queryKey: ["markets", search],
    queryFn: () =>
      api.listMarkets({
        platform: "polymarket",
        search: search || undefined,
        limit: 100,
      }),
  });

  // Filter and categorize markets based on selected filter
  const { markets, liveMarkets } = useMemo(() => {
    if (!data?.markets) {
      return { markets: [], liveMarkets: [] };
    }

    let filtered = data.markets;

    // Apply filter
    switch (filter) {
      case "trending":
        // Sort by volume (highest first) - top 20
        filtered = [...filtered]
          .sort((a, b) => parseFloat(b.volume) - parseFloat(a.volume))
          .slice(0, 20);
        break;
      case "expiring":
        filtered = filtered
          .filter(isExpiringSoon)
          .sort((a, b) => {
            const aTime = a.close_time ? new Date(a.close_time).getTime() : Infinity;
            const bTime = b.close_time ? new Date(b.close_time).getTime() : Infinity;
            return aTime - bTime;
          });
        break;
      case "new":
        filtered = filtered
          .filter(isNewMarket)
          .sort((a, b) => {
            const aTime = a.created_at ? new Date(a.created_at).getTime() : 0;
            const bTime = b.created_at ? new Date(b.created_at).getTime() : 0;
            return bTime - aTime;
          });
        break;
      case "crypto":
        filtered = filtered.filter(m => hasMatchingTag(m, CRYPTO_TAGS));
        break;
      case "politics":
        filtered = filtered.filter(m => hasMatchingTag(m, POLITICS_TAGS));
        break;
      case "sports":
        filtered = filtered.filter(m => m.is_sports || hasMatchingTag(m, SPORTS_TAGS));
        break;
      case "all":
      default:
        // No filtering
        break;
    }

    const sportsMarkets = filtered.filter(
      (m) => m.is_sports && m.home_team && m.away_team
    );
    const live = sportsMarkets.filter((m) => m.is_live);

    return { markets: filtered, liveMarkets: live };
  }, [data?.markets, filter]);

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
