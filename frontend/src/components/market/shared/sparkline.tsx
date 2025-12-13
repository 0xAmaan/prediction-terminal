"use client";

import { useMemo } from "react";

interface SparklineProps {
  data: number[];
  width?: number;
  height?: number;
  className?: string;
  strokeWidth?: number;
  /** If true, use green if trend is up, red if down. If false, use neutral color */
  colorByTrend?: boolean;
  /** Override color - takes precedence over colorByTrend */
  color?: string;
}

export const Sparkline = ({
  data,
  width = 60,
  height = 24,
  className = "",
  strokeWidth = 1.5,
  colorByTrend = true,
  color,
}: SparklineProps) => {
  const pathData = useMemo(() => {
    if (!data || data.length < 2) return "";

    const min = Math.min(...data);
    const max = Math.max(...data);
    const range = max - min || 1;

    // Padding to prevent clipping at edges
    const padding = 2;
    const chartWidth = width - padding * 2;
    const chartHeight = height - padding * 2;

    const points = data.map((value, index) => {
      const x = padding + (index / (data.length - 1)) * chartWidth;
      const y = padding + chartHeight - ((value - min) / range) * chartHeight;
      return `${x},${y}`;
    });

    return `M ${points.join(" L ")}`;
  }, [data, width, height]);

  const strokeColor = useMemo(() => {
    if (color) return color;
    if (!colorByTrend || data.length < 2) return "#7D8B96"; // Fey grey-500

    const first = data[0];
    const last = data[data.length - 1];
    // Fey colors: teal for up, red for down
    return last >= first ? "#4DBE95" : "#D84F68";
  }, [data, colorByTrend, color]);

  if (!data || data.length < 2) {
    return (
      <div
        className={`flex items-center justify-center text-muted-foreground text-xs ${className}`}
        style={{ width, height }}
      >
        —
      </div>
    );
  }

  return (
    <svg
      width={width}
      height={height}
      className={className}
      viewBox={`0 0 ${width} ${height}`}
    >
      <path
        d={pathData}
        fill="none"
        stroke={strokeColor}
        strokeWidth={strokeWidth}
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
};
