"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import type { NewsItem, PredictionMarket } from "@/lib/types";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { api } from "@/lib/api";

interface NewsCardProps {
  item: NewsItem;
  compact?: boolean;
}

interface MarketInfo {
  id: string;
  platform: string;
  title: string;
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
  const [markets, setMarkets] = useState<MarketInfo[]>([]);
  const [expanded, setExpanded] = useState(false);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!item.related_market_ids || item.related_market_ids.length === 0) return;

    setLoading(true);

    // Fetch market details for all related market IDs (Polymarket only)
    const fetchMarkets = async () => {
      const marketPromises = item.related_market_ids!.map(async (marketId) => {
        try {
          const market = await api.getMarket("polymarket", marketId);
          return { id: marketId, platform: "polymarket", title: market.title };
        } catch {
          return null;
        }
      });

      const results = await Promise.all(marketPromises);
      const validMarkets = results.filter((m): m is MarketInfo => m !== null);
      setMarkets(validMarkets);
      setLoading(false);
    };

    fetchMarkets();
  }, [item.related_market_ids]);

  const displayedMarkets = expanded ? markets : markets.slice(0, 2);

  return (
    <Card className="border-border/30 hover:border-border/60 transition-all hover:shadow-lg bg-card/50 backdrop-blur-sm">
      <CardContent className={compact ? "p-3" : "p-4"}>
        <a
          href={item.url}
          target="_blank"
          rel="noopener noreferrer"
          className="block group"
        >
          <div className="space-y-3">
            {/* Source and time */}
            <div className="flex items-center gap-2">
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
              <span className="text-xs text-muted-foreground/60">•</span>
              <span className="text-xs text-muted-foreground/80">
                {formatTimeAgo(item.published_at)}
              </span>
            </div>

            {/* Title */}
            <h3 className="font-semibold text-base leading-snug group-hover:text-primary transition-colors">
              {item.title}
            </h3>

            {/* Summary */}
            {!compact && item.summary && (
              <p className="text-sm text-muted-foreground/90 line-clamp-2 leading-relaxed">
                {item.summary}
              </p>
            )}
          </div>
        </a>

        {/* Related Markets - Outside the article link */}
        {item.related_market_ids && item.related_market_ids.length > 0 && (
          <div className="flex items-start gap-2 flex-wrap pt-3 mt-3 border-t border-border/20" onClick={(e) => e.stopPropagation()}>
            <span className="text-xs text-muted-foreground/60 pt-1">
              Related markets:
            </span>
            {loading ? (
              <span className="text-xs text-muted-foreground/60">Loading...</span>
            ) : (
              <>
                {displayedMarkets.map((market) => (
                  <Link
                    key={market.id}
                    href={`/market/${market.platform}/${market.id}`}
                    onClick={(e) => e.stopPropagation()}
                  >
                    <Badge
                      variant="outline"
                      className="text-xs px-2 py-1 bg-primary/5 border-primary/20 text-primary hover:bg-primary/15 cursor-pointer transition-colors max-w-[300px] truncate"
                    >
                      {market.title}
                    </Badge>
                  </Link>
                ))}
                {markets.length > 2 && (
                  <button
                    onClick={(e) => {
                      e.preventDefault();
                      e.stopPropagation();
                      setExpanded(!expanded);
                    }}
                    className="text-xs text-primary hover:text-primary/80 transition-colors"
                  >
                    {expanded ? "Show less" : `+${markets.length - 2} more`}
                  </button>
                )}
              </>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
};
