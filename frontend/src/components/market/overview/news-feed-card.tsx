"use client";

import { Newspaper, ExternalLink, Clock } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import type { Platform, NewsItem as APINewsItem } from "@/lib/types";
import { formatDistanceToNow } from "date-fns";

// Fey color tokens
const fey = {
  bg100: "#070709",
  bg200: "#101116",
  bg300: "#131419",
  bg400: "#16181C",
  bg500: "#1A1B20",
  grey100: "#EEF0F1",
  grey300: "#B6BEC4",
  grey500: "#7D8B96",
  grey700: "#4E5860",
  teal: "#4DBE95",
  red: "#D84F68",
  skyBlue: "#54BBF7",
  purple: "#6166DC",
  border: "rgba(255, 255, 255, 0.06)",
};

interface NewsFeedCardProps {
  platform: Platform;
  marketId: string;
  marketTitle?: string;
}

const NewsItemRow = ({ item }: { item: APINewsItem }) => {
  const timestamp = formatDistanceToNow(new Date(item.published_at), {
    addSuffix: true,
  });

  return (
    <a
      href={item.url}
      target="_blank"
      rel="noopener noreferrer"
      className="px-4 py-3 transition-colors hover:bg-white/[0.02] cursor-pointer block"
      style={{ borderBottom: `1px solid ${fey.border}` }}
    >
      <div className="flex items-start gap-3">
        <div className="flex-1 min-w-0">
          {/* Title */}
          <p
            className="text-sm font-medium leading-snug mb-1 line-clamp-2"
            style={{ color: fey.grey100 }}
          >
            {item.title}
          </p>

          {/* Meta */}
          <div className="flex items-center gap-2 text-xs" style={{ color: fey.grey500 }}>
            <span
              className="px-1.5 py-0.5 rounded"
              style={{ backgroundColor: `${fey.grey500}20` }}
            >
              {item.source.name}
            </span>
            <span className="flex items-center gap-1">
              <Clock className="h-3 w-3" />
              {timestamp}
            </span>
          </div>
        </div>

        {/* External link icon */}
        <ExternalLink
          className="h-4 w-4 flex-shrink-0 opacity-50 hover:opacity-100 transition-opacity"
          style={{ color: fey.grey500 }}
        />
      </div>
    </a>
  );
};

const LoadingState = () => (
  <div className="p-8 text-center">
    <div className="animate-pulse">
      <div
        className="h-12 w-12 rounded-full mx-auto mb-4"
        style={{ backgroundColor: `${fey.grey500}20` }}
      />
      <div
        className="h-4 w-32 rounded mx-auto mb-2"
        style={{ backgroundColor: `${fey.grey500}20` }}
      />
      <div
        className="h-3 w-48 rounded mx-auto"
        style={{ backgroundColor: `${fey.grey500}20` }}
      />
    </div>
  </div>
);

const EmptyState = () => (
  <div className="p-8 text-center">
    <p className="text-sm" style={{ color: fey.grey500 }}>
      No news articles found for this market.
    </p>
  </div>
);

export const NewsFeedCard = ({
  platform,
  marketId,
  marketTitle,
}: NewsFeedCardProps) => {
  // Fetch market-specific news
  const { data: newsData, isLoading } = useQuery({
    queryKey: ["market-news", platform, marketId],
    queryFn: () => api.getMarketNews(platform, marketId, 5),
    refetchInterval: 5 * 60 * 1000, // Refetch every 5 minutes
  });

  const displayItems = newsData?.items ?? [];

  return (
    <div
      className="rounded-lg overflow-hidden"
      style={{
        backgroundColor: fey.bg300,
        border: `1px solid ${fey.border}`,
      }}
    >
      {/* Header */}
      <div
        className="flex items-center gap-2 px-4 py-3"
        style={{ borderBottom: `1px solid ${fey.border}` }}
      >
        <Newspaper className="h-4 w-4" style={{ color: fey.teal }} />
        <h3
          className="text-sm font-semibold"
          style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
        >
          Related News
        </h3>
        {displayItems.length > 0 && (
          <span
            className="ml-auto text-xs px-2 py-0.5 rounded"
            style={{
              backgroundColor: `${fey.grey500}20`,
              color: fey.grey500,
            }}
          >
            {displayItems.length}
          </span>
        )}
      </div>

      {/* Content */}
      {isLoading ? (
        <LoadingState />
      ) : displayItems.length === 0 ? (
        <EmptyState />
      ) : (
        <div>
          {displayItems.map((item) => (
            <NewsItemRow key={item.id} item={item} />
          ))}
        </div>
      )}
    </div>
  );
};
