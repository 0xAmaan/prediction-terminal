# Deep Research Feature - How It Works

## The Big Picture

When a user opens the Research tab on a market (e.g., "Will Bitcoin hit $100k?"), the system:
1. **Gathers market context** (price, volume, trades, order book, resolution rules)
2. **Fetches resolution source data** (scrapes URLs mentioned in resolution criteria)
3. **Breaks the question into 6 trading-focused sub-questions** (using GPT-4o)
4. **Searches the web for each sub-question** (using Exa.ai)
5. **Synthesizes everything into a report with trading analysis** (using GPT-4o)

The left chat panel lets users ask follow-up questions, which either get answered from the existing report OR trigger new research that updates the document.

---

## Step-by-Step Flow

### 1. User Opens Research Tab

Research is integrated directly into the market detail page as a tab (not a standalone page). When the tab loads:
- Check for cached research via `GET /research/:platform/:market_id`
- If cached, display immediately
- If not, show "Start Research" button

**UI Location:** `frontend/src/components/market/views/research-view.tsx`

### 2. Market Context Gathering

Before generating questions, the system collects comprehensive market data:

```typescript
interface MarketContext {
  title: string;
  description?: string;
  current_price?: number;        // 0.0 to 1.0
  price_24h_ago?: number;        // For calculating change
  volume_24h?: number;           // 24h volume in dollars
  total_volume?: number;         // Lifetime volume
  num_traders?: number;
  recent_trades: RecentTrade[];  // Last ~10 trades
  order_book_summary?: OrderBookSummary;
  resolution_rules?: string;
  resolution_source_content: ResolutionSourceData[]; // Fetched from resolution URLs
}
```

**Resolution Source Fetching:** If the market has resolution source URLs (e.g., leaderboards, official data sources), the system fetches and extracts content from these pages to give the AI accurate resolution context.

### 3. Question Decomposition (GPT-4o)

The market context gets sent to GPT-4o with a trading-focused prompt. Unlike general research, questions are specifically designed to find **trading edge**.

**6 Required Sub-Question Types:**

| Purpose | Category | What It Finds |
|---------|----------|---------------|
| `base_rate` | historical | Historical frequency of similar events |
| `market_pricing` | analysis | What assumptions the market is pricing in |
| `catalyst` | news | Upcoming events that could move the market |
| `contrarian` | analysis | The case against market consensus |
| `resolution` | analysis | Exact resolution criteria and edge cases |
| `information_asymmetry` | news | What informed traders might know |

**Output example:**
```json
{
  "main_question": "Will Tesla reach $300?",
  "sub_questions": [
    {
      "question": "What percentage of price targets this ambitious get hit?",
      "category": "historical",
      "search_query": "stock price target accuracy prediction market",
      "purpose": "base_rate"
    },
    {
      "question": "What's priced in at the current probability?",
      "category": "analysis",
      "search_query": "Tesla stock analyst expectations 2025",
      "purpose": "market_pricing"
    }
  ]
}
```

### 4. Web Search (Exa.ai)

For each sub-question, Exa.ai searches the web:

| Category | Exa Settings |
|----------|--------------|
| `news` | `search_news()` - last 7 days, 5 results, extracts highlights |
| Others | `search_research()` - neural/semantic search, no date filter |

**What Exa returns:**
- URL, title, published date
- Full text (truncated to 1500 chars)
- Highlighted sentences (most relevant excerpts)

### 5. Report Synthesis (GPT-4o)

All search results plus market context get combined and sent to GPT-4o. The output includes both the research report AND trading analysis.

**Report Structure:**
```json
{
  "title": "Tesla $300 Price Target Analysis",
  "executive_summary": "2-3 paragraphs summarizing key findings...",
  "sections": [
    { "heading": "Recent Financial Performance", "content": "markdown with [1] citations..." }
  ],
  "key_factors": [
    { "factor": "Strong Q3 earnings beat", "impact": "bullish", "confidence": "high" }
  ],
  "confidence_assessment": "Information quality assessment...",
  "sources": [
    {
      "id": 1,
      "url": "https://...",
      "title": "Tesla Q3 Earnings Report",
      "site_name": "reuters.com",
      "favicon_url": "https://www.google.com/s2/favicons?domain=reuters.com&sz=32"
    }
  ],
  "trading_analysis": {
    "fair_value_low": 0.35,
    "fair_value_high": 0.45,
    "current_price": 0.52,
    "implied_edge": -0.12,
    "estimate_confidence": "medium",
    "fair_value_reasoning": "Based on historical base rates...",
    "catalysts": [...],
    "resolution_analysis": {...},
    "contrarian_case": {...}
  }
}
```

---

## Trading Analysis Components

The `trading_analysis` section provides actionable trading insights:

### Fair Value Estimation
- `fair_value_low` / `fair_value_high`: AI's probability range estimate
- `implied_edge`: Midpoint minus current price (positive = buy signal)
- `estimate_confidence`: high/medium/low

### Catalysts
Upcoming events that could move the market:
```json
{
  "date": "2025-01-15",
  "event": "Q4 Earnings Report",
  "expected_impact": "high",
  "direction_if_positive": "bullish"
}
```

### Resolution Analysis
```json
{
  "resolution_summary": "Resolves YES if Tesla closes at or above $300...",
  "resolution_source": "Official NASDAQ closing price",
  "ambiguity_flags": ["Unclear if after-hours counts"],
  "historical_edge_cases": ["Similar market in 2023 had disputed settlement"]
}
```

### Contrarian Analysis
```json
{
  "consensus_view": "Market is pricing in strong delivery numbers",
  "contrarian_case": "Competition from BYD could pressure margins...",
  "mispricing_reasons": ["Recency bias from Q3 beat"],
  "contrarian_triggers": ["Weak China sales in December"]
}
```

---

## The Chat Panel (Follow-Up Questions)

### How It Decides What To Do

When you ask a follow-up question, GPT-4o analyzes:

```
"Can this question be answered from the existing report, or do we need new research?"
```

**Decision Tree:**
```
User asks: "What about their China sales?"
                    |
        GPT-4o checks existing report
                    |
    +---------------+---------------+
    |                               |
Already covered?               Not covered?
    |                               |
Answer from context         Trigger new research
(instant response)          (updates document)
```

### If Answer Exists in Report

GPT-4o generates an answer using only the existing report content. No new searches.

### If New Research Needed

1. **Generate search queries** - GPT-4o creates 1-3 targeted queries
2. **Execute Exa searches** - Same as initial research
3. **Update the report** - GPT-4o receives:
   - Existing report JSON
   - New search results
   - User's question

   Returns an updated report with new sections/factors added

4. **Save new version** - S3 stores both `current.json` and timestamped versions
5. **Broadcast update** - WebSocket sends `FollowUpCompleted` to frontend

---

## Inline Citations

Report content uses inline citation markers like `[1]`, `[2]` that link to sources:

```markdown
Tesla's Q3 earnings beat expectations with revenue up 8% [1]. Analysts have mixed
views on the $300 target, with some citing competition concerns [2].
```

Each citation ID maps to a `SourceInfo` object with rich metadata (title, site name, favicon).

---

## Key API Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/research/:platform/:market_id` | POST | Start new research job |
| `/research/:platform/:market_id` | GET | Get cached research |
| `/research/job/:job_id` | GET | Get specific job by ID |
| `/research/jobs` | GET | List all jobs |
| `/research/:platform/:market_id/versions` | GET | List all versions |
| `/research/:platform/:market_id/versions/:key` | GET | Get specific version |
| `/research/:platform/:market_id/chat` | GET | Get chat history |
| `/research/:platform/:market_id/chat` | POST | Send chat message |

---

## WebSocket Protocol

Research progress broadcasts via WebSocket:

```json
{"type": "ResearchUpdate", "ResearchUpdate": {"type": "status_changed", "job_id": "...", "status": "searching"}}
{"type": "ResearchUpdate", "ResearchUpdate": {"type": "progress_update", "job_id": "...", "progress": {...}}}
{"type": "ResearchUpdate", "ResearchUpdate": {"type": "completed", "job_id": "...", "report": {...}}}
{"type": "ResearchUpdate", "ResearchUpdate": {"type": "followup_started", "job_id": "..."}}
{"type": "ResearchUpdate", "ResearchUpdate": {"type": "document_editing", "job_id": "...", "content_chunk": "..."}}
{"type": "ResearchUpdate", "ResearchUpdate": {"type": "followup_completed", "job_id": "...", "report": {...}}}
```

---

## Status Flow

```
Pending -> Decomposing -> Searching -> Analyzing -> Synthesizing -> Completed
                                                                       |
                                                                  (or Failed)
```

Progress broadcasts via WebSocket so the frontend can show "Searching: news..." etc.

---

## Key Files to Know

### Backend (Rust)

| File | Purpose |
|------|---------|
| `terminal-research/src/types.rs` | Core types (ResearchJob, MarketContext, TradingAnalysis) |
| `terminal-research/src/openai.rs` | GPT-4o prompts, JSON parsing, report synthesis |
| `terminal-research/src/exa.rs` | Exa.ai API client |
| `terminal-research/src/storage.rs` | S3 storage for reports and chat |
| `terminal-research/src/resolution_source.rs` | Resolution URL fetching |
| `terminal-services/src/research_service.rs` | Orchestrates the pipeline |
| `terminal-api/src/routes/research.rs` | HTTP endpoints |

### Frontend (React)

| File | Purpose |
|------|---------|
| `frontend/src/components/market/views/research-view.tsx` | Main research UI (in market tabs) |
| `frontend/src/components/research/research-document.tsx` | Report display with citations |
| `frontend/src/components/research/research-chat.tsx` | Chat panel for follow-ups |
| `frontend/src/components/research/trading-analysis.tsx` | Trading analysis display |
| `frontend/src/components/research/inline-citation.tsx` | Citation hover/click component |
| `frontend/src/components/research/version-history.tsx` | Version selector |
| `frontend/src/hooks/use-research.ts` | State management & WebSocket |
| `frontend/src/lib/api.ts` | REST API client |

---

## Caching

- Reports cached in S3 with 24-hour TTL
- Each follow-up research creates a new version (`v{timestamp}.json`)
- `current.json` always points to latest
- Version history viewable in UI via dropdown
