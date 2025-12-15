"use client";

import { useState, useMemo, useEffect, useCallback } from "react";
import { useParams, useSearchParams } from "next/navigation";
import { useQuery } from "@tanstack/react-query";
import Link from "next/link";
import { motion, AnimatePresence } from "framer-motion";
import { api } from "@/lib/api";
import type { Platform, PredictionMarket, MarketOption, Trade } from "@/lib/types";

// New view components
import { TradingView } from "@/components/market/views/trading-view";
import { OverviewView } from "@/components/market/views/overview-view";
import { MultiOutcomeOverviewView } from "@/components/market/views/multi-outcome-overview-view";
import { MultiOutcomeTradingView } from "@/components/market/views/multi-outcome-trading-view";
import { ResearchView } from "@/components/market/views/research-view";
import { MarketTabs, type MarketTab } from "@/components/market/market-tabs";
import { MarketBar } from "@/components/market/layout/market-bar";
import { MultiOutcomeMarketBar } from "@/components/market/layout/multi-outcome-market-bar";

// Hooks
import { useMarketStream } from "@/hooks/use-market-stream";
import type { ConnectionState } from "@/hooks/use-websocket";

// Animation variants
import { staggerContainer, staggerItem } from "@/lib/motion";

import { ArrowLeft, ExternalLink, Info } from "lucide-react";

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

// ============================================================================
// Sub-components
// ============================================================================

const PlatformBadge = ({ platform }: { platform: Platform }) => {
  const color = fey.skyBlue;
  return (
    <span
      className="text-[10px] font-medium uppercase tracking-wider px-2 py-1 rounded"
      style={{
        backgroundColor: `${color}15`,
        color: color,
      }}
    >
      Polymarket
    </span>
  );
};

const StatusBadge = ({ status }: { status: string }) => {
  const statusConfig = {
    open: { color: fey.teal, label: "Open" },
    closed: { color: "#C27C58", label: "Closed" },
    settled: { color: fey.grey500, label: "Settled" },
  };

  const config =
    statusConfig[status as keyof typeof statusConfig] || statusConfig.open;

  return (
    <span
      className="text-[10px] font-medium uppercase tracking-wider px-2 py-1 rounded flex items-center gap-1.5"
      style={{
        backgroundColor: `${config.color}15`,
        color: config.color,
      }}
    >
      <span
        className="h-1.5 w-1.5 rounded-full animate-pulse"
        style={{ backgroundColor: config.color }}
      />
      {config.label}
    </span>
  );
};

// Parse options from options_json
const parseOptions = (optionsJson: string | null): MarketOption[] => {
  if (!optionsJson) return [];
  try {
    return JSON.parse(optionsJson);
  } catch {
    return [];
  }
};

// ============================================================================
// Main Content Component with Tab Switching
// ============================================================================

interface MarketPageContentProps {
  market: PredictionMarket;
  orderBook: {
    yes_bids: Array<{ price: string; quantity: string; order_count: number | null }>;
    yes_asks: Array<{ price: string; quantity: string; order_count: number | null }>;
    no_bids: Array<{ price: string; quantity: string; order_count: number | null }>;
    no_asks: Array<{ price: string; quantity: string; order_count: number | null }>;
  } | null;
  orderBookLoading: boolean;
  trades: Trade[];
  tradesLoading: boolean;
  relatedMarkets: PredictionMarket[];
  relatedMarketsLoading: boolean;
  connectionState: ConnectionState;
  latency: number | null;
  livePrices: { yesPrice: string; noPrice: string } | null;
  priceHistory: number[];
  initialTab?: MarketTab;
}

const MarketPageContent = ({
  market,
  orderBook,
  orderBookLoading,
  trades,
  tradesLoading,
  relatedMarkets,
  relatedMarketsLoading,
  connectionState,
  latency,
  livePrices,
  priceHistory,
  initialTab = "overview",
}: MarketPageContentProps) => {
  // Tab state - use initialTab from URL query param or default to overview
  const [activeTab, setActiveTab] = useState<MarketTab>(initialTab);

  // Parse multi-outcome options early (needed for state initialization)
  const options = parseOptions(market.options_json ?? null);
  const isMultiOutcome = market.is_multi_outcome && options.length > 0;

  // Selected outcome state for multi-outcome trading view
  const [selectedOutcome, setSelectedOutcome] = useState<MarketOption | null>(null);

  // Initialize selected outcome to leading (highest price) on mount
  useEffect(() => {
    if (isMultiOutcome && options.length > 0 && !selectedOutcome) {
      const sorted = [...options].sort(
        (a, b) => parseFloat(b.yes_price) - parseFloat(a.yes_price)
      );
      setSelectedOutcome(sorted[0]);
    }
  }, [isMultiOutcome, options, selectedOutcome]);

  // Keyboard shortcuts for tab switching
  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      // Don't trigger if user is typing in an input
      if (
        event.target instanceof HTMLInputElement ||
        event.target instanceof HTMLTextAreaElement ||
        (event.target as HTMLElement).isContentEditable
      ) {
        return;
      }

      // O for Overview, T for Trading, R for Research
      if (event.key === "o" || event.key === "O") {
        setActiveTab("overview");
      } else if (event.key === "t" || event.key === "T") {
        setActiveTab("trading");
      } else if (event.key === "r" || event.key === "R") {
        setActiveTab("research");
      }
    },
    []
  );

  useEffect(() => {
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  // Use live prices from WebSocket if available
  const currentYesPrice = livePrices?.yesPrice ?? market.yes_price;
  const currentNoPrice = livePrices?.noPrice ?? market.no_price;

  // Calculate spread for MarketBar
  const spread = useMemo(() => {
    if (!orderBook || orderBook.yes_bids.length === 0 || orderBook.yes_asks.length === 0) {
      return null;
    }
    const bestBid = parseFloat(orderBook.yes_bids[0].price);
    const bestAsk = parseFloat(orderBook.yes_asks[0].price);
    return bestAsk - bestBid;
  }, [orderBook]);

  // Handle outcome click from overview grid - switch to trading tab with that outcome
  const handleOutcomeClick = useCallback((outcome: MarketOption) => {
    setSelectedOutcome(outcome);
    setActiveTab("trading");
  }, []);

  // =========================================================================
  // Tab-based Layout (for both binary and multi-outcome markets)
  // =========================================================================

  return (
    <motion.div
      className="h-screen flex flex-col overflow-hidden"
      style={{ backgroundColor: fey.bg100 }}
      initial="hidden"
      animate="visible"
      variants={staggerContainer}
    >
      {/* Minimal Header - Always show back button */}
      <motion.header
        className="sticky top-0 z-50"
        style={{ backgroundColor: fey.bg100, borderBottom: `1px solid ${fey.border}` }}
        variants={staggerItem}
      >
        <div className="px-6 lg:px-8 py-3">
          <div className="flex items-center justify-between">
            <Link href="/" className="p-2 rounded-lg transition-colors hover:bg-white/5" style={{ color: fey.grey500 }}>
              <ArrowLeft className="h-5 w-5" />
            </Link>

            {market.url && (
                <a
                  href={market.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center gap-2 px-2.5 py-1.5 rounded-lg transition-colors hover:bg-white/5"
                  style={{ border: `1px solid ${fey.border}`, color: fey.grey500 }}
                >
                  <ExternalLink className="h-4 w-4" />
                </a>
            )}
          </div>
        </div>
      </motion.header>

      {/* Tab Navigation */}
      <MarketTabs activeTab={activeTab} onTabChange={setActiveTab} />

      {/* Tab Content with Crossfade Animation */}
      <AnimatePresence mode="wait">
        {activeTab === "overview" && (
          <motion.div
            key="overview"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.2 }}
            className="flex-1 flex flex-col min-h-0"
          >
            {isMultiOutcome ? (
              <MultiOutcomeOverviewView
                market={market}
                options={options}
                trades={trades}
                onOutcomeClick={handleOutcomeClick}
              />
            ) : (
              <OverviewView
                market={market}
                priceHistory={priceHistory}
                relatedMarkets={relatedMarkets}
                relatedMarketsLoading={relatedMarketsLoading}
                trades={trades}
                livePrices={livePrices}
              />
            )}
          </motion.div>
        )}
        {activeTab === "trading" && (
          <motion.div
            key="trading"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.2 }}
            className="flex-1 flex flex-col min-h-0"
          >
            {isMultiOutcome ? (
              <MultiOutcomeTradingView
                market={market}
                options={options}
                selectedOutcome={selectedOutcome}
                onOutcomeSelect={setSelectedOutcome}
                relatedMarkets={relatedMarkets}
                relatedMarketsLoading={relatedMarketsLoading}
              />
            ) : (
              <TradingView
                market={market}
                orderBook={orderBook}
                orderBookLoading={orderBookLoading}
                trades={trades}
                relatedMarkets={relatedMarkets}
                relatedMarketsLoading={relatedMarketsLoading}
                livePrices={livePrices}
                priceHistory={priceHistory}
              />
            )}
          </motion.div>
        )}
        {activeTab === "research" && (
          <motion.div
            key="research"
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.2 }}
            className="flex-1 flex flex-col min-h-0"
          >
            <ResearchView
              platform={market.platform}
              marketId={market.id}
              market={market}
            />
          </motion.div>
        )}
      </AnimatePresence>

      {/* Fixed Market Bar Footer - Only show on Overview */}
      {activeTab === "overview" && (
        isMultiOutcome ? (
          <MultiOutcomeMarketBar
            options={options}
            selectedOutcome={selectedOutcome}
            onOutcomeSelect={handleOutcomeClick}
            volume24h={market.volume}
            lastTrade={trades[0] ?? null}
            connectionState={connectionState}
            latency={latency}
          />
        ) : (
          <MarketBar
            yesPrice={currentYesPrice}
            noPrice={currentNoPrice}
            spread={spread}
            volume24h={market.volume}
            lastTrade={trades[0] ?? null}
            connectionState={connectionState}
            latency={latency}
          />
        )
      )}
    </motion.div>
  );
};

// ============================================================================
// Loading Skeleton
// ============================================================================

const LoadingSkeleton = () => (
  <div className="min-h-screen" style={{ backgroundColor: fey.bg100 }}>
    <header style={{ backgroundColor: fey.bg100, borderBottom: `1px solid ${fey.border}` }}>
      <div className="px-8 lg:px-32 py-3">
        <div className="flex items-center gap-4">
          <div className="h-9 w-9 rounded-lg animate-pulse" style={{ backgroundColor: fey.bg300 }} />
          <div className="flex items-center gap-3">
            <div className="h-5 w-16 rounded animate-pulse" style={{ backgroundColor: fey.bg300 }} />
            <div className="h-5 w-14 rounded animate-pulse" style={{ backgroundColor: fey.bg300 }} />
          </div>
          <div className="h-5 w-96 flex-1 rounded animate-pulse" style={{ backgroundColor: fey.bg300 }} />
        </div>
      </div>
    </header>
    <div style={{ borderBottom: `1px solid ${fey.border}` }}>
      <div className="px-8 lg:px-32 py-3 flex gap-8">
        <div className="h-5 w-20 rounded animate-pulse" style={{ backgroundColor: fey.bg300 }} />
        <div className="h-5 w-16 rounded animate-pulse" style={{ backgroundColor: fey.bg300 }} />
      </div>
    </div>
    <main className="px-8 lg:px-32 py-8">
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        <div className="lg:col-span-2 space-y-6">
          <div className="h-32 rounded-lg animate-pulse" style={{ backgroundColor: fey.bg300 }} />
          <div className="h-40 rounded-lg animate-pulse" style={{ backgroundColor: fey.bg300 }} />
          <div className="h-64 rounded-lg animate-pulse" style={{ backgroundColor: fey.bg300 }} />
        </div>
        <div className="space-y-6">
          <div className="h-48 rounded-lg animate-pulse" style={{ backgroundColor: fey.bg300 }} />
          <div className="h-48 rounded-lg animate-pulse" style={{ backgroundColor: fey.bg300 }} />
        </div>
      </div>
    </main>
  </div>
);

// ============================================================================
// Error State
// ============================================================================

const ErrorState = ({ message }: { message: string }) => (
  <div className="min-h-screen flex items-center justify-center" style={{ backgroundColor: fey.bg100 }}>
    <div
      className="max-w-md w-full mx-4 rounded-lg p-8 text-center"
      style={{ backgroundColor: fey.bg300, border: `1px solid ${fey.border}` }}
    >
      <div
        className="h-12 w-12 rounded-full flex items-center justify-center mx-auto mb-4"
        style={{ backgroundColor: `${fey.red}15` }}
      >
        <Info className="h-6 w-6" style={{ color: fey.red }} />
      </div>
      <h2 className="text-lg font-semibold mb-2" style={{ color: fey.grey100 }}>
        Failed to Load Market
      </h2>
      <p className="mb-6" style={{ color: fey.grey500 }}>{message}</p>
      <Link
        href="/"
        className="inline-flex items-center gap-2 px-4 py-2 rounded-lg transition-colors"
        style={{ backgroundColor: fey.teal, color: fey.bg100 }}
      >
        <ArrowLeft className="h-4 w-4" />
        Back to Markets
      </Link>
    </div>
  </div>
);

// ============================================================================
// Main Page Component
// ============================================================================

const MarketPage = () => {
  const params = useParams();
  const searchParams = useSearchParams();
  const platform = params.platform as Platform;
  const id = params.id as string;

  // Get initial tab from URL query param
  const tabParam = searchParams.get("tab");
  const initialTab: MarketTab =
    tabParam === "research" ? "research" :
    tabParam === "trading" ? "trading" :
    "overview";

  // Fetch market data
  const {
    data: market,
    isLoading,
    error,
  } = useQuery({
    queryKey: ["market", platform, id],
    queryFn: () => api.getMarket(platform, id),
    enabled: !!platform && !!id,
    staleTime: 5 * 60 * 1000,
  });

  // Determine if this is a multi-outcome market
  const isMultiOutcome = market?.is_multi_outcome ?? false;

  // Fetch order book
  const orderBookId = platform === "kalshi" ? market?.ticker : id;
  const { data: orderBook, isLoading: orderBookLoading } = useQuery({
    queryKey: ["orderbook", platform, orderBookId],
    queryFn: () => api.getOrderBook(platform, orderBookId!),
    enabled: !!platform && !!orderBookId && !isMultiOutcome,
    staleTime: 5 * 60 * 1000,
  });

  // Fetch trades
  const { data: tradesData, isLoading: tradesLoading } = useQuery({
    queryKey: ["trades", platform, orderBookId],
    queryFn: () => api.getTrades(platform, orderBookId!, 50),
    enabled: !!platform && !!orderBookId && !isMultiOutcome,
    staleTime: 5 * 60 * 1000,
  });

  // Fetch related markets
  const { data: relatedData, isLoading: relatedMarketsLoading } = useQuery({
    queryKey: ["related", platform, id],
    queryFn: () => api.getRelatedMarkets(platform, id, 6),
    enabled: !!platform && !!id,
    staleTime: 5 * 60 * 1000,
  });

  // Fetch price history
  const { data: priceHistoryData } = useQuery({
    queryKey: ["priceHistory", platform, orderBookId, "7d"],
    queryFn: () => api.getPriceHistory(platform, orderBookId!, { timeframe: "7d" }),
    enabled: !!platform && !!orderBookId && !isMultiOutcome,
    staleTime: 5 * 60 * 1000,
  });

  // WebSocket streaming
  const wsMarketId = platform === "kalshi" ? (market?.ticker ?? id) : id;
  const {
    connectionState,
    prices: wsPrices,
    orderBook: wsOrderBook,
    trades: wsTrades,
    latency,
  } = useMarketStream({
    platform: platform as "kalshi" | "polymarket",
    marketId: wsMarketId,
    subscribePrices: true,
    subscribeOrderBook: true,
    subscribeTrades: true,
  });

  if (isLoading) {
    return <LoadingSkeleton />;
  }

  if (error || !market) {
    return <ErrorState message={error?.message || "Market not found"} />;
  }

  // Merge WebSocket data with REST data
  const mergedOrderBook = wsOrderBook
    ? {
        yes_bids: wsOrderBook.yesBids,
        yes_asks: wsOrderBook.yesAsks,
        no_bids: wsOrderBook.noBids,
        no_asks: wsOrderBook.noAsks,
      }
    : orderBook ?? null;

  // Merge trades
  const restTrades = tradesData?.trades ?? [];
  const wsTradeIds = new Set(wsTrades.map((t) => t.id));
  const uniqueRestTrades = restTrades.filter((t) => !wsTradeIds.has(t.id));
  const mergedTrades = [...wsTrades, ...uniqueRestTrades].slice(0, 50);

  // Extract price history
  const priceHistory: number[] = priceHistoryData?.candles
    ? priceHistoryData.candles.slice(-50).map((c) => parseFloat(c.close))
    : [];

  return (
    <MarketPageContent
      market={market}
      orderBook={mergedOrderBook}
      orderBookLoading={orderBookLoading && !wsOrderBook}
      trades={mergedTrades}
      tradesLoading={tradesLoading && wsTrades.length === 0}
      relatedMarkets={relatedData?.markets ?? []}
      relatedMarketsLoading={relatedMarketsLoading}
      connectionState={connectionState}
      latency={latency}
      livePrices={
        wsPrices ? { yesPrice: wsPrices.yesPrice, noPrice: wsPrices.noPrice } : null
      }
      priceHistory={priceHistory}
      initialTab={initialTab}
    />
  );
};

export default MarketPage;
