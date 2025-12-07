"use client";

import { useQuery } from "@tanstack/react-query";
import { useState, useMemo } from "react";
import { useRouter } from "next/navigation";
import { ExternalLink, TrendingUp, Clock, ArrowUp, ArrowDown, Layers } from "lucide-react";
import { api } from "@/lib/api";
import type { PredictionMarket } from "@/lib/types";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Skeleton } from "@/components/ui/skeleton";
import { PlatformBadge } from "./platform-badge";
import { PlatformFilter } from "./platform-filter";

type SortField = "volume" | "close_time" | "yes_price" | "created_at" | null;
type SortDirection = "asc" | "desc";

interface MarketsTableProps {
  search?: string;
}

const formatPrice = (price: string): string => {
  const num = parseFloat(price);
  if (isNaN(num)) return "—";
  return `${(num * 100).toFixed(1)}¢`;
};

const formatVolume = (volume: string): string => {
  const num = parseFloat(volume);
  if (isNaN(num) || num === 0) return "—";
  if (num >= 1_000_000) return `$${(num / 1_000_000).toFixed(1)}M`;
  if (num >= 1_000) return `$${(num / 1_000).toFixed(1)}K`;
  return `$${num.toFixed(0)}`;
};

const formatCloseTime = (closeTime: string | null): string => {
  if (!closeTime) return "—";
  const date = new Date(closeTime);
  const now = new Date();
  const diffMs = date.getTime() - now.getTime();
  const diffDays = Math.ceil(diffMs / (1000 * 60 * 60 * 24));

  if (diffDays < 0) return "Ended";
  if (diffDays === 0) return "Today";
  if (diffDays === 1) return "Tomorrow";
  if (diffDays < 7) return `${diffDays}d`;
  if (diffDays < 30) return `${Math.floor(diffDays / 7)}w`;
  if (diffDays < 365) return `${Math.floor(diffDays / 30)}mo`;
  return date.toLocaleDateString("en-US", { month: "short", year: "2-digit" });
};

const formatCreatedAt = (createdAt: string | null): string => {
  if (!createdAt) return "—";
  const date = new Date(createdAt);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSeconds = Math.floor(diffMs / 1000);
  const diffMinutes = Math.floor(diffSeconds / 60);
  const diffHours = Math.floor(diffMinutes / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffSeconds < 60) return "now";
  if (diffMinutes < 60) return `${diffMinutes}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;
  if (diffDays < 30) return `${Math.floor(diffDays / 7)}w ago`;
  if (diffDays < 365) return `${Math.floor(diffDays / 30)}mo ago`;
  return date.toLocaleDateString("en-US", { month: "short", year: "2-digit" });
};

const MarketImage = ({ market }: { market: PredictionMarket }) => {
  const borderColor = market.platform === "kalshi" ? "border-[#32E2AD]" : "border-[#3b82f6]";
  const bgColor = market.platform === "kalshi" ? "bg-[#32E2AD]/20" : "bg-[#3b82f6]/20";
  const textColor = market.platform === "kalshi" ? "text-[#32E2AD]" : "text-[#3b82f6]";

  if (market.image_url) {
    return (
      <div className={`w-18 h-18 shrink-0 rounded-lg border-2 ${borderColor} overflow-hidden`}>
        <img
          src={market.image_url}
          alt=""
          className="w-full h-full object-cover"
          onError={(e) => {
            // Replace with fallback on error
            const parent = e.currentTarget.parentElement?.parentElement;
            if (parent) {
              e.currentTarget.parentElement!.style.display = 'none';
              const fallback = parent.querySelector('[data-fallback]');
              if (fallback) fallback.classList.remove('hidden');
            }
          }}
        />
      </div>
    );
  }

  // Fallback: colored square with first letter
  return (
    <div className={`w-18 h-18 shrink-0 rounded-lg ${bgColor} ${borderColor} border-2 flex items-center justify-center`}>
      <span className={`text-xl font-bold ${textColor}`}>
        {market.platform === "kalshi" ? "K" : "P"}
      </span>
    </div>
  );
};

const MarketRow = ({ market, onClick }: { market: PredictionMarket; onClick: () => void }) => {
  const yesPrice = parseFloat(market.yes_price);
  const isPriceHigh = yesPrice >= 0.7;
  const isPriceLow = yesPrice <= 0.3;

  const platformClass = market.platform === "kalshi" ? "platform-kalshi" : "platform-polymarket";

  const handleRowClick = (e: React.MouseEvent) => {
    // Don't navigate if clicking on the external link
    if ((e.target as HTMLElement).closest('a')) {
      return;
    }
    onClick();
  };

  return (
    <TableRow
      className={`group market-card-row ${platformClass} border-b border-border cursor-pointer h-[72px]`}
      onClick={handleRowClick}
    >
      <TableCell className="py-5 pl-4 overflow-hidden">
        <div className="flex items-center gap-5">
          <MarketImage market={market} />
          <div className="flex-1 min-w-0 overflow-hidden">
            <div className="flex items-center gap-2">
              <p className="text-lg font-medium text-foreground leading-tight truncate" title={market.title}>
                {market.title}
              </p>
              {market.is_multi_outcome && market.outcome_count && (
                <span
                  className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-xs font-medium bg-violet-500/20 text-violet-400 shrink-0"
                  title={`${market.outcome_count} options${market.leading_outcome ? ` - Leading: ${market.leading_outcome}` : ''}`}
                >
                  <Layers className="h-3 w-3" />
                  {market.outcome_count}
                </span>
              )}
            </div>
            {market.is_multi_outcome && market.leading_outcome ? (
              <p className="text-sm text-muted-foreground mt-0.5 truncate">
                Leading: <span className="text-foreground/70">{market.leading_outcome}</span>
              </p>
            ) : market.ticker ? (
              <p className="text-sm text-muted-foreground font-mono mt-0.5 truncate">
                {market.ticker}
              </p>
            ) : null}
          </div>
          {market.url && (
            <a
              href={market.url}
              target="_blank"
              rel="noopener noreferrer"
              className="opacity-0 group-hover:opacity-100 transition-all duration-200 text-muted-foreground hover:text-foreground p-1.5 rounded hover:bg-secondary shrink-0"
              onClick={(e) => e.stopPropagation()}
            >
              <ExternalLink className="h-4 w-4" />
            </a>
          )}
        </div>
      </TableCell>
      <TableCell className="py-5 text-right w-[80px]">
        <span
          className={`text-base font-semibold font-mono ${
            isPriceHigh
              ? "text-[#32E2AD]"
              : isPriceLow
                ? "text-[#ED3A7A]"
                : "text-foreground"
          }`}
        >
          {formatPrice(market.yes_price)}
        </span>
      </TableCell>
      <TableCell className="py-5 text-right w-[70px]">
        <span className="text-base font-mono text-muted-foreground">
          {formatPrice(market.no_price)}
        </span>
      </TableCell>
      <TableCell className="py-5 text-right w-[90px]">
        <span className="text-base font-mono text-foreground">
          {formatVolume(market.volume)}
        </span>
      </TableCell>
      <TableCell className="py-5 text-right w-[90px]">
        <span className="text-base text-muted-foreground">
          {formatCreatedAt(market.created_at)}
        </span>
      </TableCell>
      <TableCell className="py-5 pr-4 text-right w-[90px]">
        <div className="flex items-center justify-end gap-1.5 text-muted-foreground">
          <Clock className="h-4 w-4" />
          <span className="text-base">{formatCloseTime(market.close_time)}</span>
        </div>
      </TableCell>
    </TableRow>
  );
};

const LoadingSkeleton = () => (
  <>
    {Array.from({ length: 10 }).map((_, i) => (
      <TableRow key={i} className="market-card-row border-0">
        <TableCell className="py-4 pl-4 w-[60%]">
          <div className="flex items-center gap-5">
            <Skeleton className="h-20 w-20 rounded-lg shrink-0" />
            <div className="space-y-1.5 flex-1">
              <Skeleton className="h-4 w-full" />
              <Skeleton className="h-3 w-24" />
            </div>
          </div>
        </TableCell>
        <TableCell className="py-4 w-[80px]">
          <Skeleton className="h-4 w-14 ml-auto" />
        </TableCell>
        <TableCell className="py-4 w-[70px]">
          <Skeleton className="h-4 w-14 ml-auto" />
        </TableCell>
        <TableCell className="py-4 w-[90px]">
          <Skeleton className="h-4 w-14 ml-auto" />
        </TableCell>
        <TableCell className="py-4 w-[90px]">
          <Skeleton className="h-4 w-14 ml-auto" />
        </TableCell>
        <TableCell className="py-4 pr-4 w-[90px]">
          <Skeleton className="h-4 w-14 ml-auto" />
        </TableCell>
      </TableRow>
    ))}
  </>
);

const SortableHeader = ({
  label,
  field,
  currentSort,
  currentDirection,
  onSort,
}: {
  label: string;
  field: SortField;
  currentSort: SortField;
  currentDirection: SortDirection;
  onSort: (field: SortField) => void;
}) => {
  const isActive = currentSort === field;

  return (
    <button
      onClick={() => onSort(field)}
      className={`flex items-center gap-1 hover:text-foreground transition-colors w-full justify-end text-sm font-medium cursor-pointer ${
        isActive ? "text-foreground" : "text-muted-foreground"
      }`}
    >
      <span>{label}</span>
      {isActive && (
        currentDirection === "desc" ? (
          <ArrowDown className="h-4 w-4" />
        ) : (
          <ArrowUp className="h-4 w-4" />
        )
      )}
    </button>
  );
};

export const MarketsTable = ({ search = "" }: MarketsTableProps) => {
  const router = useRouter();
  const [platform, setPlatform] = useState<"all" | "kalshi" | "polymarket">("all");
  const [sortField, setSortField] = useState<SortField>("created_at");
  const [sortDirection, setSortDirection] = useState<SortDirection>("desc");

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      // Three-state cycle: desc -> asc -> null (default)
      if (sortDirection === "desc") {
        setSortDirection("asc");
      } else {
        // Was asc, reset to default
        setSortField(null);
        setSortDirection("desc");
      }
    } else {
      setSortField(field);
      setSortDirection("desc");
    }
  };

  const { data, isLoading, error } = useQuery({
    queryKey: ["markets", platform, search],
    queryFn: () =>
      api.listMarkets({
        platform: platform === "all" ? undefined : platform,
        search: search || undefined,
        limit: 100,
      }),
  });

  const sortedMarkets = useMemo(() => {
    if (!data?.markets || !sortField) return data?.markets || [];

    return [...data.markets].sort((a, b) => {
      let aVal: number;
      let bVal: number;

      switch (sortField) {
        case "volume":
          aVal = parseFloat(a.volume) || 0;
          bVal = parseFloat(b.volume) || 0;
          break;
        case "close_time":
          aVal = a.close_time ? new Date(a.close_time).getTime() : Infinity;
          bVal = b.close_time ? new Date(b.close_time).getTime() : Infinity;
          break;
        case "yes_price":
          aVal = parseFloat(a.yes_price) || 0;
          bVal = parseFloat(b.yes_price) || 0;
          break;
        case "created_at":
          // Sort by created_at, putting markets without dates at the end
          aVal = a.created_at ? new Date(a.created_at).getTime() : 0;
          bVal = b.created_at ? new Date(b.created_at).getTime() : 0;
          break;
        default:
          return 0;
      }

      const multiplier = sortDirection === "desc" ? -1 : 1;
      return (aVal - bVal) * multiplier;
    });
  }, [data?.markets, sortField, sortDirection]);

  return (
    <div className="h-full flex flex-col">
      {/* Filters row */}
      <div className="shrink-0 flex items-center justify-between mb-6">
        <PlatformFilter value={platform} onChange={setPlatform} />
        {data && (
          <div className="text-base text-muted-foreground flex items-center gap-2">
            <TrendingUp className="h-4 w-4 text-primary" />
            <span className="font-medium">{data.count}</span>
            <span>markets</span>
          </div>
        )}
      </div>

      {/* Error state */}
      {error && (
        <div className="shrink-0 rounded-lg border border-destructive/50 bg-destructive/10 p-4 text-destructive text-base mb-4">
          Failed to load markets. Is the API server running?
        </div>
      )}

      {/* Table - scrollable */}
      <div className="flex-1 min-h-0 rounded-md border border-border bg-card overflow-hidden flex flex-col">
        <Table>
          <TableHeader>
            <TableRow className="hover:bg-transparent border-b border-border">
              <TableHead className="py-3 pl-4 text-sm font-medium text-muted-foreground w-[60%]">Market</TableHead>
              <TableHead className="text-right w-[80px] py-3">
                <SortableHeader
                  label="Yes"
                  field="yes_price"
                  currentSort={sortField}
                  currentDirection={sortDirection}
                  onSort={handleSort}
                />
              </TableHead>
              <TableHead className="text-right w-[70px] py-3 text-sm font-medium text-muted-foreground">No</TableHead>
              <TableHead className="text-right w-[90px] py-3">
                <SortableHeader
                  label="Volume"
                  field="volume"
                  currentSort={sortField}
                  currentDirection={sortDirection}
                  onSort={handleSort}
                />
              </TableHead>
              <TableHead className="text-right w-[90px] py-3">
                <SortableHeader
                  label="Created"
                  field="created_at"
                  currentSort={sortField}
                  currentDirection={sortDirection}
                  onSort={handleSort}
                />
              </TableHead>
              <TableHead className="text-right w-[90px] py-3 pr-4">
                <SortableHeader
                  label="Closes"
                  field="close_time"
                  currentSort={sortField}
                  currentDirection={sortDirection}
                  onSort={handleSort}
                />
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <LoadingSkeleton />
            ) : sortedMarkets.length === 0 ? (
              <TableRow>
                <TableCell
                  colSpan={6}
                  className="text-center py-12 text-muted-foreground"
                >
                  No markets found
                </TableCell>
              </TableRow>
            ) : (
              sortedMarkets.map((market) => (
                <MarketRow
                  key={`${market.platform}-${market.id}`}
                  market={market}
                  onClick={() => router.push(`/market/${market.platform}/${market.id}`)}
                />
              ))
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  );
};
