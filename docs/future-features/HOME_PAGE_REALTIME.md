# Home Page Real-Time Updates

## Overview

The home page currently displays markets using a single REST fetch with no automatic refresh. This document outlines how to add real-time price updates while respecting system constraints.

## Current State

### How it works now
- **Home page** (`frontend/src/app/page.tsx`): Uses `useQuery` to fetch markets via REST
- **No polling interval**: Data only refreshes when query key changes or manual invalidation
- **No WebSocket**: Unlike individual market pages, home doesn't subscribe to any streams
- **Display**: Shows up to 100 markets in a grid with static prices

### Individual market pages (for comparison)
- Use hybrid approach: REST for initial load + WebSocket for real-time updates
- Subscribe to specific market's `price`, `order_book`, `trades` channels
- WebSocket data overwrites REST data when available

### Why not just subscribe to all markets?
The backend architecture has constraints:
1. **Broadcast channel buffer**: 1024 messages (`subscription.rs` line 48)
2. **Message rate**: 100 markets × 2 updates/sec = 200 msgs/sec
3. **Client multiplication**: N clients × 200 msgs = channel overflow
4. **Exchange limits**: Kalshi/Polymarket WebSockets have subscription limits

## Proposed Solution

### Tier 1: Polling (Quick Win)

Add a 30-second polling interval to the home page query:

```tsx
// frontend/src/app/page.tsx
const { data: markets } = useQuery({
  queryKey: ["markets", platform, search],
  queryFn: () => fetchMarkets({ platform, search, limit: 100 }),
  staleTime: 15_000,      // Consider data stale after 15s
  refetchInterval: 30_000, // Refetch every 30s
});
```

**Pros**: Simple, no backend changes, works immediately
**Cons**: 30s delay, not true real-time

### Tier 2: Selective WebSocket (Optional Enhancement)

Subscribe to top N markets (by volume or user preference) via WebSocket:

```tsx
// Only subscribe to top 10 by volume
const topMarkets = markets?.slice(0, 10) ?? [];

const { prices } = useMultiMarketStream(
  topMarkets.map(m => ({ platform: m.platform, id: m.id })),
  { enabled: topMarkets.length > 0 }
);

// Merge WebSocket prices into REST data
const marketsWithLivePrices = markets?.map(m => ({
  ...m,
  yes_price: prices[m.id]?.yes_price ?? m.yes_price,
  isLive: !!prices[m.id],
}));
```

**Backend changes needed**:
- New subscription type: `multi_price` - lightweight price-only updates for multiple markets
- Reduced payload: Just `{market_id, yes_price, no_price}` instead of full order book

### Tier 3: Server-Sent Events (Alternative)

Instead of WebSocket per-market, use SSE for bulk price updates:

```
GET /api/markets/stream?ids=market1,market2,...
```

Server pushes price updates for all requested markets in a single stream.

**Pros**: Single connection, server controls rate
**Cons**: New infrastructure, SSE less flexible than WebSocket

## Recommended Approach

**Phase 1**: Implement Tier 1 (polling) - 30 minutes of work
**Phase 2**: Add visual staleness indicator - shows when data is >15s old
**Phase 3**: Evaluate if Tier 2 is needed based on user feedback

## Implementation Details

### Phase 1: Add Polling

**File**: `frontend/src/app/page.tsx`

```tsx
const { data: markets, dataUpdatedAt } = useQuery({
  queryKey: ["markets", platform, search],
  queryFn: () => fetchMarkets({ platform, search, limit: 100 }),
  staleTime: 15_000,
  refetchInterval: 30_000,
  refetchIntervalInBackground: false, // Don't refetch when tab is hidden
});
```

### Phase 2: Staleness Indicator

Show when data was last updated and visual cue for stale data:

```tsx
// In MarketsGrid component
const isStale = Date.now() - dataUpdatedAt > 15_000;

return (
  <div>
    <div className="flex items-center gap-2 text-xs text-muted-foreground">
      <span className={isStale ? "text-yellow-500" : "text-green-500"}>
        {isStale ? "⚠" : "●"}
      </span>
      Updated {formatDistanceToNow(dataUpdatedAt)} ago
    </div>
    {/* ... market grid */}
  </div>
);
```

### Phase 3: Selective WebSocket (if needed)

**New hook**: `frontend/src/hooks/use-multi-market-stream.ts`

```tsx
export const useMultiMarketStream = (
  markets: Array<{ platform: Platform; id: string }>,
  options?: { enabled?: boolean }
) => {
  const { subscribe, unsubscribe } = useWebSocket();
  const [prices, setPrices] = useState<Record<string, PriceUpdate>>({});

  useEffect(() => {
    if (!options?.enabled) return;

    markets.forEach(m => {
      subscribe({
        type: "price",
        platform: m.platform,
        market_id: m.id,
      });
    });

    return () => {
      markets.forEach(m => {
        unsubscribe({
          type: "price",
          platform: m.platform,
          market_id: m.id,
        });
      });
    };
  }, [markets, options?.enabled]);

  // Handle incoming price updates...

  return { prices };
};
```

**Backend optimization** (if needed): Add rate limiting per subscription key to prevent flood.

## Files to Modify

| File | Change |
|------|--------|
| `frontend/src/app/page.tsx` | Add refetchInterval to useQuery |
| `frontend/src/components/markets-grid/markets-grid.tsx` | Add staleness indicator |
| `frontend/src/hooks/use-multi-market-stream.ts` | **NEW** - only if Tier 2 needed |

## Trade-offs

| Approach | Latency | Complexity | Backend Load |
|----------|---------|------------|--------------|
| Polling (30s) | High | Low | Low |
| Polling (5s) | Medium | Low | Medium |
| Selective WebSocket | Low (top 10) | Medium | Medium |
| Full WebSocket | Lowest | High | High (not recommended) |

## Success Criteria

1. Home page prices update without manual refresh
2. User can see when data was last updated
3. No degradation of individual market page performance
4. Backend doesn't get overloaded with requests
