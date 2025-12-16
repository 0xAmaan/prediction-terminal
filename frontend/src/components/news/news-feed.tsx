"use client";

import Link from "next/link";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { useNewsStream } from "@/hooks/use-news-stream";
import type { NewsItem, Platform, PriceSignal, SuggestedAction } from "@/lib/types";

// Fey color tokens
const fey = {
  bg100: "#070709",
  bg200: "#0D0E12",
  grey100: "#EEF0F1",
  grey400: "#9BA3AB",
  grey500: "#7D8B96",
  grey600: "#5A6670",
  green: "#4DBE95",
  red: "#F25757",
  skyBlue: "#54BBF7",
  border: "rgba(255, 255, 255, 0.06)",
};

interface NewsFeedProps {
  market?: {
    platform: Platform;
    marketId: string;
  };
  maxItems?: number;
  compact?: boolean;
  // Legacy props for backward compat
  itemsPerPage?: number;
  limit?: number;
  totalItems?: number;
  title?: string;
  enablePagination?: boolean;
}

const formatTimeAgo = (dateStr: string): string => {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);

  if (diffMins < 1) return "now";
  if (diffMins < 60) return `${diffMins}m`;
  const diffHours = Math.floor(diffMins / 60);
  if (diffHours < 24) return `${diffHours}h`;
  const diffDays = Math.floor(diffHours / 24);
  return `${diffDays}d`;
};

const getActionDisplay = (
  signal: PriceSignal | null | undefined,
  action: SuggestedAction | null | undefined
): { label: string; color: string } | null => {
  if (!action || action === "hold") return null;
  if (action === "buy") return { label: "BUY", color: fey.green };
  if (action === "sell") return { label: "SELL", color: fey.red };
  return null;
};

export const NewsFeed = ({
  market,
  maxItems = 20,
  compact = false,
  limit,
  totalItems,
}: NewsFeedProps) => {
  const itemCount = maxItems || limit || totalItems || 20;

  // Fetch news from API - use enriched endpoint for global news
  const { data, isLoading, error } = useQuery({
    queryKey: market
      ? ["market-news", market.platform, market.marketId]
      : ["enriched-news"],
    queryFn: () =>
      market
        ? api.getMarketNews(market.platform, market.marketId, itemCount, true)
        : api.getEnrichedNews(),
    staleTime: 15 * 1000,
    refetchInterval: 15 * 1000,
  });

  // WebSocket for real-time updates
  const { news: liveNews } = useNewsStream({
    subscribeGlobal: !market,
    market: market
      ? { platform: market.platform, marketId: market.marketId }
      : undefined,
    maxItems: itemCount,
  });

  // Merge and deduplicate news (live takes priority)
  const restNews = data?.items ?? [];
  const newsMap = new Map<string, NewsItem>();
  restNews.forEach((item) => newsMap.set(item.id, item));
  liveNews.forEach((item) => newsMap.set(item.id, item));

  // Sort by published date (newest first) and limit
  const allNews = Array.from(newsMap.values())
    .sort((a, b) => new Date(b.published_at).getTime() - new Date(a.published_at).getTime())
    .slice(0, itemCount);

  if (isLoading) {
    return (
      <div className="space-y-0">
        {Array.from({ length: 5 }).map((_, i) => (
          <div
            key={i}
            className="py-3 animate-pulse"
            style={{ borderBottom: `1px solid ${fey.border}` }}
          >
            <div className="h-3 w-24 rounded bg-white/5 mb-2" />
            <div className="h-4 w-full rounded bg-white/5" />
          </div>
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <div className="py-8 text-center text-sm" style={{ color: fey.grey500 }}>
        Failed to load news feed
      </div>
    );
  }

  if (allNews.length === 0) {
    return (
      <div className="py-8 text-center text-sm" style={{ color: fey.grey500 }}>
        No news available
      </div>
    );
  }

  return (
    <div className="divide-y" style={{ borderColor: fey.border }}>
      {allNews.map((item, index) => (
        <NewsRow key={item.id} item={item} isNew={index < liveNews.length} />
      ))}
    </div>
  );
};

// Individual news row - compact streaming style
const NewsRow = ({ item, isNew }: { item: NewsItem; isNew?: boolean }) => {
  const hasMarketMatch =
    item.matched_market != null && item.matched_market.platform != null;
  const action = getActionDisplay(item.price_signal, item.suggested_action);

  return (
    <div
      className="py-3 px-1 transition-colors hover:bg-white/[0.02]"
      style={{
        borderColor: fey.border,
        backgroundColor: isNew ? "rgba(84, 187, 247, 0.03)" : "transparent",
      }}
    >
      {/* Market match row (if AI enriched) */}
      {hasMarketMatch && (
        <div className="flex items-center gap-2 mb-2 flex-wrap">
          <Link
            href={`/market/${item.matched_market!.platform}/${item.matched_market!.market_id}`}
            className="inline-flex items-center gap-1.5 group"
          >
            <span
              className="text-[11px] font-medium px-2 py-0.5 rounded transition-colors"
              style={{
                backgroundColor: "rgba(84, 187, 247, 0.1)",
                color: fey.skyBlue,
              }}
            >
              {item.matched_market!.title.length > 40
                ? item.matched_market!.title.slice(0, 40) + "..."
                : item.matched_market!.title}
            </span>
            <span
              className="text-[11px] font-medium"
              style={{ color: fey.grey500 }}
            >
              {(item.matched_market!.current_price * 100).toFixed(0)}%
            </span>
          </Link>

          {/* Price signal with tooltip - clickable to go to market */}
          {item.price_signal && item.price_signal !== "neutral" && (
            <Link
              href={`/market/${item.matched_market!.platform}/${item.matched_market!.market_id}${item.matched_market!.outcome ? `?outcome=${encodeURIComponent(item.matched_market!.outcome)}` : ""}`}
              className="relative group/signal"
            >
              <span
                className="text-[10px] font-semibold uppercase px-1.5 py-0.5 rounded cursor-pointer hover:opacity-80 transition-opacity"
                style={{
                  color:
                    item.price_signal === "underpriced" ? fey.green : fey.red,
                  backgroundColor:
                    item.price_signal === "underpriced"
                      ? "rgba(77, 190, 149, 0.15)"
                      : "rgba(242, 87, 87, 0.15)",
                }}
              >
                {item.price_signal}
                {item.matched_market?.outcome && (
                  <span className="ml-1 opacity-70">
                    ({item.matched_market.outcome.length > 15
                      ? item.matched_market.outcome.slice(0, 15) + "..."
                      : item.matched_market.outcome})
                  </span>
                )}
              </span>
              {/* Tooltip */}
              {item.signal_reasoning && (
                <span
                  className="absolute left-0 top-full mt-1 z-50 hidden group-hover/signal:block w-72 p-2 rounded text-[11px] font-normal normal-case leading-relaxed shadow-lg pointer-events-none"
                  style={{
                    backgroundColor: "#1A1B20",
                    color: fey.grey100,
                    border: `1px solid ${fey.border}`,
                  }}
                >
                  {item.signal_reasoning}
                </span>
              )}
            </Link>
          )}

          {/* Action badge */}
          {action && (
            <span
              className="text-[10px] font-bold tracking-wide px-2 py-0.5 rounded"
              style={{
                color: action.color,
                backgroundColor: `${action.color}20`,
                border: `1px solid ${action.color}30`,
              }}
            >
              {action.label}
            </span>
          )}
        </div>
      )}

      {/* Main news content */}
      <a
        href={item.url}
        target="_blank"
        rel="noopener noreferrer"
        className="block group"
      >
        {/* Source + time */}
        <div className="flex items-center gap-1.5 mb-1">
          {item.source.favicon_url && (
            <img
              src={item.source.favicon_url}
              alt=""
              className="w-3.5 h-3.5 rounded-sm opacity-70"
              onError={(e) => {
                (e.target as HTMLImageElement).style.display = "none";
              }}
            />
          )}
          <span className="text-[11px] font-medium" style={{ color: fey.grey400 }}>
            {item.source.name}
          </span>
          <span className="text-[11px]" style={{ color: fey.grey600 }}>
            · {formatTimeAgo(item.published_at)}
          </span>
        </div>

        {/* Headline */}
        <h3
          className="text-[13px] font-medium leading-snug group-hover:text-[#54BBF7] transition-colors"
          style={{ color: fey.grey100 }}
        >
          {item.title}
        </h3>
      </a>
    </div>
  );
};

export default NewsFeed;
