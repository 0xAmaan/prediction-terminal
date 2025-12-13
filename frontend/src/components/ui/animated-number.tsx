"use client";

import { useEffect, useRef, useState } from "react";
import { motion, AnimatePresence, useSpring, useTransform } from "framer-motion";

// Fey color tokens
const fey = {
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  teal: "#4DBE95",
  red: "#D84F68",
};

// ============================================================================
// Types
// ============================================================================

interface AnimatedNumberProps {
  value: number;
  format?: "percent" | "price" | "currency" | "decimal" | "integer";
  decimals?: number;
  prefix?: string;
  suffix?: string;
  colorByChange?: boolean;
  className?: string;
  style?: React.CSSProperties;
  duration?: number;
}

// ============================================================================
// Formatters
// ============================================================================

const formatNumber = (
  value: number,
  format: AnimatedNumberProps["format"],
  decimals: number,
  prefix: string,
  suffix: string
): string => {
  let formatted: string;

  switch (format) {
    case "percent":
      formatted = `${(value * 100).toFixed(decimals)}%`;
      break;
    case "price":
      formatted = `${(value * 100).toFixed(decimals)}¢`;
      break;
    case "currency":
      formatted = new Intl.NumberFormat("en-US", {
        style: "currency",
        currency: "USD",
        minimumFractionDigits: decimals,
        maximumFractionDigits: decimals,
      }).format(value);
      break;
    case "integer":
      formatted = Math.round(value).toLocaleString();
      break;
    case "decimal":
    default:
      formatted = value.toFixed(decimals);
  }

  return `${prefix}${formatted}${suffix}`;
};

// ============================================================================
// Animated Digit Component (for flip animation)
// ============================================================================

interface AnimatedDigitProps {
  digit: string;
  isChanging: boolean;
}

const AnimatedDigit = ({ digit, isChanging }: AnimatedDigitProps) => {
  return (
    <span className="relative inline-block overflow-hidden" style={{ height: "1.2em", lineHeight: "1.2em" }}>
      <AnimatePresence mode="popLayout">
        <motion.span
          key={digit}
          initial={isChanging ? { y: "100%", opacity: 0 } : false}
          animate={{ y: 0, opacity: 1 }}
          exit={{ y: "-100%", opacity: 0 }}
          transition={{
            type: "spring",
            stiffness: 500,
            damping: 30,
            mass: 0.8,
          }}
          className="inline-block"
        >
          {digit}
        </motion.span>
      </AnimatePresence>
    </span>
  );
};

// ============================================================================
// Main Animated Number Component
// ============================================================================

export const AnimatedNumber = ({
  value,
  format = "decimal",
  decimals = 1,
  prefix = "",
  suffix = "",
  colorByChange = false,
  className = "",
  style = {},
  duration = 0.5,
}: AnimatedNumberProps) => {
  const prevValueRef = useRef(value);
  const [displayValue, setDisplayValue] = useState(value);
  const [changeDirection, setChangeDirection] = useState<"up" | "down" | null>(null);

  // Animated spring value for smooth number transitions
  const springValue = useSpring(value, {
    stiffness: 100,
    damping: 20,
    duration: duration * 1000,
  });

  // Update spring value when value changes
  useEffect(() => {
    const prev = prevValueRef.current;

    if (value !== prev) {
      setChangeDirection(value > prev ? "up" : "down");
      prevValueRef.current = value;
      springValue.set(value);

      // Clear change direction after animation
      const timer = setTimeout(() => {
        setChangeDirection(null);
      }, 600);

      return () => clearTimeout(timer);
    }
  }, [value, springValue]);

  // Subscribe to spring value changes
  useEffect(() => {
    const unsubscribe = springValue.on("change", (v) => {
      setDisplayValue(v);
    });
    return unsubscribe;
  }, [springValue]);

  // Get color based on change direction
  const getColor = (): string => {
    if (!colorByChange || !changeDirection) {
      return style.color as string || fey.grey100;
    }
    return changeDirection === "up" ? fey.teal : fey.red;
  };

  // Format the display value
  const formattedValue = formatNumber(displayValue, format, decimals, prefix, suffix);

  // Flash background animation
  const flashColor =
    changeDirection === "up"
      ? "rgba(77, 190, 149, 0.2)"
      : changeDirection === "down"
        ? "rgba(216, 79, 104, 0.2)"
        : "transparent";

  return (
    <motion.span
      className={`font-mono tabular-nums inline-flex ${className}`}
      style={{
        ...style,
        color: colorByChange ? getColor() : style.color,
      }}
      animate={{
        backgroundColor: changeDirection
          ? [flashColor, "transparent"]
          : "transparent",
      }}
      transition={{ duration: 0.5 }}
    >
      {formattedValue.split("").map((char, i) => (
        <AnimatedDigit
          key={`${i}-${char}`}
          digit={char}
          isChanging={!!changeDirection && !isNaN(parseInt(char))}
        />
      ))}
    </motion.span>
  );
};

// ============================================================================
// Simple Animated Counter (for large numbers with spring animation only)
// ============================================================================

interface AnimatedCounterProps {
  value: number;
  className?: string;
  style?: React.CSSProperties;
  format?: (value: number) => string;
}

export const AnimatedCounter = ({
  value,
  className = "",
  style = {},
  format = (v) => v.toLocaleString(),
}: AnimatedCounterProps) => {
  const springValue = useSpring(value, {
    stiffness: 100,
    damping: 20,
  });

  const displayValue = useTransform(springValue, (v) => format(Math.round(v)));
  const [display, setDisplay] = useState(format(value));

  useEffect(() => {
    springValue.set(value);
  }, [value, springValue]);

  useEffect(() => {
    const unsubscribe = displayValue.on("change", (v) => {
      setDisplay(v);
    });
    return unsubscribe;
  }, [displayValue]);

  return (
    <motion.span
      className={`font-mono tabular-nums ${className}`}
      style={style}
    >
      {display}
    </motion.span>
  );
};

// ============================================================================
// Percentage Bar with animated fill
// ============================================================================

interface AnimatedPercentBarProps {
  value: number; // 0-1
  color?: string;
  bgColor?: string;
  height?: number;
  className?: string;
  showLabel?: boolean;
}

export const AnimatedPercentBar = ({
  value,
  color = fey.teal,
  bgColor = "rgba(255, 255, 255, 0.06)",
  height = 4,
  className = "",
  showLabel = false,
}: AnimatedPercentBarProps) => {
  const clampedValue = Math.max(0, Math.min(1, value));

  return (
    <div className={`relative ${className}`}>
      <div
        className="w-full rounded-full overflow-hidden"
        style={{ backgroundColor: bgColor, height }}
      >
        <motion.div
          className="h-full rounded-full"
          style={{ backgroundColor: color }}
          initial={{ width: 0 }}
          animate={{ width: `${clampedValue * 100}%` }}
          transition={{
            type: "spring",
            stiffness: 100,
            damping: 20,
          }}
        />
      </div>
      {showLabel && (
        <motion.span
          className="absolute right-0 top-1/2 -translate-y-1/2 text-xs font-mono"
          style={{ color }}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.3 }}
        >
          {(clampedValue * 100).toFixed(0)}%
        </motion.span>
      )}
    </div>
  );
};

export default AnimatedNumber;
