"use client";

import { motion } from "framer-motion";
import { TrendingUp, TrendingDown, Minus, Zap } from "lucide-react";
import type { TradeMomentum } from "@/hooks/use-trade-momentum";

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
  skyBlue: "#54BBF7",
  border: "rgba(255, 255, 255, 0.06)",
};

// ============================================================================
// Types
// ============================================================================

interface MomentumGaugeProps {
  momentum: TradeMomentum;
  showDetails?: boolean;
  compact?: boolean;
  className?: string;
}

// ============================================================================
// Helpers
// ============================================================================

const formatVolume = (volume: number): string => {
  if (volume >= 1_000_000) return `${(volume / 1_000_000).toFixed(1)}M`;
  if (volume >= 1_000) return `${(volume / 1_000).toFixed(1)}K`;
  return volume.toFixed(0);
};

const getMomentumLabel = (ratio: number): string => {
  if (ratio > 0.5) return "Strong Buying";
  if (ratio > 0.2) return "Buying";
  if (ratio < -0.5) return "Strong Selling";
  if (ratio < -0.2) return "Selling";
  return "Neutral";
};

// ============================================================================
// Radial Gauge Component
// ============================================================================

interface RadialGaugeProps {
  value: number; // -1 to 1
  size?: number;
  strokeWidth?: number;
}

const RadialGauge = ({ value, size = 80, strokeWidth = 6 }: RadialGaugeProps) => {
  const radius = (size - strokeWidth) / 2;
  const circumference = radius * Math.PI; // Semi-circle
  const offset = circumference * (1 - (value + 1) / 2);

  const getColor = () => {
    if (value > 0.2) return fey.teal;
    if (value < -0.2) return fey.red;
    return fey.grey500;
  };

  return (
    <div className="relative" style={{ width: size, height: size / 2 + 10 }}>
      <svg width={size} height={size / 2 + 10} className="overflow-visible">
        {/* Background arc */}
        <path
          d={`M ${strokeWidth / 2} ${size / 2} A ${radius} ${radius} 0 0 1 ${size - strokeWidth / 2} ${size / 2}`}
          fill="none"
          stroke={fey.bg400}
          strokeWidth={strokeWidth}
          strokeLinecap="round"
        />

        {/* Value arc */}
        <motion.path
          d={`M ${strokeWidth / 2} ${size / 2} A ${radius} ${radius} 0 0 1 ${size - strokeWidth / 2} ${size / 2}`}
          fill="none"
          stroke={getColor()}
          strokeWidth={strokeWidth}
          strokeLinecap="round"
          initial={{ pathLength: 0 }}
          animate={{ pathLength: (value + 1) / 2 }}
          transition={{ type: "spring", stiffness: 60, damping: 15 }}
        />

        {/* Center indicator */}
        <line
          x1={size / 2}
          y1={size / 2}
          x2={size / 2}
          y2={size / 2 - radius + strokeWidth}
          stroke={fey.grey500}
          strokeWidth={1}
          opacity={0.5}
        />

        {/* Needle */}
        <motion.g
          initial={{ rotate: -90 }}
          animate={{ rotate: -90 + (value + 1) * 90 }}
          transition={{ type: "spring", stiffness: 100, damping: 15 }}
          style={{ transformOrigin: `${size / 2}px ${size / 2}px` }}
        >
          <line
            x1={size / 2}
            y1={size / 2}
            x2={size / 2}
            y2={strokeWidth + 8}
            stroke={fey.grey100}
            strokeWidth={2}
            strokeLinecap="round"
          />
          <circle cx={size / 2} cy={size / 2} r={4} fill={fey.grey100} />
        </motion.g>
      </svg>

      {/* Labels */}
      <div
        className="absolute bottom-0 left-0 text-[10px]"
        style={{ color: fey.red }}
      >
        SELL
      </div>
      <div
        className="absolute bottom-0 right-0 text-[10px]"
        style={{ color: fey.teal }}
      >
        BUY
      </div>
    </div>
  );
};

// ============================================================================
// Main Momentum Gauge Component
// ============================================================================

export const MomentumGauge = ({
  momentum,
  showDetails = false,
  compact = false,
  className = "",
}: MomentumGaugeProps) => {
  const { momentumRatio, direction, isAccelerating, velocity, hasWhale } =
    momentum;

  const Icon =
    direction === "buy"
      ? TrendingUp
      : direction === "sell"
        ? TrendingDown
        : Minus;

  const color =
    direction === "buy" ? fey.teal : direction === "sell" ? fey.red : fey.grey500;

  if (compact) {
    return (
      <div className={`flex items-center gap-2 ${className}`}>
        <Icon className="h-4 w-4" style={{ color }} />
        <span className="text-sm font-mono font-medium" style={{ color }}>
          {(momentumRatio * 100).toFixed(0)}%
        </span>
        {isAccelerating && (
          <Zap className="h-3 w-3" style={{ color: fey.skyBlue }} />
        )}
      </div>
    );
  }

  return (
    <div
      className={`rounded-lg p-4 ${className}`}
      style={{
        backgroundColor: fey.bg300,
        border: `1px solid ${fey.border}`,
      }}
    >
      {/* Header */}
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <div
            className="p-1.5 rounded"
            style={{ backgroundColor: `${color}15` }}
          >
            <Icon className="h-4 w-4" style={{ color }} />
          </div>
          <span
            className="text-sm font-semibold"
            style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
          >
            Momentum
          </span>
        </div>

        {/* Status badges */}
        <div className="flex items-center gap-2">
          {isAccelerating && (
            <div
              className="flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px]"
              style={{ backgroundColor: `${fey.skyBlue}15`, color: fey.skyBlue }}
            >
              <Zap className="h-3 w-3" />
              Accelerating
            </div>
          )}
          {hasWhale && (
            <div
              className="px-1.5 py-0.5 rounded text-[10px]"
              style={{ backgroundColor: `${fey.teal}15`, color: fey.teal }}
            >
              🐋 Whale
            </div>
          )}
        </div>
      </div>

      {/* Gauge */}
      <div className="flex justify-center mb-3">
        <RadialGauge value={momentumRatio} size={100} />
      </div>

      {/* Label */}
      <div className="text-center mb-3">
        <span className="text-lg font-bold" style={{ color }}>
          {getMomentumLabel(momentumRatio)}
        </span>
      </div>

      {/* Stats Grid */}
      {showDetails && (
        <div
          className="grid grid-cols-2 gap-3 pt-3"
          style={{ borderTop: `1px solid ${fey.border}` }}
        >
          {/* Buy Volume */}
          <div>
            <span
              className="text-[10px] uppercase tracking-wider block mb-1"
              style={{ color: fey.grey500 }}
            >
              Buy Volume
            </span>
            <span
              className="text-sm font-mono font-medium"
              style={{ color: fey.teal }}
            >
              {formatVolume(momentum.buyVolume)}
            </span>
            <span className="text-[10px] ml-1" style={{ color: fey.grey500 }}>
              ({momentum.buyCount})
            </span>
          </div>

          {/* Sell Volume */}
          <div>
            <span
              className="text-[10px] uppercase tracking-wider block mb-1"
              style={{ color: fey.grey500 }}
            >
              Sell Volume
            </span>
            <span
              className="text-sm font-mono font-medium"
              style={{ color: fey.red }}
            >
              {formatVolume(momentum.sellVolume)}
            </span>
            <span className="text-[10px] ml-1" style={{ color: fey.grey500 }}>
              ({momentum.sellCount})
            </span>
          </div>

          {/* Net Flow */}
          <div>
            <span
              className="text-[10px] uppercase tracking-wider block mb-1"
              style={{ color: fey.grey500 }}
            >
              Net Flow
            </span>
            <span
              className="text-sm font-mono font-medium"
              style={{ color: momentum.netVolume >= 0 ? fey.teal : fey.red }}
            >
              {momentum.netVolume >= 0 ? "+" : ""}
              {formatVolume(momentum.netVolume)}
            </span>
          </div>

          {/* Velocity */}
          <div>
            <span
              className="text-[10px] uppercase tracking-wider block mb-1"
              style={{ color: fey.grey500 }}
            >
              Velocity
            </span>
            <span
              className="text-sm font-mono font-medium"
              style={{ color: fey.grey100 }}
            >
              {velocity.toFixed(1)}/min
            </span>
          </div>
        </div>
      )}
    </div>
  );
};

export default MomentumGauge;
