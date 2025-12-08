"use client";

import Link from "next/link";
import { colors } from "./colors";
import { formatPercent, formatVolume, formatGameTime } from "./utils";
import { getTeamAbbrev } from "./team-abbrev";
import type { PredictionMarket } from "@/lib/types";

interface SportsCardProps {
  market: PredictionMarket;
  index: number;
  href: string;
}

export const SportsCard = ({ market, index, href }: SportsCardProps) => {
  const scores = market.score?.split(" - ").map((s) => parseInt(s.trim())) || [
    0, 0,
  ];
  const teamALeading = market.is_live && scores[0] > scores[1];
  const teamBLeading = market.is_live && scores[1] > scores[0];

  const teamAOdds = market.home_odds ? parseFloat(market.home_odds) : 0;
  const teamBOdds = market.away_odds ? parseFloat(market.away_odds) : 0;
  const teamAFavored = teamAOdds > teamBOdds;

  return (
    <Link
      href={href}
      className="group relative h-[310px] rounded-xl overflow-hidden transition-all duration-200 hover:-translate-y-0.5 font-[Inter,system-ui,sans-serif] cursor-pointer block"
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
            {market.category || "Sports"}
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
            <span className="text-[12px]" style={{ color: colors.textMuted }}>
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
                  background: teamALeading
                    ? colors.greenMuted
                    : `${colors.textMuted}15`,
                  color: teamALeading ? colors.green : colors.textMuted,
                  border: `1px solid ${teamALeading ? colors.greenBorder : colors.border}`,
                }}
              >
                {getTeamAbbrev(market.home_team || "")}
              </div>
              <span
                className="text-[15px] font-medium"
                style={{
                  color: teamALeading
                    ? colors.textPrimary
                    : colors.textSecondary,
                }}
              >
                {market.home_team}
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
                {formatPercent(teamAOdds)}
              </span>
            </div>
          </div>

          {/* Team B */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div
                className="w-10 h-10 rounded-lg flex items-center justify-center text-[10px] font-bold"
                style={{
                  background: teamBLeading
                    ? colors.greenMuted
                    : `${colors.textMuted}15`,
                  color: teamBLeading ? colors.green : colors.textMuted,
                  border: `1px solid ${teamBLeading ? colors.greenBorder : colors.border}`,
                }}
              >
                {getTeamAbbrev(market.away_team || "")}
              </div>
              <span
                className="text-[15px] font-medium"
                style={{
                  color: teamBLeading
                    ? colors.textPrimary
                    : colors.textSecondary,
                }}
              >
                {market.away_team}
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
                {formatPercent(teamBOdds)}
              </span>
            </div>
          </div>
        </div>

        {/* Action Buttons */}
        <div className="grid grid-cols-2 gap-3 mt-4">
          <button
            className="py-3 rounded-lg font-medium text-[15px] transition-all duration-200 truncate px-2"
            style={{
              background: teamAFavored
                ? colors.greenMuted
                : `${colors.textMuted}15`,
              border: `1px solid ${teamAFavored ? colors.greenBorder : colors.border}`,
              color: teamAFavored ? colors.green : colors.textSecondary,
            }}
          >
            {market.home_team}
          </button>
          <button
            className="py-3 rounded-lg font-medium text-[15px] transition-all duration-200 truncate px-2"
            style={{
              background: !teamAFavored
                ? colors.greenMuted
                : `${colors.textMuted}15`,
              border: `1px solid ${!teamAFavored ? colors.greenBorder : colors.border}`,
              color: !teamAFavored ? colors.green : colors.textSecondary,
            }}
          >
            {market.away_team}
          </button>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between mt-4">
          <span className="text-[13px]" style={{ color: colors.textMuted }}>
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
    </Link>
  );
};
