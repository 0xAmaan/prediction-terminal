"use client";

import type { NewsItem } from "@/lib/types";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

interface NewsCardProps {
  item: NewsItem;
  compact?: boolean;
}

const formatTimeAgo = (dateStr: string): string => {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);

  if (diffMins < 1) return "just now";
  if (diffMins < 60) return `${diffMins}m ago`;
  const diffHours = Math.floor(diffMins / 60);
  if (diffHours < 24) return `${diffHours}h ago`;
  const diffDays = Math.floor(diffHours / 24);
  if (diffDays < 7) return `${diffDays}d ago`;
  return date.toLocaleDateString();
};

export const NewsCard = ({ item, compact = false }: NewsCardProps) => {
  return (
    <Card className="border-border/30 hover:border-border/60 transition-all hover:shadow-lg bg-card/50 backdrop-blur-sm">
      <CardContent className={compact ? "p-3" : "p-4"}>
        <a
          href={item.url}
          target="_blank"
          rel="noopener noreferrer"
          className="block group"
        >
          <div className="flex gap-4">
            {/* Thumbnail - always show */}
            <div className="shrink-0 w-32 h-32 rounded-lg overflow-hidden bg-gradient-to-br from-primary/10 to-primary/5">
              {item.image_url ? (
                <img
                  src={item.image_url}
                  alt=""
                  className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
                  onError={(e) => {
                    (e.target as HTMLImageElement).style.display = "none";
                  }}
                />
              ) : (
                <div className="w-full h-full flex items-center justify-center text-muted-foreground/30">
                  <svg
                    className="w-12 h-12"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={1.5}
                      d="M19 20H5a2 2 0 01-2-2V6a2 2 0 012-2h10a2 2 0 012 2v1m2 13a2 2 0 01-2-2V7m2 13a2 2 0 002-2V9a2 2 0 00-2-2h-2m-4-3H9M7 16h6M7 8h6v4H7V8z"
                    />
                  </svg>
                </div>
              )}
            </div>

            {/* Content */}
            <div className="flex-1 min-w-0">
              {/* Source and time */}
              <div className="flex items-center gap-2 mb-2">
                {item.source.favicon_url && (
                  <img
                    src={item.source.favicon_url}
                    alt=""
                    className="w-4 h-4 rounded"
                    onError={(e) => {
                      (e.target as HTMLImageElement).style.display = "none";
                    }}
                  />
                )}
                <span className="text-sm font-medium text-muted-foreground">
                  {item.source.name}
                </span>
                <span className="text-xs text-muted-foreground/60">
                  •
                </span>
                <span className="text-xs text-muted-foreground/80">
                  {formatTimeAgo(item.published_at)}
                </span>
              </div>

              {/* Title */}
              <h3 className="font-semibold text-base leading-snug mb-2 line-clamp-2 group-hover:text-primary transition-colors">
                {item.title}
              </h3>

              {/* Summary */}
              {!compact && item.summary && (
                <p className="text-sm text-muted-foreground/90 line-clamp-2 leading-relaxed">
                  {item.summary}
                </p>
              )}
            </div>
          </div>
        </a>
      </CardContent>
    </Card>
  );
};
