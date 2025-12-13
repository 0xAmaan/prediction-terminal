Candlestick Chart Implementation Plan

 Context & Background

 The Goal

 Render proper OHLCV candlestick charts for Polymarket markets with buy/sell pressure coloring (green = net buying, red =
 net selling). This provides more insight than the current line charts by showing both price action (OHLCV shape) and
 market sentiment (who's pushing the market).

 Polymarket Market Structure

 - Each binary market has YES and NO tokens with separate orderbooks
 - Every trade has a side field: BUY (buying YES = bullish) or SELL (selling YES = bearish)
 - YES price + NO price ≈ $1.00
 - Current charts show YES token price as a line

 Why Buy/Sell Coloring > Traditional OHLCV

 Traditional candlesticks color based on price direction (close > open = green). Buy/sell coloring shows who was 
 aggressive:
 - Green candle: More buy volume than sell volume (buyers pushing)
 - Red candle: More sell volume than buy volume (sellers pushing)

 This reveals sentiment even when price moves opposite to the pressure (e.g., price up but mostly sells = red candle,
 indicating selling into strength).

 Current State

 - Trade struct (terminal-core/src/market.rs:391): Has side: Option<TradeSide> field
 - Polymarket API: Returns side as "BUY" or "SELL" in trade responses
 - SQLite storage (terminal-services/src/trade_storage.rs:56): Stores side as nullable TEXT
 - PriceCandle struct (terminal-core/src/market.rs:490): Only has total volume, no buy/sell split
 - CandleService (terminal-services/src/candle_service.rs:25): Ignores side field entirely
 - Frontend: Uses line charts via lightweight-charts, not candlesticks

 ---
 Implementation Plan

 Phase 1: Backend - Extend Candle Data Structure

 File: terminal-core/src/market.rs

 1. Add buy/sell volume fields to PriceCandle:
 pub struct PriceCandle {
     pub timestamp: DateTime<Utc>,
     pub open: Decimal,
     pub high: Decimal,
     pub low: Decimal,
     pub close: Decimal,
     pub volume: Decimal,           // Total volume (keep for backwards compat)
     pub buy_volume: Decimal,       // NEW: Sum of quantities where side = Buy
     pub sell_volume: Decimal,      // NEW: Sum of quantities where side = Sell
 }

 File: terminal-services/src/candle_service.rs

 2. Update build_candles() to aggregate volumes by side:
 // In the bucket processing loop:
 let buy_volume: Decimal = sorted_trades
     .iter()
     .filter(|t| t.side == Some(TradeSide::Buy))
     .map(|t| t.quantity)
     .sum();

 let sell_volume: Decimal = sorted_trades
     .iter()
     .filter(|t| t.side == Some(TradeSide::Sell))
     .map(|t| t.quantity)
     .sum();

 let volume = buy_volume + sell_volume;

 Phase 2: Backend - Ensure Multi-Outcome Markets Use CandleService

 Currently multi-outcome markets bypass CandleService and call Polymarket's /prices-history API directly (which only
 returns price points, no OHLCV).

 File: terminal-services/src/market_service.rs

 3. For get_outcome_prices(), check if we have trades stored and use CandleService if available:
   - If trades exist in TradeStorage for the token → build candles via CandleService
   - Fallback to Polymarket API if no trades (for backwards compat)

 File: terminal-api/src/routes/markets.rs

 4. Update /prices-history endpoint to return full candle data (not just price points) when available.

 Phase 3: Frontend - Switch to Candlestick Rendering

 File: frontend/src/lib/types.ts

 5. Update PriceCandle type to include new fields:
 interface PriceCandle {
   timestamp: string;
   open: string;
   high: string;
   low: string;
   close: string;
   volume: string;
   buy_volume: string;   // NEW
   sell_volume: string;  // NEW
 }

 File: frontend/src/components/market/price-chart.tsx

 6. Switch from LineSeries to CandlestickSeries in lightweight-charts:
 const series = chart.addSeries(CandlestickSeries, {
   upColor: '#22c55e',      // green
   downColor: '#ef4444',    // red
   // ...
 });

 7. Implement coloring logic based on buy/sell volume:
 const candleColor = (candle: PriceCandle) => {
   const buyVol = parseFloat(candle.buy_volume);
   const sellVol = parseFloat(candle.sell_volume);
   return buyVol >= sellVol ? '#22c55e' : '#ef4444';
 };

 8. Add a toggle for coloring mode (optional enhancement):
   - "Sentiment" mode: Color by buy/sell volume (default)
   - "Price" mode: Color by close > open

 Phase 4: Ensure Trade Collection Coverage

 File: terminal-services/src/trade_collector.rs

 9. Verify that trade collection is working for multi-outcome markets:
   - Trades need to be collected per-outcome (per token_id)
   - Backfill should trigger when viewing any outcome's chart

 ---
 Important Design Decision: Where Candlesticks Appear

 Trading View (single outcome focus): Use candlestick chart with OHLCV + buy/sell coloring.
 - This applies to BOTH binary markets AND individual outcomes within multi-outcome events
 - Example: When trading "Paul Thomas Anderson - YES", show candlesticks for that specific token
 - The trading view always shows one market/outcome at a time

 Multi-outcome overlay chart (comparison view): Keep as line chart.
 - When comparing all 10 directors simultaneously, candlesticks would be unreadable
 - This is for overview/comparison, not trading

 ---
 Files to Modify

 | File                                                   | Changes
       |
 |--------------------------------------------------------|---------------------------------------------------------------
 ------|
 | terminal-core/src/market.rs                            | Add buy_volume, sell_volume to PriceCandle
       |
 | terminal-services/src/candle_service.rs                | Aggregate volumes by trade side
       |
 | terminal-services/src/market_service.rs                | Route multi-outcome to CandleService when trades available
       |
 | terminal-api/src/routes/markets.rs                     | Update price-history endpoint response
       |
 | frontend/src/lib/types.ts                              | Add new candle fields
       |
 | frontend/src/components/market/price-chart.tsx         | Switch to CandlestickSeries for trading view, add buy/sell
 coloring |
 | frontend/src/components/market/multi-outcome-chart.tsx | No changes - stays as line chart for comparison view
       |

 ---
 Data Flow After Implementation

 Polymarket Trade (with side: "BUY"/"SELL")
     ↓
 TradeStorage (SQLite) - stores side field
     ↓
 CandleService.build_candles()
     ↓ aggregates by side
 PriceCandle { open, high, low, close, volume, buy_volume, sell_volume }
     ↓
 API Response → Frontend
     ↓
 CandlestickSeries with buy/sell coloring

 ---
 Future Enhancements (Out of Scope)

 - Volume bars below chart showing buy (green) vs sell (red) stacked
 - Tooltip showing buy/sell ratio on hover
 - Real-time candle updates via WebSocket (currently would require refresh)