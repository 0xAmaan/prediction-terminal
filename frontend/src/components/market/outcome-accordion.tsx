"use client";

import { useState, useEffect, useRef } from "react";
import { useQuery } from "@tanstack/react-query";
import { createChart, type IChartApi, ColorType, LineSeries, type UTCTimestamp } from "lightweight-charts";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Badge } from "@/components/ui/badge";
import { Loader2, TrendingUp, TrendingDown, BookOpen, LineChart, History } from "lucide-react";
import { cn } from "@/lib/utils";
import { api } from "@/lib/api";
import { OrderBook } from "./order-book";
import { TradeHistory } from "./trade-history";
import type { MarketOption, Platform } from "@/lib/types";

interface OutcomeAccordionProps {
  platform: Platform;
  eventId: string;
  options: MarketOption[];
}

/** Format price as percentage (0-100%) */
const formatPrice = (price: string | number): string => {
  const p = typeof price === "string" ? parseFloat(price) : price;
  return `${(p * 100).toFixed(0)}%`;
};

export const OutcomeAccordion = ({
  platform,
  eventId,
  options,
}: OutcomeAccordionProps) => {
  const [expandedOutcome, setExpandedOutcome] = useState<string>("");

  // Sort options by price descending
  const sortedOptions = [...options].sort((a, b) => {
    const priceA = parseFloat(a.yes_price);
    const priceB = parseFloat(b.yes_price);
    return priceB - priceA;
  });

  return (
    <Accordion
      type="single"
      collapsible
      value={expandedOutcome}
      onValueChange={setExpandedOutcome}
      className="space-y-2"
    >
      {sortedOptions.map((option, index) => {
        const price = parseFloat(option.yes_price);
        const isLeading = index === 0;

        return (
          <AccordionItem
            key={option.market_id}
            value={option.market_id}
            className="border border-white/10 rounded-lg bg-black/10 overflow-hidden"
          >
            <AccordionTrigger className="px-4 py-3 hover:no-underline hover:bg-white/5 transition-colors">
              <div className="flex items-center justify-between w-full pr-2">
                <div className="flex items-center gap-3">
                  <span className="text-xs text-muted-foreground w-6">
                    #{index + 1}
                  </span>
                  <span className={cn(
                    "font-medium text-left",
                    isLeading && "text-primary"
                  )}>
                    {option.name}
                  </span>
                  {isLeading && (
                    <Badge variant="outline" className="text-xs bg-primary/10 border-primary/30">
                      Leading
                    </Badge>
                  )}
                </div>
                <div className="flex items-center gap-2">
                  {price >= 0.5 ? (
                    <TrendingUp className="h-3.5 w-3.5 text-green-500" />
                  ) : (
                    <TrendingDown className="h-3.5 w-3.5 text-red-500" />
                  )}
                  <span className={cn(
                    "font-mono text-sm font-medium min-w-[40px] text-right",
                    price >= 0.5 ? "text-green-500" : "text-red-500"
                  )}>
                    {formatPrice(option.yes_price)}
                  </span>
                </div>
              </div>
            </AccordionTrigger>
            <AccordionContent className="px-4 pb-4">
              {/* Only fetch data when expanded */}
              {expandedOutcome === option.market_id && (
                <OutcomeDetail
                  platform={platform}
                  eventId={eventId}
                  option={option}
                />
              )}
            </AccordionContent>
          </AccordionItem>
        );
      })}
    </Accordion>
  );
};

// ============================================================================
// OutcomeDetail - Displayed when an outcome is expanded (with tabs)
// ============================================================================

interface OutcomeDetailProps {
  platform: Platform;
  eventId: string;
  option: MarketOption;
}

const OutcomeDetail = ({ platform, eventId, option }: OutcomeDetailProps) => {
  const hasError = !option.clob_token_id || !option.condition_id;

  if (hasError) {
    return (
      <div className="py-8 text-center text-muted-foreground">
        <p className="text-sm">Detailed data not available for this outcome.</p>
        <p className="text-xs mt-1">Missing token or condition ID from API.</p>
      </div>
    );
  }

  return (
    <div className="pt-2">
      <Tabs defaultValue="orderbook" className="w-full">
        <TabsList className="grid w-full grid-cols-3 bg-black/20">
          <TabsTrigger value="orderbook" className="flex items-center gap-2 data-[state=active]:bg-white/10">
            <BookOpen className="h-3.5 w-3.5" />
            Order Book
          </TabsTrigger>
          <TabsTrigger value="chart" className="flex items-center gap-2 data-[state=active]:bg-white/10">
            <LineChart className="h-3.5 w-3.5" />
            Chart
          </TabsTrigger>
          <TabsTrigger value="trades" className="flex items-center gap-2 data-[state=active]:bg-white/10">
            <History className="h-3.5 w-3.5" />
            Trades
          </TabsTrigger>
        </TabsList>

        <TabsContent value="orderbook" className="mt-4">
          <OrderBookTab platform={platform} eventId={eventId} option={option} />
        </TabsContent>

        <TabsContent value="chart" className="mt-4">
          <ChartTab platform={platform} eventId={eventId} option={option} />
        </TabsContent>

        <TabsContent value="trades" className="mt-4">
          <TradesTab platform={platform} eventId={eventId} option={option} />
        </TabsContent>
      </Tabs>
    </div>
  );
};

// ============================================================================
// Tab Components
// ============================================================================

const OrderBookTab = ({ platform, eventId, option }: OutcomeDetailProps) => {
  const {
    data: orderbook,
    isLoading,
    error,
  } = useQuery({
    queryKey: ["outcome-orderbook", platform, eventId, option.clob_token_id],
    queryFn: () => api.getOutcomeOrderBook(platform, eventId, option.clob_token_id!),
    enabled: !!option.clob_token_id,
    staleTime: 10 * 1000,
    refetchInterval: 15 * 1000,
  });

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12 text-muted-foreground">
        <Loader2 className="h-5 w-5 animate-spin mr-2" />
        Loading order book...
      </div>
    );
  }

  if (error || !orderbook) {
    return (
      <div className="text-center py-12 text-muted-foreground text-sm">
        Failed to load order book
      </div>
    );
  }

  return (
    <OrderBook
      yesBids={orderbook.yes_bids}
      yesAsks={orderbook.yes_asks}
      noBids={orderbook.no_bids}
      noAsks={orderbook.no_asks}
      isLoading={false}
      maxLevels={10}
      showNoSide={false}
    />
  );
};

const ChartTab = ({ platform, eventId, option }: OutcomeDetailProps) => {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);

  const {
    data: priceHistory,
    isLoading,
    error,
  } = useQuery({
    queryKey: ["outcome-prices", platform, eventId, option.clob_token_id],
    queryFn: () => api.getOutcomePriceHistory(platform, eventId, option.clob_token_id!, "1w"),
    enabled: !!option.clob_token_id,
    staleTime: 60 * 1000,
  });

  useEffect(() => {
    if (!chartContainerRef.current || !priceHistory || priceHistory.length === 0) return;

    // Clean up previous chart
    if (chartRef.current) {
      chartRef.current.remove();
      chartRef.current = null;
    }

    const chart = createChart(chartContainerRef.current, {
      layout: {
        background: { type: ColorType.Solid, color: "transparent" },
        textColor: "#9ca3af",
        fontFamily: "Inter, system-ui, sans-serif",
      },
      grid: {
        vertLines: { color: "rgba(255, 255, 255, 0.05)" },
        horzLines: { color: "rgba(255, 255, 255, 0.05)" },
      },
      width: chartContainerRef.current.clientWidth,
      height: 250,
      timeScale: {
        borderColor: "rgba(255, 255, 255, 0.1)",
        timeVisible: true,
        secondsVisible: false,
      },
      rightPriceScale: {
        borderColor: "rgba(255, 255, 255, 0.1)",
        scaleMargins: { top: 0.1, bottom: 0.1 },
      },
      crosshair: {
        mode: 1,
        vertLine: { color: "rgba(255, 255, 255, 0.3)", width: 1, style: 2 },
        horzLine: { color: "rgba(255, 255, 255, 0.3)", width: 1, style: 2 },
      },
    });

    chartRef.current = chart;

    const series = chart.addSeries(LineSeries, {
      color: "#22c55e",
      lineWidth: 2,
      priceFormat: {
        type: "custom",
        formatter: (price: number) => `${(price * 100).toFixed(0)}%`,
      },
    });

    const chartData = priceHistory.map((point) => ({
      time: point.t as UTCTimestamp,
      value: point.p,
    }));

    series.setData(chartData);
    chart.timeScale().fitContent();

    const handleResize = () => {
      if (chartContainerRef.current && chartRef.current) {
        chartRef.current.applyOptions({
          width: chartContainerRef.current.clientWidth,
        });
      }
    };

    window.addEventListener("resize", handleResize);

    return () => {
      window.removeEventListener("resize", handleResize);
      if (chartRef.current) {
        chartRef.current.remove();
        chartRef.current = null;
      }
    };
  }, [priceHistory]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12 text-muted-foreground" style={{ height: 250 }}>
        <Loader2 className="h-5 w-5 animate-spin mr-2" />
        Loading price history...
      </div>
    );
  }

  if (error || !priceHistory || priceHistory.length === 0) {
    return (
      <div className="text-center py-12 text-muted-foreground text-sm" style={{ height: 250 }}>
        No price history available
      </div>
    );
  }

  return <div ref={chartContainerRef} className="w-full" style={{ height: 250 }} />;
};

const TradesTab = ({ platform, eventId, option }: OutcomeDetailProps) => {
  const {
    data: trades,
    isLoading,
    error,
  } = useQuery({
    queryKey: ["outcome-trades", platform, eventId, option.condition_id],
    queryFn: () => api.getOutcomeTrades(platform, eventId, option.condition_id!, 30),
    enabled: !!option.condition_id,
    staleTime: 10 * 1000,
    refetchInterval: 15 * 1000,
  });

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12 text-muted-foreground">
        <Loader2 className="h-5 w-5 animate-spin mr-2" />
        Loading trades...
      </div>
    );
  }

  if (error || !trades) {
    return (
      <div className="text-center py-12 text-muted-foreground text-sm">
        Failed to load trades
      </div>
    );
  }

  return (
    <TradeHistory
      trades={trades.trades}
      isLoading={false}
      maxTrades={20}
    />
  );
};

export default OutcomeAccordion;
