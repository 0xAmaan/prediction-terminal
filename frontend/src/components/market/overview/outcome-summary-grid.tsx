"use client";

import { motion } from "framer-motion";
import { Crown, TrendingUp, TrendingDown, Minus } from "lucide-react";
import type { MarketOption } from "@/lib/types";
import { staggerItem } from "@/lib/motion";

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
  border: "rgba(255, 255, 255, 0.06)",
  borderHover: "rgba(255, 255, 255, 0.12)",
};

// ============================================================================
// Types
// ============================================================================

export interface OutcomeSummaryGridProps {
  options: MarketOption[];
  maxDisplay?: number;
  onOutcomeClick?: (outcome: MarketOption) => void;
  className?: string;
}

// ============================================================================
// Helpers
// ============================================================================

const formatPrice = (price: string): string => {
  const num = parseFloat(price);
  if (isNaN(num)) return "—";
  return `${(num * 100).toFixed(1)}¢`;
};

const formatPercent = (price: string): string => {
  const num = parseFloat(price);
  if (isNaN(num)) return "—";
  return `${(num * 100).toFixed(0)}%`;
};

// Get color based on probability
const getProbabilityColor = (price: string): string => {
  const num = parseFloat(price);
  if (isNaN(num)) return fey.grey500;
  if (num >= 0.5) return fey.teal;
  if (num >= 0.2) return fey.grey300;
  return fey.grey500;
};

// ============================================================================
// Outcome Card Component
// ============================================================================

interface OutcomeCardProps {
  option: MarketOption;
  isLeading: boolean;
  rank: number;
  onClick?: () => void;
}

const OutcomeCard = ({ option, isLeading, rank, onClick }: OutcomeCardProps) => {
  const probability = parseFloat(option.yes_price);
  const color = getProbabilityColor(option.yes_price);

  return (
    <motion.button
      className="w-full text-left rounded-lg p-4 transition-colors relative overflow-hidden"
      style={{
        backgroundColor: fey.bg300,
        border: `1px solid ${isLeading ? `${fey.teal}30` : fey.border}`,
      }}
      onClick={onClick}
      variants={staggerItem}
      whileHover={{
        borderColor: fey.borderHover,
        backgroundColor: fey.bg200,
      }}
      transition={{ duration: 0.2 }}
    >
      {/* Leading badge */}
      {isLeading && (
        <div
          className="absolute top-2 right-2 flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase tracking-wider"
          style={{ backgroundColor: `${fey.teal}20`, color: fey.teal }}
        >
          <Crown className="w-2.5 h-2.5" />
          Leading
        </div>
      )}

      {/* Rank indicator */}
      <div
        className="text-[10px] font-mono font-medium mb-2"
        style={{ color: fey.grey500 }}
      >
        #{rank}
      </div>

      {/* Outcome name */}
      <div
        className="text-sm font-medium mb-3 line-clamp-2 pr-16"
        style={{ color: fey.grey100 }}
      >
        {option.name}
      </div>

      {/* Probability bar */}
      <div className="space-y-1.5">
        <div className="flex items-center justify-between">
          <span
            className="text-xl font-mono font-bold"
            style={{ color, letterSpacing: "-0.02em" }}
          >
            {formatPercent(option.yes_price)}
          </span>
          <span
            className="text-xs font-mono"
            style={{ color: fey.grey500 }}
          >
            {formatPrice(option.yes_price)}
          </span>
        </div>

        {/* Visual bar */}
        <div
          className="h-1 rounded-full overflow-hidden"
          style={{ backgroundColor: `${fey.grey500}20` }}
        >
          <motion.div
            className="h-full rounded-full"
            style={{ backgroundColor: color }}
            initial={{ width: 0 }}
            animate={{ width: `${probability * 100}%` }}
            transition={{ duration: 0.5, ease: "easeOut" }}
          />
        </div>
      </div>
    </motion.button>
  );
};

// ============================================================================
// Main Component
// ============================================================================

export const OutcomeSummaryGrid = ({
  options,
  maxDisplay = 6,
  onOutcomeClick,
  className = "",
}: OutcomeSummaryGridProps) => {
  // Sort by price (highest first) and take top N
  const sortedOptions = [...options]
    .sort((a, b) => parseFloat(b.yes_price) - parseFloat(a.yes_price))
    .slice(0, maxDisplay);

  const leadingId = sortedOptions[0]?.market_id;
  const remainingCount = options.length - maxDisplay;

  return (
    <div className={className}>
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h3
          className="text-sm font-medium"
          style={{ color: fey.grey100 }}
        >
          Top Outcomes
        </h3>
        <span
          className="text-xs"
          style={{ color: fey.grey500 }}
        >
          {options.length} total
        </span>
      </div>

      {/* Grid */}
      <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
        {sortedOptions.map((option, index) => (
          <OutcomeCard
            key={option.market_id}
            option={option}
            isLeading={option.market_id === leadingId}
            rank={index + 1}
            onClick={() => onOutcomeClick?.(option)}
          />
        ))}
      </div>

      {/* More outcomes indicator */}
      {remainingCount > 0 && (
        <div
          className="mt-3 text-center py-2 rounded-lg"
          style={{
            backgroundColor: fey.bg300,
            border: `1px solid ${fey.border}`,
          }}
        >
          <span
            className="text-xs"
            style={{ color: fey.grey500 }}
          >
            +{remainingCount} more outcome{remainingCount !== 1 ? "s" : ""}
          </span>
        </div>
      )}
    </div>
  );
};

export default OutcomeSummaryGrid;
