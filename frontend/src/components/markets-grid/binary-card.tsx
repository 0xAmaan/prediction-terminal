"use client";

import Link from "next/link";
import { colors } from "./colors";
import { formatPercent, formatVolume, formatCloseTime } from "./utils";
import type { PredictionMarket } from "@/lib/types";

interface BinaryCardProps {
  market: PredictionMarket;
  index: number;
  href: string;
}

export const BinaryCard = ({ market, index, href }: BinaryCardProps) => {
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
            {market.category || "General"}
          </span>
          <span className="text-[12px]" style={{ color: colors.textMuted }}>
            {formatCloseTime(market.close_time)}
          </span>
        </div>

        {/* Title - MORE PROMINENT */}
        <h3
          className="text-[19px] font-semibold leading-snug line-clamp-3 mb-auto"
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
          <span className="text-[13px]" style={{ color: colors.textMuted }}>
            {formatVolume(market.volume)}
          </span>
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
