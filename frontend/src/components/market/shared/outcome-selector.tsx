"use client";

import { useState, useRef, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { ChevronDown, Check, TrendingUp, Crown } from "lucide-react";
import type { MarketOption } from "@/lib/types";

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

export interface OutcomeSelectorProps {
  options: MarketOption[];
  selectedOutcome: MarketOption | null;
  onSelect: (outcome: MarketOption) => void;
  variant?: "full" | "mini";
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

const getLeadingOutcome = (options: MarketOption[]): MarketOption | null => {
  if (options.length === 0) return null;
  return [...options].sort(
    (a, b) => parseFloat(b.yes_price) - parseFloat(a.yes_price)
  )[0];
};

// ============================================================================
// Outcome Option Row
// ============================================================================

interface OptionRowProps {
  option: MarketOption;
  isSelected: boolean;
  isLeading: boolean;
  onClick: () => void;
  variant: "full" | "mini";
}

const OptionRow = ({
  option,
  isSelected,
  isLeading,
  onClick,
  variant,
}: OptionRowProps) => (
  <motion.button
    className="w-full flex items-center gap-3 px-3 py-2.5 text-left transition-colors rounded-md"
    style={{
      backgroundColor: isSelected ? `${fey.teal}10` : "transparent",
    }}
    onClick={onClick}
    whileHover={{ backgroundColor: isSelected ? `${fey.teal}15` : `${fey.bg300}` }}
  >
    {/* Selection indicator */}
    <div
      className="w-4 h-4 rounded-full flex items-center justify-center flex-shrink-0"
      style={{
        backgroundColor: isSelected ? fey.teal : "transparent",
        border: isSelected ? "none" : `1.5px solid ${fey.grey500}`,
      }}
    >
      {isSelected && <Check className="w-2.5 h-2.5 text-black" />}
    </div>

    {/* Outcome name */}
    <div className="flex-1 min-w-0">
      <div className="flex items-center gap-2">
        <span
          className={`truncate ${variant === "mini" ? "text-xs" : "text-sm"} font-medium`}
          style={{ color: fey.grey100 }}
        >
          {option.name}
        </span>
        {isLeading && (
          <span
            className="flex items-center gap-0.5 px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase tracking-wider"
            style={{ backgroundColor: `${fey.teal}20`, color: fey.teal }}
          >
            <Crown className="w-2.5 h-2.5" />
            Leading
          </span>
        )}
      </div>
    </div>

    {/* Price */}
    <span
      className={`font-mono ${variant === "mini" ? "text-xs" : "text-sm"} font-medium flex-shrink-0`}
      style={{ color: fey.teal }}
    >
      {formatPrice(option.yes_price)}
    </span>
  </motion.button>
);

// ============================================================================
// Main Component
// ============================================================================

export const OutcomeSelector = ({
  options,
  selectedOutcome,
  onSelect,
  variant = "full",
  className = "",
}: OutcomeSelectorProps) => {
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

  // Close on escape
  useEffect(() => {
    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") setIsOpen(false);
    };

    document.addEventListener("keydown", handleEscape);
    return () => document.removeEventListener("keydown", handleEscape);
  }, []);

  const handleSelect = (option: MarketOption) => {
    onSelect(option);
    setIsOpen(false);
  };

  const isLeading = (option: MarketOption) =>
    leadingOutcome?.market_id === option.market_id;

  return (
    <div ref={containerRef} className={`relative ${className}`}>
      {/* Trigger button */}
      <motion.button
        className={`w-full flex items-center gap-3 rounded-lg transition-colors ${
          variant === "mini" ? "px-2.5 py-1.5" : "px-4 py-3"
        }`}
        style={{
          backgroundColor: fey.bg300,
          border: `1px solid ${isOpen ? fey.borderHover : fey.border}`,
        }}
        onClick={() => setIsOpen(!isOpen)}
        whileHover={{ borderColor: fey.borderHover }}
      >
        {selectedOutcome ? (
          <>
            <div className="flex-1 min-w-0 flex items-center gap-2">
              <span
                className={`truncate ${variant === "mini" ? "text-xs" : "text-sm"} font-medium`}
                style={{ color: fey.grey100 }}
              >
                {selectedOutcome.name}
              </span>
              {isLeading(selectedOutcome) && (
                <Crown className="w-3 h-3 flex-shrink-0" style={{ color: fey.teal }} />
              )}
            </div>
            <span
              className={`font-mono ${variant === "mini" ? "text-xs" : "text-sm"} font-medium flex-shrink-0`}
              style={{ color: fey.teal }}
            >
              {formatPrice(selectedOutcome.yes_price)}
            </span>
          </>
        ) : (
          <span
            className={`flex-1 ${variant === "mini" ? "text-xs" : "text-sm"}`}
            style={{ color: fey.grey500 }}
          >
            Select outcome...
          </span>
        )}
        <ChevronDown
          className={`w-4 h-4 flex-shrink-0 transition-transform ${isOpen ? "rotate-180" : ""}`}
          style={{ color: fey.grey500 }}
        />
      </motion.button>

      {/* Dropdown */}
      <AnimatePresence>
        {isOpen && (
          <motion.div
            className="absolute z-50 w-full mt-2 rounded-lg overflow-hidden shadow-xl"
            style={{
              backgroundColor: fey.bg200,
              border: `1px solid ${fey.border}`,
            }}
            initial={{ opacity: 0, y: -8, scale: 0.96 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -8, scale: 0.96 }}
            transition={{ duration: 0.15, ease: "easeOut" }}
          >
            {/* Header */}
            <div
              className="px-3 py-2 border-b"
              style={{ borderColor: fey.border }}
            >
              <span
                className="text-[10px] uppercase tracking-wider font-medium"
                style={{ color: fey.grey500 }}
              >
                {options.length} Outcomes
              </span>
            </div>

            {/* Options list */}
            <div
              className="max-h-64 overflow-y-auto py-1"
              style={{
                scrollbarWidth: "thin",
                scrollbarColor: `${fey.grey500} transparent`,
              }}
            >
              {[...options]
                .sort((a, b) => parseFloat(b.yes_price) - parseFloat(a.yes_price))
                .map((option) => (
                  <OptionRow
                    key={option.market_id}
                    option={option}
                    isSelected={selectedOutcome?.market_id === option.market_id}
                    isLeading={isLeading(option)}
                    onClick={() => handleSelect(option)}
                    variant={variant}
                  />
                ))}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};

export default OutcomeSelector;
