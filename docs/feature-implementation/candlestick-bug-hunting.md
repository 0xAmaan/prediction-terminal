Fix Candlestick Chart Data - Root Cause & Solution

 Problem Summary

 The candlestick chart for market 16172 shows only 89 candles (~16 days) instead of the expected 30 days (or full
 market history). The chart from the screenshot shows sparse, low-quality candle data.

 Root Cause

 The native price history path is failing silently and falling back to trade-only data.

 Server log shows:
 WARN Native price history failed, falling back to trade-based:
 Parse error: Failed to parse options_json: missing field `yes_price` at line 1 column 111

 Why it fails:

 1. market_service.rs:533 tries to parse options_json into Vec<MarketOption>
 2. It uses terminal_polymarket::MarketOption which requires fields: name, yes_price, market_id, clob_token_id
 3. But options_json in the database only has: {"clob_token_id": "...", "name": "Yes"}
 4. Deserialization fails → falls back to trade-only candles → limited to 50 stored trades → only 89 candles

 Broader Issue: Multiple Parsing Inconsistencies

 This same MarketOption parsing problem exists in two places:

 | Location                   | Line | Behavior                          | Impact                                 |
 |----------------------------|------|-----------------------------------|----------------------------------------|
 | get_native_price_history() | 533  | Returns error → triggers fallback | Degraded chart data (what you saw)     |
 | get_outcome_prices()       | 350  | Silent .unwrap_or_default()       | Multi-outcome markets may show no data |

 The line 350 issue is potentially worse because it fails completely silently.

 Solution

 Create a minimal struct in market_service.rs that matches the actual stored JSON format:

 /// Minimal struct for parsing options_json to extract clob_token_id
 /// Note: options_json stored in DB has only {name, clob_token_id}, not the full MarketOption fields
 #[derive(Debug, serde::Deserialize)]
 struct MinimalOption {
     #[serde(default)]
     name: Option<String>,
     #[serde(default)]
     clob_token_id: Option<String>,
 }

 Files to Modify

 terminal-services/src/market_service.rs

 1. Remove import: Remove MarketOption from the terminal_polymarket import line (line 7)
 2. Add local struct (around line 20):
 /// Minimal struct for parsing options_json to extract clob_token_id
 /// Note: options_json stored in DB has only {name, clob_token_id}, not the full MarketOption fields
 #[derive(Debug, serde::Deserialize)]
 struct MinimalOption {
     #[serde(default)]
     name: Option<String>,
     #[serde(default)]
     clob_token_id: Option<String>,
 }

 3. Update line 350 (get_outcome_prices): Change Vec<MarketOption> to Vec<MinimalOption>
 4. Update line 533 (get_native_price_history): Change Vec<MarketOption> to Vec<MinimalOption>

 Verification Steps

 After the fix:
 1. Rebuild: cargo build -p terminal-api
 2. Restart server
 3. Test price history: curl -s "http://localhost:3001/api/markets/polymarket/16172/history?timeframe=30D"
 4. Expected: ~348 candles covering the full market history (Dec 31, 2024 to today)
 5. Server log should show Fetching price history for token ID: 91549... instead of the fallback warning
 6. Test multi-outcome market to ensure outcome prices work

 Future Recommendations (Data Reliability)

 1. Remove silent .unwrap_or_default() patterns: These hide failures. Replace with explicit error handling or
 logging.
 2. Add data quality indicators to API responses: Include a data_source field in responses so frontend can show when
 data is degraded (e.g., "trade-based fallback" vs "native API").
 3. Consider schema validation: Add runtime checks that options_json contains expected fields when storing/retrieving
  markets.
 4. Consolidate MarketOption definitions: There are 3 different structs with the same name:
   - terminal-polymarket/src/types.rs:440 (full struct for API responses)
   - terminal-api/src/routes/markets.rs:251 (minimal for sparklines)
   - NEW: terminal-services/src/market_service.rs (minimal for parsing)

 Consider consolidating or clearly documenting which to use where.