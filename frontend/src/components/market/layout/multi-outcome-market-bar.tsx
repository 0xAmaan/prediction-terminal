"use client";

import { useState, useRef, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { AnimatedNumber } from "@/components/ui/animated-number";
import { marketBarVariants } from "@/lib/motion";
import { ChevronDown, Crown, Zap, Check } from "lucide-react";
import type { MarketOption, Trade } from "@/lib/types";
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
  border: "rgba(255, 255, 255, 0.06)",
  borderHover: "rgba(255, 255, 255, 0.12)",
};

// ============================================================================
// Types
// ============================================================================

export interface MultiOutcomeMarketBarProps {
  options: MarketOption[];
  selectedOutcome?: MarketOption | null;
  onOutcomeSelect?: (outcome: MarketOption) => void;
  volume24h?: string | null;
  lastTrade?: Trade | null;
  connectionState?: ConnectionState;
  latency?: number | null;
}

// ============================================================================
// Helpers
// ============================================================================

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

const getLeadingOutcome = (options: MarketOption[]): MarketOption | null => {
  if (options.length === 0) return null;
  return [...options].sort(
    (a, b) => parseFloat(b.yes_price) - parseFloat(a.yes_price)
  )[0];
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
// Mini Outcome Selector (for market bar)
// ============================================================================

interface MiniOutcomeSelectorProps {
  options: MarketOption[];
  selectedOutcome: MarketOption | null;
  onSelect: (outcome: MarketOption) => void;
}

const MiniOutcomeSelector = ({
  options,
  selectedOutcome,
  onSelect,
}: MiniOutcomeSelectorProps) => {
  const [isOpen, setIsOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  const leadingOutcome = getLeadingOutcome(options);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        containerRef.current &&
        !containerRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const handleSelect = (option: MarketOption) => {
    onSelect(option);
    setIsOpen(false);
  };

  return (
    <div ref={containerRef} className="relative">
      <button
        className="flex items-center gap-1.5 px-2 py-1 rounded transition-colors"
        style={{
          backgroundColor: fey.bg300,
          border: `1px solid ${isOpen ? fey.borderHover : fey.border}`,
        }}
        onClick={() => setIsOpen(!isOpen)}
      >
        <span
          className="text-xs font-medium max-w-[120px] truncate"
          style={{ color: fey.grey300 }}
        >
          {selectedOutcome?.name ?? "Select..."}
        </span>
        <ChevronDown
          className={`w-3 h-3 transition-transform ${isOpen ? "rotate-180" : ""}`}
          style={{ color: fey.grey500 }}
        />
      </button>

      <AnimatePresence>
        {isOpen && (
          <motion.div
            className="absolute bottom-full mb-2 left-0 min-w-[200px] max-w-[280px] rounded-lg overflow-hidden shadow-xl z-50"
            style={{
              backgroundColor: fey.bg200,
              border: `1px solid ${fey.border}`,
            }}
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 8 }}
            transition={{ duration: 0.15 }}
          >
            <div
              className="px-3 py-2 border-b"
              style={{ borderColor: fey.border }}
            >
              <span
                className="text-[10px] uppercase tracking-wider font-medium"
                style={{ color: fey.grey500 }}
              >
                Switch Outcome
              </span>
            </div>
            <div className="max-h-48 overflow-y-auto py-1">
              {[...options]
                .sort((a, b) => parseFloat(b.yes_price) - parseFloat(a.yes_price))
                .map((option) => {
                  const isSelected = selectedOutcome?.market_id === option.market_id;
                  const isLeading = leadingOutcome?.market_id === option.market_id;

                  return (
                    <button
                      key={option.market_id}
                      className="w-full flex items-center gap-2 px-3 py-2 text-left transition-colors"
                      style={{
                        backgroundColor: isSelected ? `${fey.teal}10` : "transparent",
                      }}
                      onClick={() => handleSelect(option)}
                      onMouseEnter={(e) => {
                        if (!isSelected) {
                          e.currentTarget.style.backgroundColor = fey.bg300;
                        }
                      }}
                      onMouseLeave={(e) => {
                        if (!isSelected) {
                          e.currentTarget.style.backgroundColor = "transparent";
                        }
                      }}
                    >
                      <div
                        className="w-3 h-3 rounded-full flex items-center justify-center flex-shrink-0"
                        style={{
                          backgroundColor: isSelected ? fey.teal : "transparent",
                          border: isSelected ? "none" : `1.5px solid ${fey.grey500}`,
                        }}
                      >
                        {isSelected && <Check className="w-2 h-2 text-black" />}
                      </div>
                      <span
                        className="text-xs font-medium truncate flex-1"
                        style={{ color: fey.grey100 }}
                      >
                        {option.name}
                      </span>
                      {isLeading && (
                        <Crown className="w-3 h-3 flex-shrink-0" style={{ color: fey.teal }} />
                      )}
                      <span
                        className="text-xs font-mono flex-shrink-0"
                        style={{ color: fey.teal }}
                      >
                        {(parseFloat(option.yes_price) * 100).toFixed(0)}%
                      </span>
                    </button>
                  );
                })}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};

// ============================================================================
// Main Component
// ============================================================================

export const MultiOutcomeMarketBar = ({
  options,
  selectedOutcome,
  onOutcomeSelect,
  volume24h,
  lastTrade,
  connectionState = "connected",
  latency,
}: MultiOutcomeMarketBarProps) => {
  const leadingOutcome = getLeadingOutcome(options);
  const displayOutcome = selectedOutcome ?? leadingOutcome;
  const price = displayOutcome ? parseFloat(displayOutcome.yes_price) : 0;
  const isLeading = displayOutcome?.market_id === leadingOutcome?.market_id;

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
          {/* Left side: Leading/Selected outcome */}
          <div className="flex items-center gap-4 lg:gap-6 flex-shrink-0">
            {/* Outcome badge and price */}
            <div className="flex items-center gap-3">
              <div className="flex items-center gap-2">
                {isLeading && (
                  <span
                    className="flex items-center gap-1 text-[10px] uppercase tracking-wider font-semibold px-1.5 py-0.5 rounded"
                    style={{
                      backgroundColor: `${fey.teal}20`,
                      color: fey.teal,
                    }}
                  >
                    <Crown className="w-2.5 h-2.5" />
                    Leading
                  </span>
                )}
                <span
                  className="text-sm font-medium max-w-[180px] truncate"
                  style={{ color: fey.grey100 }}
                >
                  {displayOutcome?.name ?? "—"}
                </span>
              </div>
              <AnimatedNumber
                value={price}
                format="price"
                decimals={1}
                colorByChange
                className="text-lg font-bold"
                style={{ color: fey.teal }}
              />
            </div>

            {/* Divider */}
            <div className="h-6 w-px" style={{ backgroundColor: fey.border }} />

            {/* Mini outcome selector */}
            {onOutcomeSelect && (
              <MiniOutcomeSelector
                options={options}
                selectedOutcome={displayOutcome}
                onSelect={onOutcomeSelect}
              />
            )}

            {/* Outcomes count - hide on small screens */}
            <div className="hidden md:block">
              <StatItem
                label="Outcomes"
                value={options.length.toString()}
              />
            </div>
          </div>

          {/* Right side: Volume and activity */}
          <div className="flex items-center gap-4 lg:gap-6 flex-shrink-0">
            {/* Volume */}
            <StatItem label="Volume" value={formatVolume(volume24h)} />

            {/* Last Trade - hide on small screens */}
            {lastTrade && (
              <div className="hidden lg:flex items-center gap-2">
                <Zap className="h-3 w-3" style={{ color: fey.teal }} />
                <span className="font-mono text-sm" style={{ color: fey.grey300 }}>
                  {(parseFloat(lastTrade.price) * 100).toFixed(1)}¢
                </span>
                {lastTrade.outcome_name && (
                  <span className="text-xs truncate max-w-[100px]" style={{ color: fey.grey500 }}>
                    ({lastTrade.outcome_name})
                  </span>
                )}
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

export default MultiOutcomeMarketBar;
