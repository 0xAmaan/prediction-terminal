"use client";

import { cn } from "@/lib/utils";
import { Clock, TrendingUp, ExternalLink, Zap } from "lucide-react";

// Extended market type for demo (includes sports fields we'll add to backend later)
export interface DemoMarket {
  id: string;
  platform: "kalshi" | "polymarket";
  title: string;
  yes_price: string;
  no_price: string;
  volume: string;
  close_time: string | null;
  image_url: string | null;
  category: string | null;
  // Multi-outcome
  is_multi_outcome: boolean;
  outcome_count: number | null;
  leading_outcome: string | null;
  options?: Array<{ name: string; yes_price: string; market_id: string }>;
  // Sports
  is_sports?: boolean;
  is_live?: boolean;
  score?: string;
  game_period?: string;
  home_team?: string;
  away_team?: string;
  home_odds?: string;
  away_odds?: string;
  spread_line?: string;
  total_line?: string;
}

// ============================================================================
// Utility Functions
// ============================================================================

const formatPrice = (price: string): string => {
  const num = parseFloat(price);
  if (isNaN(num)) return "—";
  return `${Math.round(num * 100)}¢`;
};

const formatPercent = (price: string): string => {
  const num = parseFloat(price);
  if (isNaN(num)) return "—";
  return `${Math.round(num * 100)}%`;
};

const formatVolume = (volume: string): string => {
  const num = parseFloat(volume);
  if (isNaN(num) || num === 0) return "—";
  if (num >= 1_000_000) return `$${(num / 1_000_000).toFixed(1)}M`;
  if (num >= 1_000) return `$${(num / 1_000).toFixed(0)}K`;
  return `$${num.toFixed(0)}`;
};

const formatCloseTime = (closeTime: string | null): string => {
  if (!closeTime) return "—";
  const date = new Date(closeTime);
  const now = new Date();
  const diffMs = date.getTime() - now.getTime();
  const diffDays = Math.ceil(diffMs / (1000 * 60 * 60 * 24));

  if (diffDays < 0) return "Ended";
  if (diffDays === 0) return "Today";
  if (diffDays === 1) return "1d";
  if (diffDays < 7) return `${diffDays}d`;
  if (diffDays < 30) return `${Math.floor(diffDays / 7)}w`;
  return `${Math.floor(diffDays / 30)}mo`;
};

// ============================================================================
// Binary Market Card
// ============================================================================

const BinaryCard = ({ market }: { market: DemoMarket }) => {
  const yesNum = parseFloat(market.yes_price);
  const noNum = parseFloat(market.no_price);

  return (
    <div className="group relative overflow-hidden rounded-xl bg-card border border-border/50 hover:border-border transition-all duration-300 hover:shadow-lg hover:shadow-black/20">
      {/* Platform indicator line */}
      <div
        className={cn(
          "absolute top-0 left-0 right-0 h-[2px]",
          market.platform === "kalshi" ? "bg-[#22c55e]" : "bg-[#3b82f6]"
        )}
      />

      <div className="p-4">
        {/* Header with image and title */}
        <div className="flex gap-3 mb-4">
          {market.image_url ? (
            <img
              src={market.image_url}
              alt=""
              className="w-12 h-12 rounded-lg object-cover shrink-0"
            />
          ) : (
            <div
              className={cn(
                "w-12 h-12 rounded-lg shrink-0 flex items-center justify-center text-lg font-bold",
                market.platform === "kalshi"
                  ? "bg-[#22c55e]/20 text-[#22c55e]"
                  : "bg-[#3b82f6]/20 text-[#3b82f6]"
              )}
            >
              {market.platform === "kalshi" ? "K" : "P"}
            </div>
          )}
          <div className="flex-1 min-w-0">
            <h3 className="font-medium text-sm leading-tight line-clamp-2 text-foreground group-hover:text-white transition-colors">
              {market.title}
            </h3>
            {market.category && (
              <span className="text-xs text-muted-foreground mt-1 block">
                {market.category}
              </span>
            )}
          </div>
        </div>

        {/* Yes/No Buttons */}
        <div className="grid grid-cols-2 gap-2 mb-3">
          <button
            className={cn(
              "relative py-2.5 px-3 rounded-lg font-mono text-sm font-semibold transition-all duration-200",
              "bg-[#22c55e]/10 hover:bg-[#22c55e]/20 border border-[#22c55e]/30 hover:border-[#22c55e]/50",
              "text-[#22c55e] hover:text-[#4ade80]"
            )}
          >
            <span className="text-xs text-[#22c55e]/70 block mb-0.5">Yes</span>
            <span className="text-base">{formatPercent(market.yes_price)}</span>
          </button>
          <button
            className={cn(
              "relative py-2.5 px-3 rounded-lg font-mono text-sm font-semibold transition-all duration-200",
              "bg-[#ef4444]/10 hover:bg-[#ef4444]/20 border border-[#ef4444]/30 hover:border-[#ef4444]/50",
              "text-[#ef4444] hover:text-[#f87171]"
            )}
          >
            <span className="text-xs text-[#ef4444]/70 block mb-0.5">No</span>
            <span className="text-base">{formatPercent(market.no_price)}</span>
          </button>
        </div>

        {/* Footer stats */}
        <div className="flex items-center justify-between text-xs text-muted-foreground">
          <div className="flex items-center gap-1">
            <TrendingUp className="w-3 h-3" />
            <span>{formatVolume(market.volume)} Vol.</span>
          </div>
          <div className="flex items-center gap-1">
            <Clock className="w-3 h-3" />
            <span>{formatCloseTime(market.close_time)}</span>
          </div>
        </div>
      </div>
    </div>
  );
};

// ============================================================================
// Multi-Outcome Card
// ============================================================================

const MultiOutcomeCard = ({ market }: { market: DemoMarket }) => {
  const options = market.options || [];
  const displayOptions = options.slice(0, 4);
  const remainingCount = options.length - 4;

  return (
    <div className="group relative overflow-hidden rounded-xl bg-card border border-border/50 hover:border-border transition-all duration-300 hover:shadow-lg hover:shadow-black/20">
      {/* Platform indicator */}
      <div
        className={cn(
          "absolute top-0 left-0 right-0 h-[2px]",
          market.platform === "kalshi" ? "bg-[#22c55e]" : "bg-[#3b82f6]"
        )}
      />

      <div className="p-4">
        {/* Header */}
        <div className="flex gap-3 mb-4">
          {market.image_url ? (
            <img
              src={market.image_url}
              alt=""
              className="w-12 h-12 rounded-lg object-cover shrink-0"
            />
          ) : (
            <div className="w-12 h-12 rounded-lg shrink-0 bg-gradient-to-br from-violet-500/20 to-fuchsia-500/20 flex items-center justify-center">
              <span className="text-violet-400 font-bold">{market.outcome_count}</span>
            </div>
          )}
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 mb-1">
              <h3 className="font-medium text-sm leading-tight line-clamp-1 text-foreground group-hover:text-white transition-colors">
                {market.title}
              </h3>
              <span className="shrink-0 px-1.5 py-0.5 rounded text-[10px] font-medium bg-violet-500/20 text-violet-400 border border-violet-500/30">
                {market.outcome_count}
              </span>
            </div>
            {market.category && (
              <span className="text-xs text-muted-foreground">{market.category}</span>
            )}
          </div>
        </div>

        {/* Options list */}
        <div className="space-y-2 mb-3">
          {displayOptions.map((option, idx) => {
            const prob = parseFloat(option.yes_price) * 100;
            const isLeading = option.name === market.leading_outcome;

            return (
              <div
                key={option.market_id || idx}
                className={cn(
                  "relative rounded-lg overflow-hidden transition-all duration-200",
                  isLeading ? "ring-1 ring-violet-500/50" : ""
                )}
              >
                {/* Background bar */}
                <div
                  className={cn(
                    "absolute inset-0 transition-all duration-500",
                    isLeading
                      ? "bg-gradient-to-r from-violet-500/30 to-violet-500/10"
                      : "bg-secondary/50"
                  )}
                  style={{ width: `${Math.max(prob, 5)}%` }}
                />
                {/* Content */}
                <div className="relative flex items-center justify-between px-3 py-2">
                  <span
                    className={cn(
                      "text-sm truncate",
                      isLeading ? "text-foreground font-medium" : "text-muted-foreground"
                    )}
                  >
                    {option.name}
                  </span>
                  <span
                    className={cn(
                      "font-mono text-sm shrink-0 ml-2",
                      isLeading ? "text-violet-400 font-semibold" : "text-muted-foreground"
                    )}
                  >
                    {formatPercent(option.yes_price)}
                  </span>
                </div>
              </div>
            );
          })}
        </div>

        {/* More options indicator */}
        {remainingCount > 0 && (
          <div className="text-center mb-3">
            <span className="text-xs text-muted-foreground">+{remainingCount} more options</span>
          </div>
        )}

        {/* Footer */}
        <div className="flex items-center justify-between text-xs text-muted-foreground pt-2 border-t border-border/50">
          <div className="flex items-center gap-1">
            <TrendingUp className="w-3 h-3" />
            <span>{formatVolume(market.volume)}</span>
          </div>
          <div className="flex items-center gap-1">
            <Clock className="w-3 h-3" />
            <span>{formatCloseTime(market.close_time)}</span>
          </div>
        </div>
      </div>
    </div>
  );
};

// ============================================================================
// Sports Card
// ============================================================================

const SportsCard = ({ market }: { market: DemoMarket }) => {
  const homeOdds = parseFloat(market.home_odds || "0.5");
  const awayOdds = parseFloat(market.away_odds || "0.5");

  return (
    <div className="group relative overflow-hidden rounded-xl bg-card border border-border/50 hover:border-border transition-all duration-300 hover:shadow-lg hover:shadow-black/20">
      {/* Live indicator bar */}
      {market.is_live && (
        <div className="absolute top-0 left-0 right-0 h-[2px] bg-gradient-to-r from-red-500 via-orange-500 to-red-500 animate-pulse" />
      )}

      <div className="p-4">
        {/* Live badge and category */}
        <div className="flex items-center justify-between mb-3">
          <div className="flex items-center gap-2">
            {market.is_live ? (
              <span className="flex items-center gap-1.5 px-2 py-1 rounded-full bg-red-500/20 border border-red-500/30">
                <span className="w-1.5 h-1.5 rounded-full bg-red-500 animate-pulse" />
                <span className="text-[10px] font-bold text-red-400 uppercase tracking-wider">
                  Live
                </span>
              </span>
            ) : (
              <span className="text-xs text-muted-foreground">
                {formatCloseTime(market.close_time)}
              </span>
            )}
            {market.game_period && market.is_live && (
              <span className="text-xs text-muted-foreground">{market.game_period}</span>
            )}
          </div>
          <span className="text-xs text-muted-foreground uppercase tracking-wider">
            {market.category}
          </span>
        </div>

        {/* Score display (if live) */}
        {market.is_live && market.score && (
          <div className="text-center mb-4">
            <span className="font-mono text-2xl font-bold text-foreground tracking-wider">
              {market.score}
            </span>
          </div>
        )}

        {/* Teams and odds */}
        <div className="space-y-2">
          {/* Home team */}
          <button
            className={cn(
              "w-full flex items-center justify-between p-3 rounded-lg transition-all duration-200",
              "bg-[#22c55e]/5 hover:bg-[#22c55e]/15 border border-[#22c55e]/20 hover:border-[#22c55e]/40"
            )}
          >
            <div className="flex items-center gap-3">
              <div className="w-8 h-8 rounded-full bg-[#22c55e]/20 flex items-center justify-center text-xs font-bold text-[#22c55e]">
                {market.home_team?.slice(0, 3).toUpperCase()}
              </div>
              <span className="font-medium text-sm text-foreground">{market.home_team}</span>
            </div>
            <span className="font-mono text-lg font-bold text-[#22c55e]">
              {formatPercent(market.home_odds || "0.5")}
            </span>
          </button>

          {/* Away team */}
          <button
            className={cn(
              "w-full flex items-center justify-between p-3 rounded-lg transition-all duration-200",
              "bg-[#ef4444]/5 hover:bg-[#ef4444]/15 border border-[#ef4444]/20 hover:border-[#ef4444]/40"
            )}
          >
            <div className="flex items-center gap-3">
              <div className="w-8 h-8 rounded-full bg-[#ef4444]/20 flex items-center justify-center text-xs font-bold text-[#ef4444]">
                {market.away_team?.slice(0, 3).toUpperCase()}
              </div>
              <span className="font-medium text-sm text-foreground">{market.away_team}</span>
            </div>
            <span className="font-mono text-lg font-bold text-[#ef4444]">
              {formatPercent(market.away_odds || "0.5")}
            </span>
          </button>
        </div>

        {/* Spread and totals (if available) */}
        {(market.spread_line || market.total_line) && (
          <div className="grid grid-cols-2 gap-2 mt-3">
            {market.spread_line && (
              <div className="text-center p-2 rounded-lg bg-secondary/30 border border-border/30">
                <span className="text-[10px] text-muted-foreground uppercase tracking-wider block mb-0.5">
                  Spread
                </span>
                <span className="font-mono text-sm text-foreground">{market.spread_line}</span>
              </div>
            )}
            {market.total_line && (
              <div className="text-center p-2 rounded-lg bg-secondary/30 border border-border/30">
                <span className="text-[10px] text-muted-foreground uppercase tracking-wider block mb-0.5">
                  Total
                </span>
                <span className="font-mono text-sm text-foreground">O/U {market.total_line}</span>
              </div>
            )}
          </div>
        )}

        {/* Footer */}
        <div className="flex items-center justify-between text-xs text-muted-foreground mt-3 pt-3 border-t border-border/30">
          <div className="flex items-center gap-1">
            <TrendingUp className="w-3 h-3" />
            <span>{formatVolume(market.volume)}</span>
          </div>
          <span className="text-[10px] uppercase tracking-wider">
            {market.platform}
          </span>
        </div>
      </div>
    </div>
  );
};

// ============================================================================
// Main MarketCard Component
// ============================================================================

export const MarketCard = ({ market }: { market: DemoMarket }) => {
  if (market.is_sports) {
    return <SportsCard market={market} />;
  }
  if (market.is_multi_outcome && market.options && market.options.length > 0) {
    return <MultiOutcomeCard market={market} />;
  }
  return <BinaryCard market={market} />;
};

// Export individual cards for direct use
export { BinaryCard, MultiOutcomeCard, SportsCard };
