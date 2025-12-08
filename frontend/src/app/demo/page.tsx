"use client";

import { useState } from "react";
import Link from "next/link";
import { ArrowLeft, Zap, Grid3X3, LayoutList } from "lucide-react";

// ============================================================================
// COLOR PALETTE - Luxury Terminal
// ============================================================================
const colors = {
  // Backgrounds
  pageBg: "#0c0c0e",
  cardBg: "#141416",
  cardHover: "#1a1a1d",
  elevated: "#1c1c1f",

  // Borders
  border: "#232328",
  borderHover: "#3a3a42",
  borderSubtle: "#1a1a1d",

  // Text
  textPrimary: "#fafafa",
  textSecondary: "#a1a1a6",
  textMuted: "#6a6a6f",

  // Accents
  green: "#00d4aa",
  greenMuted: "#00d4aa20",
  greenBorder: "#00d4aa40",
  red: "#ff6b6b",
  redMuted: "#ff6b6b20",
  redBorder: "#ff6b6b40",
  blue: "#5b8def",
  blueMuted: "#5b8def20",
  purple: "#a78bfa",
  purpleMuted: "#a78bfa20",
  cyan: "#22d3ee",
  cyanMuted: "#22d3ee20",
};

// ============================================================================
// MOCK DATA
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
  is_multi_outcome: boolean;
  outcome_count?: number;
  leading_outcome?: string;
  options?: Array<{ name: string; yes_price: number; market_id: string }>;
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
    image_url: null,
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
    is_sports: false,
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
    team_a: "Seahawks",
    team_b: "Falcons",
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
    team_a: "Colts",
    team_b: "Jaguars",
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
    score: "1 - 0",
    game_period: "Map 2",
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
    team_a: "Real Madrid",
    team_b: "Celta de Vigo",
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
    team_a: "Celtics",
    team_b: "Raptors",
    team_a_odds: 0.57,
    team_b_odds: 0.43,
  },
];

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

const formatPercent = (price: number): string => `${Math.round(price * 100)}%`;

const formatVolume = (vol: number): string => {
  if (vol >= 1_000_000) return `$${(vol / 1_000_000).toFixed(1)}m Vol.`;
  if (vol >= 1_000) return `$${Math.round(vol / 1_000)}k Vol.`;
  return `$${vol} Vol.`;
};

const formatCloseTime = (dateStr: string): string => {
  const date = new Date(dateStr);
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

const formatGameTime = (dateStr: string): string => {
  const date = new Date(dateStr);
  return date.toLocaleTimeString("en-US", {
    hour: "numeric",
    minute: "2-digit",
  });
};

const getTeamAbbrev = (name: string): string => {
  const abbrevMap: Record<string, string> = {
    Seahawks: "SEA",
    Falcons: "ATL",
    Colts: "IND",
    Jaguars: "JAX",
    Celtics: "BOS",
    Raptors: "TOR",
    "Real Madrid": "RMA",
    "Celta de Vigo": "CEL",
    G2: "G2",
    "Team Falcons": "TF",
  };
  return abbrevMap[name] || name.slice(0, 3).toUpperCase();
};

// ============================================================================
// CARD COMPONENTS - Luxury Terminal Style
// ============================================================================

// Binary Market Card
const BinaryCard = ({
  market,
  index,
}: {
  market: MockMarket;
  index: number;
}) => {
  return (
    <div
      className="group relative h-[280px] rounded-xl overflow-hidden transition-all duration-200 hover:-translate-y-0.5 font-[Inter,system-ui,sans-serif]"
      style={{
        background: colors.cardBg,
        border: `1px solid ${colors.border}`,
        animationDelay: `${index * 50}ms`,
      }}
    >
      {/* Hover glow effect */}
      <div
        className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none rounded-xl"
        style={{
          boxShadow: `inset 0 1px 0 0 ${colors.borderHover}, 0 4px 24px -4px rgba(0,0,0,0.5)`,
        }}
      />

      <div className="relative p-6 h-full flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between mb-2">
          <span
            className="text-[12px] font-medium uppercase tracking-wide"
            style={{ color: colors.textMuted }}
          >
            {market.category}
          </span>
          <span
            className="text-[12px]"
            style={{ color: colors.textMuted }}
          >
            {formatCloseTime(market.close_time)}
          </span>
        </div>

        {/* Title - MORE PROMINENT */}
        <h3
          className="text-[19px] font-semibold leading-snug line-clamp-2 mb-auto"
          style={{ color: colors.textPrimary }}
        >
          {market.title}
        </h3>

        {/* Yes/No Buttons */}
        <div className="grid grid-cols-2 gap-3 mt-4">
          <button
            className="py-3 rounded-lg font-medium text-[15px] transition-all duration-200"
            style={{
              background: colors.greenMuted,
              border: `1px solid ${colors.greenBorder}`,
              color: colors.green,
            }}
          >
            Yes {formatPercent(market.yes_price)}
          </button>
          <button
            className="py-3 rounded-lg font-medium text-[15px] transition-all duration-200"
            style={{
              background: colors.redMuted,
              border: `1px solid ${colors.redBorder}`,
              color: colors.red,
            }}
          >
            No {formatPercent(market.no_price)}
          </button>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between mt-4">
          <span
            className="text-[13px]"
            style={{ color: colors.textMuted }}
          >
            {formatVolume(market.volume)}
          </span>
          <span
            className="text-[11px] font-medium uppercase tracking-wide px-2.5 py-1 rounded-md"
            style={{
              background:
                market.platform === "kalshi"
                  ? colors.greenMuted
                  : colors.blueMuted,
              color:
                market.platform === "kalshi" ? colors.green : colors.blue,
            }}
          >
            {market.platform}
          </span>
        </div>
      </div>
    </div>
  );
};

// Multi-Outcome Card
const MultiOutcomeCard = ({
  market,
  index,
}: {
  market: MockMarket;
  index: number;
}) => {
  const topOptions = market.options?.slice(0, 2) || [];
  const remainingCount = (market.outcome_count || 0) - 2;

  return (
    <div
      className="group relative h-[280px] rounded-xl overflow-hidden transition-all duration-200 hover:-translate-y-0.5 font-[Inter,system-ui,sans-serif]"
      style={{
        background: colors.cardBg,
        border: `1px solid ${colors.border}`,
        animationDelay: `${index * 50}ms`,
      }}
    >
      {/* Hover glow effect */}
      <div
        className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none rounded-xl"
        style={{
          boxShadow: `inset 0 1px 0 0 ${colors.borderHover}, 0 4px 24px -4px rgba(0,0,0,0.5)`,
        }}
      />

      <div className="relative p-6 h-full flex flex-col">
        {/* Header - category only, no redundant badge */}
        <div className="flex items-center justify-between mb-2">
          <span
            className="text-[12px] font-medium uppercase tracking-wide"
            style={{ color: colors.textMuted }}
          >
            {market.category}
          </span>
        </div>

        {/* Title - MORE PROMINENT */}
        <h3
          className="text-[19px] font-semibold leading-snug line-clamp-2 mb-5"
          style={{ color: colors.textPrimary }}
        >
          {market.title}
        </h3>

        {/* Options */}
        <div className="space-y-2.5 mb-auto">
          {topOptions.map((opt) => (
            <div
              key={opt.market_id}
              className="flex items-center justify-between py-2.5 px-3 rounded-lg transition-colors"
              style={{
                background: `${colors.textMuted}10`,
              }}
            >
              <span
                className="text-[15px] truncate max-w-[160px]"
                style={{
                  color: colors.textPrimary,
                  fontWeight: 500,
                }}
              >
                {opt.name}
              </span>
              <div className="flex items-center gap-3">
                <span
                  className="text-[15px] font-semibold"
                  style={{ color: colors.textPrimary }}
                >
                  {formatPercent(opt.yes_price)}
                </span>
                <div className="flex gap-1.5">
                  <button
                    className="px-3 py-1 rounded-md text-[12px] font-semibold transition-colors"
                    style={{
                      background: colors.greenMuted,
                      color: colors.green,
                    }}
                  >
                    Yes
                  </button>
                  <button
                    className="px-3 py-1 rounded-md text-[12px] font-semibold transition-colors"
                    style={{
                      background: colors.redMuted,
                      color: colors.red,
                    }}
                  >
                    No
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between pt-4">
          <div className="flex items-center gap-3">
            <span
              className="text-[13px]"
              style={{ color: colors.textMuted }}
            >
              {formatVolume(market.volume)}
            </span>
            {remainingCount > 0 && (
              <span
                className="text-[13px] font-medium cursor-pointer hover:underline"
                style={{ color: colors.purple }}
              >
                +{remainingCount} more
              </span>
            )}
          </div>
          <span
            className="text-[11px] font-medium uppercase tracking-wide px-2.5 py-1 rounded-md"
            style={{
              background:
                market.platform === "kalshi"
                  ? colors.greenMuted
                  : colors.blueMuted,
              color:
                market.platform === "kalshi" ? colors.green : colors.blue,
            }}
          >
            {market.platform}
          </span>
        </div>
      </div>
    </div>
  );
};

// Sports Card
const SportsCard = ({
  market,
  index,
}: {
  market: MockMarket;
  index: number;
}) => {
  const scores = market.score?.split(" - ").map((s) => parseInt(s.trim())) || [
    0, 0,
  ];
  const teamALeading = market.is_live && scores[0] > scores[1];
  const teamBLeading = market.is_live && scores[1] > scores[0];
  const teamAFavored = (market.team_a_odds || 0) > (market.team_b_odds || 0);

  return (
    <div
      className="group relative h-[280px] rounded-xl overflow-hidden transition-all duration-200 hover:-translate-y-0.5 font-[Inter,system-ui,sans-serif]"
      style={{
        background: colors.cardBg,
        border: `1px solid ${colors.border}`,
        animationDelay: `${index * 50}ms`,
      }}
    >
      {/* Live indicator bar */}
      {market.is_live && (
        <div
          className="absolute top-0 left-0 right-0 h-[2px]"
          style={{
            background: `linear-gradient(90deg, ${colors.cyan}, ${colors.green}, ${colors.cyan})`,
          }}
        />
      )}

      {/* Hover glow effect */}
      <div
        className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none rounded-xl"
        style={{
          boxShadow: `inset 0 1px 0 0 ${colors.borderHover}, 0 4px 24px -4px rgba(0,0,0,0.5)`,
        }}
      />

      <div className="relative p-6 h-full flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between mb-2">
          <span
            className="text-[12px] font-medium uppercase tracking-wide"
            style={{ color: colors.textMuted }}
          >
            {market.category}
          </span>
          {market.is_live ? (
            <div className="flex items-center gap-2">
              <div className="flex items-center gap-1.5">
                <span className="relative flex h-2 w-2">
                  <span
                    className="animate-ping absolute inline-flex h-full w-full rounded-full opacity-75"
                    style={{ background: colors.cyan }}
                  />
                  <span
                    className="relative inline-flex rounded-full h-2 w-2"
                    style={{ background: colors.cyan }}
                  />
                </span>
                <span
                  className="text-[11px] font-bold uppercase tracking-wide"
                  style={{ color: colors.cyan }}
                >
                  Live
                </span>
              </div>
              {market.game_period && (
                <span
                  className="text-[11px]"
                  style={{ color: colors.textMuted }}
                >
                  {market.game_period}
                </span>
              )}
            </div>
          ) : (
            <span
              className="text-[12px]"
              style={{ color: colors.textMuted }}
            >
              {formatGameTime(market.close_time)}
            </span>
          )}
        </div>

        {/* Teams */}
        <div className="space-y-3 flex-1">
          {/* Team A */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div
                className="w-10 h-10 rounded-lg flex items-center justify-center text-[10px] font-bold"
                style={{
                  background: teamALeading ? colors.greenMuted : `${colors.textMuted}15`,
                  color: teamALeading ? colors.green : colors.textMuted,
                  border: `1px solid ${teamALeading ? colors.greenBorder : colors.border}`,
                }}
              >
                {getTeamAbbrev(market.team_a || "")}
              </div>
              <span
                className="text-[15px] font-medium"
                style={{
                  color: teamALeading
                    ? colors.textPrimary
                    : colors.textSecondary,
                }}
              >
                {market.team_a}
              </span>
            </div>
            <div className="flex items-center gap-3">
              {market.is_live && (
                <span
                  className="text-xl font-bold tabular-nums w-8 text-center"
                  style={{
                    color: teamALeading ? colors.green : colors.textMuted,
                  }}
                >
                  {scores[0]}
                </span>
              )}
              <span
                className="text-[14px] font-semibold w-12 text-right"
                style={{ color: colors.textSecondary }}
              >
                {formatPercent(market.team_a_odds || 0)}
              </span>
            </div>
          </div>

          {/* Team B */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div
                className="w-10 h-10 rounded-lg flex items-center justify-center text-[10px] font-bold"
                style={{
                  background: teamBLeading ? colors.greenMuted : `${colors.textMuted}15`,
                  color: teamBLeading ? colors.green : colors.textMuted,
                  border: `1px solid ${teamBLeading ? colors.greenBorder : colors.border}`,
                }}
              >
                {getTeamAbbrev(market.team_b || "")}
              </div>
              <span
                className="text-[15px] font-medium"
                style={{
                  color: teamBLeading
                    ? colors.textPrimary
                    : colors.textSecondary,
                }}
              >
                {market.team_b}
              </span>
            </div>
            <div className="flex items-center gap-3">
              {market.is_live && (
                <span
                  className="text-xl font-bold tabular-nums w-8 text-center"
                  style={{
                    color: teamBLeading ? colors.green : colors.textMuted,
                  }}
                >
                  {scores[1]}
                </span>
              )}
              <span
                className="text-[14px] font-semibold w-12 text-right"
                style={{ color: colors.textSecondary }}
              >
                {formatPercent(market.team_b_odds || 0)}
              </span>
            </div>
          </div>
        </div>

        {/* Action Buttons */}
        <div className="grid grid-cols-2 gap-3 mt-4">
          <button
            className="py-3 rounded-lg font-medium text-[15px] transition-all duration-200"
            style={{
              background: teamAFavored ? colors.greenMuted : `${colors.textMuted}15`,
              border: `1px solid ${teamAFavored ? colors.greenBorder : colors.border}`,
              color: teamAFavored ? colors.green : colors.textSecondary,
            }}
          >
            {market.team_a}
          </button>
          <button
            className="py-3 rounded-lg font-medium text-[15px] transition-all duration-200"
            style={{
              background: !teamAFavored ? colors.greenMuted : `${colors.textMuted}15`,
              border: `1px solid ${!teamAFavored ? colors.greenBorder : colors.border}`,
              color: !teamAFavored ? colors.green : colors.textSecondary,
            }}
          >
            {market.team_b}
          </button>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between mt-4">
          <span
            className="text-[13px]"
            style={{ color: colors.textMuted }}
          >
            {formatVolume(market.volume)}
          </span>
          <span
            className="text-[11px] font-medium uppercase tracking-wide px-2.5 py-1 rounded-md"
            style={{
              background: colors.blueMuted,
              color: colors.blue,
            }}
          >
            {market.platform}
          </span>
        </div>
      </div>
    </div>
  );
};

// Unified Market Card
const MarketCard = ({
  market,
  index,
}: {
  market: MockMarket;
  index: number;
}) => {
  if (market.is_sports) {
    return <SportsCard market={market} index={index} />;
  }
  if (market.is_multi_outcome) {
    return <MultiOutcomeCard market={market} index={index} />;
  }
  return <BinaryCard market={market} index={index} />;
};

// ============================================================================
// COMPACT VARIANTS
// ============================================================================

const CompactBinaryCard = ({
  market,
  index,
}: {
  market: MockMarket;
  index: number;
}) => {
  return (
    <div
      className="group relative rounded-xl overflow-hidden transition-all duration-200 hover:-translate-y-0.5 font-[Inter,system-ui,sans-serif]"
      style={{
        background: colors.cardBg,
        border: `1px solid ${colors.border}`,
        animationDelay: `${index * 50}ms`,
      }}
    >
      <div className="p-5">
        <h3
          className="text-[15px] font-medium leading-tight line-clamp-2 mb-4"
          style={{ color: colors.textPrimary }}
        >
          {market.title}
        </h3>
        <div className="flex items-center gap-2">
          <button
            className="flex-1 py-2.5 rounded-lg text-[14px] font-medium"
            style={{
              background: colors.greenMuted,
              border: `1px solid ${colors.greenBorder}`,
              color: colors.green,
            }}
          >
            Yes {formatPercent(market.yes_price)}
          </button>
          <button
            className="flex-1 py-2.5 rounded-lg text-[14px] font-medium"
            style={{
              background: colors.redMuted,
              border: `1px solid ${colors.redBorder}`,
              color: colors.red,
            }}
          >
            No {formatPercent(market.no_price)}
          </button>
        </div>
      </div>
    </div>
  );
};

const CompactSportsCard = ({
  market,
  index,
}: {
  market: MockMarket;
  index: number;
}) => {
  const scores = market.score?.split(" - ").map((s) => parseInt(s.trim())) || [
    0, 0,
  ];

  return (
    <div
      className="group relative rounded-xl overflow-hidden transition-all duration-200 hover:-translate-y-0.5 font-[Inter,system-ui,sans-serif]"
      style={{
        background: colors.cardBg,
        border: `1px solid ${colors.border}`,
        animationDelay: `${index * 50}ms`,
      }}
    >
      {market.is_live && (
        <div
          className="absolute top-0 left-0 right-0 h-[2px]"
          style={{ background: colors.cyan }}
        />
      )}
      <div className="p-5">
        <div className="flex items-center justify-between mb-3">
          <span
            className="text-[11px] font-medium uppercase tracking-wide"
            style={{ color: colors.textMuted }}
          >
            {market.category}
          </span>
          {market.is_live && (
            <div className="flex items-center gap-1.5">
              <span
                className="h-2 w-2 rounded-full animate-pulse"
                style={{ background: colors.cyan }}
              />
              <span
                className="text-[11px] font-bold uppercase"
                style={{ color: colors.cyan }}
              >
                Live
              </span>
            </div>
          )}
        </div>
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <span
              className="text-[15px] font-semibold"
              style={{ color: colors.textPrimary }}
            >
              {market.team_a}
            </span>
            {market.is_live ? (
              <span
                className="text-xl font-bold"
                style={{ color: colors.textPrimary }}
              >
                {scores[0]}
              </span>
            ) : (
              <span
                className="text-[14px] font-semibold"
                style={{ color: colors.green }}
              >
                {formatPercent(market.team_a_odds || 0)}
              </span>
            )}
          </div>
          <div className="flex items-center justify-between">
            <span
              className="text-[15px] font-medium"
              style={{ color: colors.textSecondary }}
            >
              {market.team_b}
            </span>
            {market.is_live ? (
              <span
                className="text-xl font-bold"
                style={{ color: colors.textMuted }}
              >
                {scores[1]}
              </span>
            ) : (
              <span
                className="text-[14px] font-medium"
                style={{ color: colors.textMuted }}
              >
                {formatPercent(market.team_b_odds || 0)}
              </span>
            )}
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

  const binaryMarkets = MOCK_MARKETS.filter(
    (m) => !m.is_multi_outcome && !m.is_sports
  );
  const multiMarkets = MOCK_MARKETS.filter((m) => m.is_multi_outcome);
  const sportsMarkets = MOCK_MARKETS.filter((m) => m.is_sports);
  const liveMarkets = sportsMarkets.filter((m) => m.is_live);

  return (
    <div className="min-h-screen font-[Inter,system-ui,sans-serif]" style={{ background: colors.pageBg }}>
      {/* Header */}
      <header
        className="sticky top-0 z-50 backdrop-blur-xl"
        style={{
          background: `${colors.pageBg}ee`,
          borderBottom: `1px solid ${colors.border}`,
        }}
      >
        <div className="max-w-[1400px] mx-auto px-8 py-5">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-5">
              <Link
                href="/"
                className="p-2 rounded-lg transition-colors hover:bg-white/5"
                style={{ color: colors.textMuted }}
              >
                <ArrowLeft className="h-5 w-5" />
              </Link>
              <div>
                <h1
                  className="text-2xl font-semibold"
                  style={{ color: colors.textPrimary }}
                >
                  Market Card Designs
                </h1>
                <p className="text-sm mt-0.5" style={{ color: colors.textMuted }}>
                  Luxury Terminal Style
                </p>
              </div>
            </div>

            {/* View mode toggle */}
            <div
              className="flex items-center gap-1 p-1.5 rounded-lg"
              style={{ background: colors.elevated }}
            >
              <button
                onClick={() => setViewMode("standard")}
                className="p-2.5 rounded-md transition-all"
                style={{
                  background:
                    viewMode === "standard" ? colors.cardBg : "transparent",
                  color:
                    viewMode === "standard"
                      ? colors.textPrimary
                      : colors.textMuted,
                }}
              >
                <Grid3X3 className="h-4 w-4" />
              </button>
              <button
                onClick={() => setViewMode("compact")}
                className="p-2.5 rounded-md transition-all"
                style={{
                  background:
                    viewMode === "compact" ? colors.cardBg : "transparent",
                  color:
                    viewMode === "compact"
                      ? colors.textPrimary
                      : colors.textMuted,
                }}
              >
                <LayoutList className="h-4 w-4" />
              </button>
            </div>
          </div>
        </div>
      </header>

      <main className="max-w-[1400px] mx-auto px-8 py-10">
        {/* Filter tabs */}
        <div className="flex items-center gap-3 mb-10 overflow-x-auto pb-2">
          {[
            { id: "all", label: "All Markets", count: MOCK_MARKETS.length },
            { id: "binary", label: "Binary", count: binaryMarkets.length },
            { id: "multi", label: "Multi-Outcome", count: multiMarkets.length },
            { id: "sports", label: "Sports", count: sportsMarkets.length },
          ].map((tab) => (
            <button
              key={tab.id}
              onClick={() => setFilter(tab.id as FilterType)}
              className="px-5 py-3 rounded-lg text-[14px] font-medium transition-all whitespace-nowrap"
              style={{
                background:
                  filter === tab.id ? colors.purpleMuted : colors.elevated,
                border: `1px solid ${filter === tab.id ? colors.purple : colors.border}`,
                color:
                  filter === tab.id ? colors.purple : colors.textSecondary,
              }}
            >
              {tab.label}
              <span className="ml-2 opacity-60">{tab.count}</span>
            </button>
          ))}
        </div>

        {/* Live Sports Section */}
        {(filter === "all" || filter === "sports") && liveMarkets.length > 0 && (
          <section className="mb-12">
            <div className="flex items-center gap-4 mb-6">
              <div className="flex items-center gap-2">
                <Zap className="h-5 w-5" style={{ color: colors.cyan }} />
                <h2
                  className="text-xl font-semibold"
                  style={{ color: colors.textPrimary }}
                >
                  Live Now
                </h2>
              </div>
              <div
                className="flex-1 h-px"
                style={{
                  background: `linear-gradient(90deg, ${colors.cyan}50, transparent)`,
                }}
              />
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
              {liveMarkets.map((market, i) =>
                viewMode === "standard" ? (
                  <SportsCard key={market.id} market={market} index={i} />
                ) : (
                  <CompactSportsCard key={market.id} market={market} index={i} />
                )
              )}
            </div>
          </section>
        )}

        {/* Main Grid */}
        <section>
          {(filter === "all" || filter === "sports") &&
            liveMarkets.length > 0 && (
              <div className="flex items-center gap-4 mb-6">
                <h2
                  className="text-xl font-semibold"
                  style={{ color: colors.textPrimary }}
                >
                  All Markets
                </h2>
                <div
                  className="flex-1 h-px"
                  style={{ background: colors.border }}
                />
              </div>
            )}

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
            {filteredMarkets
              .filter((m) => !(filter === "all" && m.is_live))
              .map((market, i) => {
                if (viewMode === "compact") {
                  if (market.is_sports) {
                    return (
                      <CompactSportsCard
                        key={market.id}
                        market={market}
                        index={i}
                      />
                    );
                  }
                  return (
                    <CompactBinaryCard
                      key={market.id}
                      market={market}
                      index={i}
                    />
                  );
                }
                return <MarketCard key={market.id} market={market} index={i} />;
              })}
          </div>
        </section>
      </main>
    </div>
  );
};

export default DemoPage;
