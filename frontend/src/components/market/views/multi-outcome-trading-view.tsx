"use client";

import { useState, useMemo, useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import { motion, AnimatePresence } from "framer-motion";
import type { PredictionMarket, MarketOption, Trade, OrderBook, PriceHistoryPoint } from "@/lib/types";
import { api } from "@/lib/api";

// Components
import { OutcomeSelector } from "@/components/market/shared/outcome-selector";
import { Card as WorkspaceCard, StatDisplay } from "@/components/market/layout/market-workspace";
import { OrderBookV2 } from "@/components/market/orderbook/order-book-v2";
import { PriceChart } from "@/components/market/price-chart";
import { TradeHistory } from "@/components/market/trade-history";
import { RelatedMarkets } from "@/components/market/related-markets";
import { MomentumGauge } from "@/components/market/tradeflow/momentum-gauge";
import { PressureBar } from "@/components/market/tradeflow/pressure-bar";
import { IntelligencePanel } from "@/components/market/intelligence/intelligence-panel";

// Hooks
import { getProModeFeatures } from "@/hooks/use-pro-mode";
import { useTradeMomentum } from "@/hooks/use-trade-momentum";
import { useMarketSentiment } from "@/hooks/use-market-sentiment";

// Animation variants
import { staggerContainer, staggerItem } from "@/lib/motion";

import { Info, Activity, BarChart3, Clock, DollarSign, TrendingUp, Crown, Loader2 } from "lucide-react";

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
  purple: "#6166DC",
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

// ============================================================================
// Types
// ============================================================================

export interface MultiOutcomeTradingViewProps {
  market: PredictionMarket;
  options: MarketOption[];
  selectedOutcome: MarketOption | null;
  onOutcomeSelect: (outcome: MarketOption) => void;
  relatedMarkets: PredictionMarket[];
  relatedMarketsLoading: boolean;
  proMode: boolean;
  toggleProMode: () => void;
}

// ============================================================================
// Main Component
// ============================================================================

export const MultiOutcomeTradingView = ({
  market,
  options,
  selectedOutcome,
  onOutcomeSelect,
  relatedMarkets,
  relatedMarketsLoading,
  proMode,
  toggleProMode,
}: MultiOutcomeTradingViewProps) => {
  const features = getProModeFeatures(proMode);

  // Get leading outcome for display
  const leadingOutcome = useMemo(() => {
    if (options.length === 0) return null;
    return [...options].sort(
      (a, b) => parseFloat(b.yes_price) - parseFloat(a.yes_price)
    )[0];
  }, [options]);

  // Initialize selected outcome to leading if not set
  useEffect(() => {
    if (!selectedOutcome && leadingOutcome) {
      onOutcomeSelect(leadingOutcome);
    }
  }, [selectedOutcome, leadingOutcome, onOutcomeSelect]);

  // The outcome ID to use for queries (clob_token_id for Polymarket)
  const outcomeId = selectedOutcome?.clob_token_id ?? selectedOutcome?.market_id ?? "";
  const isLeading = selectedOutcome?.market_id === leadingOutcome?.market_id;

  // Fetch orderbook for selected outcome
  const { data: orderBook, isLoading: orderBookLoading } = useQuery({
    queryKey: ["outcome-orderbook", market.platform, market.id, outcomeId],
    queryFn: () => api.getOutcomeOrderBook(market.platform, market.id, outcomeId),
    enabled: !!outcomeId,
    refetchInterval: 5000,
  });

  // Fetch trades for selected outcome
  const { data: tradesData, isLoading: tradesLoading } = useQuery({
    queryKey: ["outcome-trades", market.platform, market.id, outcomeId],
    queryFn: () => api.getOutcomeTrades(market.platform, market.id, outcomeId, 50),
    enabled: !!outcomeId,
    refetchInterval: 10000,
  });

  // Fetch price history for selected outcome
  const { data: priceHistoryData } = useQuery({
    queryKey: ["outcome-price-history", market.platform, market.id, outcomeId],
    queryFn: () => api.getOutcomePriceHistory(market.platform, market.id, outcomeId, "1w"),
    enabled: !!outcomeId,
  });

  const trades = tradesData?.trades ?? [];
  const priceHistory = useMemo(() => {
    if (!priceHistoryData) return [];
    return priceHistoryData.map((p) => p.p);
  }, [priceHistoryData]);

  // Current price from selected outcome
  const currentPrice = selectedOutcome ? parseFloat(selectedOutcome.yes_price) : 0;

  // Trade momentum analysis
  const tradeMomentum = useTradeMomentum({
    trades,
    windowSeconds: 60,
    whaleThreshold: 2,
  });

  // Calculate order book imbalance for sentiment
  const orderBookImbalance = useMemo(() => {
    if (!orderBook) return 0;
    const bidQty = orderBook.yes_bids.reduce((sum, b) => sum + parseFloat(b.quantity), 0);
    const askQty = orderBook.yes_asks.reduce((sum, a) => sum + parseFloat(a.quantity), 0);
    const total = bidQty + askQty;
    return total > 0 ? (bidQty - askQty) / total : 0;
  }, [orderBook]);

  // Market sentiment
  const sentiment = useMarketSentiment({
    orderBookImbalance,
    tradeMomentum,
    currentPrice,
    previousPrice: priceHistory.length > 1 ? priceHistory[priceHistory.length - 2] : currentPrice,
    volume24h: market.volume ? parseFloat(market.volume) : 0,
    averageVolume: market.volume ? parseFloat(market.volume) * 0.8 : 0,
  });

  // Calculate spread
  const spread = useMemo(() => {
    if (!orderBook || orderBook.yes_bids.length === 0 || orderBook.yes_asks.length === 0) {
      return null;
    }
    const bestBid = parseFloat(orderBook.yes_bids[0].price);
    const bestAsk = parseFloat(orderBook.yes_asks[0].price);
    return bestAsk - bestBid;
  }, [orderBook]);

  return (
    <div className="flex-1 overflow-auto">
      {/* Outcome Selector Header */}
      <div
        className="sticky top-0 z-40 px-6 lg:px-8 py-4"
        style={{
          backgroundColor: fey.bg100,
          borderBottom: `1px solid ${fey.border}`,
        }}
      >
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
            <span
              className="text-xs uppercase tracking-wider font-medium"
              style={{ color: fey.grey500 }}
            >
              Trading
            </span>
            {isLeading && (
              <span
                className="flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-semibold uppercase tracking-wider"
                style={{ backgroundColor: `${fey.teal}20`, color: fey.teal }}
              >
                <Crown className="w-2.5 h-2.5" />
                Leading
              </span>
            )}
          </div>
          <div className="flex-1 max-w-md">
            <OutcomeSelector
              options={options}
              selectedOutcome={selectedOutcome}
              onSelect={onOutcomeSelect}
              variant="full"
            />
          </div>
          <span
            className="text-xs"
            style={{ color: fey.grey500 }}
          >
            {options.length} outcomes
          </span>
        </div>
      </div>

      {/* Mobile Layout: Stacked */}
      <div className="lg:hidden p-6 pb-24 space-y-6">
        {/* Selected Outcome Price */}
        <div
          className="rounded-lg p-4"
          style={{ backgroundColor: fey.bg300, border: `1px solid ${fey.border}` }}
        >
          <div className="flex items-center justify-between mb-2">
            <span className="text-xs" style={{ color: fey.grey500 }}>
              {selectedOutcome?.name ?? "Select an outcome"}
            </span>
            {isLeading && (
              <Crown className="w-4 h-4" style={{ color: fey.teal }} />
            )}
          </div>
          <div className="text-3xl font-bold font-mono" style={{ color: fey.teal }}>
            {(currentPrice * 100).toFixed(1)}¢
          </div>
        </div>

        {/* Price Chart */}
        {selectedOutcome && (
          <PriceChart
            platform={market.platform}
            marketId={outcomeId}
            currentPrice={currentPrice}
            height={280}
            title={`${selectedOutcome.name} - Price History`}
          />
        )}

        {/* Order Book */}
        <OrderBookV2
          yesBids={orderBook?.yes_bids ?? []}
          yesAsks={orderBook?.yes_asks ?? []}
          isLoading={orderBookLoading}
          maxLevels={5}
          showHeatmap={false}
          showImbalance={false}
          showWalls={false}
          proMode={false}
        />

        {/* Stats Grid */}
        <WorkspaceCard title="Key Stats" icon={<BarChart3 className="h-4 w-4" style={{ color: fey.skyBlue }} />}>
          <div className="grid grid-cols-2 gap-3">
            <StatDisplay
              label="24h Volume"
              value={market.volume ? `$${(parseFloat(market.volume) / 1000).toFixed(1)}K` : "—"}
              icon={<DollarSign className="h-3 w-3" />}
            />
            <StatDisplay
              label="Outcomes"
              value={options.length.toString()}
            />
            <StatDisplay
              label="Time Remaining"
              value={formatTimeRemaining(market.close_time)}
              icon={<Clock className="h-3 w-3" />}
            />
            {spread !== null && (
              <StatDisplay label="Spread" value={`${(spread * 100).toFixed(1)}¢`} />
            )}
          </div>
        </WorkspaceCard>

        {/* Related Markets */}
        <RelatedMarkets
          markets={relatedMarkets}
          currentMarketId={market.id}
          isLoading={relatedMarketsLoading}
          maxDisplay={3}
        />
      </div>

      {/* Desktop Layout: 3-column grid */}
      <div className="hidden lg:grid grid-cols-12 gap-6 p-6 lg:p-8 pb-24">
        {/* Left Rail - Outcome Info & Stats */}
        <div className="col-span-3 space-y-6">
          {/* Selected Outcome Pulse */}
          <WorkspaceCard title="Outcome Pulse" icon={<TrendingUp className="h-4 w-4" style={{ color: fey.teal }} />}>
            <div className="space-y-4">
              {/* Outcome Name */}
              <div>
                <div className="flex items-center gap-2 mb-1">
                  <span
                    className="text-sm font-medium truncate"
                    style={{ color: fey.grey100 }}
                  >
                    {selectedOutcome?.name ?? "—"}
                  </span>
                  {isLeading && (
                    <Crown className="w-3.5 h-3.5 flex-shrink-0" style={{ color: fey.teal }} />
                  )}
                </div>
                <span className="text-3xl font-bold font-mono" style={{ color: fey.teal }}>
                  {(currentPrice * 100).toFixed(1)}¢
                </span>
              </div>

              {/* Mini Sparkline */}
              {priceHistory.length > 0 && (
                <div className="pt-2" style={{ borderTop: `1px solid ${fey.border}` }}>
                  <svg className="w-full h-12" viewBox="0 0 100 30" preserveAspectRatio="none">
                    <polyline
                      fill="none"
                      stroke={fey.teal}
                      strokeWidth="1.5"
                      points={priceHistory.slice(-20).map((p, i, arr) => {
                        const x = (i / (arr.length - 1)) * 100;
                        const min = Math.min(...arr);
                        const max = Math.max(...arr);
                        const y = max !== min ? 30 - ((p - min) / (max - min)) * 28 : 15;
                        return `${x},${y}`;
                      }).join(" ")}
                    />
                  </svg>
                </div>
              )}
            </div>
          </WorkspaceCard>

          {/* Key Stats */}
          <WorkspaceCard title="Key Stats" icon={<BarChart3 className="h-4 w-4" style={{ color: fey.skyBlue }} />}>
            <div className="space-y-3">
              <StatDisplay
                label="Total Outcomes"
                value={options.length.toString()}
              />
              <StatDisplay
                label="24h Volume"
                value={market.volume ? `$${(parseFloat(market.volume) / 1000).toFixed(1)}K` : "—"}
                icon={<DollarSign className="h-3 w-3" />}
              />
              <StatDisplay
                label="Liquidity"
                value={market.liquidity ? `$${(parseFloat(market.liquidity) / 1000).toFixed(1)}K` : "—"}
              />
              <StatDisplay
                label="Time Remaining"
                value={formatTimeRemaining(market.close_time)}
                icon={<Clock className="h-3 w-3" />}
              />
              {spread !== null && (
                <StatDisplay label="Spread" value={`${(spread * 100).toFixed(1)}¢`} />
              )}
            </div>
          </WorkspaceCard>

          {/* Momentum Gauge (Pro Mode) */}
          {features.showMomentumGauge && trades.length > 0 && (
            <motion.div
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.2 }}
            >
              <MomentumGauge momentum={tradeMomentum} showDetails={true} />
            </motion.div>
          )}

          {/* Pressure Bar (Pro Mode) */}
          {features.showPressureBar && trades.length > 0 && (
            <motion.div
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.3 }}
            >
              <WorkspaceCard title="Buy/Sell Pressure" icon={<Activity className="h-4 w-4" style={{ color: fey.purple }} />}>
                <PressureBar
                  buyVolume={tradeMomentum.buyVolume}
                  sellVolume={tradeMomentum.sellVolume}
                  showLabels={true}
                  showValues={true}
                />
              </WorkspaceCard>
            </motion.div>
          )}
        </div>

        {/* Center - Chart & Trades */}
        <div className="col-span-5 space-y-6">
          {/* Price Chart for selected outcome */}
          {selectedOutcome && (
            <PriceChart
              platform={market.platform}
              marketId={outcomeId}
              currentPrice={currentPrice}
              height={320}
              title={`${selectedOutcome.name} - Price History`}
            />
          )}

          {/* Trade History for selected outcome */}
          {selectedOutcome && (
            <WorkspaceCard
              title={`Recent Trades - ${selectedOutcome.name}`}
              icon={<Activity className="h-4 w-4" style={{ color: fey.skyBlue }} />}
            >
              {tradesLoading ? (
                <div className="flex items-center justify-center py-8">
                  <Loader2 className="w-5 h-5 animate-spin" style={{ color: fey.grey500 }} />
                </div>
              ) : trades.length > 0 ? (
                <div className="max-h-64 overflow-y-auto">
                  <TradeHistory trades={trades} maxTrades={20} />
                </div>
              ) : (
                <div className="py-6 text-center">
                  <span className="text-sm" style={{ color: fey.grey500 }}>
                    No recent trades
                  </span>
                </div>
              )}
            </WorkspaceCard>
          )}

          {/* Market Details */}
          {(market.description || market.resolution_source) && (
            <WorkspaceCard title="About This Market" icon={<Info className="h-4 w-4" style={{ color: fey.grey500 }} />}>
              <div className="space-y-3">
                {market.description && (
                  <p className="text-sm leading-relaxed" style={{ color: fey.grey300 }}>
                    {market.description}
                  </p>
                )}
                {market.resolution_source && (
                  <div>
                    <div className="text-[10px] uppercase tracking-wider mb-1" style={{ color: fey.grey500 }}>
                      Resolution Source
                    </div>
                    <p className="text-sm leading-relaxed" style={{ color: fey.grey300 }}>
                      {market.resolution_source}
                    </p>
                  </div>
                )}
              </div>
            </WorkspaceCard>
          )}
        </div>

        {/* Right Rail - Order Book & Intelligence */}
        <div className="col-span-4 space-y-6">
          {/* Order Book for selected outcome */}
          <OrderBookV2
            yesBids={orderBook?.yes_bids ?? []}
            yesAsks={orderBook?.yes_asks ?? []}
            isLoading={orderBookLoading}
            maxLevels={features.showOrderBookDepth}
            showHeatmap={features.showHeatmap}
            showImbalance={features.showImbalanceMeter}
            showWalls={features.showWallDetection}
            proMode={proMode}
          />

          {/* Intelligence Panel (Pro Mode) */}
          {features.showSentimentGauge && trades.length > 0 && (
            <motion.div
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.5 }}
            >
              <IntelligencePanel
                sentiment={sentiment}
                marketTitle={selectedOutcome?.name ?? market.title}
                platform={market.platform}
                showDetails={proMode}
              />
            </motion.div>
          )}

          {/* Related Markets */}
          <RelatedMarkets
            markets={relatedMarkets}
            currentMarketId={market.id}
            isLoading={relatedMarketsLoading}
            maxDisplay={proMode ? 5 : 3}
          />
        </div>
      </div>
    </div>
  );
};

export default MultiOutcomeTradingView;
