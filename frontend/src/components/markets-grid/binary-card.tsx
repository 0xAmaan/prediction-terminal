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
      className="group relative h-[310px] rounded-lg overflow-hidden transition-all duration-200 cursor-pointer block"
      style={{
        background: colors.cardBg,
        border: `1px solid ${colors.border}`,
        animationDelay: `${index * 50}ms`,
      }}
    >
      {/* Fey-style hover effect - subtle background shift */}
      <div
        className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity duration-200 pointer-events-none rounded-lg"
        style={{
          background: colors.cardHover,
          borderColor: colors.borderHover,
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
          <span className="text-[11px]" style={{ color: colors.textMuted }}>
            {formatCloseTime(market.close_time)}
          </span>
        </div>

        {/* Title - Fey typography */}
        <h3
          className="text-[18px] font-semibold leading-snug line-clamp-3 mb-auto"
          style={{ color: colors.textPrimary, letterSpacing: "-0.02em" }}
        >
          {market.title}
        </h3>

        {/* Yes/No Buttons - Fey accent colors */}
        <div className="grid grid-cols-2 gap-3 mt-4">
          <button
            className="py-2.5 rounded-lg font-medium text-[14px] transition-all duration-200"
            style={{
              background: colors.tealMuted,
              border: `1px solid ${colors.tealBorder}`,
              color: colors.teal,
            }}
          >
            Yes {formatPercent(market.yes_price)}
          </button>
          <button
            className="py-2.5 rounded-lg font-medium text-[14px] transition-all duration-200"
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
          <span className="text-[12px]" style={{ color: colors.textMuted }}>
            {formatVolume(market.volume)}
          </span>
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
