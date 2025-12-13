"use client";

import Link from "next/link";
import { ArrowLeft, ExternalLink, Clock, Crown } from "lucide-react";
import type { PredictionMarket, MarketOption } from "@/lib/types";
import { OutcomeSelector } from "@/components/market/shared/outcome-selector";

// Fey color tokens
const fey = {
  bg100: "#070709",
  bg200: "#101116",
  bg300: "#131419",
  bg400: "#16181C",
  grey100: "#EEF0F1",
  grey300: "#B6BEC4",
  grey500: "#7D8B96",
  teal: "#4DBE95",
  red: "#D84F68",
  skyBlue: "#54BBF7",
  border: "rgba(255, 255, 255, 0.06)",
};

interface MultiOutcomeTradingHeaderProps {
  market: PredictionMarket;
  options: MarketOption[];
  selectedOutcome: MarketOption | null;
  onOutcomeSelect: (outcome: MarketOption) => void;
  isLeading: boolean;
  className?: string;
}

const formatVolume = (volume: string | null): string => {
  if (!volume) return "—";
  const num = parseFloat(volume);
  if (num >= 1_000_000) return `$${(num / 1_000_000).toFixed(2)}M`;
  if (num >= 1_000) return `$${(num / 1_000).toFixed(1)}K`;
  return `$${num.toFixed(2)}`;
};

const formatTimeRemaining = (dateStr: string | null): string => {
  if (!dateStr) return "—";
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = date.getTime() - now.getTime();

  if (diffMs < 0) return "Ended";

  const days = Math.floor(diffMs / (1000 * 60 * 60 * 24));
  const hours = Math.floor((diffMs % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));

  if (days > 30) return `${Math.floor(days / 30)} months`;
  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h`;
  return "< 1h";
};

const StatBadge = ({
  label,
  value,
  valueColor = fey.grey100,
}: {
  label: string;
  value: string;
  valueColor?: string;
}) => (
  <div className="flex items-center gap-2">
    <span className="text-xs" style={{ color: fey.grey500 }}>
      {label}
    </span>
    <span className="text-sm font-mono font-semibold" style={{ color: valueColor }}>
      {value}
    </span>
  </div>
);

export const MultiOutcomeTradingHeader = ({
  market,
  options,
  selectedOutcome,
  onOutcomeSelect,
  isLeading,
  className = "",
}: MultiOutcomeTradingHeaderProps) => {
  const currentPrice = selectedOutcome
    ? (parseFloat(selectedOutcome.yes_price) * 100).toFixed(1)
    : "—";

  return (
    <header
      className={`px-6 py-3 flex items-center gap-4 ${className}`}
      style={{
        backgroundColor: fey.bg100,
        borderBottom: `1px solid ${fey.border}`,
      }}
    >
      {/* Back Button */}
      <Link
        href="/"
        className="p-2 rounded-lg transition-colors hover:bg-white/5"
        style={{ color: fey.grey500 }}
      >
        <ArrowLeft className="h-5 w-5" />
      </Link>

      {/* Outcome Selector */}
      <div className="flex items-center gap-3 flex-1 min-w-0">
        {isLeading && (
          <span
            className="flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase tracking-wider shrink-0"
            style={{ backgroundColor: `${fey.teal}20`, color: fey.teal }}
          >
            <Crown className="w-2.5 h-2.5" />
            Leading
          </span>
        )}
        <div className="max-w-xs">
          <OutcomeSelector
            options={options}
            selectedOutcome={selectedOutcome}
            onSelect={onOutcomeSelect}
            variant="mini"
          />
        </div>
        <span className="text-xs shrink-0" style={{ color: fey.grey500 }}>
          {options.length} outcomes
        </span>
      </div>

      {/* Stats Row */}
      <div className="flex items-center gap-6 shrink-0">
        <StatBadge label="Price" value={`${currentPrice}¢`} valueColor={fey.teal} />
        <div className="w-px h-4" style={{ backgroundColor: fey.border }} />
        <StatBadge label="Volume" value={formatVolume(market.volume)} />
        <div className="w-px h-4" style={{ backgroundColor: fey.border }} />
        <StatBadge label="Liquidity" value={formatVolume(market.liquidity)} />
        <div className="w-px h-4" style={{ backgroundColor: fey.border }} />
        <div className="flex items-center gap-2">
          <Clock className="h-3.5 w-3.5" style={{ color: fey.grey500 }} />
          <span className="text-sm font-mono" style={{ color: fey.skyBlue }}>
            {formatTimeRemaining(market.close_time)}
          </span>
        </div>
      </div>

      {/* External Link */}
      {market.url && (
        <a
          href={market.url}
          target="_blank"
          rel="noopener noreferrer"
          className="p-2 rounded-lg transition-colors hover:bg-white/5 shrink-0"
          style={{ color: fey.grey500, border: `1px solid ${fey.border}` }}
        >
          <ExternalLink className="h-4 w-4" />
        </a>
      )}
    </header>
  );
};

export default MultiOutcomeTradingHeader;
