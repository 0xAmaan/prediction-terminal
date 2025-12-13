"use client";

import { useState } from "react";
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
  /** Items per page (backward compat: also accepts 'limit') */
  itemsPerPage?: number;
  /** Backward compatibility for limit */
  limit?: number;
  /** Total items to fetch (for pagination) */
  totalItems?: number;
  /** Title for the feed */
  title?: string;
  /** Show in compact mode */
  compact?: boolean;
  /** Enable pagination */
  enablePagination?: boolean;
}

const fey = {
  bg100: "#070709",
  bg300: "#131419",
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  teal: "#4DBE95",
  border: "rgba(255, 255, 255, 0.06)",
};

export const NewsFeed = ({
  market,
  itemsPerPage,
  limit,
  totalItems = 50,
  title,
  compact = false,
  enablePagination = true,
}: NewsFeedProps) => {
  const [currentPage, setCurrentPage] = useState(1);

  // Support both 'limit' (backward compat) and 'itemsPerPage'
  const perPage = itemsPerPage ?? limit ?? 10;

  // REST API for initial data and periodic refresh (RSS polls every 5 seconds)
  const { data, isLoading, error } = useQuery({
    queryKey: market
      ? ["market-news", market.platform, market.marketId]
      : ["global-news"],
    queryFn: () =>
      market
        ? api.getMarketNews(market.platform, market.marketId, totalItems)
        : api.getGlobalNews({ limit: totalItems }),
    staleTime: 5 * 1000, // 5 seconds (match RSS polling)
    refetchInterval: 5 * 1000, // 5 seconds (match RSS polling)
  });

  // WebSocket for real-time updates
  const { news: liveNews } = useNewsStream({
    subscribeGlobal: !market,
    market: market
      ? { platform: market.platform, marketId: market.marketId }
      : undefined,
    maxItems: totalItems,
  });

  // Merge REST data with live updates (live takes priority)
  const restNews = data?.items ?? [];
  const liveIds = new Set(liveNews.map((n) => n.id));
  const uniqueRestNews = restNews.filter((n) => !liveIds.has(n.id));
  const allNews = [...liveNews, ...uniqueRestNews];

  // Pagination logic
  const totalPages = Math.ceil(allNews.length / perPage);
  const startIndex = (currentPage - 1) * perPage;
  const endIndex = startIndex + perPage;
  const currentPageNews = allNews.slice(startIndex, endIndex);

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

  // Generate page numbers to display (like Google: 1 2 3 ... 10)
  const getPageNumbers = () => {
    const pages: (number | string)[] = [];
    const maxVisible = 7; // Max page numbers to show

    if (totalPages <= maxVisible) {
      // Show all pages if total is small
      for (let i = 1; i <= totalPages; i++) {
        pages.push(i);
      }
    } else {
      // Always show first page
      pages.push(1);

      if (currentPage > 3) {
        pages.push("...");
      }

      // Show pages around current page
      const start = Math.max(2, currentPage - 1);
      const end = Math.min(totalPages - 1, currentPage + 1);

      for (let i = start; i <= end; i++) {
        pages.push(i);
      }

      if (currentPage < totalPages - 2) {
        pages.push("...");
      }

      // Always show last page
      if (totalPages > 1) {
        pages.push(totalPages);
      }
    }

    return pages;
  };

  return (
    <div className="space-y-4">
      {title && <h2 className="text-base font-semibold">{title}</h2>}

      {/* News items */}
      <div className="space-y-3">
        {currentPageNews.length === 0 ? (
          <div className="text-center py-6 text-muted-foreground text-sm">
            No news available
          </div>
        ) : (
          currentPageNews.map((item) => (
            <NewsCard key={item.id} item={item} compact={compact} />
          ))
        )}
      </div>

      {/* Pagination controls */}
      {enablePagination && totalPages > 1 && (
        <div className="flex items-center justify-center gap-2 pt-4 pb-2">
          {/* Previous button */}
          <button
            onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
            disabled={currentPage === 1}
            className="px-3 py-1.5 rounded text-sm font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
            style={{
              backgroundColor: currentPage === 1 ? fey.bg300 : fey.bg300,
              border: `1px solid ${fey.border}`,
              color: fey.grey100,
            }}
          >
            Previous
          </button>

          {/* Page numbers */}
          {getPageNumbers().map((page, idx) =>
            page === "..." ? (
              <span
                key={`ellipsis-${idx}`}
                className="px-2 py-1.5 text-sm"
                style={{ color: fey.grey500 }}
              >
                ...
              </span>
            ) : (
              <button
                key={page}
                onClick={() => setCurrentPage(page as number)}
                className="px-3 py-1.5 rounded text-sm font-medium transition-colors min-w-[36px]"
                style={{
                  backgroundColor:
                    currentPage === page ? fey.teal : fey.bg300,
                  border: `1px solid ${
                    currentPage === page ? fey.teal : fey.border
                  }`,
                  color: currentPage === page ? fey.bg100 : fey.grey100,
                }}
              >
                {page}
              </button>
            )
          )}

          {/* Next button */}
          <button
            onClick={() => setCurrentPage((p) => Math.min(totalPages, p + 1))}
            disabled={currentPage === totalPages}
            className="px-3 py-1.5 rounded text-sm font-medium transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
            style={{
              backgroundColor:
                currentPage === totalPages ? fey.bg300 : fey.bg300,
              border: `1px solid ${fey.border}`,
              color: fey.grey100,
            }}
          >
            Next
          </button>
        </div>
      )}
    </div>
  );
};
