"use client";

import { useMemo } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Activity } from "lucide-react";
import type { ProcessedTrade } from "@/hooks/use-trade-momentum";

// Fey color tokens
const fey = {
  bg300: "#131419",
  bg400: "#16181C",
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  teal: "#4DBE95",
  tealMuted: "rgba(77, 190, 149, 0.3)",
  red: "#D84F68",
  redMuted: "rgba(216, 79, 104, 0.3)",
  skyBlue: "#54BBF7",
  border: "rgba(255, 255, 255, 0.06)",
};

// ============================================================================
// Types
// ============================================================================

interface BubbleTimelineProps {
  trades: ProcessedTrade[];
  maxTrades?: number;
  height?: number;
  showLabels?: boolean;
  className?: string;
}

// ============================================================================
// Helpers
// ============================================================================

const formatTime = (timestamp: string): string => {
  const date = new Date(timestamp);
  const now = new Date();
  const diff = now.getTime() - date.getTime();

  if (diff < 60000) return "now";
  if (diff < 3600000) return `${Math.floor(diff / 60000)}m`;
  return `${Math.floor(diff / 3600000)}h`;
};

const formatQuantity = (qty: number): string => {
  if (qty >= 1_000_000) return `${(qty / 1_000_000).toFixed(1)}M`;
  if (qty >= 1_000) return `${(qty / 1_000).toFixed(1)}K`;
  return qty.toFixed(0);
};

// Calculate bubble size from relative size (log scale)
const getBubbleSize = (relativeSize: number, minSize = 8, maxSize = 32): number => {
  // Log scale to prevent huge bubbles
  const logSize = Math.log(relativeSize + 1);
  const normalizedSize = Math.min(logSize / 2, 1);
  return minSize + normalizedSize * (maxSize - minSize);
};

// ============================================================================
// Trade Bubble Component
// ============================================================================

interface TradeBubbleProps {
  trade: ProcessedTrade;
  index: number;
  totalTrades: number;
}

const TradeBubble = ({ trade, index, totalTrades }: TradeBubbleProps) => {
  const isBuy = trade.side?.toLowerCase() === "buy";
  const size = getBubbleSize(trade.relativeSize);
  const color = isBuy ? fey.teal : fey.red;
  const mutedColor = isBuy ? fey.tealMuted : fey.redMuted;

  // Position from right (newest on right)
  const position = ((totalTrades - index - 1) / Math.max(totalTrades - 1, 1)) * 100;

  return (
    <motion.div
      className="absolute flex flex-col items-center"
      style={{
        left: `${position}%`,
        transform: "translateX(-50%)",
        bottom: 0,
      }}
      initial={{ scale: 0, opacity: 0, y: 20 }}
      animate={{ scale: 1, opacity: 1, y: 0 }}
      exit={{ scale: 0, opacity: 0 }}
      transition={{
        type: "spring",
        stiffness: 500,
        damping: 25,
        delay: index * 0.02,
      }}
      whileHover={{ scale: 1.2 }}
    >
      {/* Whale indicator */}
      {trade.isWhale && (
        <motion.div
          className="absolute -top-3 text-[10px]"
          initial={{ opacity: 0, y: 5 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
        >
          🐋
        </motion.div>
      )}

      {/* Bubble */}
      <motion.div
        className="rounded-full relative cursor-pointer"
        style={{
          width: size,
          height: size,
          backgroundColor: mutedColor,
          border: `2px solid ${color}`,
        }}
        whileHover={{
          boxShadow: `0 0 12px ${color}`,
        }}
      >
        {/* Inner glow for whales */}
        {trade.isWhale && (
          <motion.div
            className="absolute inset-0 rounded-full"
            style={{ backgroundColor: color }}
            animate={{ opacity: [0.3, 0.6, 0.3] }}
            transition={{ duration: 1.5, repeat: Infinity }}
          />
        )}
      </motion.div>

      {/* Tooltip on hover (handled by parent) */}
    </motion.div>
  );
};

// ============================================================================
// Main Bubble Timeline Component
// ============================================================================

export const BubbleTimeline = ({
  trades,
  maxTrades = 30,
  height = 80,
  showLabels = true,
  className = "",
}: BubbleTimelineProps) => {
  const displayTrades = useMemo(
    () => trades.slice(0, maxTrades),
    [trades, maxTrades],
  );

  // Calculate stats
  const stats = useMemo(() => {
    const buys = displayTrades.filter((t) => t.side?.toLowerCase() === "buy");
    const sells = displayTrades.filter((t) => t.side?.toLowerCase() !== "buy");
    return {
      buyCount: buys.length,
      sellCount: sells.length,
      whaleCount: displayTrades.filter((t) => t.isWhale).length,
    };
  }, [displayTrades]);

  return (
    <div
      className={`rounded-lg overflow-hidden ${className}`}
      style={{
        backgroundColor: fey.bg300,
        border: `1px solid ${fey.border}`,
      }}
    >
      {/* Header */}
      <div
        className="px-4 py-3 flex items-center justify-between"
        style={{ borderBottom: `1px solid ${fey.border}` }}
      >
        <div className="flex items-center gap-2">
          <div
            className="p-1.5 rounded"
            style={{ backgroundColor: "rgba(84, 187, 247, 0.1)" }}
          >
            <Activity className="h-4 w-4" style={{ color: fey.skyBlue }} />
          </div>
          <span
            className="text-sm font-semibold"
            style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
          >
            Trade Flow
          </span>
        </div>

        {/* Stats */}
        <div className="flex items-center gap-3 text-[10px]">
          <div className="flex items-center gap-1">
            <div
              className="h-2 w-2 rounded-full"
              style={{ backgroundColor: fey.teal }}
            />
            <span style={{ color: fey.grey500 }}>{stats.buyCount}</span>
          </div>
          <div className="flex items-center gap-1">
            <div
              className="h-2 w-2 rounded-full"
              style={{ backgroundColor: fey.red }}
            />
            <span style={{ color: fey.grey500 }}>{stats.sellCount}</span>
          </div>
          {stats.whaleCount > 0 && (
            <span style={{ color: fey.grey500 }}>🐋 {stats.whaleCount}</span>
          )}
        </div>
      </div>

      {/* Timeline */}
      <div className="px-4 py-3">
        <div className="relative" style={{ height }}>
          {/* Center line */}
          <div
            className="absolute left-0 right-0 top-1/2 h-px"
            style={{ backgroundColor: fey.border }}
          />

          {/* Bubbles */}
          <AnimatePresence mode="popLayout">
            {displayTrades.map((trade, index) => (
              <TradeBubble
                key={trade.id}
                trade={trade}
                index={index}
                totalTrades={displayTrades.length}
              />
            ))}
          </AnimatePresence>

          {/* Empty state */}
          {displayTrades.length === 0 && (
            <div
              className="absolute inset-0 flex items-center justify-center text-sm"
              style={{ color: fey.grey500 }}
            >
              No recent trades
            </div>
          )}
        </div>

        {/* Time labels */}
        {showLabels && displayTrades.length > 0 && (
          <div
            className="flex items-center justify-between mt-2 text-[10px]"
            style={{ color: fey.grey500 }}
          >
            <span>
              {formatTime(displayTrades[displayTrades.length - 1]?.timestamp)}
            </span>
            <span>→ now</span>
          </div>
        )}
      </div>
    </div>
  );
};

// ============================================================================
// Compact Trade Strip (horizontal bar version)
// ============================================================================

interface TradeStripProps {
  trades: ProcessedTrade[];
  maxTrades?: number;
  height?: number;
  className?: string;
}

export const TradeStrip = ({
  trades,
  maxTrades = 50,
  height = 24,
  className = "",
}: TradeStripProps) => {
  const displayTrades = useMemo(
    () => trades.slice(0, maxTrades),
    [trades, maxTrades],
  );

  return (
    <div
      className={`relative overflow-hidden rounded ${className}`}
      style={{ height, backgroundColor: fey.bg400 }}
    >
      <AnimatePresence mode="popLayout">
        {displayTrades.map((trade, index) => {
          const isBuy = trade.side?.toLowerCase() === "buy";
          const width = Math.max(2, Math.min(8, trade.relativeSize * 3));
          const position =
            ((maxTrades - index - 1) / maxTrades) * 100;

          return (
            <motion.div
              key={trade.id}
              className="absolute top-0 bottom-0"
              style={{
                left: `${position}%`,
                width,
                backgroundColor: isBuy ? fey.teal : fey.red,
                opacity: trade.isWhale ? 1 : 0.6,
              }}
              initial={{ scaleY: 0 }}
              animate={{ scaleY: 1 }}
              exit={{ scaleY: 0, opacity: 0 }}
              transition={{ duration: 0.2 }}
            />
          );
        })}
      </AnimatePresence>
    </div>
  );
};

export default BubbleTimeline;
