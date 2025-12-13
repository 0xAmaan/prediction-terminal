"use client";

import { useEffect, useRef, useState } from "react";
import { createChart, type IChartApi, ColorType, LineSeries, type UTCTimestamp } from "lightweight-charts";
import { useQuery } from "@tanstack/react-query";
import { TrendingUp, Loader2 } from "lucide-react";
import { api } from "@/lib/api";
import type { OutcomePriceHistory } from "@/lib/types";

// Fey color tokens
const fey = {
  bg300: "#131419",
  bg400: "#16181C",
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  skyBlue: "#54BBF7",
  border: "rgba(255, 255, 255, 0.06)",
};

type TimeFrame = "1H" | "24H" | "7D" | "30D" | "ALL";

const TIMEFRAME_TO_INTERVAL: Record<TimeFrame, string> = {
  "1H": "1h",
  "24H": "1d",
  "7D": "1w",
  "30D": "max",
  "ALL": "max",
};

interface MultiOutcomeChartProps {
  platform: string;
  marketId: string;
  height?: number;
  title?: string;
  top?: number;
}

export const MultiOutcomeChart = ({
  platform,
  marketId,
  height = 350,
  title = "Price History",
  top = 5,
}: MultiOutcomeChartProps) => {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const [timeframe, setTimeframe] = useState<TimeFrame>("7D");

  const { data: outcomes, isLoading, error } = useQuery({
    queryKey: ["multi-outcome-prices", platform, marketId, timeframe, top],
    queryFn: () =>
      api.getMultiOutcomePrices(platform, marketId, {
        top,
        interval: TIMEFRAME_TO_INTERVAL[timeframe],
      }),
    staleTime: 5 * 60 * 1000, // Price history doesn't change rapidly
  });

  useEffect(() => {
    if (!chartContainerRef.current || !outcomes || outcomes.length === 0) return;

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
      height,
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
        vertLine: {
          color: "rgba(255, 255, 255, 0.3)",
          width: 1,
          style: 2,
        },
        horzLine: {
          color: "rgba(255, 255, 255, 0.3)",
          width: 1,
          style: 2,
        },
      },
      handleScroll: { mouseWheel: true, pressedMouseMove: true },
      handleScale: { axisPressedMouseMove: true, mouseWheel: true, pinch: true },
    });

    chartRef.current = chart;

    // Add a line series for each outcome
    outcomes.forEach((outcome) => {
      const series = chart.addSeries(LineSeries, {
        color: outcome.color,
        lineWidth: 2,
        title: outcome.name.substring(0, 20), // Truncate long names
        priceFormat: {
          type: "custom",
          formatter: (price: number) => `${(price * 100).toFixed(0)}%`,
        },
      });

      // Convert price history to chart data
      const chartData = outcome.history.map((point) => ({
        time: point.t as UTCTimestamp,
        value: point.p,
      }));

      if (chartData.length > 0) {
        series.setData(chartData);
      }
    });

    chart.timeScale().fitContent();

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
      if (chartRef.current) {
        chartRef.current.remove();
        chartRef.current = null;
      }
    };
  }, [outcomes, height]);

  const timeframes: TimeFrame[] = ["1H", "24H", "7D", "30D", "ALL"];

  return (
    <div
      className="rounded-lg"
      style={{
        backgroundColor: fey.bg300,
        border: `1px solid ${fey.border}`,
      }}
    >
      <div className="p-5 pb-2">
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
        {/* Legend */}
        {outcomes && outcomes.length > 0 && (
          <div className="flex flex-wrap gap-3 mt-3">
            {outcomes.map((outcome) => (
              <div key={outcome.market_id} className="flex items-center gap-1.5">
                <div
                  className="w-2.5 h-2.5 rounded-full"
                  style={{ backgroundColor: outcome.color }}
                />
                <span
                  className="text-xs truncate max-w-[120px]"
                  style={{ color: fey.grey500 }}
                >
                  {outcome.name}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
      <div className="pt-0 px-5 pb-5">
        {isLoading ? (
          <div
            className="flex items-center justify-center"
            style={{ height, color: fey.grey500 }}
          >
            <Loader2 className="h-6 w-6 animate-spin mr-2" />
            Loading price history...
          </div>
        ) : error ? (
          <div
            className="flex items-center justify-center"
            style={{ height, color: fey.grey500 }}
          >
            Failed to load price history
          </div>
        ) : !outcomes || outcomes.length === 0 ? (
          <div
            className="flex items-center justify-center"
            style={{ height, color: fey.grey500 }}
          >
            No price history available
          </div>
        ) : (
          <div ref={chartContainerRef} className="w-full" style={{ height }} />
        )}
      </div>
    </div>
  );
};
