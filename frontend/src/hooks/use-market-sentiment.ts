"use client";

import { useMemo } from "react";
import type { TradeMomentum } from "./use-trade-momentum";

// ============================================================================
// Types
// ============================================================================

export interface MarketSentiment {
  /** Overall sentiment score: -100 (extremely bearish) to +100 (extremely bullish) */
  score: number;
  /** Sentiment label */
  label:
    | "Extremely Bearish"
    | "Bearish"
    | "Slightly Bearish"
    | "Neutral"
    | "Slightly Bullish"
    | "Bullish"
    | "Extremely Bullish";
  /** Confidence in the sentiment (0-1) */
  confidence: number;
  /** Individual component scores */
  components: {
    orderBookImbalance: number;
    tradeFlowMomentum: number;
    priceTrend: number;
    volumeActivity: number;
  };
  /** Signals contributing to sentiment */
  signals: SentimentSignal[];
}

export interface SentimentSignal {
  type: "bullish" | "bearish" | "neutral";
  source: string;
  description: string;
  weight: number;
}

// ============================================================================
// Sentiment Calculation Hook
// ============================================================================

interface UseMarketSentimentOptions {
  /** Order book imbalance ratio (-1 to 1) */
  orderBookImbalance?: number;
  /** Trade momentum data */
  tradeMomentum?: TradeMomentum | null;
  /** Current price */
  currentPrice?: number;
  /** Previous price (for trend) */
  previousPrice?: number;
  /** 24h volume */
  volume24h?: number;
  /** Average volume */
  averageVolume?: number;
}

export const useMarketSentiment = ({
  orderBookImbalance = 0,
  tradeMomentum = null,
  currentPrice = 0,
  previousPrice = 0,
  volume24h = 0,
  averageVolume = 0,
}: UseMarketSentimentOptions): MarketSentiment => {
  return useMemo(() => {
    const signals: SentimentSignal[] = [];

    // ========================================
    // Component 1: Order Book Imbalance
    // Weight: 30%
    // ========================================
    const obScore = orderBookImbalance * 100; // -100 to 100

    if (orderBookImbalance > 0.3) {
      signals.push({
        type: "bullish",
        source: "Order Book",
        description: "Strong bid-side liquidity",
        weight: 0.3,
      });
    } else if (orderBookImbalance < -0.3) {
      signals.push({
        type: "bearish",
        source: "Order Book",
        description: "Strong ask-side liquidity",
        weight: 0.3,
      });
    }

    // ========================================
    // Component 2: Trade Flow Momentum
    // Weight: 35%
    // ========================================
    let tfScore = 0;

    if (tradeMomentum) {
      tfScore = tradeMomentum.momentumRatio * 100;

      if (tradeMomentum.direction === "buy") {
        signals.push({
          type: "bullish",
          source: "Trade Flow",
          description: tradeMomentum.isAccelerating
            ? "Accelerating buy pressure"
            : "Net buying activity",
          weight: tradeMomentum.isAccelerating ? 0.4 : 0.3,
        });
      } else if (tradeMomentum.direction === "sell") {
        signals.push({
          type: "bearish",
          source: "Trade Flow",
          description: tradeMomentum.isAccelerating
            ? "Accelerating sell pressure"
            : "Net selling activity",
          weight: tradeMomentum.isAccelerating ? 0.4 : 0.3,
        });
      }

      if (tradeMomentum.hasWhale) {
        const whaleDirection =
          tradeMomentum.largestTrade?.side?.toLowerCase() === "buy"
            ? "bullish"
            : "bearish";
        signals.push({
          type: whaleDirection,
          source: "Whale Activity",
          description: `Large ${whaleDirection === "bullish" ? "buy" : "sell"} order detected`,
          weight: 0.2,
        });
      }
    }

    // ========================================
    // Component 3: Price Trend
    // Weight: 20%
    // ========================================
    let ptScore = 0;

    if (currentPrice > 0 && previousPrice > 0) {
      const priceChange =
        ((currentPrice - previousPrice) / previousPrice) * 100;
      ptScore = Math.max(-100, Math.min(100, priceChange * 10)); // Scale up small moves

      if (priceChange > 2) {
        signals.push({
          type: "bullish",
          source: "Price Trend",
          description: `Price up ${priceChange.toFixed(1)}%`,
          weight: 0.2,
        });
      } else if (priceChange < -2) {
        signals.push({
          type: "bearish",
          source: "Price Trend",
          description: `Price down ${Math.abs(priceChange).toFixed(1)}%`,
          weight: 0.2,
        });
      }
    }

    // ========================================
    // Component 4: Volume Activity
    // Weight: 15%
    // ========================================
    let vaScore = 0;

    if (volume24h > 0 && averageVolume > 0) {
      const volumeRatio = volume24h / averageVolume;
      vaScore = Math.max(-100, Math.min(100, (volumeRatio - 1) * 50));

      if (volumeRatio > 1.5) {
        // High volume with momentum direction matters
        const direction =
          tfScore > 0 ? "bullish" : tfScore < 0 ? "bearish" : "neutral";
        signals.push({
          type: direction,
          source: "Volume",
          description: `${(volumeRatio * 100).toFixed(0)}% of average volume`,
          weight: 0.15,
        });
      }
    }

    // ========================================
    // Calculate Weighted Score
    // ========================================
    const weightedScore =
      obScore * 0.3 + tfScore * 0.35 + ptScore * 0.2 + vaScore * 0.15;

    // Clamp to -100 to 100
    const score = Math.max(-100, Math.min(100, weightedScore));

    // ========================================
    // Determine Label
    // ========================================
    const getLabel = (s: number): MarketSentiment["label"] => {
      if (s >= 60) return "Extremely Bullish";
      if (s >= 30) return "Bullish";
      if (s >= 10) return "Slightly Bullish";
      if (s <= -60) return "Extremely Bearish";
      if (s <= -30) return "Bearish";
      if (s <= -10) return "Slightly Bearish";
      return "Neutral";
    };

    // ========================================
    // Calculate Confidence
    // ========================================
    // Confidence is based on data availability and signal agreement
    let dataPoints = 0;
    if (orderBookImbalance !== 0) dataPoints++;
    if (tradeMomentum) dataPoints++;
    if (currentPrice > 0 && previousPrice > 0) dataPoints++;
    if (volume24h > 0 && averageVolume > 0) dataPoints++;

    const dataConfidence = dataPoints / 4;

    // Signal agreement: do bullish/bearish signals agree?
    const bullishSignals = signals.filter((s) => s.type === "bullish").length;
    const bearishSignals = signals.filter((s) => s.type === "bearish").length;
    const totalDirectionalSignals = bullishSignals + bearishSignals;

    let signalAgreement = 0.5; // Default if no signals
    if (totalDirectionalSignals > 0) {
      const majoritySignals = Math.max(bullishSignals, bearishSignals);
      signalAgreement = majoritySignals / totalDirectionalSignals;
    }

    const confidence = dataConfidence * 0.6 + signalAgreement * 0.4;

    return {
      score,
      label: getLabel(score),
      confidence,
      components: {
        orderBookImbalance: obScore,
        tradeFlowMomentum: tfScore,
        priceTrend: ptScore,
        volumeActivity: vaScore,
      },
      signals,
    };
  }, [
    orderBookImbalance,
    tradeMomentum,
    currentPrice,
    previousPrice,
    volume24h,
    averageVolume,
  ]);
};

export default useMarketSentiment;
