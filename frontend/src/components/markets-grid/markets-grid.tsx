"use client";

import { useQuery } from "@tanstack/react-query";
import { useState, useMemo } from "react";
import { useRouter } from "next/navigation";
import { Zap } from "lucide-react";
import { api } from "@/lib/api";
import type { PredictionMarket } from "@/lib/types";
import { colors } from "./colors";
import { BinaryCard } from "./binary-card";
import { MultiOutcomeCard } from "./multi-outcome-card";
import { SportsCard } from "./sports-card";
import { PlatformFilter } from "../platform-filter";

interface MarketsGridProps {
  search?: string;
}

// Card dispatcher - determines which card type to render
const MarketCard = ({
  market,
  index,
  onClick,
}: {
  market: PredictionMarket;
  index: number;
  onClick: () => void;
}) => {
  // Sports cards require team data to render properly
  if (market.is_sports && market.home_team && market.away_team) {
    return <SportsCard market={market} index={index} onClick={onClick} />;
  }
  if (market.is_multi_outcome) {
    return <MultiOutcomeCard market={market} index={index} onClick={onClick} />;
  }
  return <BinaryCard market={market} index={index} onClick={onClick} />;
};

// Loading skeleton
const LoadingSkeleton = () => (
  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
    {Array.from({ length: 9 }).map((_, i) => (
      <div
        key={i}
        className="h-[280px] rounded-xl animate-pulse"
        style={{ background: colors.cardBg, border: `1px solid ${colors.border}` }}
      >
        <div className="p-6 h-full flex flex-col">
          <div className="flex justify-between mb-4">
            <div className="h-4 w-20 rounded" style={{ background: colors.elevated }} />
            <div className="h-4 w-12 rounded" style={{ background: colors.elevated }} />
          </div>
          <div className="h-6 w-full rounded mb-2" style={{ background: colors.elevated }} />
          <div className="h-6 w-3/4 rounded mb-auto" style={{ background: colors.elevated }} />
          <div className="grid grid-cols-2 gap-3 mt-4">
            <div className="h-12 rounded-lg" style={{ background: colors.elevated }} />
            <div className="h-12 rounded-lg" style={{ background: colors.elevated }} />
          </div>
          <div className="flex justify-between mt-4">
            <div className="h-4 w-24 rounded" style={{ background: colors.elevated }} />
            <div className="h-6 w-20 rounded" style={{ background: colors.elevated }} />
          </div>
        </div>
      </div>
    ))}
  </div>
);

export const MarketsGrid = ({ search = "" }: MarketsGridProps) => {
  const router = useRouter();
  const [platform, setPlatform] = useState<"all" | "kalshi" | "polymarket">("all");

  const { data, isLoading, error } = useQuery({
    queryKey: ["markets", platform, search],
    queryFn: () =>
      api.listMarkets({
        platform: platform === "all" ? undefined : platform,
        search: search || undefined,
        limit: 100,
      }),
  });

  // Filter and categorize markets
  const { markets, liveMarkets } = useMemo(() => {
    if (!data?.markets) {
      return { markets: [], liveMarkets: [] };
    }

    const sportsMarkets = data.markets.filter(
      (m) => m.is_sports && m.home_team && m.away_team
    );
    const live = sportsMarkets.filter((m) => m.is_live);

    return { markets: data.markets, liveMarkets: live };
  }, [data?.markets]);

  const handleMarketClick = (market: PredictionMarket) => {
    router.push(`/market/${market.platform}/${market.id}`);
  };

  return (
    <div className="h-full flex flex-col font-[Inter,system-ui,sans-serif]">
      {/* Filters row */}
      <div className="shrink-0 flex items-center justify-between mb-8">
        <PlatformFilter value={platform} onChange={setPlatform} />
      </div>

      {/* Error state */}
      {error && (
        <div
          className="shrink-0 rounded-lg p-4 text-base mb-4"
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
      {isLoading && <LoadingSkeleton />}

      {/* Content */}
      {!isLoading && !error && (
        <div className="flex-1 overflow-y-auto">
          {/* Live Sports Section */}
          {liveMarkets.length > 0 && (
            <section className="mb-12">
              <div className="flex items-center gap-4 mb-6">
                <div className="flex items-center gap-2">
                  <Zap className="h-5 w-5" style={{ color: colors.cyan }} />
                  <h2
                    className="text-xl font-semibold"
                    style={{ color: colors.textPrimary }}
                  >
                    Live Now
                  </h2>
                </div>
                <div
                  className="flex-1 h-px"
                  style={{
                    background: `linear-gradient(90deg, ${colors.cyan}50, transparent)`,
                  }}
                />
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
                {liveMarkets.map((market, i) => (
                  <MarketCard
                    key={market.id}
                    market={market}
                    index={i}
                    onClick={() => handleMarketClick(market)}
                  />
                ))}
              </div>
            </section>
          )}

          {/* All Markets Section */}
          <section>
            {liveMarkets.length > 0 && (
              <div className="flex items-center gap-4 mb-6">
                <h2
                  className="text-xl font-semibold"
                  style={{ color: colors.textPrimary }}
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
                    onClick={() => handleMarketClick(market)}
                  />
                ))}
            </div>

            {markets.length === 0 && (
              <div
                className="text-center py-12"
                style={{ color: colors.textMuted }}
              >
                No markets found
              </div>
            )}
          </section>
        </div>
      )}
    </div>
  );
};
