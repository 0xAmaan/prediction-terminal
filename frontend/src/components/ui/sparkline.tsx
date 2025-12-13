"use client";

import { useMemo } from "react";
import type { PriceHistoryPoint } from "@/lib/types";

// Fey color tokens
const fey = {
  teal: "#4DBE95",
  red: "#D84F68",
  grey500: "#7D8B96",
};

// ============================================================================
// Types
// ============================================================================

interface SparklineProps {
  data: PriceHistoryPoint[];
  width?: number;
  height?: number;
  color?: string;
  strokeWidth?: number;
  showArea?: boolean;
  className?: string;
}

// ============================================================================
// Helper Functions
// ============================================================================

const getMinMax = (data: PriceHistoryPoint[]): { min: number; max: number } => {
  if (data.length === 0) return { min: 0, max: 1 };

  let min = data[0].p;
  let max = data[0].p;

  for (const point of data) {
    if (point.p < min) min = point.p;
    if (point.p > max) max = point.p;
  }

  // Add small padding to avoid flat lines at edges
  const range = max - min;
  const padding = range * 0.1 || 0.01;

  return { min: min - padding, max: max + padding };
};

const generatePath = (
  data: PriceHistoryPoint[],
  width: number,
  height: number,
  min: number,
  max: number
): string => {
  if (data.length === 0) return "";

  const range = max - min;
  const xStep = width / Math.max(data.length - 1, 1);

  const points = data.map((point, i) => {
    const x = i * xStep;
    const y = height - ((point.p - min) / range) * height;
    return `${x.toFixed(2)},${y.toFixed(2)}`;
  });

  return `M${points.join("L")}`;
};

const generateAreaPath = (
  data: PriceHistoryPoint[],
  width: number,
  height: number,
  min: number,
  max: number
): string => {
  if (data.length === 0) return "";

  const linePath = generatePath(data, width, height, min, max);
  const xEnd = ((data.length - 1) * width) / Math.max(data.length - 1, 1);

  // Close the path to create a filled area
  return `${linePath}L${xEnd.toFixed(2)},${height}L0,${height}Z`;
};

// ============================================================================
// Main Sparkline Component
// ============================================================================

export const Sparkline = ({
  data,
  width = 100,
  height = 32,
  color,
  strokeWidth = 1.5,
  showArea = true,
  className = "",
}: SparklineProps) => {
  // Determine color based on price direction (first vs last)
  const computedColor = useMemo(() => {
    if (color) return color;
    if (data.length < 2) return fey.grey500;

    const firstPrice = data[0].p;
    const lastPrice = data[data.length - 1].p;

    return lastPrice >= firstPrice ? fey.teal : fey.red;
  }, [data, color]);

  // Generate SVG paths
  const { linePath, areaPath } = useMemo(() => {
    const { min, max } = getMinMax(data);
    return {
      linePath: generatePath(data, width, height, min, max),
      areaPath: showArea ? generateAreaPath(data, width, height, min, max) : "",
    };
  }, [data, width, height, showArea]);

  if (data.length === 0) {
    return (
      <div
        className={`flex items-center justify-center ${className}`}
        style={{ width, height }}
      >
        <span className="text-xs text-[#7D8B96]">-</span>
      </div>
    );
  }

  return (
    <svg
      width={width}
      height={height}
      viewBox={`0 0 ${width} ${height}`}
      className={className}
      preserveAspectRatio="none"
    >
      {/* Gradient definition for area fill */}
      {showArea && (
        <defs>
          <linearGradient
            id={`sparkline-gradient-${computedColor.replace("#", "")}`}
            x1="0%"
            y1="0%"
            x2="0%"
            y2="100%"
          >
            <stop offset="0%" stopColor={computedColor} stopOpacity={0.2} />
            <stop offset="100%" stopColor={computedColor} stopOpacity={0} />
          </linearGradient>
        </defs>
      )}

      {/* Area fill */}
      {showArea && areaPath && (
        <path
          d={areaPath}
          fill={`url(#sparkline-gradient-${computedColor.replace("#", "")})`}
        />
      )}

      {/* Line */}
      {linePath && (
        <path
          d={linePath}
          fill="none"
          stroke={computedColor}
          strokeWidth={strokeWidth}
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      )}
    </svg>
  );
};

// ============================================================================
// Skeleton Sparkline (for loading state)
// ============================================================================

interface SparklineSkeletonProps {
  width?: number;
  height?: number;
  className?: string;
}

export const SparklineSkeleton = ({
  width = 100,
  height = 32,
  className = "",
}: SparklineSkeletonProps) => {
  return (
    <div
      className={`animate-pulse bg-white/5 rounded ${className}`}
      style={{ width, height }}
    />
  );
};

export default Sparkline;
