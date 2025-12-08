"use client";

import { colors } from "./colors";
import { formatPercent, formatVolume } from "./utils";
import type { PredictionMarket, MarketOption } from "@/lib/types";

interface MultiOutcomeCardProps {
  market: PredictionMarket;
  index: number;
  onClick?: () => void;
}

export const MultiOutcomeCard = ({
  market,
  index,
  onClick,
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
    <div
      onClick={onClick}
      className="group relative h-[310px] rounded-xl overflow-hidden transition-all duration-200 hover:-translate-y-0.5 font-[Inter,system-ui,sans-serif] cursor-pointer"
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
            {market.category || "General"}
          </span>
        </div>

        {/* Title - MORE PROMINENT */}
        <h3
          className="text-[19px] font-semibold leading-snug line-clamp-3 mb-5"
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
            <span className="text-[13px]" style={{ color: colors.textMuted }}>
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
              color: market.platform === "kalshi" ? colors.green : colors.blue,
            }}
          >
            {market.platform}
          </span>
        </div>
      </div>
    </div>
  );
};
