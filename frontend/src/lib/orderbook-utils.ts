/**
 * Order Book Utility Functions
 * Calculations for heatmap, imbalance, wall detection, and other analytics
 */

// ============================================================================
// Types
// ============================================================================

export interface OrderBookLevel {
  price: string;
  quantity: string;
  order_count?: number | null;
}

export interface ProcessedLevel extends OrderBookLevel {
  priceNum: number;
  quantityNum: number;
  intensity: number; // 0-1, for heatmap
  isWall: boolean; // Large order detection
  cumulativeQuantity: number;
  distanceFromMid: number; // Distance from mid price (in price units)
}

export interface OrderBookMetrics {
  midPrice: number;
  spread: number;
  spreadPercent: number;
  totalBidQty: number;
  totalAskQty: number;
  imbalanceRatio: number; // -1 to 1, negative = more asks, positive = more bids
  bidAskRatio: number; // bids / asks
  wallThreshold: number;
  maxQuantity: number;
}

// ============================================================================
// Processing Functions
// ============================================================================

/**
 * Calculate order book metrics from raw bid/ask levels
 */
export const calculateMetrics = (
  bids: OrderBookLevel[],
  asks: OrderBookLevel[],
): OrderBookMetrics => {
  const parsedBids = bids.map((l) => ({
    price: parseFloat(l.price),
    qty: parseFloat(l.quantity),
  }));

  const parsedAsks = asks.map((l) => ({
    price: parseFloat(l.price),
    qty: parseFloat(l.quantity),
  }));

  // Calculate totals
  const totalBidQty = parsedBids.reduce((sum, b) => sum + b.qty, 0);
  const totalAskQty = parsedAsks.reduce((sum, a) => sum + a.qty, 0);

  // Best bid/ask
  const bestBid = Math.max(...parsedBids.map((b) => b.price), 0);
  const bestAsk = Math.min(...parsedAsks.map((a) => a.price), 1);

  // Mid price and spread
  const midPrice = (bestBid + bestAsk) / 2;
  const spread = bestAsk - bestBid;
  const spreadPercent = midPrice > 0 ? (spread / midPrice) * 100 : 0;

  // Imbalance: positive = bid heavy, negative = ask heavy
  const totalQty = totalBidQty + totalAskQty;
  const imbalanceRatio =
    totalQty > 0 ? (totalBidQty - totalAskQty) / totalQty : 0;

  const bidAskRatio = totalAskQty > 0 ? totalBidQty / totalAskQty : 1;

  // Wall threshold: 2x the average quantity
  const allQuantities = [...parsedBids, ...parsedAsks].map((l) => l.qty);
  const avgQuantity =
    allQuantities.length > 0
      ? allQuantities.reduce((a, b) => a + b, 0) / allQuantities.length
      : 0;
  const wallThreshold = avgQuantity * 2;

  const maxQuantity = Math.max(...allQuantities, 1);

  return {
    midPrice,
    spread,
    spreadPercent,
    totalBidQty,
    totalAskQty,
    imbalanceRatio,
    bidAskRatio,
    wallThreshold,
    maxQuantity,
  };
};

/**
 * Process raw levels into enhanced levels with heatmap intensity
 */
export const processLevels = (
  levels: OrderBookLevel[],
  metrics: OrderBookMetrics,
  isBid: boolean,
): ProcessedLevel[] => {
  let cumulative = 0;

  // Sort: bids descending, asks ascending
  const sorted = [...levels].sort((a, b) => {
    const priceA = parseFloat(a.price);
    const priceB = parseFloat(b.price);
    return isBid ? priceB - priceA : priceA - priceB;
  });

  return sorted.map((level) => {
    const priceNum = parseFloat(level.price);
    const quantityNum = parseFloat(level.quantity);
    cumulative += quantityNum;

    // Distance from mid (0 = at mid, higher = further away)
    const distanceFromMid = Math.abs(priceNum - metrics.midPrice);

    // Intensity based on:
    // 1. Quantity relative to max (40% weight)
    // 2. Distance from mid - closer = hotter (40% weight)
    // 3. Order count if available (20% weight)
    const qtyIntensity =
      metrics.maxQuantity > 0 ? quantityNum / metrics.maxQuantity : 0;

    // Distance intensity (closer to mid = higher intensity)
    const maxDistance = 0.1; // 10 cent max for intensity calc
    const distanceIntensity = Math.max(0, 1 - distanceFromMid / maxDistance);

    // Combined intensity (0-1)
    const intensity = qtyIntensity * 0.6 + distanceIntensity * 0.4;

    // Wall detection
    const isWall = quantityNum > metrics.wallThreshold;

    return {
      ...level,
      priceNum,
      quantityNum,
      intensity: Math.min(1, Math.max(0, intensity)),
      isWall,
      cumulativeQuantity: cumulative,
      distanceFromMid,
    };
  });
};

// ============================================================================
// Color Functions
// ============================================================================

/**
 * Generate heatmap color based on intensity and side
 */
export const getHeatmapColor = (
  intensity: number,
  isBid: boolean,
  isWall: boolean = false,
): string => {
  const baseColor = isBid
    ? { r: 77, g: 190, b: 149 } // teal
    : { r: 216, g: 79, b: 104 }; // red

  // Minimum opacity 0.05, max 0.4 for regular, 0.6 for walls
  const maxOpacity = isWall ? 0.6 : 0.35;
  const minOpacity = 0.05;
  const opacity = minOpacity + intensity * (maxOpacity - minOpacity);

  return `rgba(${baseColor.r}, ${baseColor.g}, ${baseColor.b}, ${opacity})`;
};

/**
 * Get text color based on intensity
 */
export const getTextColor = (intensity: number, isBid: boolean): string => {
  const baseColor = isBid ? "#4DBE95" : "#D84F68";
  const mutedColor = "#7D8B96";

  // Lerp between muted and bright based on intensity
  if (intensity > 0.5) {
    return baseColor;
  }
  return mutedColor;
};

// ============================================================================
// Formatting Functions
// ============================================================================

export const formatPrice = (price: number): string => {
  return `${(price * 100).toFixed(1)}¢`;
};

export const formatQuantity = (quantity: number): string => {
  if (quantity >= 1_000_000) {
    return `${(quantity / 1_000_000).toFixed(1)}M`;
  }
  if (quantity >= 1_000) {
    return `${(quantity / 1_000).toFixed(1)}K`;
  }
  return quantity.toFixed(0);
};

export const formatSpread = (spread: number): string => {
  return `${(spread * 100).toFixed(2)}¢`;
};

export const formatImbalance = (ratio: number): string => {
  const percent = Math.abs(ratio * 100).toFixed(0);
  if (ratio > 0) return `${percent}% bid`;
  if (ratio < 0) return `${percent}% ask`;
  return "balanced";
};

// ============================================================================
// Analysis Functions
// ============================================================================

/**
 * Detect significant price levels (walls)
 */
export const detectWalls = (
  levels: ProcessedLevel[],
  threshold: number = 2,
): ProcessedLevel[] => {
  return levels.filter((l) => l.isWall);
};

/**
 * Calculate depth at different price points
 * Returns cumulative quantity at each % distance from mid
 */
export const calculateDepthProfile = (
  bids: ProcessedLevel[],
  asks: ProcessedLevel[],
  midPrice: number,
  steps: number[] = [0.01, 0.02, 0.05, 0.1], // 1%, 2%, 5%, 10%
): { bidDepth: number[]; askDepth: number[] } => {
  const bidDepth = steps.map((step) => {
    const threshold = midPrice * (1 - step);
    return bids
      .filter((b) => b.priceNum >= threshold)
      .reduce((sum, b) => sum + b.quantityNum, 0);
  });

  const askDepth = steps.map((step) => {
    const threshold = midPrice * (1 + step);
    return asks
      .filter((a) => a.priceNum <= threshold)
      .reduce((sum, a) => sum + a.quantityNum, 0);
  });

  return { bidDepth, askDepth };
};

/**
 * Calculate market "heat" - how active/volatile the order book is
 * Returns 0-100 score
 */
export const calculateMarketHeat = (
  metrics: OrderBookMetrics,
  previousMetrics?: OrderBookMetrics,
): number => {
  let heat = 50; // Base heat

  // Factor 1: Imbalance (higher imbalance = more heat)
  heat += Math.abs(metrics.imbalanceRatio) * 20;

  // Factor 2: Spread (tighter spread = less heat, wider = more)
  if (metrics.spreadPercent > 2) heat += 15;
  else if (metrics.spreadPercent > 1) heat += 5;
  else heat -= 10;

  // Factor 3: Total liquidity (more = less heat)
  const totalLiquidity = metrics.totalBidQty + metrics.totalAskQty;
  if (totalLiquidity < 10000) heat += 15;
  else if (totalLiquidity > 100000) heat -= 10;

  // Factor 4: Change from previous (if available)
  if (previousMetrics) {
    const imbalanceChange = Math.abs(
      metrics.imbalanceRatio - previousMetrics.imbalanceRatio,
    );
    heat += imbalanceChange * 30;
  }

  return Math.max(0, Math.min(100, heat));
};
