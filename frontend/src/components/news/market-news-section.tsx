"use client";

import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { NewsFeed } from "./news-feed";
import type { Platform } from "@/lib/types";

interface MarketNewsSectionProps {
  platform: Platform;
  marketId: string;
}

export const MarketNewsSection = ({
  platform,
  marketId,
}: MarketNewsSectionProps) => {
  return (
    <Card className="border-border/30 bg-card/30">
      <CardHeader className="pb-3">
        <CardTitle className="text-sm font-medium flex items-center gap-2">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            className="h-4 w-4 text-primary"
          >
            <path d="M4 22h16a2 2 0 0 0 2-2V4a2 2 0 0 0-2-2H8a2 2 0 0 0-2 2v16a2 2 0 0 1-2 2Zm0 0a2 2 0 0 1-2-2v-9c0-1.1.9-2 2-2h2" />
            <path d="M18 14h-8" />
            <path d="M15 18h-5" />
            <path d="M10 6h8v4h-8V6Z" />
          </svg>
          Related News
        </CardTitle>
      </CardHeader>
      <CardContent className="pt-0">
        <NewsFeed market={{ platform, marketId }} limit={5} compact />
      </CardContent>
    </Card>
  );
};
