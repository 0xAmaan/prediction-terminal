"use client";

import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { useNewsStream } from "@/hooks/use-news-stream";
import { NewsCard } from "./news-card";
import { Skeleton } from "@/components/ui/skeleton";
import type { NewsItem, Platform } from "@/lib/types";

interface NewsFeedProps {
  /** Market context for contextual news */
  market?: {
    platform: Platform;
    marketId: string;
  };
  /** Maximum items to display */
  limit?: number;
  /** Title for the feed */
  title?: string;
  /** Show in compact mode */
  compact?: boolean;
}

export const NewsFeed = ({
  market,
  limit = 10,
  title,
  compact = false,
}: NewsFeedProps) => {
  // REST API for initial data and periodic refresh (RSS polls every 5 seconds)
  const { data, isLoading, error } = useQuery({
    queryKey: market
      ? ["market-news", market.platform, market.marketId]
      : ["global-news"],
    queryFn: () =>
      market
        ? api.getMarketNews(market.platform, market.marketId, limit)
        : api.getGlobalNews({ limit }),
    staleTime: 5 * 1000, // 5 seconds (match RSS polling)
    refetchInterval: 5 * 1000, // 5 seconds (match RSS polling)
  });

  // WebSocket for real-time updates
  const { news: liveNews } = useNewsStream({
    subscribeGlobal: !market,
    market: market
      ? { platform: market.platform, marketId: market.marketId }
      : undefined,
    maxItems: limit,
  });

  // Merge REST data with live updates (live takes priority)
  const restNews = data?.items ?? [];
  const liveIds = new Set(liveNews.map((n) => n.id));
  const uniqueRestNews = restNews.filter((n) => !liveIds.has(n.id));
  const allNews = [...liveNews, ...uniqueRestNews].slice(0, limit);

  if (isLoading) {
    return (
      <div className="space-y-3">
        {title && <h2 className="text-base font-semibold">{title}</h2>}
        {Array.from({ length: 3 }).map((_, i) => (
          <Skeleton key={i} className="h-24 rounded-xl" />
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <div className="space-y-3">
        {title && <h2 className="text-base font-semibold">{title}</h2>}
        <div className="text-center py-6 text-muted-foreground text-sm">
          Failed to load news
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {title && <h2 className="text-base font-semibold">{title}</h2>}
      {allNews.length === 0 ? (
        <div className="text-center py-6 text-muted-foreground text-sm">
          No news available
        </div>
      ) : (
        allNews.map((item) => (
          <NewsCard key={item.id} item={item} compact={compact} />
        ))
      )}
    </div>
  );
};
