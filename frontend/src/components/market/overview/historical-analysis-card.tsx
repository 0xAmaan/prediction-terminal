"use client";

import { useMemo } from "react";
import { ChartLine, TrendingUp, TrendingDown } from "lucide-react";
import {
  calculatePriceRange,
  calculateVolatility,
  formatPriceRange,
  formatVolatility,
  formatVolume,
  formatTradeCount,
} from "@/lib/market-analytics";

// Fey color tokens
const fey = {
  bg300: "#131419",
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  teal: "#4DBE95",
  red: "#D84F68",
  purple: "#6166DC",
  border: "rgba(255, 255, 255, 0.06)",
};

interface HistoricalAnalysisCardProps {
  priceHistory: number[];
  volume24h: string | null;
  tradeCount?: number;
}

// Individual stat column
const StatColumn = ({
  label,
  value,
  subValue,
}: {
  label: string;
  value: string;
  subValue?: string;
}) => (
  <div className="text-center">
    <div
      className="text-[10px] uppercase tracking-wider font-medium mb-1"
      style={{ color: fey.grey500 }}
    >
      {label}
    </div>
    <div
      className="text-base font-mono font-semibold"
      style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
    >
      {value}
    </div>
    {subValue && (
      <div className="text-[10px] mt-0.5" style={{ color: fey.grey500 }}>
        {subValue}
      </div>
    )}
  </div>
);

export const HistoricalAnalysisCard = ({
  priceHistory,
  volume24h,
  tradeCount = 0,
}: HistoricalAnalysisCardProps) => {
  // Calculate metrics from price history
  const metrics = useMemo(() => {
    const priceRange = calculatePriceRange(priceHistory);
    const volatilityMetrics = calculateVolatility(priceHistory);

    // Determine price trend
    let trend: "up" | "down" | "neutral" = "neutral";
    if (priceHistory.length >= 2) {
      const first = priceHistory[0];
      const last = priceHistory[priceHistory.length - 1];
      if (last > first * 1.01) trend = "up";
      else if (last < first * 0.99) trend = "down";
    }

    return {
      priceRange,
      volatility: volatilityMetrics?.volatility ?? 0,
      trend,
    };
  }, [priceHistory]);

  // Format volume
  const formattedVolume = volume24h
    ? formatVolume(parseFloat(volume24h))
    : "—";

  // Format trade count
  const formattedTradeCount = tradeCount > 0 ? formatTradeCount(tradeCount) : "—";

  // Volatility level
  const volatilityLevel =
    metrics.volatility > 10 ? "High" : metrics.volatility > 5 ? "Medium" : "Low";

  return (
    <div
      className="rounded-lg overflow-hidden"
      style={{
        backgroundColor: fey.bg300,
        border: `1px solid ${fey.border}`,
      }}
    >
      {/* Header */}
      <div
        className="flex items-center justify-between px-5 py-3"
        style={{ borderBottom: `1px solid ${fey.border}` }}
      >
        <div className="flex items-center gap-2">
          <ChartLine className="h-4 w-4" style={{ color: fey.purple }} />
          <h3
            className="text-sm font-semibold"
            style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
          >
            Historical Analysis
          </h3>
        </div>
        {/* Trend indicator */}
        {metrics.trend !== "neutral" && (
          <div className="flex items-center gap-1.5">
            {metrics.trend === "up" ? (
              <TrendingUp className="h-3.5 w-3.5" style={{ color: fey.teal }} />
            ) : (
              <TrendingDown className="h-3.5 w-3.5" style={{ color: fey.red }} />
            )}
            <span
              className="text-xs font-medium"
              style={{ color: metrics.trend === "up" ? fey.teal : fey.red }}
            >
              {metrics.trend === "up" ? "Trending Up" : "Trending Down"}
            </span>
          </div>
        )}
      </div>

      {/* Compact horizontal stats row */}
      <div className="px-5 py-4">
        <div className="grid grid-cols-4 gap-4">
          <StatColumn
            label="7D Range"
            value={metrics.priceRange ? formatPriceRange(metrics.priceRange) : "—"}
          />
          <StatColumn
            label="Volatility"
            value={metrics.volatility > 0 ? formatVolatility(metrics.volatility) : "—"}
            subValue={volatilityLevel}
          />
          <StatColumn
            label="24h Volume"
            value={formattedVolume}
          />
          <StatColumn
            label="Trades (7D)"
            value={formattedTradeCount}
          />
        </div>
      </div>
    </div>
  );
};
