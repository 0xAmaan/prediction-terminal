# Whale Tracking Implementation Guide

## Overview

This document describes how to implement whale tracking for Polymarket prediction markets. The goal is to identify wallets with historically high win rates ("smart money") and surface their positions on markets the user is researching, integrating this data into the research document.

**Why this matters:** 86% of Polymarket accounts have negative P&L. Following the 14% of profitable traders provides actionable alpha that pure news/research cannot.

---

## Architecture Decision

### Option A: Third-Party API Integration (Recommended for MVP)

Use existing whale tracking services that already index Polymarket's on-chain data:

| Service | API Available | Key Features | Pricing |
|---------|--------------|--------------|---------|
| [Polywhaler](https://polywhaler.com) | Yes | Whale alerts, AI predictions, $10k+ trade tracking | Freemium |
| [PolyTrack](https://polytrackhq.app) | Yes | Cluster detection, wallet linking, historical PnL | Paid tiers |
| [Polygrid](https://polygrid.xyz) | Yes | Live trade feed, insider tracking | Free tier |
| [PolyWatch](https://polywatch.tech) | Telegram bot | Free whale alerts >$25k | Free |

**Recommendation:** Start with Polygrid or build a minimal indexer using Polymarket's public APIs.

### Option B: Build Own Indexer (More Control, More Work)

Index Polygon blockchain directly using:
- Polymarket CLOB contract addresses
- Goldsky or The Graph for indexing
- Store in local SQLite/Postgres

---

## Implementation Plan

### Phase 1: Data Model & Types

#### 1.1 Add Types to `terminal-core/src/lib.rs`

```rust
// Add to terminal-core/src/lib.rs or create terminal-core/src/whale.rs

/// A tracked wallet with historical performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedWallet {
    /// Polygon wallet address (0x...)
    pub address: String,
    /// Human-readable label if known (e.g., "Theo Wallet 3")
    pub label: Option<String>,
    /// Total profit/loss in USD
    pub total_pnl: f64,
    /// Win rate (0.0 to 1.0)
    pub win_rate: f64,
    /// Total number of resolved positions
    pub total_positions: u32,
    /// Total volume traded in USD
    pub total_volume: f64,
    /// Wallet tier based on performance
    pub tier: WalletTier,
    /// Last activity timestamp
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WalletTier {
    /// Top 1% - $100k+ profit, >60% win rate
    Elite,
    /// Top 5% - $10k+ profit, >55% win rate
    Whale,
    /// Top 15% - Profitable, >50% win rate
    Profitable,
    /// Unclassified or insufficient data
    Unknown,
}

/// A whale's position on a specific market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhalePosition {
    /// The wallet holding this position
    pub wallet: TrackedWallet,
    /// Market ID (Polymarket condition_id or token_id)
    pub market_id: String,
    /// Position side: "yes" or "no"
    pub side: String,
    /// Position size in shares
    pub size: f64,
    /// Average entry price (0.0 to 1.0)
    pub avg_entry_price: f64,
    /// Total USD value of position
    pub usd_value: f64,
    /// When position was opened/last modified
    pub timestamp: DateTime<Utc>,
}

/// Aggregated whale sentiment for a market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhaleConsensus {
    /// Market ID
    pub market_id: String,
    /// Number of tracked whales with YES positions
    pub yes_whale_count: u32,
    /// Number of tracked whales with NO positions
    pub no_whale_count: u32,
    /// Total USD in YES positions from whales
    pub yes_whale_volume: f64,
    /// Total USD in NO positions from whales
    pub no_whale_volume: f64,
    /// Weighted average entry price for YES whales
    pub avg_yes_entry: Option<f64>,
    /// Weighted average entry price for NO whales
    pub avg_no_entry: Option<f64>,
    /// Overall signal: "bullish", "bearish", or "neutral"
    pub signal: WhaleSignal,
    /// Confidence in the signal based on whale count and volume
    pub confidence: SignalConfidence,
    /// Individual whale positions for detail view
    pub positions: Vec<WhalePosition>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WhaleSignal {
    Bullish,  // Whales favor YES
    Bearish,  // Whales favor NO
    Neutral,  // Mixed or insufficient data
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalConfidence {
    High,    // 5+ whales, >$100k total
    Medium,  // 2-4 whales, >$25k total
    Low,     // 1 whale or <$25k total
}
```

#### 1.2 Add Types to Frontend `frontend/src/lib/types.ts`

```typescript
// Add to frontend/src/lib/types.ts

export type WalletTier = "elite" | "whale" | "profitable" | "unknown";
export type WhaleSignal = "bullish" | "bearish" | "neutral";
export type SignalConfidence = "high" | "medium" | "low";

export interface TrackedWallet {
  address: string;
  label: string | null;
  total_pnl: number;
  win_rate: number;
  total_positions: number;
  total_volume: number;
  tier: WalletTier;
  last_active: string;
}

export interface WhalePosition {
  wallet: TrackedWallet;
  market_id: string;
  side: "yes" | "no";
  size: number;
  avg_entry_price: number;
  usd_value: number;
  timestamp: string;
}

export interface WhaleConsensus {
  market_id: string;
  yes_whale_count: number;
  no_whale_count: number;
  yes_whale_volume: number;
  no_whale_volume: number;
  avg_yes_entry: number | null;
  avg_no_entry: number | null;
  signal: WhaleSignal;
  confidence: SignalConfidence;
  positions: WhalePosition[];
}
```

---

### Phase 2: Create Whale Tracking Service

#### 2.1 Create New Crate `terminal-whale/`

```
terminal-whale/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── client.rs      # API client for whale data
│   ├── indexer.rs     # Optional: own blockchain indexer
│   └── analyzer.rs    # Consensus calculation logic
```

#### 2.2 `terminal-whale/Cargo.toml`

```toml
[package]
name = "terminal-whale"
version = "0.1.0"
edition = "2024"

[dependencies]
terminal-core = { path = "../terminal-core" }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2.0"
```

#### 2.3 `terminal-whale/src/client.rs`

Implement client that fetches whale data. Two approaches:

**Approach A: Use Polymarket's Public Activity API**

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use terminal_core::{TrackedWallet, WhalePosition, WalletTier};

const POLYMARKET_ACTIVITY_API: &str = "https://data-api.polymarket.com/activity";
const POLYMARKET_PROFILE_API: &str = "https://polymarket.com/api/profile";

pub struct WhaleClient {
    client: Client,
    /// Known whale addresses (can be loaded from config/db)
    tracked_wallets: Vec<String>,
}

impl WhaleClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            // Seed with known profitable wallets
            // These can be discovered via leaderboards or on-chain analysis
            tracked_wallets: vec![
                // Add known whale addresses here
                // Can be populated from: https://polymarket.com/leaderboard
            ],
        }
    }

    /// Fetch positions for a specific market from tracked whales
    pub async fn get_whale_positions(&self, market_id: &str) -> Result<Vec<WhalePosition>, Error> {
        let mut positions = Vec::new();

        for wallet_addr in &self.tracked_wallets {
            // Fetch wallet's positions
            let url = format!(
                "https://data-api.polymarket.com/positions?user={}&market={}",
                wallet_addr, market_id
            );

            let resp: PositionsResponse = self.client
                .get(&url)
                .send()
                .await?
                .json()
                .await?;

            // Convert to WhalePosition if they have a position
            if let Some(pos) = resp.positions.first() {
                let wallet = self.get_wallet_stats(wallet_addr).await?;
                positions.push(WhalePosition {
                    wallet,
                    market_id: market_id.to_string(),
                    side: pos.outcome.to_lowercase(),
                    size: pos.size,
                    avg_entry_price: pos.avg_price,
                    usd_value: pos.size * pos.avg_price,
                    timestamp: pos.timestamp,
                });
            }
        }

        Ok(positions)
    }

    /// Fetch wallet stats from Polymarket profile API
    async fn get_wallet_stats(&self, address: &str) -> Result<TrackedWallet, Error> {
        let url = format!("{}/{}", POLYMARKET_PROFILE_API, address);
        let resp: ProfileResponse = self.client.get(&url).send().await?.json().await?;

        let tier = match (resp.profit_loss, resp.positions_won as f64 / resp.positions_total as f64) {
            (pnl, wr) if pnl > 100_000.0 && wr > 0.60 => WalletTier::Elite,
            (pnl, wr) if pnl > 10_000.0 && wr > 0.55 => WalletTier::Whale,
            (pnl, wr) if pnl > 0.0 && wr > 0.50 => WalletTier::Profitable,
            _ => WalletTier::Unknown,
        };

        Ok(TrackedWallet {
            address: address.to_string(),
            label: resp.username,
            total_pnl: resp.profit_loss,
            win_rate: resp.positions_won as f64 / resp.positions_total.max(1) as f64,
            total_positions: resp.positions_total,
            total_volume: resp.volume,
            tier,
            last_active: resp.last_trade_timestamp,
        })
    }
}
```

**Approach B: Use Third-Party Whale Tracker API**

```rust
// If using Polygrid or similar service
const POLYGRID_API: &str = "https://api.polygrid.xyz/v1";

impl WhaleClient {
    pub async fn get_market_whales(&self, market_id: &str) -> Result<Vec<WhalePosition>, Error> {
        let url = format!("{}/markets/{}/whales", POLYGRID_API, market_id);

        let resp: PolygridWhalesResponse = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?
            .json()
            .await?;

        // Convert to our types
        Ok(resp.whales.into_iter().map(|w| w.into()).collect())
    }
}
```

#### 2.4 `terminal-whale/src/analyzer.rs`

```rust
use terminal_core::{WhaleConsensus, WhalePosition, WhaleSignal, SignalConfidence};

pub fn calculate_consensus(positions: Vec<WhalePosition>) -> WhaleConsensus {
    let yes_positions: Vec<_> = positions.iter().filter(|p| p.side == "yes").collect();
    let no_positions: Vec<_> = positions.iter().filter(|p| p.side == "no").collect();

    let yes_volume: f64 = yes_positions.iter().map(|p| p.usd_value).sum();
    let no_volume: f64 = no_positions.iter().map(|p| p.usd_value).sum();
    let total_volume = yes_volume + no_volume;

    // Calculate weighted average entry prices
    let avg_yes_entry = if yes_volume > 0.0 {
        Some(yes_positions.iter().map(|p| p.avg_entry_price * p.usd_value).sum::<f64>() / yes_volume)
    } else {
        None
    };

    let avg_no_entry = if no_volume > 0.0 {
        Some(no_positions.iter().map(|p| p.avg_entry_price * p.usd_value).sum::<f64>() / no_volume)
    } else {
        None
    };

    // Determine signal
    let signal = if yes_volume > no_volume * 1.5 {
        WhaleSignal::Bullish
    } else if no_volume > yes_volume * 1.5 {
        WhaleSignal::Bearish
    } else {
        WhaleSignal::Neutral
    };

    // Determine confidence
    let whale_count = positions.len();
    let confidence = match (whale_count, total_volume) {
        (c, v) if c >= 5 && v > 100_000.0 => SignalConfidence::High,
        (c, v) if c >= 2 && v > 25_000.0 => SignalConfidence::Medium,
        _ => SignalConfidence::Low,
    };

    WhaleConsensus {
        market_id: positions.first().map(|p| p.market_id.clone()).unwrap_or_default(),
        yes_whale_count: yes_positions.len() as u32,
        no_whale_count: no_positions.len() as u32,
        yes_whale_volume: yes_volume,
        no_whale_volume: no_volume,
        avg_yes_entry,
        avg_no_entry,
        signal,
        confidence,
        positions,
    }
}
```

---

### Phase 3: API Endpoint

#### 3.1 Add Endpoint to `terminal-api/src/routes/`

Create `terminal-api/src/routes/whale.rs`:

```rust
use axum::{
    extract::{Path, State},
    Json,
};
use terminal_core::WhaleConsensus;
use crate::AppState;

/// GET /api/markets/:platform/:market_id/whales
pub async fn get_market_whales(
    State(state): State<AppState>,
    Path((platform, market_id)): Path<(String, String)>,
) -> Result<Json<WhaleConsensus>, ApiError> {
    // Only Polymarket has on-chain data
    if platform != "polymarket" {
        return Ok(Json(WhaleConsensus::empty(&market_id)));
    }

    let positions = state.whale_client.get_whale_positions(&market_id).await?;
    let consensus = terminal_whale::calculate_consensus(positions);

    Ok(Json(consensus))
}
```

Register in router:

```rust
// In terminal-api/src/routes/mod.rs or main router
.route("/api/markets/:platform/:market_id/whales", get(whale::get_market_whales))
```

---

### Phase 4: Integrate Into Research

This is the key integration - whale data should appear in the research document.

#### 4.1 Update Research Types

Add whale consensus to the research report. In `terminal-research/src/openai.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesizedReport {
    pub title: String,
    pub executive_summary: String,
    pub sections: Vec<ReportSection>,
    pub key_factors: Vec<KeyFactor>,
    pub confidence_assessment: String,
    pub sources: Vec<String>,
    pub trading_analysis: Option<TradingAnalysis>,
    // ADD THIS:
    pub whale_analysis: Option<WhaleAnalysis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhaleAnalysis {
    /// The whale consensus data
    pub consensus: WhaleConsensus,
    /// AI-generated interpretation of whale activity
    pub interpretation: String,
    /// How whale sentiment compares to market price
    pub vs_market: String,
    /// Notable individual positions to highlight
    pub notable_positions: Vec<NotablePosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotablePosition {
    pub wallet_label: String,
    pub wallet_tier: WalletTier,
    pub side: String,
    pub size_usd: f64,
    pub entry_vs_current: f64,  // How much they're up/down
    pub significance: String,   // AI explanation of why this matters
}
```

#### 4.2 Update Research Service

In `terminal-services/src/research_service.rs`, fetch whale data during research:

```rust
impl ResearchService {
    pub async fn run_research(&self, job: &mut ResearchJob) -> Result<()> {
        // ... existing decomposition and search steps ...

        // NEW: Fetch whale data for Polymarket markets
        let whale_consensus = if job.platform == Platform::Polymarket {
            self.whale_client.get_whale_positions(&job.market_id).await.ok()
                .map(|positions| terminal_whale::calculate_consensus(positions))
        } else {
            None
        };

        // Pass whale data to synthesis
        let report = self.openai_client.synthesize_report(
            &job.market_title,
            &decomposed,
            &search_results,
            &market_context,
            whale_consensus.as_ref(),  // NEW PARAMETER
        ).await?;

        // ... rest of pipeline ...
    }
}
```

#### 4.3 Update Synthesis Prompt

In `terminal-research/src/openai.rs`, update the synthesis prompt to include whale data:

```rust
fn build_synthesis_prompt(
    market_title: &str,
    questions: &DecomposedQuestions,
    search_results: &[SearchResultGroup],
    market_context: &MarketContext,
    whale_consensus: Option<&WhaleConsensus>,  // NEW
) -> String {
    let mut prompt = format!(
        r#"You are a prediction market research analyst. Synthesize a comprehensive report.

## Market
Title: {market_title}
Current Price: {price:.1}%
24h Volume: ${volume:.0}

## Research Data
{research_sections}
"#,
        market_title = market_title,
        price = market_context.current_price.unwrap_or(0.0) * 100.0,
        volume = market_context.volume_24h.unwrap_or(0.0),
        research_sections = format_search_results(search_results),
    );

    // ADD WHALE DATA TO PROMPT
    if let Some(whales) = whale_consensus {
        prompt.push_str(&format!(r#"
## Whale Activity (Smart Money Tracking)
This data shows positions from historically profitable traders (>50% win rate, positive P&L).

- YES whale count: {} traders
- NO whale count: {} traders
- YES whale volume: ${:.0}
- NO whale volume: ${:.0}
- Average YES entry price: {:.1}%
- Average NO entry price: {:.1}%
- Overall whale signal: {:?}
- Signal confidence: {:?}

Notable positions:
{}
"#,
            whales.yes_whale_count,
            whales.no_whale_count,
            whales.yes_whale_volume,
            whales.no_whale_volume,
            whales.avg_yes_entry.unwrap_or(0.0) * 100.0,
            whales.avg_no_entry.unwrap_or(0.0) * 100.0,
            whales.signal,
            whales.confidence,
            format_notable_positions(&whales.positions),
        ));
    }

    prompt.push_str(r#"
## Output Format

Return a JSON object with this structure:
{
  "title": "...",
  "executive_summary": "...",
  "sections": [...],
  "key_factors": [...],
  "confidence_assessment": "...",
  "sources": [...],
  "trading_analysis": {...},
  "whale_analysis": {
    "interpretation": "Explain what whale positioning suggests about this market",
    "vs_market": "Compare whale sentiment to current market price - are they aligned or divergent?",
    "notable_positions": [
      {
        "wallet_label": "...",
        "wallet_tier": "elite|whale|profitable",
        "side": "yes|no",
        "size_usd": 50000,
        "entry_vs_current": 0.05,
        "significance": "Why this position matters"
      }
    ]
  }
}

IMPORTANT for whale_analysis:
- If whales are heavily positioned opposite to market consensus, this is a STRONG signal
- Note if whale entry prices suggest they got in early (potential information edge)
- Highlight any Elite tier wallets - these have 60%+ win rates
- If whale data is absent or insufficient, set whale_analysis to null
"#);

    prompt
}
```

---

### Phase 5: Frontend Display

#### 5.1 Create Whale Consensus Component

Create `frontend/src/components/research/whale-consensus.tsx`:

```tsx
import { TrendingUp, TrendingDown, Minus, Users, DollarSign, Target } from "lucide-react";
import type { WhaleAnalysis, WhaleConsensus } from "@/lib/types";

interface WhaleConsensusProps {
  analysis: WhaleAnalysis;
  currentPrice: number;
}

export function WhaleConsensusDisplay({ analysis, currentPrice }: WhaleConsensusProps) {
  const { consensus, interpretation, vs_market, notable_positions } = analysis;

  const signalColor = {
    bullish: "#4DBE95",
    bearish: "#D84F68",
    neutral: "#7D8B96",
  }[consensus.signal];

  const SignalIcon = {
    bullish: TrendingUp,
    bearish: TrendingDown,
    neutral: Minus,
  }[consensus.signal];

  return (
    <div className="space-y-4 p-4 rounded-lg" style={{ backgroundColor: "rgba(255,255,255,0.02)" }}>
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium flex items-center gap-2">
          <Users className="h-4 w-4" />
          Whale Consensus
        </h3>
        <div
          className="flex items-center gap-1 px-2 py-1 rounded text-xs font-medium"
          style={{ backgroundColor: `${signalColor}20`, color: signalColor }}
        >
          <SignalIcon className="h-3 w-3" />
          {consensus.signal.toUpperCase()}
          <span className="opacity-60">({consensus.confidence})</span>
        </div>
      </div>

      {/* Volume bars */}
      <div className="space-y-2">
        <div className="flex items-center gap-2">
          <span className="text-xs w-8" style={{ color: "#4DBE95" }}>YES</span>
          <div className="flex-1 h-2 rounded-full overflow-hidden" style={{ backgroundColor: "rgba(255,255,255,0.1)" }}>
            <div
              className="h-full rounded-full"
              style={{
                width: `${(consensus.yes_whale_volume / (consensus.yes_whale_volume + consensus.no_whale_volume)) * 100}%`,
                backgroundColor: "#4DBE95"
              }}
            />
          </div>
          <span className="text-xs w-20 text-right" style={{ color: "#7D8B96" }}>
            ${(consensus.yes_whale_volume / 1000).toFixed(0)}k ({consensus.yes_whale_count})
          </span>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-xs w-8" style={{ color: "#D84F68" }}>NO</span>
          <div className="flex-1 h-2 rounded-full overflow-hidden" style={{ backgroundColor: "rgba(255,255,255,0.1)" }}>
            <div
              className="h-full rounded-full"
              style={{
                width: `${(consensus.no_whale_volume / (consensus.yes_whale_volume + consensus.no_whale_volume)) * 100}%`,
                backgroundColor: "#D84F68"
              }}
            />
          </div>
          <span className="text-xs w-20 text-right" style={{ color: "#7D8B96" }}>
            ${(consensus.no_whale_volume / 1000).toFixed(0)}k ({consensus.no_whale_count})
          </span>
        </div>
      </div>

      {/* Interpretation */}
      <p className="text-sm" style={{ color: "#EEF0F1" }}>{interpretation}</p>
      <p className="text-xs" style={{ color: "#7D8B96" }}>{vs_market}</p>

      {/* Notable positions */}
      {notable_positions.length > 0 && (
        <div className="space-y-2 pt-2 border-t" style={{ borderColor: "rgba(255,255,255,0.06)" }}>
          <span className="text-xs font-medium" style={{ color: "#7D8B96" }}>Notable Positions</span>
          {notable_positions.map((pos, i) => (
            <div key={i} className="flex items-center justify-between text-xs">
              <div className="flex items-center gap-2">
                <span
                  className="px-1.5 py-0.5 rounded"
                  style={{
                    backgroundColor: pos.wallet_tier === "elite" ? "rgba(255,215,0,0.2)" : "rgba(255,255,255,0.05)",
                    color: pos.wallet_tier === "elite" ? "#FFD700" : "#7D8B96"
                  }}
                >
                  {pos.wallet_tier}
                </span>
                <span style={{ color: "#EEF0F1" }}>{pos.wallet_label || "Anon"}</span>
              </div>
              <div className="flex items-center gap-2">
                <span style={{ color: pos.side === "yes" ? "#4DBE95" : "#D84F68" }}>
                  {pos.side.toUpperCase()}
                </span>
                <span style={{ color: "#7D8B96" }}>${(pos.size_usd / 1000).toFixed(0)}k</span>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
```

#### 5.2 Integrate Into Research Document

In `frontend/src/components/research/research-document.tsx`, add the whale section:

```tsx
import { WhaleConsensusDisplay } from "./whale-consensus";

// Inside the component, after trading analysis section:
{report.whale_analysis && (
  <section className="space-y-4">
    <h2 className="text-lg font-semibold" style={{ color: fey.grey100 }}>
      Smart Money Analysis
    </h2>
    <WhaleConsensusDisplay
      analysis={report.whale_analysis}
      currentPrice={parseFloat(market?.yes_price || "0")}
    />
  </section>
)}
```

---

### Phase 6: Data Sources for Whale Discovery

#### 6.1 Polymarket Leaderboard Scraping

The Polymarket leaderboard at `https://polymarket.com/leaderboard` shows top traders. These can be periodically scraped to populate the tracked wallet list.

```rust
// Pseudocode for leaderboard scraping
pub async fn scrape_leaderboard(&self) -> Result<Vec<String>> {
    // Polymarket leaderboard API (unofficial)
    let url = "https://polymarket.com/api/leaderboard?period=all&limit=100";
    let resp: LeaderboardResponse = self.client.get(url).send().await?.json().await?;

    Ok(resp.users
        .into_iter()
        .filter(|u| u.profit > 10_000.0 && u.win_rate > 0.55)
        .map(|u| u.address)
        .collect())
}
```

#### 6.2 On-Chain Discovery

For more sophisticated tracking, query Polygon for large trades:

```rust
// Using ethers-rs to query Polygon
const POLYMARKET_CTF_ADDRESS: &str = "0x4D97DCd97eC945f40cF65F87097ACe5EA0476045";

pub async fn find_large_trades(&self, min_size_usd: f64) -> Result<Vec<Trade>> {
    // Query Transfer events from CTF contract
    // Filter by size > min_size_usd
    // Return unique addresses that made large trades
}
```

---

## Environment Variables

Add to `.env.local`:

```bash
# Whale tracking (optional - for third-party APIs)
POLYGRID_API_KEY=your_key_here
POLYWHALER_API_KEY=your_key_here

# Or use Polymarket's public APIs (no key needed)
# The client will fall back to public endpoints
```

---

## Testing

### Manual Testing Checklist

1. [ ] `/api/markets/polymarket/{market_id}/whales` returns whale data
2. [ ] Research report includes `whale_analysis` section
3. [ ] Whale consensus displays correctly in UI
4. [ ] Signal colors match (green=bullish, red=bearish, grey=neutral)
5. [ ] Notable positions show wallet tier badges
6. [ ] Empty state handles gracefully (no whales = null analysis)

### Test Markets

Good test cases with likely whale activity:
- High-volume political markets
- Crypto price markets
- Markets near resolution

---

## Future Enhancements

1. **Real-time whale alerts**: WebSocket notifications when a whale enters/exits
2. **Historical whale accuracy**: Track how often whales are right on resolved markets
3. **Cluster detection**: Group related wallets (like Théo's 11 accounts)
4. **Whale following**: Let users "follow" specific successful wallets
5. **Entry timing analysis**: Show if whales got in before price moved

---

## Key Files Summary

| File | Purpose |
|------|---------|
| `terminal-core/src/whale.rs` | Core whale types |
| `terminal-whale/src/client.rs` | API client for whale data |
| `terminal-whale/src/analyzer.rs` | Consensus calculation |
| `terminal-api/src/routes/whale.rs` | REST endpoint |
| `terminal-services/src/research_service.rs` | Integration point |
| `terminal-research/src/openai.rs` | Synthesis prompt update |
| `frontend/src/lib/types.ts` | TypeScript types |
| `frontend/src/components/research/whale-consensus.tsx` | UI component |
| `frontend/src/components/research/research-document.tsx` | Integration point |

---

## References

- [Polymarket Data API](https://docs.polymarket.com/)
- [PolyTrack Whale Tracking](https://polytrackhq.app)
- [Polywhaler](https://polywhaler.com)
- [Polygon Block Explorer](https://polygonscan.com)
