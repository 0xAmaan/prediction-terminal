"use client";

import Link from "next/link";
import { colors } from "./colors";
import { formatPercent, formatVolume } from "./utils";
import type { PredictionMarket, MarketOption } from "@/lib/types";

interface MultiOutcomeCardProps {
  market: PredictionMarket;
  index: number;
  href: string;
}

export const MultiOutcomeCard = ({
  market,
  index,
  href,
}: MultiOutcomeCardProps) => {
  // Parse options from JSON
  let options: MarketOption[] = [];
  if (market.options_json) {
    try {
      options = JSON.parse(market.options_json);
    } catch {
      // Ignore parse errors
    }
  }

  const topOptions = options.slice(0, 2);
  const remainingCount = (market.outcome_count || 0) - 2;

  return (
    <Link
      href={href}
      className="group relative h-[310px] rounded-lg overflow-hidden transition-all duration-200 cursor-pointer block"
      style={{
        background: colors.cardBg,
        border: `1px solid ${colors.border}`,
        animationDelay: `${index * 50}ms`,
      }}
    >
      {/* Fey-style hover effect */}
      <div
        className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-200 pointer-events-none rounded-lg"
        style={{
          background: colors.cardHover,
        }}
      />

      <div className="relative p-5 h-full flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between mb-3">
          <span
            className="text-[11px] font-medium uppercase tracking-wider"
            style={{ color: colors.textMuted }}
          >
            {market.category || "General"}
          </span>
        </div>

        {/* Title - Fey typography */}
        <h3
          className="text-[18px] font-semibold leading-snug line-clamp-3 mb-4"
          style={{ color: colors.textPrimary, letterSpacing: "-0.02em" }}
        >
          {market.title}
        </h3>

        {/* Options - Fey style */}
        <div className="space-y-2 mb-auto">
          {topOptions.map((opt) => (
            <div
              key={opt.market_id}
              className="flex items-center justify-between py-2 px-3 rounded-lg transition-colors"
              style={{
                background: "rgba(255, 255, 255, 0.03)",
              }}
            >
              <span
                className="text-[14px] truncate max-w-[150px]"
                style={{
                  color: colors.textPrimary,
                  fontWeight: 500,
                }}
              >
                {opt.name}
              </span>
              <div className="flex items-center gap-2.5">
                <span
                  className="text-[14px] font-semibold font-mono"
                  style={{ color: colors.textPrimary }}
                >
                  {formatPercent(opt.yes_price)}
                </span>
                <div className="flex gap-1">
                  <button
                    className="px-2.5 py-0.5 rounded text-[11px] font-medium transition-colors"
                    style={{
                      background: colors.tealMuted,
                      color: colors.teal,
                    }}
                  >
                    Yes
                  </button>
                  <button
                    className="px-2.5 py-0.5 rounded text-[11px] font-medium transition-colors"
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
        <div className="flex items-center justify-between pt-3">
          <div className="flex items-center gap-3">
            <span className="text-[12px]" style={{ color: colors.textMuted }}>
              {formatVolume(market.volume)}
            </span>
            {remainingCount > 0 && (
              <span
                className="text-[12px] font-medium cursor-pointer hover:underline"
                style={{ color: colors.purple }}
              >
                +{remainingCount} more
              </span>
            )}
          </div>
          <span
            className="text-[10px] font-medium uppercase tracking-wider px-2 py-1 rounded"
            style={{
              // KALSHI_DISABLED: was conditional on market.platform === "kalshi"
              background: colors.polymarketMuted,
              color: colors.polymarket,
            }}
          >
            Polymarket
          </span>
        </div>
      </div>
    </Link>
  );
};
