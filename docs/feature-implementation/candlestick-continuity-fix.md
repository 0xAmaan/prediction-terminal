# Candlestick Chart Continuity Fix

## Summary

Fixed candlestick charts to render as continuous, connected price flows instead of disconnected independent candles.

## Problem

Candlestick charts were rendering as scattered, disconnected vertical bars that didn't visually connect to each other. This made charts unreadable, especially for low-activity markets where:
- Each candle appeared as isolated dots/dashes
- Price movements between candles weren't visually connected
- The overall price trend was hard to follow

### Root Cause

Each candlestick was calculated independently from price points within its time bucket only. The close of candle N had no relationship to the open of candle N+1, causing visual gaps.

**Before (disconnected):**
```
Candle 1: open=0.015, close=0.016
Candle 2: open=0.014, close=0.015  <- Gap! Open doesn't match previous close
Candle 3: open=0.017, close=0.016  <- Another gap!
```

## Solution

Modified `transformCandleData()` in `frontend/src/components/market/price-chart.tsx` to create **continuous candlesticks** where each candle's open equals the previous candle's close.

**After (continuous):**
```
Candle 1: open=0.015, close=0.016
Candle 2: open=0.016, close=0.015  <- Open = previous close
Candle 3: open=0.015, close=0.016  <- Continuous flow
```

### Key Code Change

```typescript
const transformCandleData = (
  candles: PriceCandle[],
  colorMode: ColorMode
): CandlestickData<UTCTimestamp>[] => {
  // First pass: dedupe and sort
  const sortedCandles = candles
    .map((candle) => ({
      candle,
      time: Math.floor(new Date(candle.timestamp).getTime() / 1000),
    }))
    .filter(/* dedupe by timestamp */)
    .sort((a, b) => a.time - b.time);

  // Second pass: create continuous candlesticks
  let prevClose: number | null = null;

  return sortedCandles.map(({ candle, time }) => {
    const close = parseFloat(candle.close);

    // KEY: Make candles continuous - open = previous close
    const open = prevClose !== null ? prevClose : parseFloat(candle.open);
    prevClose = close;

    // Adjust high/low to include the continuous open
    const high = Math.max(originalHigh, open, close);
    const low = Math.min(originalLow, open, close);

    return { time, open, high, low, close, color, wickColor };
  });
};
```

## Affected Components

The fix applies to all candlestick charts via the shared `PriceChart` component:

| View | File | Usage |
|------|------|-------|
| Binary Overview | `overview-view.tsx` | Market overview page |
| Binary Trading | `trading-view.tsx` | Trading view for binary markets |
| Multi-Outcome Trading | `multi-outcome-trading-view.tsx` | Trading view for multi-outcome events |

## Data Flow

```
Polymarket API (PriceHistoryPoint: timestamp + price)
    ↓
Backend (candle_service.rs: builds OHLCV candles per time bucket)
    ↓
Frontend API response (PriceCandle[])
    ↓
transformCandleData() ← FIX APPLIED HERE
    ↓
lightweight-charts (CandlestickSeries)
```

## Visual Result

**Before:** Scattered dots/dashes with no visual connection
**After:** Smooth, connected candlestick flow that clearly shows price movement over time

## Related Fixes (Same Session)

### 1. Timeframe Filtering (Phase 1)
- **Issue:** 30D showed full market history instead of last 30 days
- **Fix:** Added `from_filter` parameter to `build_hybrid_candles()` in `candle_service.rs`
- **Files:** `terminal-api/src/routes/markets.rs`, `terminal-services/src/candle_service.rs`, `terminal-polymarket/src/client.rs`

### 2. Fidelity Override
- **Issue:** Wrong data granularity for different timeframes
- **Fix:** Added `fidelity_override` parameter to Polymarket client
- **Files:** `terminal-polymarket/src/client.rs`, `terminal-services/src/market_service.rs`

## Configuration

Timeframe settings in `terminal-api/src/routes/markets.rs`:

| Timeframe | Polymarket Interval | Fidelity | Candle Size | Time Filter |
|-----------|--------------------|---------:|-------------|-------------|
| 1H        | 1h                 | 1        | 1 minute    | Last 1 hour |
| 24H       | 1d                 | 15       | 15 minutes  | Last 24h    |
| 7D        | 1w                 | 60       | 1 hour      | Last 7 days |
| 30D       | max                | 240      | 4 hours     | Last 30 days|
| ALL       | max                | 1440     | 1 day       | None        |

## Testing

1. **Low-activity market (e.g., "Kim Jong Un visits US")** - Should show smooth connected candles
2. **Active market** - Should look normal with proper OHLC representation
3. **All timeframes (1H, 24H, 7D, 30D, ALL)** - Should all render continuously
4. **All views (overview, trading, multi-outcome)** - Should all use the fix

## Date

December 13, 2025
