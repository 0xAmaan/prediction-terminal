# Plan: Trading-Focused Research Output

## Problem Statement

The current deep research feature produces **general information** about a market topic. But prediction market traders don't need information—they need **edge**: insights that help identify when the market price is wrong.

**Current output:** "Here's what's happening with X"
**What traders need:** "Should I buy or sell X at the current price, and how much?"

---

## Goals

Transform research output from generic analysis to actionable trading intelligence:

1. **Fair Value Estimation** - AI estimates probability range and compares to market price
2. **Catalyst Timeline** - Upcoming events that could move the market
3. **Resolution Criteria Analysis** - Exact rules and potential edge cases
4. **Contrarian Analysis** - Why the crowd might be wrong
5. **Trader-Oriented Decomposition** - Better sub-questions for finding edge

---

## Step 1: Add Trading-Focused Types

**File:** `terminal-research/src/types.rs`

Add new structs after the existing `SynthesizedReport`:

```rust
/// Trading-focused analysis that accompanies the research report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingAnalysis {
    /// Estimated fair probability range (e.g., 0.52 to 0.58)
    pub fair_value_low: f64,
    pub fair_value_high: f64,

    /// Current market price for comparison
    pub current_price: f64,

    /// Calculated edge: midpoint of fair value minus current price
    /// Positive = market underpriced (buy signal), Negative = overpriced (sell signal)
    pub implied_edge: f64,

    /// How confident is the AI in this fair value estimate
    pub estimate_confidence: EstimateConfidence,

    /// Reasoning for the fair value estimate
    pub fair_value_reasoning: String,

    /// Upcoming events that could move the market
    pub catalysts: Vec<Catalyst>,

    /// Analysis of resolution rules and potential issues
    pub resolution_analysis: ResolutionAnalysis,

    /// The case against the current market consensus
    pub contrarian_case: ContrarianAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EstimateConfidence {
    /// Strong evidence, narrow range
    High,
    /// Decent evidence, moderate uncertainty
    Medium,
    /// Limited evidence, wide range, speculative
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalyst {
    /// When the event occurs (if known)
    pub date: Option<String>,
    /// Description of the event
    pub event: String,
    /// How much could this move the market
    pub expected_impact: CatalystImpact,
    /// Which direction if the event is favorable
    pub direction_if_positive: Option<Direction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalystImpact {
    /// Could move market 10%+
    High,
    /// Could move market 5-10%
    Medium,
    /// Could move market 1-5%
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Bullish,
    Bearish,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionAnalysis {
    /// Plain English summary of how this market resolves
    pub resolution_summary: String,
    /// The exact source/criteria used for resolution
    pub resolution_source: Option<String>,
    /// Potential ambiguities or edge cases
    pub ambiguity_flags: Vec<String>,
    /// Historical edge cases from similar markets (if any)
    pub historical_edge_cases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContrarianAnalysis {
    /// What the market consensus appears to be
    pub consensus_view: String,
    /// The case for why consensus might be wrong
    pub contrarian_case: String,
    /// Specific reasons the crowd could be mispricing
    pub mispricing_reasons: Vec<String>,
    /// What would need to happen for contrarian view to win
    pub contrarian_triggers: Vec<String>,
}
```

**Also update `SynthesizedReport`** to include the trading analysis:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesizedReport {
    pub title: String,
    pub executive_summary: String,
    pub sections: Vec<ReportSection>,
    pub key_factors: Vec<KeyFactor>,
    pub confidence_assessment: String,
    pub sources: Vec<String>,

    /// NEW: Trading-specific analysis
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trading_analysis: Option<TradingAnalysis>,
}
```

**Export from:** `terminal-research/src/lib.rs`

Add to the pub use statement:
```rust
pub use types::{
    TradingAnalysis, EstimateConfidence, Catalyst, CatalystImpact,
    Direction, ResolutionAnalysis, ContrarianAnalysis,
    // ... existing exports
};
```

---

## Step 2: Update Question Decomposition Prompt

**File:** `terminal-research/src/openai.rs`

Replace the system prompt in `decompose_question` (around line 80-120) with trader-focused decomposition:

```rust
let system_prompt = r#"You are a quantitative prediction market analyst. Your job is to find EDGE—where the market price might be wrong.

You will receive a market question with current price and trading data. Decompose it into 5-6 sub-questions designed to find trading edge, NOT just general information.

REQUIRED sub-question types (include at least one of each):

1. BASE RATE: "What is the historical frequency of [this type of event]?"
   - Category: "historical"
   - Example: "What percentage of incumbent presidents win re-election historically?"

2. MARKET PRICING: "What assumptions is the market making at [current price]?"
   - Category: "analysis"
   - Helps understand what's priced in

3. CATALYSTS: "What upcoming events could move this market significantly?"
   - Category: "news"
   - Focus on dated, scheduled events

4. CONTRARIAN: "What's the case against the current market consensus?"
   - Category: "analysis"
   - If market is high, search for bear case; if low, search for bull case

5. RESOLUTION: "How exactly does this market resolve and what are edge cases?"
   - Category: "analysis"
   - Focus on the specific resolution criteria

6. INFORMATION ASYMMETRY: "What might informed traders know that isn't public?"
   - Category: "news"
   - Recent insider activity, smart money movements

Respond with valid JSON:
{
  "main_question": "The original market question",
  "sub_questions": [
    {
      "question": "Specific sub-question",
      "category": "news|analysis|historical|expert_opinion",
      "search_query": "Optimized search query",
      "purpose": "base_rate|market_pricing|catalyst|contrarian|resolution|information_asymmetry"
    }
  ]
}

Generate exactly 6 sub-questions, one for each purpose."#;
```

Update the `DecomposedQuestions` struct to include purpose:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubQuestion {
    pub question: String,
    pub category: String,
    pub search_query: String,
    #[serde(default)]
    pub purpose: Option<String>,  // NEW: base_rate, catalyst, contrarian, etc.
}
```

---

## Step 3: Create Trading Analysis Synthesis

**File:** `terminal-research/src/openai.rs`

Add a new method to generate trading analysis after the main report:

```rust
/// Generate trading-focused analysis from research results
pub async fn generate_trading_analysis(
    &self,
    context: &MarketContext,
    report: &SynthesizedReport,
    search_results: &[SearchResultWithQuestion],
) -> Result<TradingAnalysis, TerminalError> {
    let system_prompt = r#"You are a quantitative trading analyst for prediction markets.
Your job is to translate research into actionable trading intelligence.

You will receive:
1. Market context (current price, volume, order flow)
2. A research report with findings
3. Raw search results

Your task is to produce a TradingAnalysis with:

1. FAIR VALUE ESTIMATE
   - Estimate a probability RANGE (not a point estimate)
   - Be specific: "52-58%" not "around 55%"
   - Consider base rates, current evidence, and uncertainty
   - Compare to current market price to identify edge
   - If genuinely uncertain, use a wide range (e.g., 40-60%)

2. CATALYSTS
   - List specific upcoming events with dates when known
   - Estimate impact level (high/medium/low)
   - Indicate likely direction if event is favorable

3. RESOLUTION ANALYSIS
   - Summarize EXACTLY how this market resolves
   - Flag ANY ambiguities in resolution criteria
   - Note historical edge cases from similar markets

4. CONTRARIAN ANALYSIS
   - State what the market consensus appears to be
   - Make the strongest case AGAINST that consensus
   - List specific reasons the crowd might be wrong
   - What would need to happen for the contrarian view to win

Be intellectually honest. If there's no edge, say so. If uncertainty is high, reflect that in a wide fair value range.

Respond with valid JSON matching the TradingAnalysis schema."#;

    let user_prompt = format!(
        r#"## Market
Title: {title}
Current Price: {price:.1}%

## Market Data
- 24h Volume: {volume}
- Recent Trade Flow: {trade_flow}
- Order Book: {orderbook}

## Research Report Summary
{executive_summary}

## Key Factors Found
{key_factors}

## Raw Search Findings
{search_findings}

Generate a TradingAnalysis for this market."#,
        title = context.title,
        price = context.current_price * 100.0,
        volume = format_volume(context.volume_24h),
        trade_flow = summarize_trade_flow(&context.recent_trades),
        orderbook = format_order_book(&context.order_book_summary),
        executive_summary = report.executive_summary,
        key_factors = format_key_factors(&report.key_factors),
        search_findings = summarize_search_results(search_results),
    );

    // Call GPT-4o with JSON mode
    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o")
        .response_format(ChatCompletionResponseFormat {
            r#type: ChatCompletionResponseFormatType::JsonObject,
        })
        .messages(vec![
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(system_prompt)
                    .build()?,
            ),
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessageArgs::default()
                    .content(user_prompt)
                    .build()?,
            ),
        ])
        .build()?;

    let response = self.client.chat().create(request).await?;
    let content = response.choices[0]
        .message
        .content
        .as_ref()
        .ok_or_else(|| TerminalError::Api("No content in response".into()))?;

    let mut analysis: TradingAnalysis = serde_json::from_str(content)?;

    // Calculate implied edge
    let fair_value_midpoint = (analysis.fair_value_low + analysis.fair_value_high) / 2.0;
    analysis.implied_edge = fair_value_midpoint - analysis.current_price;
    analysis.current_price = context.current_price;

    Ok(analysis)
}
```

Add helper functions:

```rust
fn summarize_trade_flow(trades: &[RecentTrade]) -> String {
    if trades.is_empty() {
        return "No recent trades".to_string();
    }

    let buys: Vec<_> = trades.iter().filter(|t| t.side == "buy").collect();
    let sells: Vec<_> = trades.iter().filter(|t| t.side == "sell").collect();

    let buy_volume: f64 = buys.iter().map(|t| t.size).sum();
    let sell_volume: f64 = sells.iter().map(|t| t.size).sum();

    format!(
        "{} buys (${:.0}), {} sells (${:.0}) - {}",
        buys.len(),
        buy_volume,
        sells.len(),
        sell_volume,
        if buy_volume > sell_volume * 1.5 {
            "bullish flow"
        } else if sell_volume > buy_volume * 1.5 {
            "bearish flow"
        } else {
            "balanced"
        }
    )
}

fn format_key_factors(factors: &[KeyFactor]) -> String {
    factors
        .iter()
        .map(|f| format!("- {} ({}, {} confidence)", f.factor, f.impact, f.confidence))
        .collect::<Vec<_>>()
        .join("\n")
}

fn summarize_search_results(results: &[SearchResultWithQuestion]) -> String {
    results
        .iter()
        .take(10)  // Limit context size
        .map(|r| format!(
            "Q: {}\nFindings: {}",
            r.question,
            r.results.iter()
                .take(2)
                .filter_map(|res| res.text.as_ref())
                .map(|t| truncate(t, 200))
                .collect::<Vec<_>>()
                .join(" | ")
        ))
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
```

---

## Step 4: Update Research Pipeline

**File:** `terminal-services/src/research_service.rs`

Modify `run_research_pipeline` to generate trading analysis after the main report:

Find the synthesis step (around line 200-220) and add trading analysis generation:

```rust
// Step 4: Synthesize report
self.update_status(job_id, ResearchStatus::Synthesizing).await;
self.update_progress(job_id, "Synthesizing research findings...", 4, 5, None).await;

let report = self.openai_client
    .synthesize_report(&context, &questions, &all_results)
    .await?;

// Step 5: Generate trading analysis (NEW)
self.update_progress(job_id, "Generating trading analysis...", 5, 5, None).await;

let trading_analysis = self.openai_client
    .generate_trading_analysis(&context, &report, &all_results)
    .await
    .ok();  // Don't fail the whole job if trading analysis fails

// Attach trading analysis to report
let final_report = SynthesizedReport {
    trading_analysis,
    ..report
};

Ok(final_report)
```

Update `ResearchProgress` to reflect 5 steps:

```rust
// In types.rs, update the step counts in progress updates
// Or just update the total_steps parameter in update_progress calls
```

---

## Step 5: Update Frontend Types

**File:** `frontend/src/lib/types.ts`

Add TypeScript interfaces matching the Rust types:

```typescript
export interface TradingAnalysis {
  fair_value_low: number;
  fair_value_high: number;
  current_price: number;
  implied_edge: number;
  estimate_confidence: 'high' | 'medium' | 'low';
  fair_value_reasoning: string;
  catalysts: Catalyst[];
  resolution_analysis: ResolutionAnalysis;
  contrarian_case: ContrarianAnalysis;
}

export interface Catalyst {
  date: string | null;
  event: string;
  expected_impact: 'high' | 'medium' | 'low';
  direction_if_positive: 'bullish' | 'bearish' | null;
}

export interface ResolutionAnalysis {
  resolution_summary: string;
  resolution_source: string | null;
  ambiguity_flags: string[];
  historical_edge_cases: string[];
}

export interface ContrarianAnalysis {
  consensus_view: string;
  contrarian_case: string;
  mispricing_reasons: string[];
  contrarian_triggers: string[];
}

// Update SynthesizedReport interface
export interface SynthesizedReport {
  title: string;
  executive_summary: string;
  sections: ReportSection[];
  key_factors: KeyFactor[];
  confidence_assessment: string;
  sources: string[];
  trading_analysis?: TradingAnalysis;  // NEW
}
```

---

## Step 6: Create Trading Analysis UI Component

**File:** `frontend/src/components/research/trading-analysis.tsx` (NEW FILE)

```tsx
'use client';

import { TradingAnalysis } from '@/lib/types';
import { cn } from '@/lib/utils';
import {
  TrendUp,
  TrendDown,
  Warning,
  Calendar,
  Target,
  Scales
} from '@phosphor-icons/react';

interface TradingAnalysisProps {
  analysis: TradingAnalysis;
}

export function TradingAnalysisPanel({ analysis }: TradingAnalysisProps) {
  const edgePercent = (analysis.implied_edge * 100).toFixed(1);
  const isPositiveEdge = analysis.implied_edge > 0.02; // 2% threshold
  const isNegativeEdge = analysis.implied_edge < -0.02;

  return (
    <div className="space-y-6">
      {/* Fair Value Card */}
      <div className="rounded-lg border border-grey-800 bg-grey-900/50 p-4">
        <div className="flex items-center gap-2 mb-3">
          <Target className="h-5 w-5 text-grey-400" />
          <h3 className="font-medium text-grey-100">Fair Value Estimate</h3>
          <span className={cn(
            "ml-auto px-2 py-0.5 rounded text-xs",
            analysis.estimate_confidence === 'high' && "bg-green-500/20 text-green-400",
            analysis.estimate_confidence === 'medium' && "bg-yellow-500/20 text-yellow-400",
            analysis.estimate_confidence === 'low' && "bg-grey-500/20 text-grey-400",
          )}>
            {analysis.estimate_confidence} confidence
          </span>
        </div>

        <div className="flex items-baseline gap-4 mb-3">
          <div className="text-3xl font-bold text-grey-100">
            {(analysis.fair_value_low * 100).toFixed(0)}–{(analysis.fair_value_high * 100).toFixed(0)}%
          </div>
          <div className="text-grey-500">
            vs market at {(analysis.current_price * 100).toFixed(1)}%
          </div>
        </div>

        {/* Edge indicator */}
        <div className={cn(
          "flex items-center gap-2 p-2 rounded",
          isPositiveEdge && "bg-green-500/10",
          isNegativeEdge && "bg-red-500/10",
          !isPositiveEdge && !isNegativeEdge && "bg-grey-800/50"
        )}>
          {isPositiveEdge ? (
            <TrendUp className="h-5 w-5 text-green-400" />
          ) : isNegativeEdge ? (
            <TrendDown className="h-5 w-5 text-red-400" />
          ) : (
            <Scales className="h-5 w-5 text-grey-400" />
          )}
          <span className={cn(
            "font-medium",
            isPositiveEdge && "text-green-400",
            isNegativeEdge && "text-red-400",
            !isPositiveEdge && !isNegativeEdge && "text-grey-400"
          )}>
            {isPositiveEdge
              ? `+${edgePercent}% edge (market may be underpriced)`
              : isNegativeEdge
              ? `${edgePercent}% edge (market may be overpriced)`
              : "No significant edge detected"
            }
          </span>
        </div>

        <p className="text-sm text-grey-400 mt-3">
          {analysis.fair_value_reasoning}
        </p>
      </div>

      {/* Catalysts */}
      {analysis.catalysts.length > 0 && (
        <div className="rounded-lg border border-grey-800 bg-grey-900/50 p-4">
          <div className="flex items-center gap-2 mb-3">
            <Calendar className="h-5 w-5 text-grey-400" />
            <h3 className="font-medium text-grey-100">Upcoming Catalysts</h3>
          </div>
          <div className="space-y-2">
            {analysis.catalysts.map((catalyst, i) => (
              <div key={i} className="flex items-start gap-3 text-sm">
                <span className={cn(
                  "px-1.5 py-0.5 rounded text-xs shrink-0",
                  catalyst.expected_impact === 'high' && "bg-red-500/20 text-red-400",
                  catalyst.expected_impact === 'medium' && "bg-yellow-500/20 text-yellow-400",
                  catalyst.expected_impact === 'low' && "bg-grey-500/20 text-grey-400",
                )}>
                  {catalyst.expected_impact}
                </span>
                <span className="text-grey-300">{catalyst.event}</span>
                {catalyst.date && (
                  <span className="text-grey-500 ml-auto shrink-0">{catalyst.date}</span>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Resolution Analysis */}
      <div className="rounded-lg border border-grey-800 bg-grey-900/50 p-4">
        <div className="flex items-center gap-2 mb-3">
          <Warning className="h-5 w-5 text-grey-400" />
          <h3 className="font-medium text-grey-100">Resolution Criteria</h3>
        </div>
        <p className="text-sm text-grey-300 mb-3">
          {analysis.resolution_analysis.resolution_summary}
        </p>
        {analysis.resolution_analysis.ambiguity_flags.length > 0 && (
          <div className="mt-3 p-2 rounded bg-yellow-500/10 border border-yellow-500/20">
            <p className="text-xs font-medium text-yellow-400 mb-1">Potential Ambiguities:</p>
            <ul className="text-xs text-yellow-300/80 space-y-1">
              {analysis.resolution_analysis.ambiguity_flags.map((flag, i) => (
                <li key={i}>• {flag}</li>
              ))}
            </ul>
          </div>
        )}
      </div>

      {/* Contrarian View */}
      <div className="rounded-lg border border-grey-800 bg-grey-900/50 p-4">
        <div className="flex items-center gap-2 mb-3">
          <Scales className="h-5 w-5 text-grey-400" />
          <h3 className="font-medium text-grey-100">Contrarian Analysis</h3>
        </div>
        <div className="space-y-3 text-sm">
          <div>
            <p className="text-grey-500 text-xs mb-1">Market Consensus</p>
            <p className="text-grey-300">{analysis.contrarian_case.consensus_view}</p>
          </div>
          <div>
            <p className="text-grey-500 text-xs mb-1">The Case Against</p>
            <p className="text-grey-300">{analysis.contrarian_case.contrarian_case}</p>
          </div>
          {analysis.contrarian_case.mispricing_reasons.length > 0 && (
            <div>
              <p className="text-grey-500 text-xs mb-1">Why Crowd Might Be Wrong</p>
              <ul className="text-grey-400 space-y-1">
                {analysis.contrarian_case.mispricing_reasons.map((reason, i) => (
                  <li key={i}>• {reason}</li>
                ))}
              </ul>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
```

---

## Step 7: Integrate into Research Document

**File:** `frontend/src/components/research/research-document.tsx`

Import and add the trading analysis panel:

```tsx
import { TradingAnalysisPanel } from './trading-analysis';

// In the component, after executive summary but before sections:
{report.trading_analysis && (
  <div className="mb-8">
    <h2 className="text-lg font-semibold text-grey-100 mb-4">Trading Analysis</h2>
    <TradingAnalysisPanel analysis={report.trading_analysis} />
  </div>
)}
```

---

## Step 8: Add Better Search Queries for Each Purpose

**File:** `terminal-services/src/research_service.rs`

Update the search logic to use purpose-specific search strategies:

```rust
async fn execute_search_for_question(
    &self,
    question: &SubQuestion,
) -> Result<Vec<ExaSearchResult>, TerminalError> {
    let search_config = match question.purpose.as_deref() {
        Some("base_rate") => SearchConfig {
            query: format!("{} historical statistics data", question.search_query),
            num_results: 5,
            search_type: "auto",
            include_domains: vec!["wikipedia.org", "statista.com", "pewresearch.org"],
        },
        Some("catalyst") => SearchConfig {
            query: format!("{} upcoming schedule calendar 2024 2025", question.search_query),
            num_results: 5,
            search_type: "news",
            days_back: 30,
        },
        Some("contrarian") => SearchConfig {
            query: format!("{} skeptic criticism unlikely wrong", question.search_query),
            num_results: 5,
            search_type: "auto",
        },
        Some("resolution") => SearchConfig {
            query: format!("{} rules criteria definition resolution", question.search_query),
            num_results: 5,
            search_type: "auto",
            include_domains: vec!["polymarket.com", "kalshi.com"],
        },
        _ => SearchConfig::default_for(&question.search_query),
    };

    self.exa_client.search(search_config).await
}
```

---

## Step 9: Add Tests

**File:** `terminal-research/src/openai.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_implied_edge_calculation() {
        let analysis = TradingAnalysis {
            fair_value_low: 0.52,
            fair_value_high: 0.58,
            current_price: 0.45,
            implied_edge: 0.0, // Will be calculated
            // ... other fields
        };

        let midpoint = (analysis.fair_value_low + analysis.fair_value_high) / 2.0;
        let edge = midpoint - analysis.current_price;

        assert!((edge - 0.10).abs() < 0.001); // ~10% edge
    }

    #[test]
    fn test_trade_flow_summary() {
        let trades = vec![
            RecentTrade { price: 0.5, size: 100.0, side: "buy".into(), timestamp: "".into() },
            RecentTrade { price: 0.5, size: 100.0, side: "buy".into(), timestamp: "".into() },
            RecentTrade { price: 0.5, size: 50.0, side: "sell".into(), timestamp: "".into() },
        ];

        let summary = summarize_trade_flow(&trades);
        assert!(summary.contains("bullish flow"));
    }
}
```

---

## Files Modified Summary

| File | Changes |
|------|---------|
| `terminal-research/src/types.rs` | Add `TradingAnalysis`, `Catalyst`, `ResolutionAnalysis`, `ContrarianAnalysis` |
| `terminal-research/src/lib.rs` | Export new types |
| `terminal-research/src/openai.rs` | Update decomposition prompt, add `generate_trading_analysis` method |
| `terminal-services/src/research_service.rs` | Add Step 5 to pipeline, purpose-specific search |
| `frontend/src/lib/types.ts` | Add TypeScript interfaces |
| `frontend/src/components/research/trading-analysis.tsx` | New component |
| `frontend/src/components/research/research-document.tsx` | Integrate trading analysis panel |

---

## Verification Steps

1. `cargo build -p terminal-research` - Check types compile
2. `cargo build -p terminal-services` - Check service compiles
3. `cargo test -p terminal-research` - Run unit tests
4. `cd frontend && bun run build` - Check frontend builds
5. Manual test: Run research on a market and verify:
   - Fair value range is displayed
   - Implied edge is calculated correctly
   - Catalysts have dates when available
   - Resolution analysis flags ambiguities
   - Contrarian case argues against consensus
