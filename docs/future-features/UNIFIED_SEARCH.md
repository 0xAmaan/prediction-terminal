# Unified Search Bar for Polymarket and Kalshi

## Overview

Build a unified search bar that queries both Polymarket and Kalshi prediction markets simultaneously, displaying combined results mixed by relevance/volume.

**Key Decisions:**
- Replace existing search functionality (not a new feature)
- Results mixed by relevance (not grouped by platform)
- Start with fetch-on-demand for Kalshi (can optimize with caching later)

## Feasibility Assessment

### Polymarket API
**Status: Fully Supported**

Polymarket has a dedicated search endpoint designed for this exact use case:

```
GET https://gamma-api.polymarket.com/public-search?q={query}
```

**Parameters:**
| Parameter | Description |
|-----------|-------------|
| `q` | Search query string (required) |
| `limit_per_type` | Number of results per type |
| `page` | Pagination |
| `events_status` | Filter by status |
| `events_tag` | Filter by tag |
| `sort` | Sort order |
| `ascending` | Boolean |

**Response Structure:**
- `events[]` - Array of market groups containing nested `markets[]`
- `tags[]` - Related category tags
- Returns hierarchical data with event metadata, market details, prices, and volumes

**Authentication:** None required for public search

---

### Kalshi API
**Status: No Direct Search Endpoint**

Kalshi does NOT have a unified search endpoint like Polymarket's `/public-search`. Instead, it offers:

1. **Category/Tag Filtering:**
   - `GET /search/tags_by_categories` - Tags organized by series categories
   - `GET /search/filters_by_sport` - Sport-based filters

2. **Market Listing with Filters:**
   ```
   GET https://api.elections.kalshi.com/trade-api/v2/markets
   ```
   **Parameters:**
   - `status` - Filter by: `unopened`, `open`, `closed`, `settled`
   - `event_ticker` - Filter by specific event
   - `series_ticker` - Filter by specific series
   - `ticker` - Filter by specific market ticker
   - `limit` (1-1000, default: 100)
   - `cursor` - Pagination cursor

**Authentication:** None required for public market data

**Implication:** For Kalshi, we must implement client-side search by:
1. Fetching markets from the API
2. Filtering results by title/description match

---

## Current Implementation Analysis

### Existing Search (`terminal-services/src/market_service.rs:104-127`)

```rust
pub async fn search_markets(
    &self,
    query: &str,
    platform: Option<Platform>,
    limit: Option<usize>,
) -> Result<Vec<PredictionMarket>, TerminalError> {
    let query_lower = query.to_lowercase();
    let markets = match platform {
        Some(p) => self.get_markets_by_platform(p, None).await?,
        None => self.get_all_markets(None).await?,
    };

    let mut filtered: Vec<PredictionMarket> = markets
        .into_iter()
        .filter(|m| m.title.to_lowercase().contains(&query_lower))
        .collect();
    // ...
}
```

**Current Limitations:**
1. Fetches ALL markets before filtering (inefficient)
2. Simple substring match only (no relevance scoring)
3. No caching - refetches on every search
4. Doesn't use Polymarket's native search API

---

## Recommended Implementation

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Frontend                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              Unified Search Bar Component                │   │
│  │  - Debounced input (300ms)                              │   │
│  │  - Platform filter toggle (All/Kalshi/Polymarket)       │   │
│  │  - Loading states per platform                          │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Backend API                                  │
│  GET /api/markets/search?q={query}&platform={all|kalshi|poly}   │
└─────────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┴───────────────┐
              ▼                               ▼
┌─────────────────────────┐   ┌─────────────────────────┐
│    Polymarket Search    │   │     Kalshi Search       │
│  ────────────────────── │   │  ────────────────────── │
│  Native /public-search  │   │  Client-side filtering  │
│  Server-side relevance  │   │  with cached market     │
│                         │   │  index                  │
└─────────────────────────┘   └─────────────────────────┘
```

### Phase 1: Backend - Add Native Polymarket Search

**File: `terminal-polymarket/src/client.rs`**

Add a new method to use Polymarket's native search:

```rust
/// Search markets using the native public-search endpoint
pub async fn search(&self, query: &str, limit: Option<u32>) -> Result<Vec<PredictionMarket>, TerminalError> {
    let limit = limit.unwrap_or(20);
    let url = format!(
        "{}/public-search?q={}&limit_per_type={}&events_status=active",
        self.base_url,
        urlencoding::encode(query),
        limit
    );
    // Parse response and convert to PredictionMarket
}
```

**New types needed in `terminal-polymarket/src/types.rs`:**
```rust
pub struct SearchResponse {
    pub events: Vec<PolymarketEvent>,
    pub tags: Vec<SearchTag>,
}
```

---

### Phase 2: Backend - Kalshi Search (Client-Side)

**File: `terminal-kalshi/src/client.rs`**

Add client-side search method. Since Kalshi has no search API, we fetch markets and filter:

```rust
/// Search markets by title (client-side filtering)
/// Fetches open markets and filters by query match
pub async fn search(
    &self,
    query: &str,
    limit: Option<usize>,
) -> Result<Vec<PredictionMarket>, TerminalError> {
    // Fetch open markets (uses existing grouped method)
    let markets = self.list_markets_grouped(Some("open"), Some(500)).await?;

    let query_lower = query.to_lowercase();
    let mut results: Vec<_> = markets
        .into_iter()
        .filter(|m| m.title.to_lowercase().contains(&query_lower))
        .collect();

    // Sort by volume (proxy for relevance)
    results.sort_by(|a, b| b.volume.cmp(&a.volume));

    if let Some(l) = limit {
        results.truncate(l);
    }
    Ok(results)
}
```

**Note:** This approach fetches markets on each search. Since Polymarket search is instant (server-side), users will see Polymarket results quickly while Kalshi results may take 1-2 seconds longer. We can add caching as a future optimization if needed.

---

### Phase 3: Backend - Unified Search Service

**File: `terminal-services/src/market_service.rs`**

Update `search_markets` to use platform-native search and mix results by volume:

```rust
pub async fn search_markets(
    &self,
    query: &str,
    platform: Option<Platform>,
    limit: Option<usize>,
) -> Result<Vec<PredictionMarket>, TerminalError> {
    let limit_per_platform = limit.unwrap_or(20);

    match platform {
        Some(Platform::Polymarket) => {
            self.polymarket.search(query, Some(limit_per_platform as u32)).await
        }
        Some(Platform::Kalshi) => {
            self.kalshi.search(query, Some(limit_per_platform)).await
        }
        None => {
            // Search both platforms concurrently
            let (poly_results, kalshi_results) = tokio::join!(
                self.polymarket.search(query, Some(limit_per_platform as u32)),
                self.kalshi.search(query, Some(limit_per_platform))
            );

            let mut combined = Vec::new();
            if let Ok(markets) = poly_results {
                combined.extend(markets);
            }
            if let Ok(markets) = kalshi_results {
                combined.extend(markets);
            }

            // Mix results by volume (relevance proxy)
            // This ensures "presidential election" shows both platforms' top results
            combined.sort_by(|a, b| b.volume.cmp(&a.volume));
            combined.truncate(limit.unwrap_or(40));

            Ok(combined)
        }
    }
}
```

---

### Phase 4: Frontend (Minimal Changes)

**File: `frontend/src/app/page.tsx`**

The existing frontend search implementation should work without changes since:
- It already uses debouncing (300ms)
- It already calls the backend search API
- Results are already rendered in a unified grid

The backend changes (using native Polymarket search + mixed results) will automatically improve the search quality. No frontend changes required unless we want to add loading state improvements.

---

## Files to Modify

| File | Changes |
|------|---------|
| `terminal-polymarket/src/client.rs` | Add `search()` method using `/public-search` |
| `terminal-polymarket/src/types.rs` | Add `SearchResponse` type for parsing search results |
| `terminal-kalshi/src/client.rs` | Add `search()` method (client-side filtering) |
| `terminal-services/src/market_service.rs` | Update `search_markets()` to use new methods |

**No frontend changes required** - the existing search UI will automatically use the improved backend.

---

## API Response Format

The unified search should return the existing `PredictionMarket` type, which both platforms already normalize to:

```typescript
interface PredictionMarket {
  id: string;
  title: string;
  platform: "kalshi" | "polymarket";
  probability?: number;
  volume?: number;
  outcomes?: MarketOutcome[];
  url?: string;
  // ... other fields
}
```

---

## Performance Considerations

1. **Polymarket:** Uses server-side search - instant results
2. **Kalshi:** Client-side filtering - 1-2 second delay per search (fetches ~500 markets)
3. **Parallel Execution:** Both searches run concurrently with `tokio::join!`
4. **Overall:** Polymarket results will appear fast; Kalshi results may lag slightly

**Future optimization (if needed):** Add market caching for Kalshi to make searches instant

---

## Future Enhancements (Out of Scope)

1. **Elasticsearch/Meilisearch** - Local search index for both platforms
2. **Fuzzy matching** - Handle typos and partial matches
3. **Search suggestions** - Autocomplete as user types
4. **Search history** - Remember recent searches
5. **Advanced filters** - Category, volume range, expiry date

---

## Summary

**Yes, building a unified search bar is feasible.**

- **Polymarket:** Has native search API (`/public-search`) - direct integration
- **Kalshi:** No search API - client-side filtering required

**Implementation scope:** Backend only (4 files), no frontend changes needed.

**Example search for "presidential election":**
- Polymarket results return instantly (server-side search)
- Kalshi results return after 1-2 seconds (client-side filter)
- Both are combined and sorted by volume
- User sees mixed results from both platforms
