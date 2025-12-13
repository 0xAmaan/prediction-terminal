"use client";

import { useEffect, useRef, useState } from "react";
import {
  createChart,
  type IChartApi,
  ColorType,
  CandlestickSeries,
  LineSeries,
  type CandlestickData,
  type LineData,
  type UTCTimestamp,
} from "lightweight-charts";
import { useQuery } from "@tanstack/react-query";
import { TrendingUp } from "lucide-react";
import { api } from "@/lib/api";
import type { PriceCandle } from "@/lib/types";

// Fey color tokens
const fey = {
  bg300: "#131419",
  bg400: "#16181C",
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  teal: "#4DBE95",
  red: "#D84F68",
  skyBlue: "#54BBF7",
  border: "rgba(255, 255, 255, 0.06)",
};

// ============================================================================
// Types
// ============================================================================

type ColorMode = "sentiment" | "price";
type ChartType = "candlestick" | "line";

interface PriceChartProps {
  /** Platform for fetching price history */
  platform?: string;
  /** Market ID for fetching price history */
  marketId?: string;
  /** Current YES price for real-time updates */
  currentPrice?: number;
  /** Chart height */
  height?: number;
  /** Loading state */
  isLoading?: boolean;
  /** Optional title */
  title?: string;
  /** Color mode: "sentiment" (buy/sell volume) or "price" (close vs open) */
  colorMode?: ColorMode;
  /** Chart type: "candlestick" (default) or "line" */
  chartType?: ChartType;
}

type TimeFrame = "1H" | "24H" | "7D" | "30D" | "ALL";

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Determine candle color based on mode
 * - sentiment: green if buy_volume >= sell_volume (net buying pressure)
 *              falls back to price mode if no volume data
 * - price: green if close >= open (price went up)
 */
const getCandleColor = (candle: PriceCandle, mode: ColorMode): string => {
  const close = parseFloat(candle.close);
  const open = parseFloat(candle.open);

  if (mode === "sentiment") {
    const buyVol = parseFloat(candle.buy_volume || "0");
    const sellVol = parseFloat(candle.sell_volume || "0");

    // If we have volume data, use it for coloring
    if (buyVol > 0 || sellVol > 0) {
      return buyVol >= sellVol ? fey.teal : fey.red;
    }
    // No volume data - fall back to price direction
    return close >= open ? fey.teal : fey.red;
  }
  // price mode
  return close >= open ? fey.teal : fey.red;
};

/**
 * Generate mock candlestick data for demo purposes
 */
const generateMockCandleData = (
  timeframe: TimeFrame,
  basePrice: number = 0.5
): CandlestickData<UTCTimestamp>[] => {
  const now = Math.floor(Date.now() / 1000);
  const candles: CandlestickData<UTCTimestamp>[] = [];

  let interval: number;
  let count: number;

  switch (timeframe) {
    case "1H":
      interval = 60;
      count = 60;
      break;
    case "24H":
      interval = 60 * 15;
      count = 96;
      break;
    case "7D":
      interval = 60 * 60;
      count = 168;
      break;
    case "30D":
      interval = 60 * 60 * 4;
      count = 180;
      break;
    case "ALL":
      interval = 60 * 60 * 24;
      count = 90;
      break;
  }

  let price = basePrice;
  for (let i = count; i >= 0; i--) {
    const open = price;
    const change = (Math.random() - 0.5) * 0.04;
    const reversion = (basePrice - price) * 0.1;
    price = Math.max(0.01, Math.min(0.99, price + change + reversion));
    const close = price;

    const high = Math.max(open, close) + Math.random() * 0.01;
    const low = Math.min(open, close) - Math.random() * 0.01;

    // Random buy/sell pressure for mock data
    const isBuyPressure = Math.random() > 0.5;
    const color = isBuyPressure ? fey.teal : fey.red;

    candles.push({
      time: (now - i * interval) as UTCTimestamp,
      open,
      high,
      low,
      close,
      color,
      wickColor: color,
    });
  }

  return candles;
};

/**
 * Transform API candle data to lightweight-charts candlestick format with colors
 * Creates continuous candlesticks where each candle's open = previous candle's close
 */
const transformCandleData = (
  candles: PriceCandle[],
  colorMode: ColorMode
): CandlestickData<UTCTimestamp>[] => {
  const seen = new Set<number>();

  // First pass: dedupe and sort
  const sortedCandles = candles
    .map((candle) => ({
      candle,
      time: Math.floor(new Date(candle.timestamp).getTime() / 1000),
    }))
    .filter((d) => {
      if (seen.has(d.time)) return false;
      seen.add(d.time);
      return true;
    })
    .sort((a, b) => a.time - b.time);

  // Second pass: create continuous candlesticks
  let prevClose: number | null = null;

  return sortedCandles.map(({ candle, time }) => {
    const color = getCandleColor(candle, colorMode);

    const originalOpen = parseFloat(candle.open);
    const originalHigh = parseFloat(candle.high);
    const originalLow = parseFloat(candle.low);
    const close = parseFloat(candle.close);

    // Make candles continuous: open = previous close
    const open = prevClose !== null ? prevClose : originalOpen;
    prevClose = close;

    // Adjust high/low to include the continuous open
    const high = Math.max(originalHigh, open, close);
    const low = Math.min(originalLow, open, close);

    return {
      time: time as UTCTimestamp,
      open,
      high,
      low,
      close,
      color,
      wickColor: color,
    };
  });
};

/**
 * Transform API candle data to lightweight-charts line format (close prices only)
 */
const transformLineData = (candles: PriceCandle[]): LineData<UTCTimestamp>[] => {
  const seen = new Set<number>();

  return candles
    .map((candle) => ({
      time: Math.floor(new Date(candle.timestamp).getTime() / 1000) as UTCTimestamp,
      value: parseFloat(candle.close),
    }))
    .filter((d) => {
      if (seen.has(d.time)) return false;
      seen.add(d.time);
      return true;
    })
    .sort((a, b) => a.time - b.time);
};

// ============================================================================
// Chart Component
// ============================================================================

export const PriceChart = ({
  platform,
  marketId,
  currentPrice,
  height,
  isLoading = false,
  title = "Price History",
  colorMode = "sentiment",
  chartType = "candlestick",
}: PriceChartProps) => {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const seriesRef = useRef<any>(null);
  const [timeframe, setTimeframe] = useState<TimeFrame>("30D");
  const [activeColorMode, setActiveColorMode] = useState<ColorMode>(colorMode);
  const [containerHeight, setContainerHeight] = useState<number>(height ?? 300);
  const isLineChart = chartType === "line";
  const wrapperRef = useRef<HTMLDivElement>(null);

  // Observe container size when height is not specified
  useEffect(() => {
    if (height !== undefined || !wrapperRef.current) return;

    const resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const newHeight = entry.contentRect.height;
        if (newHeight > 100) {
          // Subtract header height (~60px)
          setContainerHeight(Math.max(200, newHeight - 60));
        }
      }
    });

    resizeObserver.observe(wrapperRef.current);
    return () => resizeObserver.disconnect();
  }, [height]);

  // Fetch price history from API
  const { data: priceHistory, isLoading: historyLoading } = useQuery({
    queryKey: ["priceHistory", platform, marketId, timeframe],
    queryFn: () => api.getPriceHistory(platform!, marketId!, { timeframe }),
    enabled: !!platform && !!marketId,
    staleTime: 5 * 60 * 1000,
  });

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
      height: containerHeight,
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

    // Create series based on chart type
    if (isLineChart) {
      // Line chart for overview
      const series = chart.addSeries(LineSeries, {
        color: fey.teal,
        lineWidth: 2,
        priceFormat: {
          type: "custom",
          formatter: (price: number) => `${(price * 100).toFixed(1)}¢`,
        },
      });

      // Get line chart data
      if (priceHistory && priceHistory.candles.length > 0) {
        const lineData = transformLineData(priceHistory.candles);
        series.setData(lineData);
      }

      seriesRef.current = series;
    } else {
      // Candlestick chart for trading view
      const series = chart.addSeries(CandlestickSeries, {
        upColor: fey.teal,
        downColor: fey.red,
        wickUpColor: fey.teal,
        wickDownColor: fey.red,
        borderVisible: false,
        priceFormat: {
          type: "custom",
          formatter: (price: number) => `${(price * 100).toFixed(1)}¢`,
        },
      });

      // Get candlestick chart data
      let chartData: CandlestickData<UTCTimestamp>[];
      if (priceHistory && priceHistory.candles.length > 0) {
        chartData = transformCandleData(priceHistory.candles, activeColorMode);
      } else {
        chartData = generateMockCandleData(timeframe, currentPrice ?? 0.5);
      }

      series.setData(chartData);
      seriesRef.current = series;
    }
    chart.timeScale().fitContent();
    chartRef.current = chart;

    // Handle resize
    const handleResize = () => {
      if (chartContainerRef.current && chartRef.current) {
        chartRef.current.applyOptions({
          width: chartContainerRef.current.clientWidth,
          height: containerHeight,
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
  }, [priceHistory, containerHeight, isChartLoading, timeframe, currentPrice, activeColorMode, isLineChart]);

  // Update data when currentPrice changes (real-time updates)
  useEffect(() => {
    if (seriesRef.current && currentPrice && priceHistory?.candles.length) {
      const now = Math.floor(Date.now() / 1000) as UTCTimestamp;
      const lastCandle = priceHistory.candles[priceHistory.candles.length - 1];
      const lastClose = parseFloat(lastCandle?.close || "0");

      if (isLineChart) {
        // Line chart: simple value update
        seriesRef.current.update({
          time: now,
          value: currentPrice,
        });
      } else {
        // Candlestick: full OHLC update
        const color = currentPrice >= lastClose ? fey.teal : fey.red;
        seriesRef.current.update({
          time: now,
          open: lastClose,
          high: Math.max(lastClose, currentPrice),
          low: Math.min(lastClose, currentPrice),
          close: currentPrice,
          color,
          wickColor: color,
        });
      }
    }
  }, [currentPrice, priceHistory?.candles, isLineChart]);

  const timeframes: TimeFrame[] = ["1H", "24H", "7D", "30D", "ALL"];

  if (isChartLoading) {
    return (
      <div
        ref={wrapperRef}
        className="rounded-lg h-full flex flex-col"
        style={{
          backgroundColor: fey.bg300,
          border: `1px solid ${fey.border}`,
        }}
      >
        <div className="p-5 pb-2">
          <div className="flex items-center gap-2">
            <div
              className="p-1.5 rounded"
              style={{ backgroundColor: "rgba(84, 187, 247, 0.1)" }}
            >
              <TrendingUp className="h-4 w-4" style={{ color: fey.skyBlue }} />
            </div>
            <span
              className="text-base font-semibold"
              style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
            >
              {title}
            </span>
          </div>
        </div>
        <div className="px-5 pb-5 flex-1">
          <div
            className="animate-pulse rounded-lg h-full"
            style={{ minHeight: "200px", backgroundColor: fey.bg400 }}
          />
        </div>
      </div>
    );
  }

  return (
    <div
      ref={wrapperRef}
      className="rounded-lg h-full flex flex-col"
      style={{
        backgroundColor: fey.bg300,
        border: `1px solid ${fey.border}`,
      }}
    >
      <div className="p-5 pb-2 shrink-0">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div
              className="p-1.5 rounded"
              style={{ backgroundColor: "rgba(84, 187, 247, 0.1)" }}
            >
              <TrendingUp className="h-4 w-4" style={{ color: fey.skyBlue }} />
            </div>
            <span
              className="text-base font-semibold"
              style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
            >
              {title}
            </span>
          </div>
          <div className="flex items-center gap-3">
            {/* Color mode toggle - only show for candlestick charts */}
            {!isLineChart && (
              <div className="flex gap-1">
                <button
                  className="h-7 px-2.5 text-xs font-medium rounded transition-colors"
                  style={{
                    backgroundColor:
                      activeColorMode === "sentiment" ? "rgba(77, 190, 149, 0.1)" : "transparent",
                    color: activeColorMode === "sentiment" ? fey.teal : fey.grey500,
                  }}
                  onClick={() => setActiveColorMode("sentiment")}
                  title="Color by buy/sell pressure"
                >
                  Sentiment
                </button>
                <button
                  className="h-7 px-2.5 text-xs font-medium rounded transition-colors"
                  style={{
                    backgroundColor:
                      activeColorMode === "price" ? "rgba(84, 187, 247, 0.1)" : "transparent",
                    color: activeColorMode === "price" ? fey.skyBlue : fey.grey500,
                  }}
                  onClick={() => setActiveColorMode("price")}
                  title="Color by price direction"
                >
                  Price
                </button>
              </div>
            )}
            {/* Timeframe selector */}
            <div className="flex gap-1">
              {timeframes.map((tf) => (
                <button
                  key={tf}
                  className="h-7 px-2.5 text-xs font-medium rounded transition-colors"
                  style={{
                    backgroundColor: timeframe === tf ? "rgba(84, 187, 247, 0.1)" : "transparent",
                    color: timeframe === tf ? fey.skyBlue : fey.grey500,
                  }}
                  onClick={() => setTimeframe(tf)}
                >
                  {tf}
                </button>
              ))}
            </div>
          </div>
        </div>
      </div>
      <div className="px-0 pb-4 flex-1 min-h-0">
        <div ref={chartContainerRef} className="w-full h-full" />
      </div>
    </div>
  );
};
