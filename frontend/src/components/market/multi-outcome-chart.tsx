"use client";

import { useEffect, useRef, useState } from "react";
import {
  createChart,
  type IChartApi,
  type ISeriesApi,
  ColorType,
  LineSeries,
  type UTCTimestamp,
} from "lightweight-charts";
import { useQuery } from "@tanstack/react-query";
import { TrendingUp, Loader2 } from "lucide-react";
import { api } from "@/lib/api";

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

// Types for crosshair labels
interface SeriesInfo {
  series: ISeriesApi<"Line">;
  name: string;
  color: string;
  marketId: string;
  data: { time: number; value: number }[]; // Store data for interpolation
}

interface LabelData {
  name: string;
  color: string;
  value: number;
  y: number;
}

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
  const seriesMapRef = useRef<Map<string, SeriesInfo>>(new Map());
  const [timeframe, setTimeframe] = useState<TimeFrame>("30D");
  const [labels, setLabels] = useState<LabelData[]>([]);
  const [crosshairX, setCrosshairX] = useState<number | null>(null);
  const [overlayLeft, setOverlayLeft] = useState<number | null>(null);

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

    // Clean up previous chart and series map
    if (chartRef.current) {
      chartRef.current.remove();
      chartRef.current = null;
    }
    seriesMapRef.current.clear();
    setLabels([]);
    setCrosshairX(null);

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
        title: "", // Hide Y-axis labels - we have the legend + floating labels
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

      // Store series info for crosshair labels (including data for interpolation)
      seriesMapRef.current.set(outcome.market_id, {
        series,
        name: outcome.name,
        color: outcome.color,
        marketId: outcome.market_id,
        data: chartData.map((d) => ({ time: d.time as number, value: d.value })),
      });
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

    // Helper to find value at timestamp (exact match or interpolate)
    const findValueAtTime = (
      data: { time: number; value: number }[],
      targetTime: number
    ): number | null => {
      if (data.length === 0) return null;

      // Binary search for closest point
      let left = 0;
      let right = data.length - 1;

      // Handle edge cases
      if (targetTime <= data[0].time) return data[0].value;
      if (targetTime >= data[right].time) return data[right].value;

      while (left < right - 1) {
        const mid = Math.floor((left + right) / 2);
        if (data[mid].time === targetTime) return data[mid].value;
        if (data[mid].time < targetTime) left = mid;
        else right = mid;
      }

      // Interpolate between left and right
      const t1 = data[left].time;
      const t2 = data[right].time;
      const v1 = data[left].value;
      const v2 = data[right].value;
      const ratio = (targetTime - t1) / (t2 - t1);
      return v1 + (v2 - v1) * ratio;
    };

    // Subscribe to crosshair move for floating labels and overlay positioning
    chart.subscribeCrosshairMove((param) => {
      if (!param.point || !param.time || param.point.x < 0 || param.point.y < 0) {
        setLabels([]);
        setCrosshairX(null);
        setOverlayLeft(null);
        return;
      }

      setCrosshairX(param.point.x);
      setOverlayLeft(param.point.x);
      const newLabels: LabelData[] = [];
      const targetTime = param.time as number;

      seriesMapRef.current.forEach((info) => {
        // First try exact match from param.seriesData
        const exactData = param.seriesData.get(info.series);
        let value: number | null = null;

        if (exactData && "value" in exactData) {
          value = exactData.value;
        } else {
          // Fall back to interpolation
          value = findValueAtTime(info.data, targetTime);
        }

        if (value !== null) {
          const y = info.series.priceToCoordinate(value);
          if (y !== null) {
            newLabels.push({
              name: info.name,
              color: info.color,
              value,
              y,
            });
          }
        }
      });

      // Sort by Y position and add spacing to prevent overlap
      newLabels.sort((a, b) => a.y - b.y);
      const minSpacing = 24;
      for (let i = 1; i < newLabels.length; i++) {
        const prevY = newLabels[i - 1].y;
        if (newLabels[i].y - prevY < minSpacing) {
          newLabels[i].y = prevY + minSpacing;
        }
      }

      setLabels(newLabels);
    });

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
          <div className="relative">
            <div ref={chartContainerRef} className="w-full" style={{ height }} />
            {/* Gray overlay for area after cursor */}
            {overlayLeft !== null && (
              <div
                className="absolute pointer-events-none"
                style={{
                  left: overlayLeft,
                  top: 0,
                  right: 0,
                  height,
                  backgroundColor: "rgba(19, 20, 25, 0.75)",
                  zIndex: 5,
                }}
              />
            )}
            {/* Floating crosshair labels */}
            {crosshairX !== null &&
              labels.map((label, i) => (
                <div
                  key={i}
                  className="absolute pointer-events-none px-2 py-1 rounded text-xs font-semibold text-white whitespace-nowrap shadow-lg"
                  style={{
                    left: Math.min(crosshairX + 8, (chartContainerRef.current?.clientWidth ?? 300) - 150),
                    top: Math.max(4, Math.min(label.y, height - 24)),
                    transform: "translateY(-50%)",
                    backgroundColor: label.color,
                    zIndex: 10,
                  }}
                >
                  {label.name.length > 18 ? `${label.name.substring(0, 18)}…` : label.name}{" "}
                  {(label.value * 100).toFixed(1)}%
                </div>
              ))}
          </div>
        )}
      </div>
    </div>
  );
};
