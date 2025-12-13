"use client";

import { ProbabilityBar } from "../shared/probability-bar";
import { Sparkline } from "../shared/sparkline";
import { TrendingUp, TrendingDown, Minus } from "lucide-react";

// Fey color tokens
const fey = {
  bg300: "#131419",
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  teal: "#4DBE95",
  red: "#D84F68",
  border: "rgba(255, 255, 255, 0.06)",
};

interface MarketSummaryProps {
  yesPrice: string;
  noPrice: string;
  priceHistory?: number[];
  change24h?: number;
  className?: string;
}

export const MarketSummary = ({
  yesPrice,
  priceHistory = [],
  change24h,
  className = "",
}: MarketSummaryProps) => {
  const probability = parseFloat(yesPrice) * 100;
  const probabilityStr = probability.toFixed(1);

  // Determine trend for styling
  const isPositive = change24h !== undefined && change24h > 0;
  const isNegative = change24h !== undefined && change24h < 0;
  const isNeutral = change24h === undefined || change24h === 0;

  const TrendIcon = isPositive ? TrendingUp : isNegative ? TrendingDown : Minus;

  // Get probability color based on value
  const getProbabilityColor = () => {
    if (probability >= 70) return fey.teal;
    if (probability <= 30) return fey.red;
    return fey.grey100;
  };

  // Get trend color
  const getTrendColor = () => {
    if (isPositive) return fey.teal;
    if (isNegative) return fey.red;
    return fey.grey500;
  };

  return (
    <div
      className={`rounded-lg ${className}`}
      style={{
        backgroundColor: fey.bg300,
        border: `1px solid ${fey.border}`,
      }}
    >
      <div className="p-6">
        <div className="flex items-start justify-between gap-6">
          {/* Left: Probability display */}
          <div className="flex-1">
            <div
              className="text-[10px] uppercase tracking-wider font-medium mb-2"
              style={{ color: fey.grey500 }}
            >
              YES Probability
            </div>
            <div className="flex items-baseline gap-2">
              <span
                className="text-4xl font-semibold"
                style={{
                  color: getProbabilityColor(),
                  letterSpacing: "-0.02em",
                }}
              >
                {probabilityStr}%
              </span>
            </div>

            {/* Change indicator */}
            {change24h !== undefined && (
              <div
                className="flex items-center gap-1.5 mt-2"
                style={{ color: getTrendColor() }}
              >
                <TrendIcon className="h-3.5 w-3.5" />
                <span className="text-sm font-medium">
                  {isPositive && "+"}
                  {change24h.toFixed(2)}%
                </span>
                <span
                  className="text-xs ml-1"
                  style={{ color: fey.grey500 }}
                >
                  24h
                </span>
              </div>
            )}

            {/* Probability bar */}
            <div className="mt-4">
              <ProbabilityBar probability={probability} size="md" />
            </div>
          </div>

          {/* Right: Sparkline */}
          {priceHistory.length > 1 && (
            <div className="flex flex-col items-end">
              <div
                className="text-[10px] uppercase tracking-wider font-medium mb-2"
                style={{ color: fey.grey500 }}
              >
                7 Day Trend
              </div>
              <Sparkline
                data={priceHistory}
                width={100}
                height={40}
                strokeWidth={1.5}
                colorByTrend={true}
              />
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
