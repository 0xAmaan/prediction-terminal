/**
 * Market Analytics Utilities
 * Functions for calculating price metrics, volatility, and other analytics
 */

export interface PriceRange {
  high: number;
  low: number;
  range: number;
  rangePercent: number;
}

export interface VolatilityMetrics {
  volatility: number; // Standard deviation as percentage
  avgDailyChange: number;
}

/**
 * Calculate the price range from an array of prices
 */
export const calculatePriceRange = (prices: number[]): PriceRange | null => {
  if (prices.length === 0) return null;

  const high = Math.max(...prices);
  const low = Math.min(...prices);
  const range = high - low;
  const mid = (high + low) / 2;
  const rangePercent = mid > 0 ? (range / mid) * 100 : 0;

  return { high, low, range, rangePercent };
};

/**
 * Calculate volatility (standard deviation of returns) from price history
 */
export const calculateVolatility = (
  prices: number[],
): VolatilityMetrics | null => {
  if (prices.length < 2) return null;

  // Calculate daily returns
  const returns: number[] = [];
  for (let i = 1; i < prices.length; i++) {
    if (prices[i - 1] > 0) {
      returns.push((prices[i] - prices[i - 1]) / prices[i - 1]);
    }
  }

  if (returns.length === 0) return null;

  // Calculate mean return
  const meanReturn = returns.reduce((a, b) => a + b, 0) / returns.length;

  // Calculate variance
  const variance =
    returns.reduce((sum, r) => sum + Math.pow(r - meanReturn, 2), 0) /
    returns.length;

  // Standard deviation (volatility) as percentage
  const volatility = Math.sqrt(variance) * 100;

  // Average absolute daily change as percentage
  const avgDailyChange =
    (returns.reduce((sum, r) => sum + Math.abs(r), 0) / returns.length) * 100;

  return { volatility, avgDailyChange };
};

/**
 * Format a price as cents (e.g., 0.52 -> "52¢")
 */
export const formatPriceCents = (price: number): string => {
  return `${Math.round(price * 100)}¢`;
};

/**
 * Format a price range for display
 */
export const formatPriceRange = (range: PriceRange): string => {
  return `${formatPriceCents(range.low)} - ${formatPriceCents(range.high)}`;
};

/**
 * Format volatility as percentage
 */
export const formatVolatility = (volatility: number): string => {
  return `${volatility.toFixed(1)}%`;
};

/**
 * Format volume (abbreviate large numbers)
 */
export const formatVolume = (volume: number): string => {
  if (volume >= 1_000_000) {
    return `$${(volume / 1_000_000).toFixed(1)}M`;
  }
  if (volume >= 1_000) {
    return `$${(volume / 1_000).toFixed(1)}K`;
  }
  return `$${volume.toFixed(0)}`;
};

/**
 * Format trade count with abbreviation
 */
export const formatTradeCount = (count: number): string => {
  if (count >= 1_000_000) {
    return `${(count / 1_000_000).toFixed(1)}M`;
  }
  if (count >= 1_000) {
    return `${(count / 1_000).toFixed(1)}K`;
  }
  return count.toString();
};
