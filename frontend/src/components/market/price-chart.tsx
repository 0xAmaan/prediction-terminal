"use client";

import { useEffect, useRef, useState } from "react";
import { createChart, type IChartApi, ColorType, AreaSeries } from "lightweight-charts";
import { useQuery } from "@tanstack/react-query";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { TrendingUp } from "lucide-react";
import { cn } from "@/lib/utils";
import { api } from "@/lib/api";

// ============================================================================
// Types
// ============================================================================

interface PriceDataPoint {
  time: number; // Unix timestamp in seconds
  value: number;
}

interface PriceChartProps {
  /** Platform for fetching price history */
  platform?: string;
  /** Market ID for fetching price history */
  marketId?: string;
  /** Price history data for YES outcome (overrides API fetch if provided) */
  data?: PriceDataPoint[];
  /** Current YES price for real-time updates */
  currentPrice?: number;
  /** Chart height */
  height?: number;
  /** Loading state */
  isLoading?: boolean;
  /** Optional title */
  title?: string;
}

type TimeFrame = "1H" | "24H" | "7D" | "30D" | "ALL";

// ============================================================================
// Mock Data Generator (for demo purposes when no data is available)
// ============================================================================

const generateMockData = (timeframe: TimeFrame, basePrice: number = 0.5): PriceDataPoint[] => {
  const now = Math.floor(Date.now() / 1000);
  const points: PriceDataPoint[] = [];

  let interval: number;
  let count: number;

  switch (timeframe) {
    case "1H":
      interval = 60; // 1 minute
      count = 60;
      break;
    case "24H":
      interval = 60 * 15; // 15 minutes
      count = 96;
      break;
    case "7D":
      interval = 60 * 60; // 1 hour
      count = 168;
      break;
    case "30D":
      interval = 60 * 60 * 4; // 4 hours
      count = 180;
      break;
    case "ALL":
      interval = 60 * 60 * 24; // 1 day
      count = 90;
      break;
  }

  let price = basePrice;
  for (let i = count; i >= 0; i--) {
    // Random walk with mean reversion
    const change = (Math.random() - 0.5) * 0.02;
    const reversion = (basePrice - price) * 0.1;
    price = Math.max(0.01, Math.min(0.99, price + change + reversion));

    points.push({
      time: now - i * interval,
      value: price,
    });
  }

  return points;
};

// ============================================================================
// Chart Component
// ============================================================================

export const PriceChart = ({
  platform,
  marketId,
  data,
  currentPrice,
  height = 300,
  isLoading = false,
  title = "Price History",
}: PriceChartProps) => {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const seriesRef = useRef<any>(null);
  const [timeframe, setTimeframe] = useState<TimeFrame>("24H");
  const [chartData, setChartData] = useState<PriceDataPoint[]>([]);

  // Fetch price history from API
  const { data: priceHistory, isLoading: historyLoading } = useQuery({
    queryKey: ["priceHistory", platform, marketId, timeframe],
    queryFn: () => api.getPriceHistory(platform!, marketId!, { timeframe }),
    enabled: !!platform && !!marketId,
    staleTime: 30000, // Consider data fresh for 30 seconds
    refetchInterval: 60000, // Refetch every minute
  });

  // Generate chart data based on API response or fallback to props/mock
  useEffect(() => {
    // If API returned data, use it
    if (priceHistory && priceHistory.candles.length > 0) {
      const apiData: PriceDataPoint[] = priceHistory.candles.map((candle) => ({
        time: Math.floor(new Date(candle.timestamp).getTime() / 1000),
        value: parseFloat(candle.close),
      }));
      setChartData(apiData);
    } else if (data && data.length > 0) {
      // Use provided data
      setChartData(data);
    } else if (currentPrice) {
      // Generate mock data for demo
      setChartData(generateMockData(timeframe, currentPrice));
    } else {
      setChartData(generateMockData(timeframe, 0.5));
    }
  }, [priceHistory, data, currentPrice, timeframe]);

  // Combined loading state
  const isChartLoading = isLoading || historyLoading;

  // Initialize chart
  useEffect(() => {
    if (!chartContainerRef.current || isChartLoading) return;

    const chart = createChart(chartContainerRef.current, {
      layout: {
        background: { type: ColorType.Solid, color: "transparent" },
        textColor: "#9ca3af",
        fontFamily: "'Inter', sans-serif",
      },
      grid: {
        vertLines: { color: "rgba(255, 255, 255, 0.03)" },
        horzLines: { color: "rgba(255, 255, 255, 0.03)" },
      },
      width: chartContainerRef.current.clientWidth,
      height: height,
      rightPriceScale: {
        borderColor: "rgba(255, 255, 255, 0.1)",
        scaleMargins: {
          top: 0.1,
          bottom: 0.1,
        },
      },
      timeScale: {
        borderColor: "rgba(255, 255, 255, 0.1)",
        timeVisible: true,
        secondsVisible: false,
      },
      crosshair: {
        vertLine: {
          color: "rgba(255, 255, 255, 0.2)",
          width: 1,
          style: 2,
        },
        horzLine: {
          color: "rgba(255, 255, 255, 0.2)",
          width: 1,
          style: 2,
        },
      },
      handleScroll: {
        mouseWheel: true,
        pressedMouseMove: true,
      },
      handleScale: {
        axisPressedMouseMove: true,
        mouseWheel: true,
        pinch: true,
      },
    });

    // Determine line color based on price trend
    const firstPrice = chartData[0]?.value ?? 0.5;
    const lastPrice = chartData[chartData.length - 1]?.value ?? 0.5;
    const isUp = lastPrice >= firstPrice;
    const lineColor = isUp ? "#22c55e" : "#ef4444";

    const series = chart.addSeries(AreaSeries, {
      lineColor: lineColor,
      topColor: isUp ? "rgba(34, 197, 94, 0.3)" : "rgba(239, 68, 68, 0.3)",
      bottomColor: isUp ? "rgba(34, 197, 94, 0.0)" : "rgba(239, 68, 68, 0.0)",
      lineWidth: 2,
      priceFormat: {
        type: "custom",
        formatter: (price: number) => `${(price * 100).toFixed(1)}¢`,
      },
    });

    // Format data for the chart
    const formattedData = chartData.map((d) => ({
      time: d.time as any,
      value: d.value,
    }));

    series.setData(formattedData);
    chart.timeScale().fitContent();

    chartRef.current = chart;
    seriesRef.current = series;

    // Handle resize
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
      chart.remove();
      chartRef.current = null;
      seriesRef.current = null;
    };
  }, [chartData, height, isChartLoading]);

  // Update data when currentPrice changes (real-time updates)
  useEffect(() => {
    if (seriesRef.current && currentPrice && chartData.length > 0) {
      const now = Math.floor(Date.now() / 1000);
      seriesRef.current.update({
        time: now as any,
        value: currentPrice,
      });
    }
  }, [currentPrice, chartData.length]);

  const timeframes: TimeFrame[] = ["1H", "24H", "7D", "30D", "ALL"];

  if (isChartLoading) {
    return (
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-base flex items-center gap-2">
            <TrendingUp className="h-4 w-4 text-primary" />
            {title}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div
            className="animate-pulse bg-muted rounded-lg"
            style={{ height: `${height}px` }}
          />
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-base flex items-center gap-2">
            <TrendingUp className="h-4 w-4 text-primary" />
            {title}
          </CardTitle>
          <div className="flex gap-1">
            {timeframes.map((tf) => (
              <Button
                key={tf}
                variant="ghost"
                size="sm"
                className={cn(
                  "h-7 px-2 text-xs",
                  timeframe === tf
                    ? "bg-primary/10 text-primary"
                    : "text-muted-foreground"
                )}
                onClick={() => setTimeframe(tf)}
              >
                {tf}
              </Button>
            ))}
          </div>
        </div>
      </CardHeader>
      <CardContent className="p-0 pb-4">
        <div ref={chartContainerRef} className="w-full" />
      </CardContent>
    </Card>
  );
};
