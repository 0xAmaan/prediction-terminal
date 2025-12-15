"use client";

import { motion } from "framer-motion";
import Image from "next/image";
import { ExternalLink } from "lucide-react";
import type { PredictionMarket, Trade } from "@/lib/types";

// Components
import { PriceChart } from "@/components/market/price-chart";
import { ResolutionStrategyCard } from "@/components/market/overview/resolution-strategy-card";
import { HistoricalAnalysisCard } from "@/components/market/overview/historical-analysis-card";
import { NewsFeedCard } from "@/components/market/overview/news-feed-card";
import { PlatformBadge } from "@/components/platform-badge";

// Animation variants
import { staggerContainer, staggerItem } from "@/lib/motion";

// Fey color tokens
const fey = {
  bg100: "#070709",
  bg300: "#131419",
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  teal: "#4DBE95",
  red: "#D84F68",
  skyBlue: "#54BBF7",
  border: "rgba(255, 255, 255, 0.06)",
};

// Format helpers
const formatTimeRemaining = (dateStr: string | null): string => {
  if (!dateStr) return "—";
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = date.getTime() - now.getTime();

  if (diffMs < 0) return "Ended";

  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
  const diffHours = Math.floor(
    (diffMs % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60)
  );

  if (diffDays === 0 && diffHours === 0) return "< 1 hour";
  if (diffDays === 0) return `${diffHours}h remaining`;
  if (diffDays === 1) return `1 day, ${diffHours}h`;
  if (diffDays < 7) return `${diffDays} days`;
  if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks`;
  return `${Math.floor(diffDays / 30)} months`;
};

// Format volume for display
const formatVolume = (volume: string | null): string => {
  if (!volume) return "—";
  const num = parseFloat(volume);
  if (isNaN(num)) return "—";
  if (num >= 1_000_000) return `$${(num / 1_000_000).toFixed(1)}M`;
  if (num >= 1_000) return `$${(num / 1_000).toFixed(1)}K`;
  return `$${num.toFixed(0)}`;
};

// ============================================================================
// StatCard Component - Fey-style mini stat cards
// ============================================================================

interface StatCardProps {
  label: string;
  value: string;
  valueColor?: string;
  subValue?: string;
}

const StatCard = ({
  label,
  value,
  valueColor = fey.grey100,
  subValue,
}: StatCardProps) => (
  <motion.div
    className="rounded-lg p-4"
    style={{
      backgroundColor: fey.bg300,
      border: `1px solid ${fey.border}`,
    }}
    variants={staggerItem}
    whileHover={{ borderColor: "rgba(255, 255, 255, 0.12)" }}
    transition={{ duration: 0.2 }}
  >
    <div
      className="text-[11px] uppercase tracking-wider font-medium mb-2"
      style={{ color: fey.grey500 }}
    >
      {label}
    </div>
    <div
      className="text-xl font-mono font-bold"
      style={{ color: valueColor, letterSpacing: "-0.02em" }}
    >
      {value}
    </div>
    {subValue && (
      <div
        className="text-xs mt-1"
        style={{ color: fey.grey500 }}
      >
        {subValue}
      </div>
    )}
  </motion.div>
);

// ============================================================================
// Main Component
// ============================================================================

export interface OverviewViewProps {
  market: PredictionMarket;
  priceHistory: number[];
  relatedMarkets?: PredictionMarket[];
  relatedMarketsLoading?: boolean;
  trades: Trade[];
  livePrices: { yesPrice: string; noPrice: string } | null;
}

export const OverviewView = ({
  market,
  priceHistory,
  trades,
  livePrices,
}: OverviewViewProps) => {
  // Use live prices from WebSocket if available
  const currentYesPrice = livePrices?.yesPrice ?? market.yes_price;
  const currentNoPrice = livePrices?.noPrice ?? market.no_price;

  return (
    <div className="flex-1 overflow-auto">
      {/* Fey-style: Single column, centered, generous padding */}
      <div className="px-6 lg:px-8 py-8 pb-24">
        <motion.div
          className="max-w-4xl mx-auto space-y-10"
          variants={staggerContainer}
          initial="hidden"
          animate="visible"
        >
          {/* 1. Market Header - Title, image, platform, status */}
          <motion.div variants={staggerItem} className="flex items-start gap-5">
            {/* Market Image */}
            {market.image_url && (
              <div
                className="flex-shrink-0 rounded-xl overflow-hidden"
                style={{
                  width: 80,
                  height: 80,
                  backgroundColor: fey.bg300,
                  border: `1px solid ${fey.border}`,
                }}
              >
                <Image
                  src={market.image_url}
                  alt={market.title}
                  width={80}
                  height={80}
                  className="object-cover w-full h-full"
                />
              </div>
            )}

            {/* Title and metadata */}
            <div className="flex-1 min-w-0">
              <h1
                className="text-2xl md:text-3xl font-bold leading-tight mb-3"
                style={{
                  color: fey.grey100,
                  letterSpacing: "-0.02em",
                }}
              >
                {market.title}
              </h1>

              {/* Platform badge, status, and external link */}
              <div className="flex items-center gap-3">
                <PlatformBadge platform={market.platform} />
                <span
                  className="text-xs font-medium px-2 py-1 rounded"
                  style={{
                    backgroundColor:
                      market.status === "open"
                        ? `${fey.teal}15`
                        : `${fey.grey500}15`,
                    color: market.status === "open" ? fey.teal : fey.grey500,
                  }}
                >
                  {market.status === "open" ? "Open" : market.status}
                </span>
                {market.url && (
                  <a
                    href={market.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="p-1.5 rounded-lg transition-colors hover:bg-white/5"
                    style={{ color: fey.grey500, border: `1px solid ${fey.border}` }}
                  >
                    <ExternalLink className="h-4 w-4" />
                  </a>
                )}
              </div>
            </div>
          </motion.div>

          {/* 2. Price Chart - Full width, prominent */}
          <motion.div variants={staggerItem}>
            <PriceChart
              platform={market.platform}
              marketId={market.id}
              currentPrice={parseFloat(currentYesPrice)}
              height={400}
              title="Price History"
            />
          </motion.div>

          {/* 3. Key Stats Grid - Horizontal mini-cards (Fey style) */}
          <motion.div
            className="grid grid-cols-2 md:grid-cols-4 gap-4"
            variants={staggerItem}
          >
            <StatCard
              label="YES Price"
              value={`${(parseFloat(currentYesPrice) * 100).toFixed(0)}¢`}
              valueColor={fey.teal}
            />
            <StatCard
              label="NO Price"
              value={`${(parseFloat(currentNoPrice) * 100).toFixed(0)}¢`}
              valueColor={fey.red}
            />
            <StatCard
              label="24h Volume"
              value={formatVolume(market.volume)}
            />
            <StatCard
              label="Liquidity"
              value={formatVolume(market.liquidity)}
            />
          </motion.div>

          {/* Second row of stats */}
          <motion.div
            className="grid grid-cols-2 md:grid-cols-3 gap-4"
            variants={staggerItem}
          >
            <StatCard
              label="Time Remaining"
              value={formatTimeRemaining(market.close_time)}
            />
            <StatCard
              label="Recent Trades"
              value={trades.length.toString()}
              subValue="Last 7 days"
            />
            <StatCard
              label="Status"
              value={market.status === "open" ? "Active" : market.status}
              valueColor={market.status === "open" ? fey.teal : fey.grey500}
            />
          </motion.div>

          {/* 4. Historical Analysis - Compact horizontal stats */}
          <motion.div variants={staggerItem}>
            <HistoricalAnalysisCard
              priceHistory={priceHistory}
              volume24h={market.volume}
              tradeCount={trades.length}
            />
          </motion.div>

          {/* 5. Resolution Strategy - Market rules and resolution info */}
          <motion.div variants={staggerItem}>
            <ResolutionStrategyCard
              resolutionSource={market.resolution_source}
              description={market.description}
              closeTime={market.close_time}
              status={market.status}
              url={market.url}
            />
          </motion.div>

          {/* 6. News Feed */}
          <motion.div variants={staggerItem}>
            <NewsFeedCard
              platform={market.platform}
              marketId={market.id}
              marketTitle={market.title}
            />
          </motion.div>
        </motion.div>
      </div>
    </div>
  );
};
