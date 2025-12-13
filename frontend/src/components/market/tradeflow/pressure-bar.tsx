"use client";

import { motion } from "framer-motion";
import { ArrowUp, ArrowDown } from "lucide-react";

// Fey color tokens
const fey = {
  bg300: "#131419",
  bg400: "#16181C",
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  teal: "#4DBE95",
  tealMuted: "rgba(77, 190, 149, 0.2)",
  red: "#D84F68",
  redMuted: "rgba(216, 79, 104, 0.2)",
  border: "rgba(255, 255, 255, 0.06)",
};

// ============================================================================
// Types
// ============================================================================

interface PressureBarProps {
  buyVolume: number;
  sellVolume: number;
  orientation?: "horizontal" | "vertical";
  showLabels?: boolean;
  showValues?: boolean;
  height?: number;
  className?: string;
}

// ============================================================================
// Helpers
// ============================================================================

const formatVolume = (volume: number): string => {
  if (volume >= 1_000_000) return `$${(volume / 1_000_000).toFixed(1)}M`;
  if (volume >= 1_000) return `$${(volume / 1_000).toFixed(1)}K`;
  return `$${volume.toFixed(0)}`;
};

// ============================================================================
// Horizontal Pressure Bar
// ============================================================================

const HorizontalPressureBar = ({
  buyVolume,
  sellVolume,
  showLabels,
  showValues,
}: Omit<PressureBarProps, "orientation" | "height" | "className">) => {
  const total = buyVolume + sellVolume;
  const buyPercent = total > 0 ? (buyVolume / total) * 100 : 50;
  const sellPercent = 100 - buyPercent;

  const isBullish = buyPercent > 55;
  const isBearish = sellPercent > 55;

  return (
    <div className="space-y-2">
      {/* Labels */}
      {showLabels && (
        <div className="flex items-center justify-between text-[10px] uppercase tracking-wider">
          <div className="flex items-center gap-1" style={{ color: fey.teal }}>
            <ArrowUp className="h-3 w-3" />
            Buyers
          </div>
          <div className="flex items-center gap-1" style={{ color: fey.red }}>
            Sellers
            <ArrowDown className="h-3 w-3" />
          </div>
        </div>
      )}

      {/* Bar */}
      <div
        className="h-3 rounded-full overflow-hidden flex"
        style={{ backgroundColor: fey.bg400 }}
      >
        {/* Buy side */}
        <motion.div
          className="h-full relative"
          style={{ backgroundColor: fey.teal }}
          initial={{ width: "50%" }}
          animate={{ width: `${buyPercent}%` }}
          transition={{ type: "spring", stiffness: 100, damping: 20 }}
        >
          {/* Gradient overlay */}
          <div
            className="absolute inset-0"
            style={{
              background: "linear-gradient(90deg, rgba(255,255,255,0.15), transparent)",
            }}
          />
        </motion.div>

        {/* Sell side */}
        <motion.div
          className="h-full relative"
          style={{ backgroundColor: fey.red }}
          initial={{ width: "50%" }}
          animate={{ width: `${sellPercent}%` }}
          transition={{ type: "spring", stiffness: 100, damping: 20 }}
        >
          {/* Gradient overlay */}
          <div
            className="absolute inset-0"
            style={{
              background: "linear-gradient(270deg, rgba(255,255,255,0.15), transparent)",
            }}
          />
        </motion.div>
      </div>

      {/* Values */}
      {showValues && (
        <div className="flex items-center justify-between text-xs font-mono">
          <span style={{ color: fey.teal }}>
            {formatVolume(buyVolume)}
            <span className="ml-1 text-[10px]" style={{ color: fey.grey500 }}>
              ({buyPercent.toFixed(0)}%)
            </span>
          </span>
          <span style={{ color: fey.red }}>
            <span className="mr-1 text-[10px]" style={{ color: fey.grey500 }}>
              ({sellPercent.toFixed(0)}%)
            </span>
            {formatVolume(sellVolume)}
          </span>
        </div>
      )}
    </div>
  );
};

// ============================================================================
// Vertical Pressure Bar (Thermometer style)
// ============================================================================

const VerticalPressureBar = ({
  buyVolume,
  sellVolume,
  showLabels,
  showValues,
  height = 120,
}: Omit<PressureBarProps, "orientation" | "className">) => {
  const total = buyVolume + sellVolume;
  const buyPercent = total > 0 ? (buyVolume / total) * 100 : 50;
  const sellPercent = 100 - buyPercent;

  // Net pressure: positive = buy pressure, negative = sell pressure
  const netPressure = (buyVolume - sellVolume) / Math.max(total, 1);

  return (
    <div className="flex gap-3">
      {/* Thermometer */}
      <div
        className="relative rounded-full overflow-hidden"
        style={{
          width: 24,
          height,
          backgroundColor: fey.bg400,
        }}
      >
        {/* Center line */}
        <div
          className="absolute left-0 right-0 h-px"
          style={{
            top: "50%",
            backgroundColor: fey.grey500,
            opacity: 0.5,
          }}
        />

        {/* Buy pressure (fills from center up) */}
        <motion.div
          className="absolute left-0 right-0"
          style={{
            bottom: "50%",
            backgroundColor: fey.teal,
          }}
          initial={{ height: 0 }}
          animate={{ height: `${Math.max(0, netPressure) * 50}%` }}
          transition={{ type: "spring", stiffness: 100, damping: 20 }}
        />

        {/* Sell pressure (fills from center down) */}
        <motion.div
          className="absolute left-0 right-0"
          style={{
            top: "50%",
            backgroundColor: fey.red,
          }}
          initial={{ height: 0 }}
          animate={{ height: `${Math.max(0, -netPressure) * 50}%` }}
          transition={{ type: "spring", stiffness: 100, damping: 20 }}
        />

        {/* Indicator dot at current level */}
        <motion.div
          className="absolute left-1/2 -translate-x-1/2 w-3 h-3 rounded-full"
          style={{
            backgroundColor: netPressure >= 0 ? fey.teal : fey.red,
            boxShadow: `0 0 8px ${netPressure >= 0 ? fey.teal : fey.red}`,
          }}
          initial={{ top: "50%" }}
          animate={{ top: `${50 - netPressure * 50}%` }}
          transition={{ type: "spring", stiffness: 100, damping: 20 }}
        />
      </div>

      {/* Labels and values */}
      <div className="flex flex-col justify-between text-[10px]">
        {showLabels && (
          <div className="flex items-center gap-1" style={{ color: fey.teal }}>
            <ArrowUp className="h-3 w-3" />
            <span className="uppercase tracking-wider">Buy</span>
          </div>
        )}

        {showValues && (
          <div className="flex flex-col gap-1">
            <div className="font-mono" style={{ color: fey.teal }}>
              {formatVolume(buyVolume)}
            </div>
            <div className="font-mono" style={{ color: fey.red }}>
              {formatVolume(sellVolume)}
            </div>
          </div>
        )}

        {showLabels && (
          <div className="flex items-center gap-1" style={{ color: fey.red }}>
            <ArrowDown className="h-3 w-3" />
            <span className="uppercase tracking-wider">Sell</span>
          </div>
        )}
      </div>
    </div>
  );
};

// ============================================================================
// Main Pressure Bar Component
// ============================================================================

export const PressureBar = ({
  buyVolume,
  sellVolume,
  orientation = "horizontal",
  showLabels = true,
  showValues = true,
  height = 120,
  className = "",
}: PressureBarProps) => {
  if (orientation === "vertical") {
    return (
      <div className={className}>
        <VerticalPressureBar
          buyVolume={buyVolume}
          sellVolume={sellVolume}
          showLabels={showLabels}
          showValues={showValues}
          height={height}
        />
      </div>
    );
  }

  return (
    <div className={className}>
      <HorizontalPressureBar
        buyVolume={buyVolume}
        sellVolume={sellVolume}
        showLabels={showLabels}
        showValues={showValues}
      />
    </div>
  );
};

// ============================================================================
// Mini Pressure Indicator (for compact displays)
// ============================================================================

interface MiniPressureProps {
  buyVolume: number;
  sellVolume: number;
  width?: number;
}

export const MiniPressure = ({
  buyVolume,
  sellVolume,
  width = 60,
}: MiniPressureProps) => {
  const total = buyVolume + sellVolume;
  const buyPercent = total > 0 ? (buyVolume / total) * 100 : 50;

  return (
    <div className="flex items-center gap-2">
      <div
        className="h-1.5 rounded-full overflow-hidden flex"
        style={{ width, backgroundColor: fey.bg400 }}
      >
        <motion.div
          className="h-full"
          style={{ backgroundColor: fey.teal }}
          animate={{ width: `${buyPercent}%` }}
          transition={{ type: "spring", stiffness: 100, damping: 20 }}
        />
        <motion.div
          className="h-full"
          style={{ backgroundColor: fey.red }}
          animate={{ width: `${100 - buyPercent}%` }}
          transition={{ type: "spring", stiffness: 100, damping: 20 }}
        />
      </div>
      <span
        className="text-[10px] font-mono"
        style={{ color: buyPercent >= 50 ? fey.teal : fey.red }}
      >
        {buyPercent.toFixed(0)}:{(100 - buyPercent).toFixed(0)}
      </span>
    </div>
  );
};

export default PressureBar;
