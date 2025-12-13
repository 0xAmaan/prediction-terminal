"use client";

import { motion } from "framer-motion";
import {
  TrendingUp,
  TrendingDown,
  Minus,
  AlertCircle,
  CheckCircle,
  XCircle,
} from "lucide-react";
import type { MarketSentiment, SentimentSignal } from "@/hooks/use-market-sentiment";

// Fey color tokens
const fey = {
  bg300: "#131419",
  bg400: "#16181C",
  grey100: "#EEF0F1",
  grey300: "#B6BEC4",
  grey500: "#7D8B96",
  teal: "#4DBE95",
  tealMuted: "rgba(77, 190, 149, 0.15)",
  red: "#D84F68",
  redMuted: "rgba(216, 79, 104, 0.15)",
  skyBlue: "#54BBF7",
  amber: "#F5A524",
  border: "rgba(255, 255, 255, 0.06)",
};

// ============================================================================
// Types
// ============================================================================

interface SentimentGaugeProps {
  sentiment: MarketSentiment;
  showDetails?: boolean;
  compact?: boolean;
  className?: string;
}

// ============================================================================
// Helpers
// ============================================================================

const getSentimentColor = (score: number): string => {
  if (score >= 30) return fey.teal;
  if (score >= 10) return fey.teal;
  if (score <= -30) return fey.red;
  if (score <= -10) return fey.red;
  return fey.grey500;
};

const getSentimentBgColor = (score: number): string => {
  if (score >= 10) return fey.tealMuted;
  if (score <= -10) return fey.redMuted;
  return "rgba(125, 139, 150, 0.1)";
};

// ============================================================================
// Radial Gauge SVG
// ============================================================================

interface RadialGaugeSVGProps {
  score: number; // -100 to 100
  size?: number;
}

const RadialGaugeSVG = ({ score, size = 140 }: RadialGaugeSVGProps) => {
  const strokeWidth = 12;
  const radius = (size - strokeWidth) / 2;
  const center = size / 2;

  // Calculate arc parameters
  // We want a 270-degree arc (3/4 of a circle), starting from bottom-left
  const startAngle = 135; // degrees from top (clockwise)
  const sweepAngle = 270;
  const endAngle = startAngle + sweepAngle;

  // Convert score (-100 to 100) to angle
  const normalizedScore = (score + 100) / 200; // 0 to 1
  const currentAngle = startAngle + normalizedScore * sweepAngle;

  // Convert degrees to radians
  const toRad = (deg: number) => (deg * Math.PI) / 180;

  // Calculate arc path
  const startRad = toRad(startAngle - 90); // Adjust for SVG coordinate system
  const endRad = toRad(endAngle - 90);
  const currentRad = toRad(currentAngle - 90);

  const startX = center + radius * Math.cos(startRad);
  const startY = center + radius * Math.sin(startRad);
  const endX = center + radius * Math.cos(endRad);
  const endY = center + radius * Math.sin(endRad);
  const currentX = center + radius * Math.cos(currentRad);
  const currentY = center + radius * Math.sin(currentRad);

  // Large arc flag (1 if arc > 180 degrees)
  const largeArc = sweepAngle > 180 ? 1 : 0;
  const valueLargeArc = normalizedScore * sweepAngle > 180 ? 1 : 0;

  const color = getSentimentColor(score);

  return (
    <svg width={size} height={size} className="overflow-visible">
      {/* Background arc */}
      <path
        d={`M ${startX} ${startY} A ${radius} ${radius} 0 ${largeArc} 1 ${endX} ${endY}`}
        fill="none"
        stroke={fey.bg400}
        strokeWidth={strokeWidth}
        strokeLinecap="round"
      />

      {/* Gradient definition */}
      <defs>
        <linearGradient id="sentimentGradient" x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" stopColor={fey.red} />
          <stop offset="50%" stopColor={fey.grey500} />
          <stop offset="100%" stopColor={fey.teal} />
        </linearGradient>
      </defs>

      {/* Value arc */}
      <motion.path
        d={`M ${startX} ${startY} A ${radius} ${radius} 0 ${valueLargeArc} 1 ${currentX} ${currentY}`}
        fill="none"
        stroke={color}
        strokeWidth={strokeWidth}
        strokeLinecap="round"
        initial={{ pathLength: 0 }}
        animate={{ pathLength: 1 }}
        transition={{ type: "spring", stiffness: 50, damping: 15 }}
      />

      {/* Tick marks */}
      {[-100, -50, 0, 50, 100].map((tick, i) => {
        const tickNormalized = (tick + 100) / 200;
        const tickAngle = toRad(startAngle + tickNormalized * sweepAngle - 90);
        const innerRadius = radius - strokeWidth / 2 - 4;
        const outerRadius = radius - strokeWidth / 2 - 10;
        const x1 = center + innerRadius * Math.cos(tickAngle);
        const y1 = center + innerRadius * Math.sin(tickAngle);
        const x2 = center + outerRadius * Math.cos(tickAngle);
        const y2 = center + outerRadius * Math.sin(tickAngle);

        return (
          <line
            key={tick}
            x1={x1}
            y1={y1}
            x2={x2}
            y2={y2}
            stroke={fey.grey500}
            strokeWidth={tick === 0 ? 2 : 1}
            opacity={0.5}
          />
        );
      })}

      {/* Needle indicator */}
      <motion.circle
        cx={currentX}
        cy={currentY}
        r={6}
        fill={color}
        initial={{ scale: 0 }}
        animate={{ scale: 1 }}
        transition={{ type: "spring", stiffness: 200, damping: 15, delay: 0.2 }}
        style={{
          filter: `drop-shadow(0 0 6px ${color})`,
        }}
      />

      {/* Center score display */}
      <text
        x={center}
        y={center - 5}
        textAnchor="middle"
        fill={color}
        fontSize="28"
        fontWeight="bold"
        fontFamily="monospace"
      >
        <motion.tspan
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.3 }}
        >
          {score >= 0 ? "+" : ""}{score.toFixed(0)}
        </motion.tspan>
      </text>

      {/* Labels */}
      <text
        x={startX - 5}
        y={startY + 20}
        textAnchor="end"
        fill={fey.red}
        fontSize="10"
        fontWeight="500"
      >
        SELL
      </text>
      <text
        x={endX + 5}
        y={endY + 20}
        textAnchor="start"
        fill={fey.teal}
        fontSize="10"
        fontWeight="500"
      >
        BUY
      </text>
    </svg>
  );
};

// ============================================================================
// Signal Item Component
// ============================================================================

interface SignalItemProps {
  signal: SentimentSignal;
  index: number;
}

const SignalItem = ({ signal, index }: SignalItemProps) => {
  const Icon =
    signal.type === "bullish"
      ? CheckCircle
      : signal.type === "bearish"
        ? XCircle
        : AlertCircle;

  const color =
    signal.type === "bullish"
      ? fey.teal
      : signal.type === "bearish"
        ? fey.red
        : fey.grey500;

  return (
    <motion.div
      className="flex items-start gap-2 py-1"
      initial={{ opacity: 0, x: -10 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ delay: index * 0.05 }}
    >
      <Icon className="h-3.5 w-3.5 mt-0.5 flex-shrink-0" style={{ color }} />
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span
            className="text-[10px] uppercase tracking-wider font-medium"
            style={{ color }}
          >
            {signal.source}
          </span>
        </div>
        <span className="text-xs" style={{ color: fey.grey300 }}>
          {signal.description}
        </span>
      </div>
    </motion.div>
  );
};

// ============================================================================
// Component Breakdown (mini bars)
// ============================================================================

interface ComponentBreakdownProps {
  components: MarketSentiment["components"];
}

const ComponentBreakdown = ({ components }: ComponentBreakdownProps) => {
  const items = [
    { label: "Order Book", value: components.orderBookImbalance, weight: 30 },
    { label: "Trade Flow", value: components.tradeFlowMomentum, weight: 35 },
    { label: "Price Trend", value: components.priceTrend, weight: 20 },
    { label: "Volume", value: components.volumeActivity, weight: 15 },
  ];

  return (
    <div className="space-y-2">
      {items.map((item, i) => {
        const percent = (item.value + 100) / 2; // 0-100
        const color = item.value >= 10 ? fey.teal : item.value <= -10 ? fey.red : fey.grey500;

        return (
          <div key={item.label} className="space-y-1">
            <div className="flex items-center justify-between text-[10px]">
              <span style={{ color: fey.grey500 }}>
                {item.label}
                <span className="ml-1 opacity-50">({item.weight}%)</span>
              </span>
              <span className="font-mono" style={{ color }}>
                {item.value >= 0 ? "+" : ""}
                {item.value.toFixed(0)}
              </span>
            </div>
            <div
              className="h-1.5 rounded-full overflow-hidden"
              style={{ backgroundColor: fey.bg400 }}
            >
              <motion.div
                className="h-full rounded-full"
                style={{ backgroundColor: color }}
                initial={{ width: "50%" }}
                animate={{ width: `${percent}%` }}
                transition={{ type: "spring", stiffness: 100, damping: 20 }}
              />
            </div>
          </div>
        );
      })}
    </div>
  );
};

// ============================================================================
// Main Sentiment Gauge Component
// ============================================================================

export const SentimentGauge = ({
  sentiment,
  showDetails = true,
  compact = false,
  className = "",
}: SentimentGaugeProps) => {
  const { score, label, confidence, components, signals } = sentiment;

  const Icon =
    score >= 10 ? TrendingUp : score <= -10 ? TrendingDown : Minus;
  const color = getSentimentColor(score);

  if (compact) {
    return (
      <div className={`flex items-center gap-3 ${className}`}>
        <Icon className="h-4 w-4" style={{ color }} />
        <div>
          <span className="text-sm font-mono font-bold" style={{ color }}>
            {score >= 0 ? "+" : ""}{score.toFixed(0)}
          </span>
          <span className="text-xs ml-1" style={{ color: fey.grey500 }}>
            {label}
          </span>
        </div>
      </div>
    );
  }

  return (
    <div
      className={`rounded-lg overflow-hidden ${className}`}
      style={{
        backgroundColor: fey.bg300,
        border: `1px solid ${fey.border}`,
      }}
    >
      {/* Header */}
      <div
        className="px-4 py-3 flex items-center justify-between"
        style={{ borderBottom: `1px solid ${fey.border}` }}
      >
        <div className="flex items-center gap-2">
          <div
            className="p-1.5 rounded"
            style={{ backgroundColor: getSentimentBgColor(score) }}
          >
            <Icon className="h-4 w-4" style={{ color }} />
          </div>
          <span
            className="text-sm font-semibold"
            style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
          >
            Market Sentiment
          </span>
        </div>

        {/* Confidence badge */}
        <div
          className="px-2 py-0.5 rounded text-[10px] font-medium"
          style={{
            backgroundColor:
              confidence >= 0.7
                ? fey.tealMuted
                : confidence >= 0.4
                  ? "rgba(245, 165, 36, 0.15)"
                  : fey.redMuted,
            color:
              confidence >= 0.7
                ? fey.teal
                : confidence >= 0.4
                  ? fey.amber
                  : fey.red,
          }}
        >
          {(confidence * 100).toFixed(0)}% confidence
        </div>
      </div>

      {/* Gauge */}
      <div className="p-4 flex flex-col items-center">
        <RadialGaugeSVG score={score} size={140} />

        {/* Label */}
        <motion.div
          className="mt-2 px-3 py-1 rounded-full"
          style={{ backgroundColor: getSentimentBgColor(score) }}
          initial={{ scale: 0.9, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          transition={{ delay: 0.4 }}
        >
          <span className="text-sm font-semibold" style={{ color }}>
            {label}
          </span>
        </motion.div>
      </div>

      {/* Details */}
      {showDetails && (
        <>
          {/* Component Breakdown */}
          <div
            className="px-4 py-3"
            style={{ borderTop: `1px solid ${fey.border}` }}
          >
            <div
              className="text-[10px] uppercase tracking-wider mb-2"
              style={{ color: fey.grey500 }}
            >
              Component Scores
            </div>
            <ComponentBreakdown components={components} />
          </div>

          {/* Signals */}
          {signals.length > 0 && (
            <div
              className="px-4 py-3"
              style={{
                borderTop: `1px solid ${fey.border}`,
                backgroundColor: fey.bg400,
              }}
            >
              <div
                className="text-[10px] uppercase tracking-wider mb-2"
                style={{ color: fey.grey500 }}
              >
                Active Signals
              </div>
              <div className="space-y-1">
                {signals.slice(0, 4).map((signal, i) => (
                  <SignalItem key={`${signal.source}-${i}`} signal={signal} index={i} />
                ))}
              </div>
            </div>
          )}
        </>
      )}
    </div>
  );
};

// ============================================================================
// Mini Sentiment Badge (for headers/compact displays)
// ============================================================================

interface MiniSentimentProps {
  score: number;
  showLabel?: boolean;
}

export const MiniSentiment = ({ score, showLabel = true }: MiniSentimentProps) => {
  const color = getSentimentColor(score);
  const Icon = score >= 10 ? TrendingUp : score <= -10 ? TrendingDown : Minus;

  return (
    <div
      className="flex items-center gap-1.5 px-2 py-1 rounded"
      style={{ backgroundColor: getSentimentBgColor(score) }}
    >
      <Icon className="h-3 w-3" style={{ color }} />
      <span className="text-xs font-mono font-medium" style={{ color }}>
        {score >= 0 ? "+" : ""}{score.toFixed(0)}
      </span>
      {showLabel && (
        <span className="text-[10px]" style={{ color: fey.grey500 }}>
          sent
        </span>
      )}
    </div>
  );
};

export default SentimentGauge;
