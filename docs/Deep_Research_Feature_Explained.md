# Research Document Feature - How It Works

## The Big Picture

When a user clicks "Research" on a market (e.g., "Will Bitcoin hit $100k?"), the system:
1. **Breaks the question into sub-questions** (using GPT-4o)
2. **Searches the web for each sub-question** (using Exa.ai)
3. **Synthesizes everything into a report** (using GPT-4o)

The left chat panel lets users ask follow-up questions, which either get answered from the existing report OR trigger new research that updates the document.

---

## Step-by-Step Flow

### 1. User Initiates Research

**Input:** Market title + description (e.g., "Will Tesla stock reach $300 by March 2025?")

**API Call:** `POST /research/:platform/:market_id`

### 2. Question Decomposition (GPT-4o)

The market question gets sent to GPT-4o with this prompt (simplified):

```
"Break this market question into 4-6 sub-questions covering different angles"
```

**Output example:**
```json
{
  "main_question": "Will Tesla reach $300?",
  "sub_questions": [
    { "question": "What are Tesla's recent earnings?", "category": "news", "search_query": "Tesla Q3 2024 earnings results" },
    { "question": "What do analysts predict?", "category": "expert_opinion", "search_query": "Tesla stock analyst price targets 2025" },
    { "question": "Historical price movements", "category": "historical", "search_query": "Tesla stock price history volatility" }
  ]
}
```

Categories: `news`, `analysis`, `historical`, `expert_opinion`

### 3. Web Search (Exa.ai)

For each sub-question, Exa.ai searches the web:

| Category | Exa Settings |
|----------|--------------|
| `news` | `search_news()` - last 7 days, 5 results, extracts highlights |
| Others | `search_research()` - neural/semantic search, no date filter |

**What Exa returns:**
- URL, title, published date
- Full text (truncated to 1500 chars)
- Highlighted sentences (most relevant excerpts)

### 4. Report Synthesis (GPT-4o)

All search results get combined and sent to GPT-4o:

```
Market: {title}
Description: {description}

## Research Data
### Sub-question 1: Recent earnings?
- Article: "Tesla Q3 Earnings Beat Expectations"
  URL: https://...
  Highlights: "Revenue increased 8%...", "Margins improved..."

### Sub-question 2: Analyst predictions?
...
```

GPT-4o returns a structured JSON report:

```json
{
  "title": "Tesla $300 Price Target Analysis",
  "executive_summary": "2-3 paragraphs summarizing key findings...",
  "sections": [
    { "heading": "Recent Financial Performance", "content": "markdown content..." },
    { "heading": "Analyst Sentiment", "content": "..." }
  ],
  "key_factors": [
    { "factor": "Strong Q3 earnings beat", "impact": "bullish", "confidence": "high" },
    { "factor": "Rising competition from BYD", "impact": "bearish", "confidence": "medium" }
  ],
  "confidence_assessment": "Information quality assessment and gaps...",
  "sources": ["https://...", "https://..."]
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

## Where Each Part of the Report Comes From

| Section | Source |
|---------|--------|
| **Title** | GPT-4o generates based on market title |
| **Executive Summary** | GPT-4o synthesizes key findings from all search results |
| **Sections** (4-6 paragraphs) | Each maps roughly to a sub-question's research |
| **Key Factors** | GPT-4o extracts factors with impact (bullish/bearish/neutral) and confidence |
| **Confidence Assessment** | GPT-4o evaluates information quality and gaps |
| **Sources** | URLs from Exa search results |

---

## Key Files to Know

| File | Purpose |
|------|---------|
| `terminal-research/src/openai.rs` | All GPT-4o prompts and JSON parsing |
| `terminal-research/src/exa.rs` | Exa.ai API client |
| `terminal-services/src/research_service.rs` | Orchestrates the pipeline |
| `terminal-api/src/routes/research.rs` | HTTP endpoints |
| `frontend/src/components/research/research-page.tsx` | Main UI layout |
| `frontend/src/components/research/research-chat.tsx` | Chat panel |
| `frontend/src/components/research/research-document.tsx` | Report display |
| `frontend/src/hooks/use-research.ts` | State management & WebSocket |

---

## Status Flow

```
Pending -> Decomposing -> Searching -> Analyzing -> Synthesizing -> Completed
                                                                       |
                                                                  (or Failed)
```

Progress broadcasts via WebSocket so the frontend can show "Searching: news..." etc.

---

## Caching

- Reports cached in S3 with 24-hour TTL
- Each follow-up research creates a new version (`v{timestamp}.json`)
- `current.json` always points to latest
- Version history viewable in UI
