"use client";

import { useMemo } from "react";
import { Clock, Copy, ExternalLink, Check, Crown } from "lucide-react";
import { useState } from "react";
import type { PredictionMarket } from "@/lib/types";

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

interface MarketInfoPanelProps {
  market: PredictionMarket;
  yesPrice: string;
  noPrice: string;
  spread: number | null;
  className?: string;
  // Multi-outcome specific props
  outcomeName?: string;
  isLeading?: boolean;
  outcomeCount?: number;
}

const formatTimeRemaining = (dateStr: string | null): string => {
  if (!dateStr) return "—";
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = date.getTime() - now.getTime();

  if (diffMs < 0) return "Ended";

  const days = Math.floor(diffMs / (1000 * 60 * 60 * 24));
  const hours = Math.floor((diffMs % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
  const minutes = Math.floor((diffMs % (1000 * 60 * 60)) / (1000 * 60));

  if (days > 30) return `${Math.floor(days / 30)}mo`;
  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h ${minutes}m`;
  return `${minutes}m`;
};

const formatVolume = (volume: string | null): string => {
  if (!volume) return "—";
  const num = parseFloat(volume);
  if (num >= 1_000_000) return `$${(num / 1_000_000).toFixed(2)}M`;
  if (num >= 1_000) return `$${(num / 1_000).toFixed(1)}K`;
  return `$${num.toFixed(2)}`;
};

const StatCard = ({
  label,
  value,
  valueColor = fey.grey100,
}: {
  label: string;
  value: string;
  valueColor?: string;
}) => (
  <div
    className="p-3 rounded-lg text-center"
    style={{ backgroundColor: fey.bg400 }}
  >
    <div className="text-[10px] uppercase tracking-wider mb-1" style={{ color: fey.grey500 }}>
      {label}
    </div>
    <div className="text-sm font-mono font-semibold" style={{ color: valueColor }}>
      {value}
    </div>
  </div>
);

const truncateId = (id: string): string => {
  if (id.length <= 10) return id;
  return `${id.slice(0, 6)}...`;
};

export const MarketInfoPanel = ({
  market,
  yesPrice,
  noPrice,
  spread,
  className = "",
  outcomeName,
  isLeading,
  outcomeCount,
}: MarketInfoPanelProps) => {
  const [copied, setCopied] = useState(false);
  const isMultiOutcome = !!outcomeName;

  const handleCopyId = () => {
    navigator.clipboard.writeText(market.id);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const yesPct = (parseFloat(yesPrice) * 100).toFixed(1);
  const noPct = (parseFloat(noPrice) * 100).toFixed(1);
  const spreadCents = spread !== null ? (spread * 100).toFixed(1) : "—";

  return (
    <div
      className={`rounded-lg overflow-hidden flex flex-col ${className}`}
      style={{
        backgroundColor: fey.bg300,
        border: `1px solid ${fey.border}`,
      }}
    >
      {/* Header */}
      <div
        className="px-4 py-3"
        style={{ borderBottom: `1px solid ${fey.border}` }}
      >
        <span
          className="text-sm font-semibold"
          style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
        >
          {isMultiOutcome ? "Outcome Info" : "Market Info"}
        </span>
      </div>

      {/* Outcome Name (for multi-outcome markets) */}
      {isMultiOutcome && (
        <div className="px-4 pt-4 pb-2">
          <div className="flex items-center gap-2">
            <span
              className="text-sm font-medium truncate"
              style={{ color: fey.grey100 }}
            >
              {outcomeName}
            </span>
            {isLeading && (
              <span
                className="flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase tracking-wider shrink-0"
                style={{ backgroundColor: `${fey.teal}20`, color: fey.teal }}
              >
                <Crown className="w-2.5 h-2.5" />
                Leading
              </span>
            )}
          </div>
        </div>
      )}

      {/* Price Display */}
      <div className="p-4" style={{ borderBottom: `1px solid ${fey.border}` }}>
        {isMultiOutcome ? (
          // Single price for multi-outcome
          <div>
            <div className="text-[10px] uppercase tracking-wider mb-1" style={{ color: fey.grey500 }}>
              Price
            </div>
            <div className="text-2xl font-bold font-mono" style={{ color: fey.teal }}>
              {yesPct}¢
            </div>
          </div>
        ) : (
          // YES/NO prices for binary
          <div className="grid grid-cols-2 gap-4">
            <div>
              <div className="text-[10px] uppercase tracking-wider mb-1" style={{ color: fey.grey500 }}>
                YES
              </div>
              <div className="text-2xl font-bold font-mono" style={{ color: fey.teal }}>
                {yesPct}¢
              </div>
            </div>
            <div>
              <div className="text-[10px] uppercase tracking-wider mb-1" style={{ color: fey.grey500 }}>
                NO
              </div>
              <div className="text-2xl font-bold font-mono" style={{ color: fey.red }}>
                {noPct}¢
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Stats Grid */}
      <div className="p-4 flex-1" style={{ borderBottom: `1px solid ${fey.border}` }}>
        <div className="grid grid-cols-3 gap-2">
          <StatCard label="Spread" value={`${spreadCents}¢`} />
          <StatCard label="24h Vol" value={formatVolume(market.volume)} />
          <StatCard label="Liquidity" value={formatVolume(market.liquidity)} />
          <StatCard
            label="Time"
            value={formatTimeRemaining(market.close_time)}
            valueColor={fey.skyBlue}
          />
          <StatCard
            label="Outcomes"
            value={outcomeCount ? outcomeCount.toString() : "2"}
          />
          <button
            onClick={handleCopyId}
            className="p-3 rounded-lg text-center transition-colors hover:opacity-80"
            style={{ backgroundColor: fey.bg400 }}
          >
            <div className="text-[10px] uppercase tracking-wider mb-1" style={{ color: fey.grey500 }}>
              Market ID
            </div>
            <div className="text-sm font-mono font-semibold flex items-center justify-center gap-1" style={{ color: fey.grey300 }}>
              {truncateId(market.id)}
              {copied ? (
                <Check className="h-3 w-3" style={{ color: fey.teal }} />
              ) : (
                <Copy className="h-3 w-3" style={{ color: fey.grey500 }} />
              )}
            </div>
          </button>
        </div>
      </div>

      {/* Resolution Source */}
      {market.resolution_source && (
        <div className="p-4" style={{ borderBottom: `1px solid ${fey.border}` }}>
          <div className="text-[10px] uppercase tracking-wider mb-2" style={{ color: fey.grey500 }}>
            Resolution Source
          </div>
          <p className="text-xs leading-relaxed" style={{ color: fey.grey300 }}>
            {market.resolution_source}
          </p>
        </div>
      )}

      {/* External Link */}
      {market.url && (
        <div className="p-4">
          <a
            href={market.url}
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center justify-center gap-2 py-2 rounded-lg text-sm font-medium transition-colors hover:opacity-80"
            style={{ backgroundColor: fey.bg400, color: fey.skyBlue }}
          >
            <ExternalLink className="h-4 w-4" />
            View on Polymarket
          </a>
        </div>
      )}
    </div>
  );
};

export default MarketInfoPanel;
