"use client";

import { useState, useEffect, useCallback, useMemo, useRef } from "react";
import type { Trade } from "@/lib/types";

// ============================================================================
// Types
// ============================================================================

export interface TradeMomentum {
  /** Net buy volume - sell volume in the window */
  netVolume: number;
  /** Total buy volume */
  buyVolume: number;
  /** Total sell volume */
  sellVolume: number;
  /** Number of buy trades */
  buyCount: number;
  /** Number of sell trades */
  sellCount: number;
  /** Momentum ratio: -1 (all sells) to 1 (all buys) */
  momentumRatio: number;
  /** Is momentum accelerating? */
  isAccelerating: boolean;
  /** Direction: "buy" | "sell" | "neutral" */
  direction: "buy" | "sell" | "neutral";
  /** Recent trade velocity (trades per minute) */
  velocity: number;
  /** Largest trade in the window */
  largestTrade: Trade | null;
  /** Is there a "whale" trade? (>2x average) */
  hasWhale: boolean;
}

export interface ProcessedTrade extends Trade {
  /** Size relative to average (1 = average) */
  relativeSize: number;
  /** Is this a "whale" trade? */
  isWhale: boolean;
  /** Time since this trade (ms) */
  age: number;
}

// ============================================================================
// Trade Momentum Hook
// ============================================================================

interface UseTradeMomentumOptions {
  /** Trades to analyze */
  trades: Trade[];
  /** Time window in seconds (default: 60) */
  windowSeconds?: number;
  /** Whale threshold multiplier (default: 2x average) */
  whaleThreshold?: number;
  /** Update interval in ms (default: 1000) */
  updateInterval?: number;
}

export const useTradeMomentum = ({
  trades,
  windowSeconds = 60,
  whaleThreshold = 2,
  updateInterval = 1000,
}: UseTradeMomentumOptions): TradeMomentum => {
  const [momentum, setMomentum] = useState<TradeMomentum>({
    netVolume: 0,
    buyVolume: 0,
    sellVolume: 0,
    buyCount: 0,
    sellCount: 0,
    momentumRatio: 0,
    isAccelerating: false,
    direction: "neutral",
    velocity: 0,
    largestTrade: null,
    hasWhale: false,
  });

  const prevMomentumRef = useRef<number>(0);

  // Calculate momentum from trades
  const calculateMomentum = useCallback(() => {
    const now = Date.now();
    const windowMs = windowSeconds * 1000;
    const cutoff = now - windowMs;

    // Filter trades within window
    const recentTrades = trades.filter((t) => {
      const tradeTime = new Date(t.timestamp).getTime();
      return tradeTime >= cutoff;
    });

    if (recentTrades.length === 0) {
      setMomentum((prev) => ({
        ...prev,
        netVolume: 0,
        buyVolume: 0,
        sellVolume: 0,
        buyCount: 0,
        sellCount: 0,
        momentumRatio: 0,
        direction: "neutral",
        velocity: 0,
        largestTrade: null,
        hasWhale: false,
      }));
      return;
    }

    // Calculate volumes
    let buyVolume = 0;
    let sellVolume = 0;
    let buyCount = 0;
    let sellCount = 0;
    let largestTrade: Trade | null = null;
    let largestSize = 0;

    const avgSize =
      recentTrades.reduce((sum, t) => sum + parseFloat(String(t.quantity)), 0) /
      recentTrades.length;

    let hasWhale = false;

    for (const trade of recentTrades) {
      const qty = parseFloat(String(trade.quantity));
      const isBuy = trade.side?.toLowerCase() === "buy";

      if (isBuy) {
        buyVolume += qty;
        buyCount++;
      } else {
        sellVolume += qty;
        sellCount++;
      }

      if (qty > largestSize) {
        largestSize = qty;
        largestTrade = trade;
      }

      if (qty > avgSize * whaleThreshold) {
        hasWhale = true;
      }
    }

    const netVolume = buyVolume - sellVolume;
    const totalVolume = buyVolume + sellVolume;
    const momentumRatio = totalVolume > 0 ? netVolume / totalVolume : 0;

    // Determine direction
    let direction: "buy" | "sell" | "neutral" = "neutral";
    if (momentumRatio > 0.2) direction = "buy";
    else if (momentumRatio < -0.2) direction = "sell";

    // Check if accelerating
    const isAccelerating =
      Math.abs(momentumRatio) > Math.abs(prevMomentumRef.current);
    prevMomentumRef.current = momentumRatio;

    // Calculate velocity (trades per minute)
    const velocity = (recentTrades.length / windowSeconds) * 60;

    setMomentum({
      netVolume,
      buyVolume,
      sellVolume,
      buyCount,
      sellCount,
      momentumRatio,
      isAccelerating,
      direction,
      velocity,
      largestTrade,
      hasWhale,
    });
  }, [trades, windowSeconds, whaleThreshold]);

  // Update on interval
  useEffect(() => {
    calculateMomentum();
    const interval = setInterval(calculateMomentum, updateInterval);
    return () => clearInterval(interval);
  }, [calculateMomentum, updateInterval]);

  return momentum;
};

// ============================================================================
// Process Trades Hook (adds relative sizing)
// ============================================================================

export const useProcessedTrades = (
  trades: Trade[],
  maxTrades: number = 50,
  whaleThreshold: number = 2,
): ProcessedTrade[] => {
  return useMemo(() => {
    const now = Date.now();

    // Calculate average size
    const avgSize =
      trades.length > 0
        ? trades.reduce((sum, t) => sum + parseFloat(String(t.quantity)), 0) /
          trades.length
        : 0;

    return trades.slice(0, maxTrades).map((trade) => {
      const qty = parseFloat(String(trade.quantity));
      const relativeSize = avgSize > 0 ? qty / avgSize : 1;
      const isWhale = relativeSize > whaleThreshold;
      const age = now - new Date(trade.timestamp).getTime();

      return {
        ...trade,
        relativeSize,
        isWhale,
        age,
      };
    });
  }, [trades, maxTrades, whaleThreshold]);
};

// ============================================================================
// Pressure History Hook (for sparkline/charts)
// ============================================================================

interface PressurePoint {
  timestamp: number;
  ratio: number;
  buyVolume: number;
  sellVolume: number;
}

export const usePressureHistory = (
  trades: Trade[],
  bucketSeconds: number = 10,
  maxBuckets: number = 30,
): PressurePoint[] => {
  return useMemo(() => {
    if (trades.length === 0) return [];

    const now = Date.now();
    const bucketMs = bucketSeconds * 1000;
    const points: PressurePoint[] = [];

    // Create buckets going back in time
    for (let i = 0; i < maxBuckets; i++) {
      const bucketEnd = now - i * bucketMs;
      const bucketStart = bucketEnd - bucketMs;

      const bucketTrades = trades.filter((t) => {
        const time = new Date(t.timestamp).getTime();
        return time >= bucketStart && time < bucketEnd;
      });

      let buyVolume = 0;
      let sellVolume = 0;

      for (const trade of bucketTrades) {
        const qty = parseFloat(String(trade.quantity));
        if (trade.side?.toLowerCase() === "buy") {
          buyVolume += qty;
        } else {
          sellVolume += qty;
        }
      }

      const total = buyVolume + sellVolume;
      const ratio = total > 0 ? (buyVolume - sellVolume) / total : 0;

      points.unshift({
        timestamp: bucketStart,
        ratio,
        buyVolume,
        sellVolume,
      });
    }

    return points;
  }, [trades, bucketSeconds, maxBuckets]);
};

export default useTradeMomentum;
