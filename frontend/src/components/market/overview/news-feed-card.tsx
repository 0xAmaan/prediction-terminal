"use client";

import { Newspaper, Sparkles, ExternalLink, Clock } from "lucide-react";

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

export interface NewsItem {
  id: string;
  title: string;
  source: string;
  timestamp: string;
  url?: string;
  sentiment?: "positive" | "negative" | "neutral";
}

interface NewsFeedCardProps {
  marketTitle?: string;
  items?: NewsItem[];
  isPlaceholder?: boolean;
}

// Mock news items for demonstration
const mockNewsItems: NewsItem[] = [
  {
    id: "1",
    title: "Market activity surges as deadline approaches",
    source: "Market Analysis",
    timestamp: "2 hours ago",
    sentiment: "neutral",
  },
  {
    id: "2",
    title: "New data suggests shifting probabilities",
    source: "Research Digest",
    timestamp: "5 hours ago",
    sentiment: "positive",
  },
  {
    id: "3",
    title: "Expert commentary on resolution criteria",
    source: "Industry Report",
    timestamp: "1 day ago",
    sentiment: "neutral",
  },
];

const getSentimentColor = (sentiment?: NewsItem["sentiment"]) => {
  switch (sentiment) {
    case "positive":
      return fey.teal;
    case "negative":
      return fey.red;
    default:
      return fey.grey500;
  }
};

const NewsItemRow = ({ item }: { item: NewsItem }) => (
  <div
    className="px-4 py-3 transition-colors hover:bg-white/[0.02] cursor-pointer"
    style={{ borderBottom: `1px solid ${fey.border}` }}
  >
    <div className="flex items-start gap-3">
      {/* Sentiment indicator */}
      <div
        className="w-1 h-full min-h-[40px] rounded-full flex-shrink-0 mt-0.5"
        style={{ backgroundColor: getSentimentColor(item.sentiment) }}
      />

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
            {item.source}
          </span>
          <span className="flex items-center gap-1">
            <Clock className="h-3 w-3" />
            {item.timestamp}
          </span>
        </div>
      </div>

      {/* External link icon */}
      {item.url && (
        <ExternalLink
          className="h-4 w-4 flex-shrink-0 opacity-50 hover:opacity-100 transition-opacity"
          style={{ color: fey.grey500 }}
        />
      )}
    </div>
  </div>
);

const PlaceholderState = () => (
  <div className="p-8 text-center">
    <div
      className="inline-flex items-center justify-center w-12 h-12 rounded-full mb-4"
      style={{ backgroundColor: `${fey.purple}15` }}
    >
      <Sparkles className="h-6 w-6" style={{ color: fey.purple }} />
    </div>

    <h4
      className="text-sm font-semibold mb-2"
      style={{ color: fey.grey100 }}
    >
      AI-Powered News Aggregation
    </h4>
    <p
      className="text-sm mb-4 max-w-[280px] mx-auto leading-relaxed"
      style={{ color: fey.grey500 }}
    >
      This section will surface relevant news articles and events that may impact market outcomes.
    </p>

    <div
      className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-medium"
      style={{
        backgroundColor: `${fey.skyBlue}15`,
        color: fey.skyBlue,
      }}
    >
      <div
        className="w-1.5 h-1.5 rounded-full animate-pulse"
        style={{ backgroundColor: fey.skyBlue }}
      />
      Coming Soon
    </div>
  </div>
);

export const NewsFeedCard = ({
  marketTitle,
  items,
  isPlaceholder = true,
}: NewsFeedCardProps) => {
  // Use mock items if no items provided and not in placeholder mode
  const displayItems = items ?? (isPlaceholder ? [] : mockNewsItems);

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
            {displayItems.length} articles
          </span>
        )}
      </div>

      {/* Content */}
      {isPlaceholder || displayItems.length === 0 ? (
        <PlaceholderState />
      ) : (
        <div>
          {displayItems.slice(0, 5).map((item) => (
            <NewsItemRow key={item.id} item={item} />
          ))}

          {/* View All Link */}
          {displayItems.length > 5 && (
            <div className="px-4 py-3">
              <button
                className="text-sm font-medium flex items-center gap-1 transition-colors hover:opacity-80"
                style={{ color: fey.skyBlue }}
              >
                View all {displayItems.length} articles
                <ExternalLink className="h-3.5 w-3.5" />
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
