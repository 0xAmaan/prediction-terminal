"use client";

import { useParams } from "next/navigation";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { ResearchPage } from "@/components/research/research-page";
import { Loader2, AlertCircle, ArrowLeft } from "lucide-react";
import Link from "next/link";
import { Button } from "@/components/ui/button";

function parseMarketId(marketIdParam: string): { platform: string; marketId: string } | null {
  // Format: platform-marketId (e.g., "kalshi-ABC123" or "polymarket-XYZ")
  const firstHyphen = marketIdParam.indexOf("-");
  if (firstHyphen === -1) {
    return null;
  }

  const platform = marketIdParam.slice(0, firstHyphen);
  const marketId = marketIdParam.slice(firstHyphen + 1);

  if (!platform || !marketId) {
    return null;
  }

  // Validate platform
  if (platform !== "kalshi" && platform !== "polymarket") {
    return null;
  }

  return { platform, marketId };
}

export default function ResearchMarketPage() {
  const params = useParams();
  const marketIdParam = params.marketId as string;

  const parsed = parseMarketId(marketIdParam);

  // Fetch market details for context
  const {
    data: market,
    isLoading: isLoadingMarket,
    error: marketError,
  } = useQuery({
    queryKey: ["market", parsed?.platform, parsed?.marketId],
    queryFn: () => api.getMarket(parsed!.platform, parsed!.marketId),
    enabled: !!parsed,
  });

  // Error: Invalid market ID format
  if (!parsed) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <div className="text-center max-w-md">
          <AlertCircle className="h-12 w-12 mx-auto text-red-500 mb-4" />
          <h1 className="text-xl font-semibold mb-2">Invalid Research URL</h1>
          <p className="text-muted-foreground mb-4">
            The URL format should be <code className="bg-muted px-1 rounded">/research/platform-marketId</code>
            <br />
            For example: <code className="bg-muted px-1 rounded">/research/kalshi-KXBTC</code>
          </p>
          <Link href="/research">
            <Button variant="outline" className="gap-2">
              <ArrowLeft className="h-4 w-4" />
              Back to Research List
            </Button>
          </Link>
        </div>
      </div>
    );
  }

  // Loading market details
  if (isLoadingMarket) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <div className="text-center">
          <Loader2 className="h-8 w-8 animate-spin mx-auto mb-4 text-muted-foreground" />
          <p className="text-muted-foreground">Loading market details...</p>
        </div>
      </div>
    );
  }

  // Error loading market
  if (marketError) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <div className="text-center max-w-md">
          <AlertCircle className="h-12 w-12 mx-auto text-red-500 mb-4" />
          <h1 className="text-xl font-semibold mb-2">Market Not Found</h1>
          <p className="text-muted-foreground mb-4">
            Could not find market <code className="bg-muted px-1 rounded">{parsed.marketId}</code> on {parsed.platform}.
          </p>
          <div className="flex gap-2 justify-center">
            <Link href="/research">
              <Button variant="outline" className="gap-2">
                <ArrowLeft className="h-4 w-4" />
                Back to Research List
              </Button>
            </Link>
            <Link href="/">
              <Button>Browse Markets</Button>
            </Link>
          </div>
        </div>
      </div>
    );
  }

  return (
    <ResearchPage
      platform={parsed.platform}
      marketId={parsed.marketId}
      market={market}
    />
  );
}
