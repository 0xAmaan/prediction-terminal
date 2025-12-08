# Cross-Platform Market Matching

## Overview

To compare prices and find arbitrage opportunities across Kalshi and Polymarket, we need to know which markets on each platform represent the same underlying bet. This document outlines how to build a market matching system.

## The Challenge

Markets on different platforms have:
- **Different IDs**: Kalshi uses tickers like `KXINX-25020-A`, Polymarket uses event IDs like `0x123...`
- **Different phrasing**: "Will S&P 500 close above 6000?" vs "S&P 500 to exceed 6000 by year end"
- **Different resolution dates**: Sometimes close but not identical
- **Different granularity**: One platform might have 5 price points, another has 10

There's no universal identifier - matching must be built.

## Current State

### Existing infrastructure
- `UnifiedMarket` type in `terminal-core/src/market.rs` (lines 156-269)
- Has fields for both platforms: `kalshi_market`, `polymarket_market`
- `calculate_spread()` method exists but unused
- `MarketService.get_unified_markets()` has TODO for matching logic

### What's missing
- No matching algorithm
- No storage for market pairs
- No UI to create/view matches

## Proposed Solution: Phased Approach

### Phase 1: Manual Mapping (MVP)

Create a simple JSON config file for known matches:

```json
// data/market_mappings.json
{
  "mappings": [
    {
      "id": "sp500-6000-2024",
      "kalshi_ticker": "KXINX-25020-A",
      "polymarket_event_id": "0x1234...",
      "description": "S&P 500 above 6000 by Dec 31, 2024",
      "confidence": 1.0,
      "source": "manual"
    },
    {
      "id": "btc-100k-2025",
      "kalshi_ticker": "KXBTC-100K-25DEC31",
      "polymarket_event_id": "0x5678...",
      "description": "Bitcoin above $100K by end of 2025",
      "confidence": 1.0,
      "source": "manual"
    }
  ]
}
```

**Implementation**:
1. Load mappings at startup
2. When fetching unified markets, look up matches from config
3. Populate both sides of `UnifiedMarket` when match exists

### Phase 2: Database Storage + UI

Move from JSON to SQLite for persistence and add UI for creating matches:

```sql
CREATE TABLE market_mappings (
  id TEXT PRIMARY KEY,
  kalshi_ticker TEXT,
  polymarket_event_id TEXT,
  description TEXT,
  confidence REAL DEFAULT 1.0,
  source TEXT DEFAULT 'manual',  -- 'manual', 'fuzzy', 'llm'
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(kalshi_ticker, polymarket_event_id)
);

CREATE INDEX idx_kalshi ON market_mappings(kalshi_ticker);
CREATE INDEX idx_polymarket ON market_mappings(polymarket_event_id);
```

**UI Features**:
- When viewing a market, show "Link to [other platform]" button
- Search other platform's markets
- Create mapping with one click
- View all existing mappings in dedicated page

### Phase 3: Fuzzy Matching Suggestions

Add automated suggestions based on text similarity:

```rust
pub fn find_similar_markets(
    query: &str,
    candidates: &[PredictionMarket],
    threshold: f64,  // e.g., 0.7
) -> Vec<(PredictionMarket, f64)> {
    candidates
        .iter()
        .filter_map(|m| {
            let score = text_similarity(&query, &m.question);
            if score >= threshold {
                Some((m.clone(), score))
            } else {
                None
            }
        })
        .sorted_by(|a, b| b.1.partial_cmp(&a.1).unwrap())
        .collect()
}

fn text_similarity(a: &str, b: &str) -> f64 {
    // Options:
    // 1. Levenshtein distance (simple)
    // 2. Jaccard similarity on word tokens
    // 3. TF-IDF cosine similarity
    // 4. Sentence embeddings (requires ML model)
}
```

**Workflow**:
1. User views Kalshi market
2. System suggests: "Similar on Polymarket: [list with confidence scores]"
3. User confirms or dismisses suggestions
4. Confirmed matches saved with `source: 'fuzzy'`

### Phase 4: LLM-Assisted Matching (Optional)

For edge cases, use LLM to compare market semantics:

```typescript
const prompt = `
Compare these two prediction markets and determine if they represent the same bet:

Market A (Kalshi):
- Question: "${kalshiMarket.question}"
- Resolution: ${kalshiMarket.resolution_date}
- Rules: ${kalshiMarket.rules}

Market B (Polymarket):
- Question: "${polyMarket.question}"
- Resolution: ${polyMarket.end_date}
- Rules: ${polyMarket.description}

Are these the same bet? Respond with:
- SAME: If they resolve to the same outcome
- DIFFERENT: If they have different resolution criteria
- SIMILAR: If they're related but not identical

Include confidence (0-1) and brief explanation.
`;
```

**Note**: This adds cost and latency - use sparingly for ambiguous cases.

## Implementation Details

### Phase 1: JSON Config

**New file**: `terminal-services/src/market_matcher.rs`

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketMapping {
    pub id: String,
    pub kalshi_ticker: Option<String>,
    pub polymarket_event_id: Option<String>,
    pub description: String,
    pub confidence: f64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingsConfig {
    pub mappings: Vec<MarketMapping>,
}

pub struct MarketMatcher {
    by_kalshi: HashMap<String, MarketMapping>,
    by_polymarket: HashMap<String, MarketMapping>,
}

impl MarketMatcher {
    pub fn load_from_file(path: &str) -> Result<Self, Error> {
        let config: MappingsConfig = serde_json::from_str(&std::fs::read_to_string(path)?)?;

        let mut by_kalshi = HashMap::new();
        let mut by_polymarket = HashMap::new();

        for mapping in config.mappings {
            if let Some(ref ticker) = mapping.kalshi_ticker {
                by_kalshi.insert(ticker.clone(), mapping.clone());
            }
            if let Some(ref event_id) = mapping.polymarket_event_id {
                by_polymarket.insert(event_id.clone(), mapping.clone());
            }
        }

        Ok(Self { by_kalshi, by_polymarket })
    }

    pub fn find_match(&self, platform: Platform, id: &str) -> Option<&MarketMapping> {
        match platform {
            Platform::Kalshi => self.by_kalshi.get(id),
            Platform::Polymarket => self.by_polymarket.get(id),
        }
    }

    pub fn get_counterpart_id(&self, platform: Platform, id: &str) -> Option<String> {
        let mapping = self.find_match(platform, id)?;
        match platform {
            Platform::Kalshi => mapping.polymarket_event_id.clone(),
            Platform::Polymarket => mapping.kalshi_ticker.clone(),
        }
    }
}
```

**Update**: `terminal-services/src/market_service.rs`

```rust
impl MarketService {
    pub async fn get_unified_markets(&self) -> Result<Vec<UnifiedMarket>, TerminalError> {
        let (kalshi, poly) = tokio::join!(
            self.kalshi_client.get_markets(),
            self.polymarket_client.get_markets()
        );

        let mut unified = Vec::new();
        let mut matched_poly_ids = HashSet::new();

        // Process Kalshi markets, looking for Polymarket matches
        for km in kalshi.unwrap_or_default() {
            let mut um = UnifiedMarket::from_kalshi(km.clone());

            if let Some(poly_id) = self.matcher.get_counterpart_id(Platform::Kalshi, &km.ticker) {
                if let Some(pm) = poly.as_ref().ok().and_then(|p| p.iter().find(|m| m.id == poly_id)) {
                    um.polymarket_market = Some(pm.clone());
                    um.spread = um.calculate_spread();
                    matched_poly_ids.insert(poly_id);
                }
            }

            unified.push(um);
        }

        // Add unmatched Polymarket markets
        for pm in poly.unwrap_or_default() {
            if !matched_poly_ids.contains(&pm.id) {
                unified.push(UnifiedMarket::from_polymarket(pm));
            }
        }

        Ok(unified)
    }
}
```

### Phase 2: Database + API

**New API endpoints**:

```
GET  /api/mappings              - List all mappings
POST /api/mappings              - Create new mapping
GET  /api/mappings/:id          - Get specific mapping
PUT  /api/mappings/:id          - Update mapping
DELETE /api/mappings/:id        - Delete mapping
GET  /api/markets/:platform/:id/counterpart - Get matched market on other platform
```

**Frontend components**:
- `MappingsPage` - View/manage all mappings
- `LinkMarketDialog` - Modal to search and link markets
- `CounterpartBadge` - Shows linked market on market detail page

### Phase 3: Fuzzy Suggestions

**Text similarity options** (ranked by complexity):

1. **Jaccard on words** - Simple, works for similar phrasing
   ```rust
   fn jaccard(a: &str, b: &str) -> f64 {
       let a_words: HashSet<_> = a.to_lowercase().split_whitespace().collect();
       let b_words: HashSet<_> = b.to_lowercase().split_whitespace().collect();
       let intersection = a_words.intersection(&b_words).count();
       let union = a_words.union(&b_words).count();
       intersection as f64 / union as f64
   }
   ```

2. **TF-IDF + Cosine** - Better for varied phrasing, requires corpus

3. **Sentence embeddings** - Best quality, requires ML runtime (onnx?)

Recommend starting with Jaccard, upgrade if needed.

## Data Flow Example

User wants to compare "Bitcoin $100K" markets:

1. Views Kalshi market `KXBTC-100K-25DEC31`
2. Clicks "Find on Polymarket"
3. System searches Polymarket for "bitcoin 100k" (fuzzy)
4. Returns candidates with similarity scores
5. User confirms match with Polymarket event `0x789...`
6. Mapping saved to database
7. Next time viewing either market, counterpart shown automatically
8. Arbitrage page shows spread between the two

## Files to Create/Modify

| File | Action |
|------|--------|
| `data/market_mappings.json` | **NEW** - Initial manual mappings |
| `terminal-services/src/market_matcher.rs` | **NEW** - Matching logic |
| `terminal-services/src/market_service.rs` | Update to use matcher |
| `terminal-api/src/routes/mappings.rs` | **NEW** - CRUD endpoints |
| `frontend/src/app/mappings/page.tsx` | **NEW** - Mappings management UI |
| `frontend/src/components/market/link-market-dialog.tsx` | **NEW** - Link creation modal |

## Success Criteria

1. Can manually create market mappings via config file
2. Unified markets show both platforms when matched
3. Spread calculated correctly for matched pairs
4. UI allows viewing and managing mappings
5. Fuzzy suggestions help discover new matches
