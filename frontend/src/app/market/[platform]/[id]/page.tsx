"use client";

import { useParams } from "next/navigation";
import { useQuery } from "@tanstack/react-query";
import Link from "next/link";
import { api } from "@/lib/api";
import type { Platform, PredictionMarket, MarketOption } from "@/lib/types";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { OrderBook } from "@/components/market/order-book";
import { TradeHistory } from "@/components/market/trade-history";
import { PriceChart } from "@/components/market/price-chart";
import { ConnectionIndicator } from "@/components/market/connection-indicator";
import { RelatedMarkets } from "@/components/market/related-markets";
import { MarketNewsSection } from "@/components/news";
import { MultiOutcomeChart } from "@/components/market/multi-outcome-chart";
import { OutcomeAccordion } from "@/components/market/outcome-accordion";
import { useMarketStream } from "@/hooks/use-market-stream";
import type { ConnectionState } from "@/hooks/use-websocket";
import {
  ArrowLeft,
  ExternalLink,
  Clock,
  TrendingUp,
  Activity,
  Calendar,
  Info,
  DollarSign,
} from "lucide-react";

// Format helpers (same as markets-table for consistency)
const formatPrice = (price: string): string => {
  const num = parseFloat(price);
  if (isNaN(num)) return "—";
  return `${(num * 100).toFixed(1)}¢`;
};

const formatPricePercent = (price: string): string => {
  const num = parseFloat(price);
  if (isNaN(num)) return "—";
  return `${(num * 100).toFixed(1)}%`;
};

const formatVolume = (volume: string): string => {
  const num = parseFloat(volume);
  if (isNaN(num) || num === 0) return "—";
  if (num >= 1_000_000) return `$${(num / 1_000_000).toFixed(2)}M`;
  if (num >= 1_000) return `$${(num / 1_000).toFixed(1)}K`;
  return `$${num.toFixed(0)}`;
};

const formatDate = (dateStr: string | null): string => {
  if (!dateStr) return "No end date";
  const date = new Date(dateStr);
  return date.toLocaleDateString("en-US", {
    weekday: "short",
    month: "short",
    day: "numeric",
    year: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });
};

const formatTicker = (ticker: string): string => {
  // Truncate long hex addresses (Polymarket) like 0x01d5...e569
  if (ticker.startsWith("0x") && ticker.length > 20) {
    return `${ticker.slice(0, 8)}...${ticker.slice(-4)}`;
  }
  return ticker;
};

const formatTimeRemaining = (dateStr: string | null): string => {
  if (!dateStr) return "—";
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = date.getTime() - now.getTime();

  if (diffMs < 0) return "Ended";

  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
  const diffHours = Math.floor((diffMs % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));

  if (diffDays === 0 && diffHours === 0) return "< 1 hour";
  if (diffDays === 0) return `${diffHours}h remaining`;
  if (diffDays === 1) return `1 day, ${diffHours}h`;
  if (diffDays < 7) return `${diffDays} days`;
  if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks`;
  return `${Math.floor(diffDays / 30)} months`;
};

const PlatformBadge = ({ platform }: { platform: Platform }) => {
  const isKalshi = platform === "kalshi";
  return (
    <Badge
      variant="outline"
      className={`${
        isKalshi
          ? "border-[#22c55e]/50 text-[#22c55e] bg-[#22c55e]/10"
          : "border-[#3b82f6]/50 text-[#3b82f6] bg-[#3b82f6]/10"
      } font-medium`}
    >
      {isKalshi ? "Kalshi" : "Polymarket"}
    </Badge>
  );
};

const StatusBadge = ({ status }: { status: string }) => {
  const statusConfig = {
    open: { color: "bg-green-500/20 text-green-400 border-green-500/30", label: "Open" },
    closed: { color: "bg-yellow-500/20 text-yellow-400 border-yellow-500/30", label: "Closed" },
    settled: { color: "bg-gray-500/20 text-gray-400 border-gray-500/30", label: "Settled" },
  };

  const config = statusConfig[status as keyof typeof statusConfig] || statusConfig.open;

  return (
    <Badge variant="outline" className={`${config.color} font-medium`}>
      <span className="h-1.5 w-1.5 rounded-full bg-current mr-1.5 animate-pulse" />
      {config.label}
    </Badge>
  );
};

const PriceCard = ({
  label,
  price,
  variant,
}: {
  label: string;
  price: string;
  variant: "yes" | "no";
}) => {
  const num = parseFloat(price);
  const isHigh = num >= 0.7;
  const isLow = num <= 0.3;

  return (
    <Card className="border-border/30">
      <CardContent className="p-4">
        <div className="text-sm text-muted-foreground mb-1">{label}</div>
        <div
          className={`text-3xl font-bold font-mono ${
            variant === "yes"
              ? isHigh
                ? "text-[#22c55e]"
                : isLow
                  ? "text-[#ef4444]"
                  : "text-foreground"
              : "text-muted-foreground"
          }`}
        >
          {formatPrice(price)}
        </div>
        <div className="text-sm text-muted-foreground mt-1">
          {formatPricePercent(price)} implied
        </div>
      </CardContent>
    </Card>
  );
};

interface MarketPageContentProps {
  market: PredictionMarket;
  orderBook: {
    yes_bids: Array<{ price: string; quantity: string; order_count: number | null }>;
    yes_asks: Array<{ price: string; quantity: string; order_count: number | null }>;
    no_bids: Array<{ price: string; quantity: string; order_count: number | null }>;
    no_asks: Array<{ price: string; quantity: string; order_count: number | null }>;
  } | null;
  orderBookLoading: boolean;
  trades: Array<{
    id: string;
    market_id: string;
    platform: string;
    timestamp: string;
    price: string;
    quantity: string;
    outcome: string;
    side: string | null;
  }>;
  tradesLoading: boolean;
  relatedMarkets: PredictionMarket[];
  relatedMarketsLoading: boolean;
  connectionState: ConnectionState;
  latency: number | null;
  livePrices: { yesPrice: string; noPrice: string } | null;
}

// Parse options from options_json
const parseOptions = (optionsJson: string | null): MarketOption[] => {
  if (!optionsJson) return [];
  try {
    return JSON.parse(optionsJson);
  } catch {
    return [];
  }
};

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
}: MarketPageContentProps) => {
  const platformColor = market.platform === "kalshi" ? "#22c55e" : "#3b82f6";

  // Use live prices from WebSocket if available, fallback to REST data
  const currentYesPrice = livePrices?.yesPrice ?? market.yes_price;
  const currentNoPrice = livePrices?.noPrice ?? market.no_price;

  // Parse multi-outcome options
  const options = parseOptions(market.options_json ?? null);
  const isMultiOutcome = market.is_multi_outcome && options.length > 0;

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b border-border/50 bg-card/50 backdrop-blur-xl sticky top-0 z-50">
        <div className="px-32 py-4">
          <div className="flex items-center gap-4">
            <Link
              href="/"
              className="p-2 rounded-lg hover:bg-secondary/50 transition-colors text-muted-foreground hover:text-foreground"
            >
              <ArrowLeft className="h-5 w-5" />
            </Link>
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-3 mb-1">
                <PlatformBadge platform={market.platform} />
                <StatusBadge status={market.status} />
                {market.category && (
                  <Badge variant="secondary" className="text-xs">
                    {market.category}
                  </Badge>
                )}
                <ConnectionIndicator state={connectionState} latency={latency} showLabel={true} />
              </div>
              <h1 className="text-xl font-semibold truncate">{market.title}</h1>
            </div>
            {market.url && (
              <a
                href={market.url}
                target="_blank"
                rel="noopener noreferrer"
                className="flex items-center gap-2 px-4 py-2 rounded-lg border border-border/50 hover:bg-secondary/50 transition-colors text-sm"
                style={{ borderColor: `${platformColor}40` }}
              >
                <span>View on {market.platform === "kalshi" ? "Kalshi" : "Polymarket"}</span>
                <ExternalLink className="h-4 w-4" />
              </a>
            )}
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="px-32 py-8">
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* Left Column - Main Info */}
          <div className="lg:col-span-2 space-y-6">
            {/* Price Cards - different display for multi-outcome */}
            {isMultiOutcome ? (
              <div className="space-y-6">
                {/* Multi-line chart showing top outcomes */}
                <MultiOutcomeChart
                  platform={market.platform}
                  marketId={market.id}
                  height={350}
                  title="Price History"
                  top={5}
                />

                {/* Expandable outcomes list with full detail */}
                <Card className="border-border/30">
                  <CardHeader className="pb-2">
                    <CardTitle className="text-base flex items-center gap-2">
                      <Activity className="h-4 w-4 text-primary" />
                      Outcomes ({options.length})
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <OutcomeAccordion
                      platform={market.platform}
                      eventId={market.id}
                      options={options}
                    />
                  </CardContent>
                </Card>
              </div>
            ) : (
              <div className="grid grid-cols-2 gap-4">
                <PriceCard label="Yes Price" price={currentYesPrice} variant="yes" />
                <PriceCard label="No Price" price={currentNoPrice} variant="no" />
              </div>
            )}

            {/* Price Chart - only show for binary markets */}
            {!isMultiOutcome && (
              <PriceChart
                platform={market.platform}
                marketId={market.id}
                currentPrice={parseFloat(currentYesPrice)}
                height={280}
                title="Price History"
              />
            )}

            {/* Order Book - only show for binary markets */}
            {!isMultiOutcome && (
              <OrderBook
                yesBids={orderBook?.yes_bids ?? []}
                yesAsks={orderBook?.yes_asks ?? []}
                noBids={orderBook?.no_bids ?? []}
                noAsks={orderBook?.no_asks ?? []}
                isLoading={orderBookLoading}
                maxLevels={10}
              />
            )}

          </div>

          {/* Right Column - Sidebar */}
          <div className="space-y-6">
            {/* Market Stats */}
            <Card className="border-border/30">
              <CardHeader className="pb-2">
                <CardTitle className="text-base">Market Stats</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground flex items-center gap-2">
                    <TrendingUp className="h-4 w-4" />
                    Volume
                  </span>
                  <span className="font-mono font-medium">{formatVolume(market.volume)}</span>
                </div>
                {market.liquidity && (
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground flex items-center gap-2">
                      <DollarSign className="h-4 w-4" />
                      Liquidity
                    </span>
                    <span className="font-mono font-medium">{formatVolume(market.liquidity)}</span>
                  </div>
                )}
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground flex items-center gap-2">
                    <Clock className="h-4 w-4" />
                    Time Left
                  </span>
                  <span className="font-medium">{formatTimeRemaining(market.close_time)}</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-sm text-muted-foreground flex items-center gap-2">
                    <Calendar className="h-4 w-4" />
                    Closes
                  </span>
                  <span className="text-sm">{formatDate(market.close_time)}</span>
                </div>
                {market.ticker && (
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-muted-foreground flex items-center gap-2">
                      <Activity className="h-4 w-4" />
                      Ticker
                    </span>
                    <span className="font-mono text-sm" title={market.ticker}>
                      {formatTicker(market.ticker)}
                    </span>
                  </div>
                )}
              </CardContent>
            </Card>

            {/* About This Market / Resolution */}
            {(market.description || market.resolution_source) && (
              <Card className="border-border/30">
                <CardHeader className="pb-2">
                  <CardTitle className="text-base flex items-center gap-2">
                    <Info className="h-4 w-4 text-primary" />
                    About This Market
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  {market.description && (
                    <p className="text-[15px] text-foreground/80 leading-relaxed whitespace-pre-wrap">
                      {market.description}
                    </p>
                  )}
                  {market.resolution_source && (
                    <div>
                      <div className="text-xs text-muted-foreground uppercase tracking-wide mb-1.5">
                        Resolution Source
                      </div>
                      <p className="text-[15px] text-foreground/70 leading-relaxed">
                        {market.resolution_source}
                      </p>
                    </div>
                  )}
                </CardContent>
              </Card>
            )}

            {/* Trade History - only show for binary markets */}
            {!isMultiOutcome && (
              <TradeHistory trades={trades} isLoading={tradesLoading} maxTrades={20} />
            )}

            {/* Related News */}
            <MarketNewsSection platform={market.platform} marketId={market.id} />

            {/* Related Markets */}
            <RelatedMarkets
              markets={relatedMarkets}
              currentMarketId={market.id}
              isLoading={relatedMarketsLoading}
              maxDisplay={5}
            />
          </div>
        </div>
      </main>
    </div>
  );
};

const LoadingSkeleton = () => (
  <div className="min-h-screen bg-background">
    <header className="border-b border-border/50 bg-card/50">
      <div className="px-32 py-4">
        <div className="flex items-center gap-4">
          <Skeleton className="h-9 w-9 rounded-lg" />
          <div className="flex-1">
            <div className="flex items-center gap-3 mb-2">
              <Skeleton className="h-5 w-16" />
              <Skeleton className="h-5 w-14" />
            </div>
            <Skeleton className="h-6 w-96" />
          </div>
        </div>
      </div>
    </header>
    <main className="px-32 py-8">
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        <div className="lg:col-span-2 space-y-6">
          <div className="grid grid-cols-2 gap-4">
            <Skeleton className="h-32 rounded-xl" />
            <Skeleton className="h-32 rounded-xl" />
          </div>
          <Skeleton className="h-80 rounded-xl" />
          <Skeleton className="h-64 rounded-xl" />
        </div>
        <div className="space-y-6">
          <Skeleton className="h-48 rounded-xl" />
          <Skeleton className="h-48 rounded-xl" />
          <Skeleton className="h-48 rounded-xl" />
        </div>
      </div>
    </main>
  </div>
);

const ErrorState = ({ message }: { message: string }) => (
  <div className="min-h-screen bg-background flex items-center justify-center">
    <Card className="max-w-md w-full mx-4">
      <CardContent className="p-8 text-center">
        <div className="h-12 w-12 rounded-full bg-destructive/20 flex items-center justify-center mx-auto mb-4">
          <Info className="h-6 w-6 text-destructive" />
        </div>
        <h2 className="text-lg font-semibold mb-2">Failed to Load Market</h2>
        <p className="text-muted-foreground mb-6">{message}</p>
        <Link
          href="/"
          className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors"
        >
          <ArrowLeft className="h-4 w-4" />
          Back to Markets
        </Link>
      </CardContent>
    </Card>
  </div>
);

const MarketPage = () => {
  const params = useParams();
  const platform = params.platform as Platform;
  const id = params.id as string;

  // Fetch market data
  const { data: market, isLoading, error } = useQuery({
    queryKey: ["market", platform, id],
    queryFn: () => api.getMarket(platform, id),
    enabled: !!platform && !!id,
    staleTime: 30 * 1000, // 30 seconds
    refetchInterval: 60 * 1000, // Refresh every minute
  });

  // Determine if this is a multi-outcome market (skip orderbook/trades for those)
  const isMultiOutcome = market?.is_multi_outcome ?? false;

  // Fetch order book (use ticker for Kalshi, id for Polymarket)
  // Skip for multi-outcome markets - they don't have single orderbooks
  const orderBookId = platform === "kalshi" ? market?.ticker : id;
  const { data: orderBook, isLoading: orderBookLoading } = useQuery({
    queryKey: ["orderbook", platform, orderBookId],
    queryFn: () => api.getOrderBook(platform, orderBookId!),
    enabled: !!platform && !!orderBookId && !isMultiOutcome,
    staleTime: 5 * 1000, // 5 seconds
    refetchInterval: 10 * 1000, // Refresh every 10 seconds
  });

  // Fetch trades
  // Skip for multi-outcome markets - they don't have single trade feeds
  const { data: tradesData, isLoading: tradesLoading } = useQuery({
    queryKey: ["trades", platform, orderBookId],
    queryFn: () => api.getTrades(platform, orderBookId!, 50),
    enabled: !!platform && !!orderBookId && !isMultiOutcome,
    staleTime: 5 * 1000,
    refetchInterval: 15 * 1000, // Refresh every 15 seconds
  });

  // Fetch related markets
  const { data: relatedData, isLoading: relatedMarketsLoading } = useQuery({
    queryKey: ["related", platform, id],
    queryFn: () => api.getRelatedMarkets(platform, id, 6),
    enabled: !!platform && !!id,
    staleTime: 60 * 1000, // 1 minute
  });

  // WebSocket streaming for live updates
  const wsMarketId = platform === "kalshi" ? market?.ticker ?? id : id;
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

  // Merge WebSocket data with REST data (WebSocket takes priority when available)
  const mergedOrderBook = wsOrderBook
    ? {
        yes_bids: wsOrderBook.yesBids,
        yes_asks: wsOrderBook.yesAsks,
        no_bids: wsOrderBook.noBids,
        no_asks: wsOrderBook.noAsks,
      }
    : orderBook ?? null;

  // Merge trades: WebSocket trades at the top, then REST trades (deduplicated)
  const restTrades = tradesData?.trades ?? [];
  const wsTradeIds = new Set(wsTrades.map((t) => t.id));
  const uniqueRestTrades = restTrades.filter((t) => !wsTradeIds.has(t.id));
  const mergedTrades = [...wsTrades, ...uniqueRestTrades].slice(0, 50);

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
      livePrices={wsPrices ? { yesPrice: wsPrices.yesPrice, noPrice: wsPrices.noPrice } : null}
    />
  );
};

export default MarketPage;
