"use client";

import { useEffect } from "react";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { Loader2, ArrowLeft, CheckCircle, XCircle } from "lucide-react";
import Link from "next/link";
import { useResearch } from "@/hooks/use-research";
import { ResearchDocument } from "./research-document";
import type { PredictionMarket } from "@/lib/types";

interface ResearchPageProps {
  platform: string;
  marketId: string;
  market?: PredictionMarket;
}

export function ResearchPage({ platform, marketId, market }: ResearchPageProps) {
  const { job, isLoading, error, startResearch } = useResearch();

  // Start research on mount if no job exists
  useEffect(() => {
    if (!job && !isLoading && !error) {
      startResearch(platform, marketId);
    }
  }, [job, isLoading, error, platform, marketId, startResearch]);

  const progressPercent =
    job && job.progress.total_steps > 0
      ? (job.progress.completed_steps / job.progress.total_steps) * 100
      : 0;

  const isComplete = job?.status === "completed";
  const isFailed = job?.status === "failed";
  const isRunning = job && !isComplete && !isFailed;
  const showLoading = !job && !error;

  return (
    <div className="min-h-screen bg-background flex flex-col">
      {/* Header */}
      <header className="border-b border-border/50 bg-card/50 backdrop-blur-xl sticky top-0 z-50">
        <div className="px-8 py-4 flex items-center justify-between">
          <div className="flex items-center gap-4">
            <Link
              href="/research"
              className="p-2 rounded-lg hover:bg-secondary/50 transition-colors text-muted-foreground hover:text-foreground"
            >
              <ArrowLeft className="h-5 w-5" />
            </Link>
            <Link href="/" className="text-xl font-bold">
              Prediction Terminal
            </Link>
            <span className="text-muted-foreground">/</span>
            <h1 className="text-lg font-semibold">Research</h1>
          </div>
          <Badge variant="outline" className="capitalize">
            {platform}
          </Badge>
        </div>
      </header>

      {/* Market Title */}
      <div className="px-8 py-4 border-b border-border/30">
        <h2 className="text-xl font-semibold">{market?.title || job?.market_title || "Loading..."}</h2>
      </div>

      {/* Main Content */}
      <main className="flex-1 flex">
        {/* Chat Panel (placeholder for Phase 3) */}
        <div className="w-2/5 border-r border-border/30 p-6">
          <div className="h-full flex items-center justify-center text-muted-foreground">
            <p className="text-sm">Chat interface coming soon...</p>
          </div>
        </div>

        {/* Document Panel */}
        <div className="w-3/5 p-6 overflow-y-auto">
          {/* Loading State */}
          {showLoading && (
            <div className="flex items-center justify-center py-20">
              <div className="text-center">
                <Loader2 className="h-8 w-8 animate-spin mx-auto mb-4 text-muted-foreground" />
                <p className="text-muted-foreground">Starting research...</p>
              </div>
            </div>
          )}

          {/* External Error */}
          {error && !job && (
            <div className="p-4 bg-red-500/10 border border-red-500/30 rounded-lg">
              <p className="text-red-400">{error}</p>
            </div>
          )}

          {/* Progress Section */}
          {isRunning && (
            <div className="space-y-4">
              <div className="flex items-center gap-2 mb-2">
                <Loader2 className="h-5 w-5 animate-spin" />
                <span className="text-sm font-medium">Research in progress</span>
              </div>
              <Progress value={progressPercent} className="h-2" />
              <div className="flex justify-between text-sm text-muted-foreground">
                <span>{job.progress.current_step || "Initializing..."}</span>
                <span>
                  {job.progress.completed_steps}/{job.progress.total_steps} steps
                </span>
              </div>
              {job.progress.current_query && (
                <p className="text-sm italic text-muted-foreground">
                  Searching: &quot;{job.progress.current_query}&quot;
                </p>
              )}
              {job.progress.searches_total > 0 && (
                <p className="text-sm text-muted-foreground">
                  Searches: {job.progress.searches_completed}/
                  {job.progress.searches_total}
                </p>
              )}
            </div>
          )}

          {/* Error Display */}
          {isFailed && (
            <div className="p-4 bg-red-500/10 border border-red-500/30 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <XCircle className="h-5 w-5 text-red-500" />
                <span className="font-medium text-red-400">Research Failed</span>
              </div>
              <p className="text-red-400">{job?.error || "Research failed"}</p>
            </div>
          )}

          {/* Report Display */}
          {isComplete && job?.report && (
            <div className="space-y-4">
              <div className="flex items-center gap-2">
                <CheckCircle className="h-5 w-5 text-green-500" />
                <span className="text-sm font-medium text-green-400">Research Complete</span>
                {job.cached && (
                  <Badge variant="outline" className="text-xs">
                    Cached
                  </Badge>
                )}
              </div>
              <ResearchDocument report={job.report} />
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
