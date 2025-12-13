"use client";

import { motion } from "framer-motion";
import { TrendingUp, TrendingDown, Minus } from "lucide-react";

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

interface ImbalanceMeterProps {
  /** Imbalance ratio from -1 (all asks) to 1 (all bids) */
  imbalanceRatio: number;
  /** Total bid quantity */
  bidQuantity: number;
  /** Total ask quantity */
  askQuantity: number;
  /** Show detailed breakdown */
  showDetails?: boolean;
  /** Compact mode */
  compact?: boolean;
  className?: string;
}

// ============================================================================
// Helpers
// ============================================================================

const formatQuantity = (qty: number): string => {
  if (qty >= 1_000_000) return `${(qty / 1_000_000).toFixed(1)}M`;
  if (qty >= 1_000) return `${(qty / 1_000).toFixed(1)}K`;
  return qty.toFixed(0);
};

const getImbalanceLabel = (ratio: number): string => {
  if (ratio > 0.3) return "Strong Buy Pressure";
  if (ratio > 0.1) return "Buy Pressure";
  if (ratio < -0.3) return "Strong Sell Pressure";
  if (ratio < -0.1) return "Sell Pressure";
  return "Balanced";
};

const getImbalanceIcon = (ratio: number) => {
  if (ratio > 0.1) return TrendingUp;
  if (ratio < -0.1) return TrendingDown;
  return Minus;
};

// ============================================================================
// Main Component
// ============================================================================

export const ImbalanceMeter = ({
  imbalanceRatio,
  bidQuantity,
  askQuantity,
  showDetails = false,
  compact = false,
  className = "",
}: ImbalanceMeterProps) => {
  // Clamp ratio to -1 to 1
  const ratio = Math.max(-1, Math.min(1, imbalanceRatio));

  // Calculate percentages
  const total = bidQuantity + askQuantity;
  const bidPercent = total > 0 ? (bidQuantity / total) * 100 : 50;
  const askPercent = total > 0 ? (askQuantity / total) * 100 : 50;

  // Get state based on ratio
  const isBullish = ratio > 0.1;
  const isBearish = ratio < -0.1;
  const isNeutral = !isBullish && !isBearish;

  const Icon = getImbalanceIcon(ratio);
  const label = getImbalanceLabel(ratio);

  if (compact) {
    return (
      <div className={`flex items-center gap-2 ${className}`}>
        {/* Mini bar */}
        <div
          className="w-24 h-1.5 rounded-full overflow-hidden flex"
          style={{ backgroundColor: fey.bg400 }}
        >
          <motion.div
            className="h-full"
            style={{ backgroundColor: fey.teal }}
            initial={{ width: 0 }}
            animate={{ width: `${bidPercent}%` }}
            transition={{ type: "spring", stiffness: 100, damping: 20 }}
          />
          <motion.div
            className="h-full"
            style={{ backgroundColor: fey.red }}
            initial={{ width: 0 }}
            animate={{ width: `${askPercent}%` }}
            transition={{ type: "spring", stiffness: 100, damping: 20 }}
          />
        </div>
        {/* Ratio indicator */}
        <div className="flex items-center gap-1">
          <Icon
            className="h-3 w-3"
            style={{
              color: isBullish ? fey.teal : isBearish ? fey.red : fey.grey500,
            }}
          />
          <span
            className="text-xs font-mono"
            style={{
              color: isBullish ? fey.teal : isBearish ? fey.red : fey.grey500,
            }}
          >
            {bidPercent.toFixed(0)}:{askPercent.toFixed(0)}
          </span>
        </div>
      </div>
    );
  }

  return (
    <div
      className={`rounded-lg p-3 ${className}`}
      style={{
        backgroundColor: fey.bg400,
        border: `1px solid ${fey.border}`,
      }}
    >
      {/* Header */}
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <div
            className="p-1 rounded"
            style={{
              backgroundColor: isBullish
                ? fey.tealMuted
                : isBearish
                  ? fey.redMuted
                  : fey.bg300,
            }}
          >
            <Icon
              className="h-3.5 w-3.5"
              style={{
                color: isBullish ? fey.teal : isBearish ? fey.red : fey.grey500,
              }}
            />
          </div>
          <span
            className="text-xs font-medium"
            style={{
              color: isBullish ? fey.teal : isBearish ? fey.red : fey.grey100,
            }}
          >
            {label}
          </span>
        </div>
        {showDetails && (
          <span className="text-[10px]" style={{ color: fey.grey500 }}>
            {formatQuantity(total)} total
          </span>
        )}
      </div>

      {/* Main Bar */}
      <div className="relative">
        <div
          className="h-3 rounded-full overflow-hidden flex"
          style={{ backgroundColor: fey.bg300 }}
        >
          {/* Bid side */}
          <motion.div
            className="h-full relative"
            style={{ backgroundColor: fey.teal }}
            initial={{ width: 0 }}
            animate={{ width: `${bidPercent}%` }}
            transition={{ type: "spring", stiffness: 100, damping: 20 }}
          >
            {/* Gradient overlay for depth effect */}
            <div
              className="absolute inset-0"
              style={{
                background: `linear-gradient(90deg, transparent, rgba(255,255,255,0.1))`,
              }}
            />
          </motion.div>

          {/* Ask side */}
          <motion.div
            className="h-full relative"
            style={{ backgroundColor: fey.red }}
            initial={{ width: 0 }}
            animate={{ width: `${askPercent}%` }}
            transition={{ type: "spring", stiffness: 100, damping: 20 }}
          >
            {/* Gradient overlay */}
            <div
              className="absolute inset-0"
              style={{
                background: `linear-gradient(270deg, transparent, rgba(255,255,255,0.1))`,
              }}
            />
          </motion.div>
        </div>

        {/* Center line indicator */}
        <div
          className="absolute top-0 bottom-0 w-px"
          style={{
            left: "50%",
            backgroundColor: "rgba(255, 255, 255, 0.3)",
          }}
        />
      </div>

      {/* Labels */}
      <div className="flex items-center justify-between mt-2">
        <div className="flex items-center gap-1.5">
          <span
            className="text-xs font-mono font-medium"
            style={{ color: fey.teal }}
          >
            {bidPercent.toFixed(0)}%
          </span>
          {showDetails && (
            <span className="text-[10px]" style={{ color: fey.grey500 }}>
              ({formatQuantity(bidQuantity)})
            </span>
          )}
        </div>
        <div className="flex items-center gap-1.5">
          {showDetails && (
            <span className="text-[10px]" style={{ color: fey.grey500 }}>
              ({formatQuantity(askQuantity)})
            </span>
          )}
          <span
            className="text-xs font-mono font-medium"
            style={{ color: fey.red }}
          >
            {askPercent.toFixed(0)}%
          </span>
        </div>
      </div>
    </div>
  );
};

// ============================================================================
// Mini Imbalance Indicator (for use in headers/bars)
// ============================================================================

interface MiniImbalanceProps {
  ratio: number;
  size?: "sm" | "md";
  showLabel?: boolean;
}

export const MiniImbalance = ({
  ratio,
  size = "sm",
  showLabel = true,
}: MiniImbalanceProps) => {
  const isBullish = ratio > 0.1;
  const isBearish = ratio < -0.1;

  const Icon = getImbalanceIcon(ratio);
  const dimensions = size === "sm" ? "h-3 w-3" : "h-4 w-4";

  return (
    <div className="flex items-center gap-1">
      <Icon
        className={dimensions}
        style={{
          color: isBullish ? fey.teal : isBearish ? fey.red : fey.grey500,
        }}
      />
      {showLabel && (
        <span
          className={`font-mono ${size === "sm" ? "text-[10px]" : "text-xs"}`}
          style={{
            color: isBullish ? fey.teal : isBearish ? fey.red : fey.grey500,
          }}
        >
          {Math.abs(ratio * 100).toFixed(0)}%
        </span>
      )}
    </div>
  );
};

export default ImbalanceMeter;
