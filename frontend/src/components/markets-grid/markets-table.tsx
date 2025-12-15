"use client";

import { useMemo } from "react";
import Link from "next/link";
import Image from "next/image";
import { useQuery } from "@tanstack/react-query";
import { ExternalLink, ChevronRight } from "lucide-react";
import type {
  PredictionMarket,
  MarketStats,
  Timeframe,
  PriceHistoryPoint,
} from "@/lib/types";
import { api } from "@/lib/api";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Sparkline, SparklineSkeleton } from "@/components/ui/sparkline";
import { colors } from "./colors";
import { formatVolume, formatCloseTime } from "./utils";

// ============================================================================
// Types
// ============================================================================

interface MarketsTableProps {
  markets: PredictionMarket[];
  timeframe?: Timeframe;
}

// ============================================================================
// Utility Functions
// ============================================================================

// Format price as cents (e.g., 84.0¢)
const formatPriceCents = (price: number | string): string => {
  const num = typeof price === "string" ? parseFloat(price) : price;
  return `${(num * 100).toFixed(1)}¢`;
};

// Format price change (e.g., +0.81¢ or -1.2¢)
const formatPriceChange = (change: number | string): string => {
  const num = typeof change === "string" ? parseFloat(change) : change;
  const sign = num >= 0 ? "+" : "";
  return `${sign}${(num * 100).toFixed(2)}¢`;
};


// Sort markets with live ones first
const sortMarkets = (markets: PredictionMarket[]) => {
  return [...markets].sort((a, b) => {
    if (a.is_live && !b.is_live) return -1;
    if (!a.is_live && b.is_live) return 1;
    return 0;
  });
};

// ============================================================================
// Timeframe Selector Component
// ============================================================================

interface TimeframeSelectorProps {
  value: Timeframe;
  onChange: (timeframe: Timeframe) => void;
}

export const TimeframeSelector = ({
  value,
  onChange,
}: TimeframeSelectorProps) => {
  const timeframes: { value: Timeframe; label: string }[] = [
    { value: "1h", label: "1H" },
    { value: "24h", label: "24H" },
    { value: "7d", label: "7D" },
    { value: "30d", label: "30D" },
  ];

  return (
    <div
      className="inline-flex rounded-lg p-1 gap-1"
      style={{ backgroundColor: colors.secondary }}
    >
      {timeframes.map((tf) => (
        <button
          key={tf.value}
          onClick={() => onChange(tf.value)}
          className="px-3 py-1.5 text-xs font-medium rounded-md transition-all"
          style={{
            backgroundColor: value === tf.value ? colors.elevated : "transparent",
            color: value === tf.value ? colors.textPrimary : colors.textMuted,
          }}
        >
          {tf.label}
        </button>
      ))}
    </div>
  );
};

// ============================================================================
// Main Markets Table Component
// ============================================================================

export const MarketsTable = ({
  markets,
  timeframe = "24h",
}: MarketsTableProps) => {
  const sortedMarkets = sortMarkets(markets);

  // Fetch market stats for the selected timeframe
  // Use a higher limit to ensure we get stats for all markets that might be shown after filtering
  const { data: statsData, isLoading: statsLoading } = useQuery({
    queryKey: ["market-stats", timeframe],
    queryFn: () =>
      api.getMarketStats({
        timeframe,
        platform: "polymarket",
        limit: 200, // Increased from 100 to cover filtered markets
      }),
    staleTime: 30_000, // 30 seconds
    refetchInterval: 60_000, // Refetch every minute
  });

  // Create lookup maps for stats and sparklines
  const statsMap = useMemo(() => {
    const map = new Map<string, MarketStats>();
    if (statsData?.stats) {
      for (const stat of statsData.stats) {
        map.set(stat.market_id, stat);
      }
    }
    return map;
  }, [statsData?.stats]);

  const sparklinesMap = useMemo(() => {
    return statsData?.sparklines || {};
  }, [statsData?.sparklines]);

  return (
    <Table>
      <TableHeader>
        <TableRow style={{ borderColor: colors.border }}>
          <TableHead className="w-[45%]" style={{ color: colors.textMuted }}>
            Market
          </TableHead>
          <TableHead
            className="w-[100px] text-right"
            style={{ color: colors.textMuted }}
          >
            Price
          </TableHead>
          <TableHead
            className="w-[90px] text-right"
            style={{ color: colors.textMuted }}
          >
            Volume
          </TableHead>
          <TableHead
            className="w-[90px] text-right"
            style={{ color: colors.textMuted }}
          >
            Expires
          </TableHead>
          <TableHead
            className="w-[110px] text-center"
            style={{ color: colors.textMuted }}
          >
            Chart
          </TableHead>
          <TableHead
            className="w-[60px] text-center"
            style={{ color: colors.textMuted }}
          >
            {/* Action column - no header */}
          </TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {sortedMarkets.map((market) => {
          const stats = statsMap.get(market.id);
          const sparklineData = sparklinesMap[market.id] || [];
          const priceChange = stats
            ? parseFloat(stats.price_change)
            : 0;
          const isPositive = priceChange >= 0;

          return (
            <TableRow
              key={market.id}
              className="cursor-pointer group"
              style={{ borderColor: colors.border }}
            >
              {/* Market - Image + Title + Tags */}
              <TableCell className="py-4">
                <Link
                  href={`/market/${market.platform}/${market.id}`}
                  className="flex items-center gap-3"
                >
                  {/* Market Image */}
                  <div
                    className="relative h-10 w-10 rounded-lg overflow-hidden shrink-0"
                    style={{
                      backgroundColor: colors.elevated,
                      border: `1px solid ${colors.border}`,
                    }}
                  >
                    {market.image_url ? (
                      <Image
                        src={market.image_url}
                        alt=""
                        fill
                        className="object-cover"
                        sizes="40px"
                      />
                    ) : (
                      <div
                        className="h-full w-full flex items-center justify-center text-xs font-medium"
                        style={{ color: colors.textMuted }}
                      >
                        {market.title.charAt(0).toUpperCase()}
                      </div>
                    )}
                  </div>

                  {/* Title + Tags */}
                  <div className="flex-1 min-w-0">
                    {/* Line 1: Title */}
                    <span
                      className="font-medium line-clamp-1 group-hover:underline block"
                      style={{ color: colors.textPrimary }}
                    >
                      {market.title}
                    </span>
                    {/* Line 2: Metadata tags */}
                    <div className="flex items-center gap-1.5 mt-2">
                      {market.is_live && (
                        <span
                          className="px-2 py-0.5 text-xs font-semibold rounded uppercase tracking-wide"
                          style={{
                            background: colors.redMuted,
                            color: colors.red,
                          }}
                        >
                          Live
                        </span>
                      )}
                      {market.is_multi_outcome && market.outcome_count && (
                        <span
                          className="px-2 py-0.5 text-xs font-medium rounded"
                          style={{
                            background: colors.purpleMuted,
                            color: colors.purple,
                          }}
                        >
                          {market.outcome_count} outcomes
                        </span>
                      )}
                      {market.category && (
                        <span
                          className="text-xs"
                          style={{ color: colors.textMuted }}
                        >
                          {market.category}
                        </span>
                      )}
                    </div>
                  </div>
                </Link>
              </TableCell>

              {/* Price + Change */}
              <TableCell className="text-right">
                <Link
                  href={`/market/${market.platform}/${market.id}`}
                  className="block"
                >
                  <div className="flex flex-col items-end">
                    <span
                      className="font-medium tabular-nums"
                      style={{ color: colors.textPrimary }}
                    >
                      {formatPriceCents(market.yes_price)}
                    </span>
                    {stats && (
                      <span
                        className="text-xs tabular-nums"
                        style={{
                          color: isPositive ? colors.teal : colors.red,
                        }}
                      >
                        {formatPriceChange(stats.price_change)}
                      </span>
                    )}
                    {!stats && statsLoading && (
                      <div
                        className="h-3 w-10 rounded animate-pulse mt-0.5"
                        style={{ background: colors.elevated }}
                      />
                    )}
                  </div>
                </Link>
              </TableCell>

              {/* Volume - prefer API's 24hr volume when available */}
              <TableCell className="text-right">
                <Link
                  href={`/market/${market.platform}/${market.id}`}
                  className="block"
                >
                  <span style={{ color: colors.textSecondary }}>
                    {market.volume_24hr
                      ? formatVolume(market.volume_24hr).replace(" Vol.", "")
                      : stats
                        ? formatVolume(stats.volume).replace(" Vol.", "")
                        : formatVolume(market.volume).replace(" Vol.", "")}
                  </span>
                </Link>
              </TableCell>


              {/* Expiration */}
              <TableCell className="text-right">
                <Link
                  href={`/market/${market.platform}/${market.id}`}
                  className="block"
                >
                  <span style={{ color: colors.textMuted }}>
                    {formatCloseTime(market.close_time)}
                  </span>
                </Link>
              </TableCell>

              {/* Sparkline Chart */}
              <TableCell className="text-center">
                <Link
                  href={`/market/${market.platform}/${market.id}`}
                  className="flex justify-center"
                >
                  {sparklineData.length > 0 ? (
                    <Sparkline
                      data={sparklineData}
                      width={90}
                      height={28}
                      strokeWidth={1.5}
                      showArea={true}
                    />
                  ) : statsLoading ? (
                    <SparklineSkeleton width={90} height={28} />
                  ) : (
                    <span
                      className="text-xs"
                      style={{ color: colors.textMuted }}
                    >
                      -
                    </span>
                  )}
                </Link>
              </TableCell>

              {/* Action Button */}
              <TableCell className="text-center">
                <Link
                  href={`/market/${market.platform}/${market.id}`}
                  className="inline-flex items-center justify-center h-8 w-8 rounded-lg transition-colors"
                  style={{
                    backgroundColor: colors.elevated,
                    color: colors.textMuted,
                  }}
                >
                  <ChevronRight className="h-4 w-4" />
                </Link>
              </TableCell>
            </TableRow>
          );
        })}
      </TableBody>
    </Table>
  );
};

// ============================================================================
// Loading Skeleton
// ============================================================================

export const TableLoadingSkeleton = () => (
  <div className="w-full">
    {/* Header skeleton */}
    <div
      className="flex items-center h-12 border-b"
      style={{ borderColor: colors.border }}
    >
      <div className="w-[45%] px-3">
        <div
          className="h-3 w-16 rounded animate-pulse"
          style={{ background: colors.elevated }}
        />
      </div>
      <div className="w-[100px] px-3 flex justify-end">
        <div
          className="h-3 w-10 rounded animate-pulse"
          style={{ background: colors.elevated }}
        />
      </div>
      <div className="w-[90px] px-3 flex justify-end">
        <div
          className="h-3 w-12 rounded animate-pulse"
          style={{ background: colors.elevated }}
        />
      </div>
      <div className="w-[50px] px-3 flex justify-center">
        <div
          className="h-3 w-6 rounded animate-pulse"
          style={{ background: colors.elevated }}
        />
      </div>
      <div className="w-[90px] px-3 flex justify-end">
        <div
          className="h-3 w-10 rounded animate-pulse"
          style={{ background: colors.elevated }}
        />
      </div>
      <div className="w-[110px] px-3 flex justify-center">
        <div
          className="h-3 w-16 rounded animate-pulse"
          style={{ background: colors.elevated }}
        />
      </div>
      <div className="w-[60px] px-3" />
    </div>
    {/* Row skeletons */}
    {Array.from({ length: 10 }).map((_, i) => (
      <div
        key={i}
        className="flex items-center h-16 border-b"
        style={{ borderColor: colors.border }}
      >
        <div className="w-[45%] px-3 flex items-center gap-3">
          <div
            className="h-10 w-10 rounded-lg animate-pulse shrink-0"
            style={{ background: colors.elevated }}
          />
          <div className="flex-1">
            <div
              className="h-4 w-full max-w-xs rounded animate-pulse mb-1"
              style={{ background: colors.elevated }}
            />
            <div
              className="h-3 w-20 rounded animate-pulse"
              style={{ background: colors.elevated }}
            />
          </div>
        </div>
        <div className="w-[100px] px-3 flex flex-col items-end gap-1">
          <div
            className="h-4 w-12 rounded animate-pulse"
            style={{ background: colors.elevated }}
          />
          <div
            className="h-3 w-10 rounded animate-pulse"
            style={{ background: colors.elevated }}
          />
        </div>
        <div className="w-[90px] px-3 flex justify-end">
          <div
            className="h-4 w-14 rounded animate-pulse"
            style={{ background: colors.elevated }}
          />
        </div>
        <div className="w-[50px] px-3 flex justify-center">
          <div
            className="h-8 w-6 rounded animate-pulse"
            style={{ background: colors.elevated }}
          />
        </div>
        <div className="w-[90px] px-3 flex justify-end">
          <div
            className="h-4 w-10 rounded animate-pulse"
            style={{ background: colors.elevated }}
          />
        </div>
        <div className="w-[110px] px-3 flex justify-center">
          <div
            className="h-7 w-20 rounded animate-pulse"
            style={{ background: colors.elevated }}
          />
        </div>
        <div className="w-[60px] px-3 flex justify-center">
          <div
            className="h-8 w-8 rounded-lg animate-pulse"
            style={{ background: colors.elevated }}
          />
        </div>
      </div>
    ))}
  </div>
);
