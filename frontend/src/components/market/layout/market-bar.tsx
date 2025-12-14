"use client";

import { motion } from "framer-motion";
import { AnimatedNumber } from "@/components/ui/animated-number";
import { marketBarVariants } from "@/lib/motion";
import { TrendingUp, TrendingDown, Minus, Activity, Zap } from "lucide-react";
import { ConnectionIndicator } from "../connection-indicator";
import type { ConnectionState } from "@/hooks/use-websocket";

// Fey color tokens
const fey = {
  bg100: "#070709",
  bg200: "#101116",
  bg300: "#131419",
  grey100: "#EEF0F1",
  grey300: "#B6BEC4",
  grey500: "#7D8B96",
  teal: "#4DBE95",
  red: "#D84F68",
  skyBlue: "#54BBF7",
  border: "rgba(255, 255, 255, 0.06)",
};

// ============================================================================
// Types
// ============================================================================

interface Trade {
  id: string;
  price: string;
  quantity: string;
  side?: string | null;
  timestamp: string;
}

interface MarketBarProps {
  yesPrice: string;
  noPrice: string;
  spread?: number | null;
  volume24h?: string | null;
  lastTrade?: Trade | null;
  change24h?: number;
  connectionState?: ConnectionState;
  latency?: number | null;
}

// ============================================================================
// Helpers
// ============================================================================

const parsePrice = (price: string): number => {
  const num = parseFloat(price);
  return isNaN(num) ? 0 : num;
};

const formatSpread = (spread: number | null | undefined): string => {
  if (spread === undefined || spread === null) return "—";
  return `${(spread * 100).toFixed(1)}¢`;
};

const formatVolume = (volume: string | null | undefined): string => {
  if (!volume) return "—";
  const num = parseFloat(volume);
  if (isNaN(num)) return "—";

  if (num >= 1_000_000) {
    return `$${(num / 1_000_000).toFixed(1)}M`;
  }
  if (num >= 1_000) {
    return `$${(num / 1_000).toFixed(1)}K`;
  }
  return `$${num.toFixed(0)}`;
};

// ============================================================================
// Stat Item Component
// ============================================================================

interface StatItemProps {
  label: string;
  value: React.ReactNode;
  color?: string;
  icon?: React.ReactNode;
}

const StatItem = ({ label, value, color = fey.grey100, icon }: StatItemProps) => (
  <div className="flex items-center gap-3">
    <span
      className="text-[10px] uppercase tracking-wider font-medium"
      style={{ color: fey.grey500 }}
    >
      {label}
    </span>
    <div className="flex items-center gap-1.5">
      {icon}
      <span className="font-mono text-sm font-medium" style={{ color }}>
        {value}
      </span>
    </div>
  </div>
);

// ============================================================================
// Spread Visualizer (mini bid-ask visualization)
// ============================================================================

interface SpreadVisualizerProps {
  bidPrice: number;
  askPrice: number;
}

const SpreadVisualizer = ({ bidPrice, askPrice }: SpreadVisualizerProps) => {
  const spread = askPrice - bidPrice;
  const midpoint = (bidPrice + askPrice) / 2;

  return (
    <div className="flex items-center gap-2">
      {/* Bid */}
      <div className="flex items-center gap-1">
        <span
          className="text-[10px] uppercase tracking-wider"
          style={{ color: fey.grey500 }}
        >
          Bid
        </span>
        <span className="font-mono text-sm font-medium" style={{ color: fey.teal }}>
          {(bidPrice * 100).toFixed(1)}¢
        </span>
      </div>

      {/* Visual spread bar */}
      <div className="w-16 h-1 rounded-full relative overflow-hidden" style={{ backgroundColor: fey.bg300 }}>
        <motion.div
          className="absolute inset-y-0 left-0 rounded-full"
          style={{ backgroundColor: fey.teal }}
          initial={{ width: 0 }}
          animate={{ width: `${bidPrice * 100}%` }}
          transition={{ type: "spring", stiffness: 100, damping: 20 }}
        />
        <motion.div
          className="absolute inset-y-0 right-0 rounded-full"
          style={{ backgroundColor: fey.red }}
          initial={{ width: 0 }}
          animate={{ width: `${(1 - askPrice) * 100}%` }}
          transition={{ type: "spring", stiffness: 100, damping: 20 }}
        />
      </div>

      {/* Ask */}
      <div className="flex items-center gap-1">
        <span
          className="text-[10px] uppercase tracking-wider"
          style={{ color: fey.grey500 }}
        >
          Ask
        </span>
        <span className="font-mono text-sm font-medium" style={{ color: fey.red }}>
          {(askPrice * 100).toFixed(1)}¢
        </span>
      </div>

      {/* Spread value */}
      <div
        className="px-2 py-0.5 rounded text-xs font-mono"
        style={{ backgroundColor: fey.bg300, color: fey.grey500 }}
      >
        {(spread * 100).toFixed(1)}¢ spread
      </div>
    </div>
  );
};

// ============================================================================
// Main Market Bar Component
// ============================================================================

export const MarketBar = ({
  yesPrice,
  noPrice,
  spread,
  volume24h,
  lastTrade,
  change24h,
  connectionState = "connected",
  latency,
}: MarketBarProps) => {
  // Extract last trade info
  const lastTradePrice = lastTrade?.price;
  const lastTradeSide = lastTrade?.side?.toLowerCase() as "buy" | "sell" | undefined;
  const yesPriceNum = parsePrice(yesPrice);
  const noPriceNum = parsePrice(noPrice);

  // Calculate spread if not provided
  const calculatedSpread = spread !== null && spread !== undefined
    ? spread
    : Math.abs(1 - yesPriceNum - noPriceNum);

  // Determine 24h change color and icon
  const getChangeDisplay = () => {
    if (change24h === undefined) return null;

    const isPositive = change24h >= 0;
    const color = change24h === 0 ? fey.grey500 : isPositive ? fey.teal : fey.red;
    const Icon = change24h === 0 ? Minus : isPositive ? TrendingUp : TrendingDown;

    return {
      color,
      icon: <Icon className="h-3 w-3" style={{ color }} />,
      value: `${isPositive ? "+" : ""}${(change24h * 100).toFixed(1)}%`,
    };
  };

  const changeDisplay = getChangeDisplay();

  return (
    <motion.div
      className="fixed bottom-0 left-0 right-0 z-50"
      style={{
        backgroundColor: fey.bg200,
        borderTop: `1px solid ${fey.border}`,
        backdropFilter: "blur(12px)",
      }}
      variants={marketBarVariants}
      initial="hidden"
      animate="visible"
    >
      <div className="px-6 lg:px-8 py-4">
        <div className="flex items-center justify-between gap-4">
          {/* Left side: Core pricing */}
          <div className="flex items-center gap-4 lg:gap-6 flex-shrink-0">
            {/* YES Price (Primary) */}
            <div className="flex items-center gap-2">
              <span
                className="text-[10px] uppercase tracking-wider font-semibold px-1.5 py-0.5 rounded"
                style={{
                  backgroundColor: `${fey.teal}20`,
                  color: fey.teal,
                }}
              >
                Yes
              </span>
              <AnimatedNumber
                value={yesPriceNum}
                format="price"
                decimals={1}
                colorByChange
                className="text-lg font-bold"
                style={{ color: fey.teal }}
              />
            </div>

            {/* Divider */}
            <div className="h-6 w-px" style={{ backgroundColor: fey.border }} />

            {/* NO Price */}
            <div className="flex items-center gap-2">
              <span
                className="text-[10px] uppercase tracking-wider font-semibold px-1.5 py-0.5 rounded"
                style={{
                  backgroundColor: `${fey.red}20`,
                  color: fey.red,
                }}
              >
                No
              </span>
              <AnimatedNumber
                value={noPriceNum}
                format="price"
                decimals={1}
                colorByChange
                className="text-lg font-bold"
                style={{ color: fey.red }}
              />
            </div>

            {/* Divider - hide on small screens */}
            <div className="hidden md:block h-6 w-px" style={{ backgroundColor: fey.border }} />

            {/* Spread - hide on small screens */}
            <div className="hidden md:block">
              <StatItem label="Spread" value={formatSpread(calculatedSpread)} />
            </div>
          </div>

          {/* Right side: Volume and activity */}
          <div className="flex items-center gap-4 lg:gap-6 flex-shrink-0">
            {/* 24h Change - hide on small screens */}
            {changeDisplay && (
              <div className="hidden md:block">
                <StatItem
                  label="24h"
                  value={changeDisplay.value}
                  color={changeDisplay.color}
                  icon={changeDisplay.icon}
                />
              </div>
            )}

            {/* Volume */}
            <StatItem label="Volume" value={formatVolume(volume24h)} />

            {/* Last Trade - hide on small screens */}
            {lastTradePrice && (
              <div className="hidden lg:flex items-center gap-2">
                <Zap
                  className="h-3 w-3"
                  style={{
                    color: lastTradeSide === "buy" ? fey.teal : fey.red,
                  }}
                />
                <span
                  className="font-mono text-sm"
                  style={{
                    color: lastTradeSide === "buy" ? fey.teal : fey.red,
                  }}
                >
                  {(parseFloat(lastTradePrice) * 100).toFixed(1)}¢
                </span>
              </div>
            )}

            {/* Connection Status */}
            <ConnectionIndicator
              state={connectionState}
              latency={latency}
              showLabel={true}
            />
          </div>
        </div>
      </div>
    </motion.div>
  );
};

export default MarketBar;
