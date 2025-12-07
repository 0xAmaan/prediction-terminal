"use client";

import { useState } from "react";
import Link from "next/link";
import { ArrowLeft, Zap, Grid3X3, LayoutList, Sparkles } from "lucide-react";

// ============================================================================
// MOCK DATA - Simulating different market types
// ============================================================================

interface MockMarket {
  id: string;
  platform: "kalshi" | "polymarket";
  title: string;
  yes_price: number;
  no_price: number;
  volume: number;
  image_url: string | null;
  close_time: string;
  category: string;
  // Multi-outcome fields
  is_multi_outcome: boolean;
  outcome_count?: number;
  leading_outcome?: string;
  options?: Array<{ name: string; yes_price: number; market_id: string }>;
  // Sports fields
  is_sports: boolean;
  is_live?: boolean;
  score?: string;
  game_period?: string;
  team_a?: string;
  team_b?: string;
  team_a_odds?: number;
  team_b_odds?: number;
}

const MOCK_MARKETS: MockMarket[] = [
  // Binary markets
  {
    id: "binary-1",
    platform: "polymarket",
    title: "Russia x Ukraine ceasefire in 2025?",
    yes_price: 0.06,
    no_price: 0.94,
    volume: 50_000_000,
    image_url: "https://polymarket.com/images/markets/ukraine.png",
    close_time: "2025-12-31T23:59:59Z",
    category: "Politics",
    is_multi_outcome: false,
    is_sports: false,
  },
  {
    id: "binary-2",
    platform: "kalshi",
    title: "Fed decision in December?",
    yes_price: 0.93,
    no_price: 0.07,
    volume: 280_400_000,
    image_url: null,
    close_time: "2025-12-18T18:00:00Z",
    category: "Economics",
    is_multi_outcome: false,
    is_sports: false,
  },
  {
    id: "binary-3",
    platform: "polymarket",
    title: "Bitcoin above $150K by March 2025?",
    yes_price: 0.24,
    no_price: 0.76,
    volume: 15_200_000,
    image_url: null,
    close_time: "2025-03-31T23:59:59Z",
    category: "Crypto",
    is_multi_outcome: false,
    is_sports: false,
  },
  // Multi-outcome markets
  {
    id: "multi-1",
    platform: "polymarket",
    title: "Time 2025 Person of the Year",
    yes_price: 0.41,
    no_price: 0.59,
    volume: 19_000_000,
    image_url: null,
    close_time: "2025-12-15T12:00:00Z",
    category: "Culture",
    is_multi_outcome: true,
    outcome_count: 8,
    leading_outcome: "Artificial Intelligence",
    options: [
      { name: "Artificial Intelligence", yes_price: 0.41, market_id: "m1" },
      { name: "Jensen Huang", yes_price: 0.21, market_id: "m2" },
      { name: "Elon Musk", yes_price: 0.15, market_id: "m3" },
      { name: "Donald Trump", yes_price: 0.12, market_id: "m4" },
      { name: "Sam Altman", yes_price: 0.06, market_id: "m5" },
      { name: "Taylor Swift", yes_price: 0.03, market_id: "m6" },
      { name: "Volodymyr Zelenskyy", yes_price: 0.01, market_id: "m7" },
      { name: "Other", yes_price: 0.01, market_id: "m8" },
    ],
    is_sports: false,
  },
  {
    id: "multi-2",
    platform: "polymarket",
    title: "College Football Champion 2026",
    yes_price: 0.30,
    no_price: 0.70,
    volume: 3_000_000,
    image_url: null,
    close_time: "2026-01-15T23:59:59Z",
    category: "Sports",
    is_multi_outcome: true,
    outcome_count: 12,
    leading_outcome: "Ohio State",
    options: [
      { name: "Ohio State", yes_price: 0.30, market_id: "c1" },
      { name: "Indiana", yes_price: 0.21, market_id: "c2" },
      { name: "Georgia", yes_price: 0.18, market_id: "c3" },
      { name: "Texas", yes_price: 0.12, market_id: "c4" },
    ],
    is_sports: false, // Not live sports
  },
  {
    id: "multi-3",
    platform: "kalshi",
    title: "Democratic Presidential Nominee 2028",
    yes_price: 0.355,
    no_price: 0.645,
    volume: 348_300_000,
    image_url: null,
    close_time: "2028-08-31T23:59:59Z",
    category: "Politics",
    is_multi_outcome: true,
    outcome_count: 128,
    leading_outcome: "Gavin Newsom",
    options: [
      { name: "Gavin Newsom", yes_price: 0.355, market_id: "d1" },
      { name: "Pete Buttigieg", yes_price: 0.12, market_id: "d2" },
      { name: "Alexandria Ocasio-Cortez", yes_price: 0.08, market_id: "d3" },
      { name: "Gretchen Whitmer", yes_price: 0.06, market_id: "d4" },
    ],
    is_sports: false,
  },
  // Sports - Live
  {
    id: "sports-1",
    platform: "polymarket",
    title: "Seahawks vs Falcons",
    yes_price: 0.84,
    no_price: 0.17,
    volume: 4_000_000,
    image_url: null,
    close_time: "2025-12-07T20:00:00Z",
    category: "NFL",
    is_multi_outcome: false,
    is_sports: true,
    is_live: true,
    score: "13 - 6",
    game_period: "Q3",
    team_a: "SEA",
    team_b: "ATL",
    team_a_odds: 0.84,
    team_b_odds: 0.17,
  },
  {
    id: "sports-2",
    platform: "polymarket",
    title: "Colts vs Jaguars",
    yes_price: 0.04,
    no_price: 0.97,
    volume: 4_000_000,
    image_url: null,
    close_time: "2025-12-07T20:00:00Z",
    category: "NFL",
    is_multi_outcome: false,
    is_sports: true,
    is_live: true,
    score: "10 - 28",
    game_period: "Q4",
    team_a: "IND",
    team_b: "JAX",
    team_a_odds: 0.04,
    team_b_odds: 0.97,
  },
  {
    id: "sports-3",
    platform: "polymarket",
    title: "G2 vs Team Falcons",
    yes_price: 0.42,
    no_price: 0.58,
    volume: 720_000,
    image_url: null,
    close_time: "2025-12-07T22:00:00Z",
    category: "Counter Strike",
    is_multi_outcome: false,
    is_sports: true,
    is_live: true,
    score: "0 - 0",
    game_period: "Map 1",
    team_a: "G2",
    team_b: "Team Falcons",
    team_a_odds: 0.42,
    team_b_odds: 0.58,
  },
  {
    id: "sports-4",
    platform: "polymarket",
    title: "Real Madrid vs Celta de Vigo",
    yes_price: 0.78,
    no_price: 0.08,
    volume: 507_000,
    image_url: null,
    close_time: "2025-12-07T21:00:00Z",
    category: "La Liga",
    is_multi_outcome: false,
    is_sports: true,
    is_live: false,
    team_a: "Real Madrid CF",
    team_b: "RC Celta de Vigo",
    team_a_odds: 0.78,
    team_b_odds: 0.08,
  },
  {
    id: "sports-5",
    platform: "polymarket",
    title: "Celtics vs Raptors",
    yes_price: 0.57,
    no_price: 0.43,
    volume: 1_000_000,
    image_url: null,
    close_time: "2025-12-07T19:30:00Z",
    category: "NBA",
    is_multi_outcome: false,
    is_sports: true,
    is_live: false,
    team_a: "BOS",
    team_b: "TOR",
    team_a_odds: 0.57,
    team_b_odds: 0.43,
  },
];

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

const formatPrice = (price: number): string => `${(price * 100).toFixed(0)}%`;
const formatPriceCents = (price: number): string => `${(price * 100).toFixed(1)}¢`;

const formatVolume = (vol: number): string => {
  if (vol >= 1_000_000) return `$${(vol / 1_000_000).toFixed(1)}M`;
  if (vol >= 1_000) return `$${(vol / 1_000).toFixed(0)}K`;
  return `$${vol}`;
};

const formatCloseTime = (dateStr: string): string => {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = date.getTime() - now.getTime();
  const diffDays = Math.ceil(diffMs / (1000 * 60 * 60 * 24));
  if (diffDays < 0) return "Ended";
  if (diffDays === 0) return "Today";
  if (diffDays === 1) return "Tomorrow";
  if (diffDays < 7) return `${diffDays}d`;
  if (diffDays < 30) return `${Math.floor(diffDays / 7)}w`;
  return `${Math.floor(diffDays / 30)}mo`;
};

// ============================================================================
// CARD COMPONENTS
// ============================================================================

// Binary Market Card - Clean Yes/No with prominent action buttons
const BinaryCard = ({ market }: { market: MockMarket }) => {
  const platformColor = market.platform === "kalshi" ? "#22c55e" : "#3b82f6";
  const isHighProb = market.yes_price >= 0.7;
  const isLowProb = market.yes_price <= 0.3;

  return (
    <div
      className="group relative bg-[#1a1a1c] border border-[#2a2a2e] rounded-xl h-[200px] overflow-hidden transition-all duration-300 hover:border-[#3a3a3f] hover:shadow-lg hover:shadow-white/5"
      style={{
        ["--platform-color" as string]: platformColor,
      }}
    >
      {/* Platform accent line */}
      <div
        className="absolute top-0 left-0 right-0 h-[2px] opacity-60 group-hover:opacity-100 transition-opacity"
        style={{ background: platformColor }}
      />

      {/* Content */}
      <div className="p-4">
        {/* Header with category */}
        <div className="flex items-center justify-between mb-2">
          <span className="text-xs font-mono uppercase tracking-wider text-[#888]">
            {market.category}
          </span>
          <span className="text-xs font-mono text-[#888]">
            {formatCloseTime(market.close_time)}
          </span>
        </div>

        {/* Title */}
        <h3 className="text-base font-medium text-white leading-snug mb-3 line-clamp-2">
          {market.title}
        </h3>

        {/* Price Display */}
        <div className="flex items-center gap-3 mb-4">
          <div
            className={`flex-1 text-center py-3 rounded-xl border transition-all ${
              isHighProb
                ? "bg-emerald-500/10 border-emerald-500/30 text-emerald-400"
                : isLowProb
                ? "bg-rose-500/10 border-rose-500/30 text-rose-400"
                : "bg-[#18181b] border-[#27272a] text-[#fafafa]"
            }`}
          >
            <div className="text-2xl font-mono font-bold tracking-tight">
              {formatPrice(market.yes_price)}
            </div>
            <div className="text-[10px] uppercase tracking-wider opacity-60 mt-0.5">
              Yes
            </div>
          </div>
          <div className="flex-1 text-center py-3 rounded-xl bg-[#18181b] border border-[#27272a]">
            <div className="text-2xl font-mono font-bold tracking-tight text-[#888]">
              {formatPrice(market.no_price)}
            </div>
            <div className="text-[10px] uppercase tracking-wider text-[#888] mt-0.5">
              No
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between text-xs text-[#888]">
          <span className="font-mono">{formatVolume(market.volume)} Vol.</span>
          <span
            className="px-2 py-0.5 rounded-full text-[9px] font-medium uppercase tracking-wider"
            style={{
              background: `${platformColor}15`,
              color: platformColor,
            }}
          >
            {market.platform}
          </span>
        </div>
      </div>
    </div>
  );
};

// Multi-Outcome Card - Shows top options with probability bars
const MultiOutcomeCard = ({ market }: { market: MockMarket }) => {
  const platformColor = market.platform === "kalshi" ? "#22c55e" : "#3b82f6";
  const topOptions = market.options?.slice(0, 2) || [];
  const remainingCount = (market.outcome_count || 0) - 2;

  return (
    <div
      className="group relative bg-[#1a1a1c] border border-[#2a2a2e] rounded-xl h-[200px] overflow-hidden transition-all duration-300 hover:border-[#3a3a3f] hover:shadow-lg hover:shadow-white/5"
    >
      {/* Platform accent */}
      <div
        className="absolute top-0 left-0 right-0 h-[2px] opacity-60 group-hover:opacity-100 transition-opacity"
        style={{ background: platformColor }}
      />

      <div className="p-4">
        {/* Header */}
        <div className="flex items-center justify-between mb-2">
          <span className="text-xs font-mono uppercase tracking-wider text-[#888]">
            {market.category}
          </span>
          <div className="flex items-center gap-2">
            <span className="px-1.5 py-0.5 rounded bg-violet-500/20 text-violet-400 text-[10px] font-mono">
              {market.outcome_count} options
            </span>
          </div>
        </div>

        {/* Title */}
        <h3 className="text-base font-medium text-white leading-snug mb-3 line-clamp-2">
          {market.title}
        </h3>

        {/* Options with probability bars */}
        <div className="space-y-2 mb-3">
          {topOptions.map((opt, i) => (
            <div key={opt.market_id} className="relative">
              <div className="flex items-center justify-between relative z-10 py-2 px-3">
                <span
                  className={`text-sm truncate max-w-[180px] ${
                    i === 0 ? "text-white font-medium" : "text-[#a1a1aa]"
                  }`}
                >
                  {opt.name}
                </span>
                <span
                  className={`text-sm font-mono font-medium ${
                    i === 0 ? "text-emerald-400" : "text-[#888]"
                  }`}
                >
                  {formatPrice(opt.yes_price)}
                </span>
              </div>
              {/* Probability bar background */}
              <div
                className={`absolute inset-0 rounded-lg transition-all ${
                  i === 0 ? "bg-emerald-500/10" : "bg-[#18181b]"
                }`}
                style={{
                  clipPath: `inset(0 ${100 - opt.yes_price * 100}% 0 0 round 8px)`,
                }}
              />
              <div className="absolute inset-0 rounded-lg bg-[#18181b] -z-10" />
            </div>
          ))}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between text-xs text-[#888]">
          <span className="font-mono">{formatVolume(market.volume)} Vol.</span>
          {remainingCount > 0 && (
            <span className="text-violet-400">+{remainingCount} more</span>
          )}
        </div>
      </div>
    </div>
  );
};

// Sports Card - Team vs Team with live score
const SportsCard = ({ market }: { market: MockMarket }) => {
  const platformColor = "#3b82f6";
  const scores = market.score?.split(" - ").map((s) => parseInt(s.trim())) || [0, 0];
  const teamALeading = scores[0] > scores[1];
  const teamBLeading = scores[1] > scores[0];

  return (
    <div className="group relative bg-[#1a1a1c] border border-[#2a2a2e] rounded-xl h-[200px] overflow-hidden transition-all duration-300 hover:border-[#3a3a3f] hover:shadow-lg hover:shadow-white/5">
      {/* Live indicator strip */}
      {market.is_live && (
        <div className="absolute top-0 left-0 right-0 h-[2px] bg-gradient-to-r from-cyan-500 via-cyan-400 to-cyan-500 animate-pulse" />
      )}

      <div className="p-4">
        {/* Header */}
        <div className="flex items-center justify-between mb-3">
          <span className="text-xs font-mono uppercase tracking-wider text-[#888]">
            {market.category}
          </span>
          {market.is_live ? (
            <div className="flex items-center gap-1.5">
              <span className="relative flex h-2 w-2">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-cyan-400 opacity-75" />
                <span className="relative inline-flex rounded-full h-2 w-2 bg-cyan-500" />
              </span>
              <span className="text-xs font-mono text-cyan-400 uppercase tracking-wider">
                Live
              </span>
              {market.game_period && (
                <span className="text-xs font-mono text-[#888] ml-1">
                  {market.game_period}
                </span>
              )}
            </div>
          ) : (
            <span className="text-xs font-mono text-[#888]">
              {new Date(market.close_time).toLocaleTimeString("en-US", {
                hour: "numeric",
                minute: "2-digit",
              })}
            </span>
          )}
        </div>

        {/* Teams and Score */}
        <div className="grid grid-cols-[1fr_auto_1fr] gap-3 items-center mb-3">
          {/* Team A */}
          <div className="text-center min-w-0">
            <div
              className={`text-base font-bold mb-1 truncate ${
                teamALeading ? "text-white" : "text-[#888]"
              }`}
            >
              {market.team_a}
            </div>
            {market.is_live && market.score && (
              <div
                className={`text-4xl font-mono font-black ${
                  teamALeading ? "text-emerald-400" : "text-[#666]"
                }`}
              >
                {scores[0]}
              </div>
            )}
          </div>

          {/* Center divider */}
          <div className="flex flex-col items-center gap-1">
            <span className="text-xs font-mono text-[#666]">VS</span>
          </div>

          {/* Team B */}
          <div className="text-center min-w-0">
            <div
              className={`text-base font-bold mb-1 truncate ${
                teamBLeading ? "text-white" : "text-[#888]"
              }`}
            >
              {market.team_b}
            </div>
            {market.is_live && market.score && (
              <div
                className={`text-4xl font-mono font-black ${
                  teamBLeading ? "text-emerald-400" : "text-[#666]"
                }`}
              >
                {scores[1]}
              </div>
            )}
          </div>
        </div>

        {/* Odds buttons */}
        <div className="grid grid-cols-2 gap-2 mb-3">
          <button
            className={`py-2 rounded-lg font-mono text-sm font-semibold transition-all truncate ${
              (market.team_a_odds || 0) > 0.5
                ? "bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 hover:bg-emerald-500/30"
                : "bg-[#18181b] text-[#a1a1aa] border border-[#27272a] hover:border-[#3f3f46]"
            }`}
          >
            <span className="truncate">{market.team_a}</span> {formatPrice(market.team_a_odds || 0)}
          </button>
          <button
            className={`py-2 rounded-lg font-mono text-sm font-semibold transition-all truncate ${
              (market.team_b_odds || 0) > 0.5
                ? "bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 hover:bg-emerald-500/30"
                : "bg-[#18181b] text-[#a1a1aa] border border-[#27272a] hover:border-[#3f3f46]"
            }`}
          >
            <span className="truncate">{market.team_b}</span> {formatPrice(market.team_b_odds || 0)}
          </button>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between text-xs text-[#888]">
          <span className="font-mono">{formatVolume(market.volume)} Vol.</span>
          <span
            className="px-2 py-0.5 rounded-full text-[9px] font-medium uppercase tracking-wider"
            style={{
              background: `${platformColor}15`,
              color: platformColor,
            }}
          >
            {market.platform}
          </span>
        </div>
      </div>
    </div>
  );
};

// Unified Market Card that routes to the right variant
const MarketCard = ({ market }: { market: MockMarket }) => {
  if (market.is_sports) {
    return <SportsCard market={market} />;
  }
  if (market.is_multi_outcome) {
    return <MultiOutcomeCard market={market} />;
  }
  return <BinaryCard market={market} />;
};

// ============================================================================
// COMPACT VARIANTS
// ============================================================================

const CompactBinaryCard = ({ market }: { market: MockMarket }) => {
  const platformColor = market.platform === "kalshi" ? "#22c55e" : "#3b82f6";

  return (
    <div
      className="group relative bg-[#0f0f11] border border-[#1f1f23] rounded-xl p-4 transition-all duration-200 hover:border-[#2a2a2f]"
      style={{ ["--platform-color" as string]: platformColor }}
    >
      <div
        className="absolute top-0 left-0 w-1 h-full rounded-l-xl"
        style={{ background: platformColor }}
      />

      <h3 className="text-[13px] font-medium text-[#fafafa] leading-tight mb-3 line-clamp-2 pl-2">
        {market.title}
      </h3>

      <div className="flex items-center gap-2 pl-2">
        <button className="flex-1 py-1.5 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-xs font-mono font-semibold">
          Yes {formatPrice(market.yes_price)}
        </button>
        <button className="flex-1 py-1.5 rounded-lg bg-rose-500/10 border border-rose-500/20 text-rose-400 text-xs font-mono font-semibold">
          No {formatPrice(market.no_price)}
        </button>
      </div>
    </div>
  );
};

const CompactSportsCard = ({ market }: { market: MockMarket }) => {
  const scores = market.score?.split(" - ").map((s) => parseInt(s.trim())) || [0, 0];

  return (
    <div className="group relative bg-[#0f0f11] border border-[#1f1f23] rounded-xl p-4 transition-all duration-200 hover:border-[#2a2a2f]">
      {market.is_live && (
        <div className="absolute top-0 left-0 right-0 h-[2px] bg-cyan-500 rounded-t-xl" />
      )}

      <div className="flex items-center justify-between mb-2">
        <span className="text-[9px] font-mono text-[#52525b] uppercase">
          {market.category}
        </span>
        {market.is_live && (
          <div className="flex items-center gap-1">
            <span className="h-1.5 w-1.5 rounded-full bg-cyan-500 animate-pulse" />
            <span className="text-[9px] font-mono text-cyan-400">LIVE</span>
          </div>
        )}
      </div>

      <div className="flex items-center justify-between">
        <div className="flex-1">
          <div className="flex items-center justify-between mb-1">
            <span className="text-sm font-semibold text-[#fafafa]">{market.team_a}</span>
            {market.is_live && <span className="text-lg font-mono font-bold text-[#fafafa]">{scores[0]}</span>}
            {!market.is_live && <span className="text-sm font-mono text-emerald-400">{formatPrice(market.team_a_odds || 0)}</span>}
          </div>
          <div className="flex items-center justify-between">
            <span className="text-sm font-semibold text-[#71717a]">{market.team_b}</span>
            {market.is_live && <span className="text-lg font-mono font-bold text-[#71717a]">{scores[1]}</span>}
            {!market.is_live && <span className="text-sm font-mono text-[#71717a]">{formatPrice(market.team_b_odds || 0)}</span>}
          </div>
        </div>
      </div>
    </div>
  );
};

// ============================================================================
// MAIN DEMO PAGE
// ============================================================================

type ViewMode = "standard" | "compact";
type FilterType = "all" | "binary" | "multi" | "sports";

const DemoPage = () => {
  const [viewMode, setViewMode] = useState<ViewMode>("standard");
  const [filter, setFilter] = useState<FilterType>("all");

  const filteredMarkets = MOCK_MARKETS.filter((m) => {
    if (filter === "all") return true;
    if (filter === "binary") return !m.is_multi_outcome && !m.is_sports;
    if (filter === "multi") return m.is_multi_outcome;
    if (filter === "sports") return m.is_sports;
    return true;
  });

  const binaryMarkets = MOCK_MARKETS.filter((m) => !m.is_multi_outcome && !m.is_sports);
  const multiMarkets = MOCK_MARKETS.filter((m) => m.is_multi_outcome);
  const sportsMarkets = MOCK_MARKETS.filter((m) => m.is_sports);
  const liveMarkets = sportsMarkets.filter((m) => m.is_live);

  return (
    <div className="min-h-screen bg-[#09090b]">
      {/* Header */}
      <header className="sticky top-0 z-50 bg-[#09090b]/80 backdrop-blur-xl border-b border-[#1f1f23]">
        <div className="max-w-7xl mx-auto px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <Link
                href="/"
                className="p-2 rounded-lg hover:bg-[#18181b] transition-colors text-[#71717a] hover:text-[#fafafa]"
              >
                <ArrowLeft className="h-5 w-5" />
              </Link>
              <div>
                <h1 className="text-xl font-semibold text-[#fafafa] flex items-center gap-2">
                  <Sparkles className="h-5 w-5 text-violet-400" />
                  Market Card Designs
                </h1>
                <p className="text-sm text-[#52525b]">
                  Demo variations for review
                </p>
              </div>
            </div>

            {/* View mode toggle */}
            <div className="flex items-center gap-2 p-1 bg-[#18181b] rounded-lg">
              <button
                onClick={() => setViewMode("standard")}
                className={`p-2 rounded-md transition-all ${
                  viewMode === "standard"
                    ? "bg-[#27272a] text-[#fafafa]"
                    : "text-[#71717a] hover:text-[#fafafa]"
                }`}
              >
                <Grid3X3 className="h-4 w-4" />
              </button>
              <button
                onClick={() => setViewMode("compact")}
                className={`p-2 rounded-md transition-all ${
                  viewMode === "compact"
                    ? "bg-[#27272a] text-[#fafafa]"
                    : "text-[#71717a] hover:text-[#fafafa]"
                }`}
              >
                <LayoutList className="h-4 w-4" />
              </button>
            </div>
          </div>
        </div>
      </header>

      <main className="max-w-7xl mx-auto px-6 py-8">
        {/* Filter tabs */}
        <div className="flex items-center gap-2 mb-8 overflow-x-auto pb-2">
          {[
            { id: "all", label: "All Markets", count: MOCK_MARKETS.length },
            { id: "binary", label: "Binary", count: binaryMarkets.length },
            { id: "multi", label: "Multi-Outcome", count: multiMarkets.length },
            { id: "sports", label: "Sports", count: sportsMarkets.length },
          ].map((tab) => (
            <button
              key={tab.id}
              onClick={() => setFilter(tab.id as FilterType)}
              className={`px-4 py-2 rounded-lg text-sm font-medium transition-all whitespace-nowrap ${
                filter === tab.id
                  ? "bg-violet-500/20 text-violet-400 border border-violet-500/30"
                  : "bg-[#18181b] text-[#71717a] border border-[#27272a] hover:border-[#3f3f46]"
              }`}
            >
              {tab.label}
              <span className="ml-2 text-xs opacity-60">{tab.count}</span>
            </button>
          ))}
        </div>

        {/* Live Sports Section (if viewing all or sports) */}
        {(filter === "all" || filter === "sports") && liveMarkets.length > 0 && (
          <section className="mb-10">
            <div className="flex items-center gap-3 mb-4">
              <div className="flex items-center gap-2">
                <Zap className="h-5 w-5 text-cyan-400" />
                <h2 className="text-lg font-semibold text-[#fafafa]">Live Now</h2>
              </div>
              <div className="flex-1 h-px bg-gradient-to-r from-cyan-500/50 to-transparent" />
            </div>

            <div
              className={
                viewMode === "standard"
                  ? "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4"
                  : "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3"
              }
            >
              {liveMarkets.map((market) =>
                viewMode === "standard" ? (
                  <SportsCard key={market.id} market={market} />
                ) : (
                  <CompactSportsCard key={market.id} market={market} />
                )
              )}
            </div>
          </section>
        )}

        {/* Main Grid */}
        <section>
          {(filter === "all" || filter === "sports") && liveMarkets.length > 0 && (
            <div className="flex items-center gap-3 mb-4">
              <h2 className="text-lg font-semibold text-[#fafafa]">All Markets</h2>
              <div className="flex-1 h-px bg-[#27272a]" />
            </div>
          )}

          <div
            className={
              viewMode === "standard"
                ? "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4"
                : "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3"
            }
          >
            {filteredMarkets
              .filter((m) => !(filter === "all" && m.is_live))
              .map((market) => {
                if (viewMode === "compact") {
                  if (market.is_sports) {
                    return <CompactSportsCard key={market.id} market={market} />;
                  }
                  return <CompactBinaryCard key={market.id} market={market} />;
                }
                return <MarketCard key={market.id} market={market} />;
              })}
          </div>
        </section>

        {/* Design Notes */}
        <section className="mt-16 p-6 bg-[#0f0f11] border border-[#1f1f23] rounded-2xl">
          <h2 className="text-lg font-semibold text-[#fafafa] mb-4">
            Design Notes
          </h2>
          <div className="grid md:grid-cols-3 gap-6 text-sm text-[#a1a1aa]">
            <div>
              <h3 className="font-medium text-[#fafafa] mb-2">Binary Cards</h3>
              <p>
                Clean Yes/No display with prominent probability. Green/red color coding
                for high/low probability. Platform accent at top.
              </p>
            </div>
            <div>
              <h3 className="font-medium text-[#fafafa] mb-2">Multi-Outcome Cards</h3>
              <p>
                Shows top 4 options with probability bars. Leading option highlighted
                in green. Badge shows total option count.
              </p>
            </div>
            <div>
              <h3 className="font-medium text-[#fafafa] mb-2">Sports Cards</h3>
              <p>
                Team vs Team layout. Live indicator with animated pulse. Score
                display for live games. Quick bet buttons per team.
              </p>
            </div>
          </div>
        </section>
      </main>
    </div>
  );
};

export default DemoPage;
