# Plan: Add Market Data Context to Research AI

## Overview

Currently, the research feature only passes `market.title` and `market.description` to GPT-4o. This plan adds real-time market data (price, volume, trades, order book) so the AI has accurate context when generating research.

## Goal

When the AI researches a market, it should know:
- Current probability/price
- Recent price movement (trend)
- Trading volume
- Recent trades (to see momentum)
- Order book depth (liquidity)

---

## Step 1: Create a MarketContext Struct

**File:** `terminal-research/src/types.rs`

Add a new struct to hold market context data:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketContext {
    pub title: String,
    pub description: Option<String>,
    pub current_price: Option<f64>,        // 0.0 to 1.0 (probability)
    pub price_24h_ago: Option<f64>,        // For calculating change
    pub volume_24h: Option<f64>,           // In dollars
    pub total_volume: Option<f64>,         // Lifetime volume
    pub num_traders: Option<u64>,          // Number of unique traders
    pub recent_trades: Vec<RecentTrade>,   // Last ~10 trades
    pub order_book_summary: Option<OrderBookSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentTrade {
    pub price: f64,
    pub size: f64,
    pub side: String,  // "buy" or "sell"
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookSummary {
    pub best_bid: Option<f64>,
    pub best_ask: Option<f64>,
    pub spread: Option<f64>,
    pub bid_depth_10pct: f64,  // Total $ within 10% of best bid
    pub ask_depth_10pct: f64,  // Total $ within 10% of best ask
}
```

**Export from:** `terminal-research/src/lib.rs`

---

## Step 2: Update OpenAI Client Functions

**File:** `terminal-research/src/openai.rs`

### 2a. Update `decompose_question` signature

Change from:
```rust
pub async fn decompose_question(
    &self,
    market_title: &str,
    market_description: &str,
) -> Result<DecomposedQuestions, TerminalError>
```

To:
```rust
pub async fn decompose_question(
    &self,
    context: &MarketContext,
) -> Result<DecomposedQuestions, TerminalError>
```

### 2b. Update the system prompt in `decompose_question`

Add market data awareness to the system prompt (around line 80):

```rust
let system_prompt = r#"You are a research analyst specializing in prediction markets.
Your task is to decompose a market question into sub-questions that will help gather comprehensive information.

You will receive:
1. Market title and description
2. Current market data (price, volume, recent trades)

Use the market data to inform your sub-questions. For example:
- If price recently moved significantly, ask "What caused the recent price movement?"
- If volume is high, consider what's driving interest
- Look at trade flow to understand market sentiment

For each sub-question, assign a category:
- "news": Recent news and developments
- "analysis": Expert analysis and opinions
- "historical": Historical precedents and data
- "expert_opinion": Domain expert perspectives

Respond with valid JSON in this exact format:
{
  "main_question": "The original question",
  "sub_questions": [
    {
      "question": "A specific sub-question",
      "category": "news",
      "search_query": "Optimized search query for this question"
    }
  ]
}

Generate 4-6 diverse sub-questions covering different angles."#;
```

### 2c. Update the user prompt in `decompose_question`

Replace the user_prompt construction (around line 103) with:

```rust
let user_prompt = format!(
    r#"## Market
Title: {}
Description: {}

## Current Market Data
- Current Price: {}
- 24h Change: {}
- 24h Volume: {}
- Total Volume: {}
{}
{}

Decompose this into research sub-questions that account for the current market state."#,
    context.title,
    context.description.as_deref().unwrap_or("No description"),
    format_price(context.current_price),
    format_price_change(context.current_price, context.price_24h_ago),
    format_volume(context.volume_24h),
    format_volume(context.total_volume),
    format_recent_trades(&context.recent_trades),
    format_order_book(&context.order_book_summary),
);
```

### 2d. Add helper formatting functions

Add at the bottom of the file:

```rust
fn format_price(price: Option<f64>) -> String {
    price.map(|p| format!("{:.1}%", p * 100.0))
        .unwrap_or_else(|| "Unknown".to_string())
}

fn format_price_change(current: Option<f64>, previous: Option<f64>) -> String {
    match (current, previous) {
        (Some(c), Some(p)) => {
            let change = (c - p) * 100.0;
            if change >= 0.0 {
                format!("+{:.1}%", change)
            } else {
                format!("{:.1}%", change)
            }
        }
        _ => "Unknown".to_string(),
    }
}

fn format_volume(volume: Option<f64>) -> String {
    volume.map(|v| {
        if v >= 1_000_000.0 {
            format!("${:.1}M", v / 1_000_000.0)
        } else if v >= 1_000.0 {
            format!("${:.1}K", v / 1_000.0)
        } else {
            format!("${:.0}", v)
        }
    }).unwrap_or_else(|| "Unknown".to_string())
}

fn format_recent_trades(trades: &[RecentTrade]) -> String {
    if trades.is_empty() {
        return String::new();
    }

    let mut output = String::from("\n## Recent Trades\n");
    for trade in trades.iter().take(10) {
        output.push_str(&format!(
            "- {} {:.1}% (${:.0}) at {}\n",
            trade.side.to_uppercase(),
            trade.price * 100.0,
            trade.size,
            trade.timestamp
        ));
    }
    output
}

fn format_order_book(summary: &Option<OrderBookSummary>) -> String {
    match summary {
        Some(ob) => format!(
            "\n## Order Book\n- Best Bid: {}\n- Best Ask: {}\n- Spread: {}\n- Bid Depth (10%): ${:.0}\n- Ask Depth (10%): ${:.0}",
            format_price(ob.best_bid),
            format_price(ob.best_ask),
            ob.spread.map(|s| format!("{:.1}%", s * 100.0)).unwrap_or_else(|| "Unknown".to_string()),
            ob.bid_depth_10pct,
            ob.ask_depth_10pct,
        ),
        None => String::new(),
    }
}
```

### 2e. Update `synthesize_report` similarly

Update the signature and user prompt to include market context. The AI should reference the market data when writing the executive summary and confidence assessment.

---

## Step 3: Update Research Service

**File:** `terminal-services/src/research_service.rs`

### 3a. Add method to build MarketContext

Add a new method around line 150:

```rust
async fn build_market_context(
    &self,
    platform: Platform,
    market_id: &str,
) -> Result<MarketContext, TerminalError> {
    // Fetch market details
    let market = self.market_service.get_market(platform, market_id).await?;

    // Fetch recent trades
    let trades = self.market_service
        .get_recent_trades(platform, market_id, 10)
        .await
        .unwrap_or_default();

    // Fetch order book
    let order_book = self.market_service
        .get_order_book(platform, market_id)
        .await
        .ok();

    // Build context
    Ok(MarketContext {
        title: market.title,
        description: market.description,
        current_price: market.outcomes.first().and_then(|o| o.price),
        price_24h_ago: None, // TODO: Fetch from candle history
        volume_24h: market.volume_24h,
        total_volume: market.volume,
        num_traders: market.num_traders,
        recent_trades: trades.into_iter().map(|t| RecentTrade {
            price: t.price,
            size: t.size,
            side: t.side.to_string(),
            timestamp: t.timestamp.to_rfc3339(),
        }).collect(),
        order_book_summary: order_book.map(|ob| OrderBookSummary {
            best_bid: ob.bids.first().map(|b| b.price),
            best_ask: ob.asks.first().map(|a| a.price),
            spread: calculate_spread(&ob),
            bid_depth_10pct: calculate_depth(&ob.bids, 0.10),
            ask_depth_10pct: calculate_depth(&ob.asks, 0.10),
        }),
    })
}
```

### 3b. Update `run_research_pipeline`

Modify line 154-234 to use the new context:

```rust
async fn run_research_pipeline(
    &self,
    job: &ResearchJob,
    _market: &terminal_core::PredictionMarket,  // Keep for backwards compat or remove
) -> Result<SynthesizedReport, TerminalError> {
    let job_id = &job.id;

    // Build rich market context
    let context = self.build_market_context(job.platform, &job.market_id).await?;

    // Step 1: Decompose question (now with context)
    self.update_status(job_id, ResearchStatus::Decomposing).await;
    self.update_progress(job_id, "Analyzing market question...", 1, 4, None).await;

    let questions = self.openai_client
        .decompose_question(&context)  // Changed from (title, description)
        .await?;

    // ... rest of pipeline stays the same until synthesis ...

    // Step 4: Synthesize report (now with context)
    let report = self.openai_client
        .synthesize_report(&context, &questions, &search_results)  // Changed
        .await?;

    Ok(report)
}
```

---

## Step 4: Fetch 24h Price History

**File:** `terminal-services/src/research_service.rs`

To calculate `price_24h_ago`, fetch candle data:

```rust
// In build_market_context, after fetching market:
let price_24h_ago = self.candle_service
    .get_candles(platform, market_id, "1h", 24)
    .await
    .ok()
    .and_then(|candles| candles.first().map(|c| c.close));
```

This requires `CandleService` to be available in `ResearchService`. Either:
- Add it as a field in `ResearchService`
- Or fetch via `MarketService` if it has a method

---

## Step 5: Update Follow-Up Research

**File:** `terminal-services/src/research_service.rs`

The `execute_followup_research` method (line 569) also needs market context when updating reports. Update `openai_client.update_report` to accept `MarketContext`.

---

## Step 6: Add Tests

**File:** `terminal-research/src/openai.rs` (add test module)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_price() {
        assert_eq!(format_price(Some(0.73)), "73.0%");
        assert_eq!(format_price(None), "Unknown");
    }

    #[test]
    fn test_format_price_change() {
        assert_eq!(format_price_change(Some(0.75), Some(0.70)), "+5.0%");
        assert_eq!(format_price_change(Some(0.65), Some(0.70)), "-5.0%");
    }

    #[test]
    fn test_format_volume() {
        assert_eq!(format_volume(Some(1_500_000.0)), "$1.5M");
        assert_eq!(format_volume(Some(50_000.0)), "$50.0K");
        assert_eq!(format_volume(Some(500.0)), "$500");
    }
}
```

---

## Step 7: Verify & Test

1. Run `cargo build -p terminal-research` to check compilation
2. Run `cargo build -p terminal-services` to check service compilation
3. Run `cargo test -p terminal-research` to run unit tests
4. Manual test: Start a research job and verify the logs show market data being passed
5. Check the generated report references market data appropriately

---

## Files Modified Summary

| File | Changes |
|------|---------|
| `terminal-research/src/types.rs` | Add `MarketContext`, `RecentTrade`, `OrderBookSummary` |
| `terminal-research/src/lib.rs` | Export new types |
| `terminal-research/src/openai.rs` | Update function signatures, prompts, add formatters |
| `terminal-services/src/research_service.rs` | Add `build_market_context`, update pipeline calls |

---

## Optional Enhancements (Future)

- [ ] Include outcome names for multi-outcome markets
- [ ] Add market category/tags for better search queries
- [ ] Include related markets data
- [ ] Add historical volatility metrics
- [ ] Include social sentiment if available
