"use client";

import { useMemo } from "react";
import { motion, AnimatePresence } from "framer-motion";
import Link from "next/link";
import type { Platform, PredictionMarket, Trade } from "@/lib/types";

// Workspace components
import { Card as WorkspaceCard, StatDisplay } from "@/components/market/layout/market-workspace";
import { MarketBar } from "@/components/market/layout/market-bar";
import { OrderBookV2 } from "@/components/market/orderbook/order-book-v2";
import { BubbleTimeline } from "@/components/market/tradeflow/bubble-timeline";
import { MomentumGauge } from "@/components/market/tradeflow/momentum-gauge";
import { PressureBar } from "@/components/market/tradeflow/pressure-bar";
import { IntelligencePanel } from "@/components/market/intelligence/intelligence-panel";
import { PriceChart } from "@/components/market/price-chart";
import { RelatedMarkets } from "@/components/market/related-markets";
import { KeyboardShortcutsHelp } from "@/components/market/ui/pro-mode-toggle";

// Hooks
import { getProModeFeatures, useKeyboardShortcuts } from "@/hooks/use-pro-mode";
import { useTradeMomentum, useProcessedTrades } from "@/hooks/use-trade-momentum";
import { useMarketSentiment } from "@/hooks/use-market-sentiment";

// Animation variants
import { staggerContainer, staggerItem } from "@/lib/motion";

import { Info, Activity, BarChart3, Clock, DollarSign, TrendingUp } from "lucide-react";

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

interface OrderBookData {
  yes_bids: Array<{ price: string; quantity: string; order_count: number | null }>;
  yes_asks: Array<{ price: string; quantity: string; order_count: number | null }>;
  no_bids: Array<{ price: string; quantity: string; order_count: number | null }>;
  no_asks: Array<{ price: string; quantity: string; order_count: number | null }>;
}

export interface TradingViewProps {
  market: PredictionMarket;
  orderBook: OrderBookData | null;
  orderBookLoading: boolean;
  trades: Trade[];
  relatedMarkets: PredictionMarket[];
  relatedMarketsLoading: boolean;
  livePrices: { yesPrice: string; noPrice: string } | null;
  priceHistory: number[];
  proMode: boolean;
  toggleProMode: () => void;
}

export const TradingView = ({
  market,
  orderBook,
  orderBookLoading,
  trades,
  relatedMarkets,
  relatedMarketsLoading,
  livePrices,
  priceHistory,
  proMode,
  toggleProMode,
}: TradingViewProps) => {
  const features = getProModeFeatures(proMode);

  // Keyboard shortcuts
  const { showHelp, setShowHelp } = useKeyboardShortcuts(
    [
      { key: "o", description: "Toggle order book", action: () => {} },
      { key: "t", description: "Toggle trade flow", action: () => {} },
    ],
    features.keyboardShortcuts
  );

  // Use live prices from WebSocket if available, fallback to REST data
  const currentYesPrice = livePrices?.yesPrice ?? market.yes_price;
  const currentNoPrice = livePrices?.noPrice ?? market.no_price;

  // Trade momentum analysis
  const tradeMomentum = useTradeMomentum({
    trades,
    windowSeconds: 60,
    whaleThreshold: 2,
  });

  const processedTrades = useProcessedTrades(trades, 30, 2);

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
    currentPrice: parseFloat(currentYesPrice),
    previousPrice: priceHistory.length > 1 ? priceHistory[priceHistory.length - 2] : parseFloat(currentYesPrice),
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
      {/* Main Workspace */}
      <div>
        {/* Mobile Layout: Stacked */}
        <div className="lg:hidden p-6 space-y-6 pb-24">
          {/* Price Chart - Full width on mobile */}
          <PriceChart
            platform={market.platform}
            marketId={market.id}
            currentPrice={parseFloat(currentYesPrice)}
            height={280}
            title="Price History"
          />

          {/* Price Cards Row */}
          <div className="grid grid-cols-2 gap-4">
            <div
              className="rounded-lg p-4"
              style={{ backgroundColor: fey.bg300, border: `1px solid ${fey.border}` }}
            >
              <span className="text-xs" style={{ color: fey.grey500 }}>YES</span>
              <div className="text-2xl font-bold font-mono" style={{ color: fey.teal }}>
                {(parseFloat(currentYesPrice) * 100).toFixed(0)}¢
              </div>
            </div>
            <div
              className="rounded-lg p-4"
              style={{ backgroundColor: fey.bg300, border: `1px solid ${fey.border}` }}
            >
              <span className="text-xs" style={{ color: fey.grey500 }}>NO</span>
              <div className="text-2xl font-bold font-mono" style={{ color: fey.red }}>
                {(parseFloat(currentNoPrice) * 100).toFixed(0)}¢
              </div>
            </div>
          </div>

          {/* Order Book - Full width on mobile */}
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
          {/* Left Rail - Market Pulse & Stats */}
          <motion.div className="col-span-3 space-y-6 overflow-y-auto" variants={staggerItem}>
            {/* Market Pulse */}
            <WorkspaceCard title="Market Pulse" icon={<TrendingUp className="h-4 w-4" style={{ color: fey.teal }} />}>
              <div className="space-y-4">
                {/* YES Price */}
                <div className="flex items-center justify-between">
                  <span className="text-sm" style={{ color: fey.grey500 }}>YES</span>
                  <span className="text-2xl font-bold font-mono" style={{ color: fey.teal }}>
                    {(parseFloat(currentYesPrice) * 100).toFixed(0)}¢
                  </span>
                </div>

                {/* NO Price */}
                <div className="flex items-center justify-between">
                  <span className="text-sm" style={{ color: fey.grey500 }}>NO</span>
                  <span className="text-2xl font-bold font-mono" style={{ color: fey.red }}>
                    {(parseFloat(currentNoPrice) * 100).toFixed(0)}¢
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
            {features.showMomentumGauge && (
              <motion.div
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.2 }}
              >
                <MomentumGauge momentum={tradeMomentum} showDetails={true} />
              </motion.div>
            )}

            {/* Pressure Bar (Pro Mode) */}
            {features.showPressureBar && (
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
          </motion.div>

          {/* Center - Chart & Trade Flow */}
          <motion.div className="col-span-5 space-y-6 overflow-y-auto" variants={staggerItem}>
            {/* Price Chart */}
            <PriceChart
              platform={market.platform}
              marketId={market.id}
              currentPrice={parseFloat(currentYesPrice)}
              height={320}
              title="Price History"
            />

            {/* Trade Flow Strip (Pro Mode) */}
            {features.showBubbleTimeline && processedTrades.length > 0 && (
              <motion.div
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.4 }}
              >
                <BubbleTimeline trades={processedTrades} maxTrades={30} height={80} />
              </motion.div>
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
          </motion.div>

          {/* Right Rail - Order Book & Intelligence */}
          <motion.div className="col-span-4 space-y-6 overflow-y-auto" variants={staggerItem}>
            {/* Order Book V2 */}
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
            {features.showSentimentGauge && (
              <motion.div
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: 0.5 }}
              >
                <IntelligencePanel
                  sentiment={sentiment}
                  marketTitle={market.title}
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
          </motion.div>
        </div>
      </div>

      {/* Fixed Market Bar Footer */}
      <MarketBar
        yesPrice={currentYesPrice}
        noPrice={currentNoPrice}
        spread={spread}
        volume24h={market.volume}
        lastTrade={trades[0] ?? null}
        isConnected={true}
        latency={null}
      />

      {/* Keyboard Shortcuts Help Modal */}
      <AnimatePresence>
        {showHelp && (
          <KeyboardShortcutsHelp isOpen={showHelp} onClose={() => setShowHelp(false)} />
        )}
      </AnimatePresence>
    </div>
  );
};
