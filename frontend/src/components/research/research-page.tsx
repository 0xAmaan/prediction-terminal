"use client";

import { useEffect, useState, useCallback } from "react";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { Loader2, ArrowLeft, CheckCircle, XCircle } from "lucide-react";
import Link from "next/link";
import { useResearch } from "@/hooks/use-research";
import { ResearchDocument } from "./research-document";
import { ResearchChat } from "./research-chat";
import { api } from "@/lib/api";
import type { PredictionMarket, SynthesizedReport } from "@/lib/types";

interface ResearchPageProps {
  platform: string;
  marketId: string;
  market?: PredictionMarket;
}

export function ResearchPage({ platform, marketId, market }: ResearchPageProps) {
  const {
    job,
    isLoading,
    error,
    isFollowUpInProgress,
    streamingContent,
    startResearch,
  } = useResearch();

  // Version state
  const [selectedVersion, setSelectedVersion] = useState<string | null>(null);
  const [historicalReport, setHistoricalReport] = useState<SynthesizedReport | null>(null);
  const [isLoadingVersion, setIsLoadingVersion] = useState(false);

  // Start research on mount if no job exists
  useEffect(() => {
    if (!job && !isLoading && !error) {
      startResearch(platform, marketId);
    }
  }, [job, isLoading, error, platform, marketId, startResearch]);

  // Handle version change
  const handleVersionChange = useCallback(async (versionKey: string | null) => {
    setSelectedVersion(versionKey);

    if (versionKey === null) {
      // Switching back to current version
      setHistoricalReport(null);
      return;
    }

    // Fetch historical version
    setIsLoadingVersion(true);
    try {
      const versionData = await api.getVersion(platform, marketId, versionKey);
      setHistoricalReport(versionData.report ?? null);
    } catch (e) {
      console.error("Failed to load version:", e);
      // On error, reset to current
      setSelectedVersion(null);
      setHistoricalReport(null);
    } finally {
      setIsLoadingVersion(false);
    }
  }, [platform, marketId]);

  const progressPercent =
    job && job.progress.total_steps > 0
      ? (job.progress.completed_steps / job.progress.total_steps) * 100
      : 0;

  const isComplete = job?.status === "completed";
  const isFailed = job?.status === "failed";
  const isRunning = job && !isComplete && !isFailed;
  const showLoading = !job && !error;

  // Determine which report to display
  const isViewingHistorical = selectedVersion !== null;
  const displayReport = isViewingHistorical ? historicalReport : job?.report;

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
      <main className="flex-1 flex overflow-hidden">
        {/* Chat Panel */}
        <div className="w-2/5 border-r border-border/30 flex flex-col">
          <ResearchChat
            platform={platform}
            marketId={marketId}
            isFollowUpInProgress={isFollowUpInProgress}
            disabled={isViewingHistorical}
          />
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

          {/* Version Loading State */}
          {isLoadingVersion && (
            <div className="flex items-center justify-center py-20">
              <div className="text-center">
                <Loader2 className="h-8 w-8 animate-spin mx-auto mb-4 text-muted-foreground" />
                <p className="text-muted-foreground">Loading version...</p>
              </div>
            </div>
          )}

          {/* Report Display */}
          {isComplete && displayReport && !isLoadingVersion && (
            <div className="space-y-4">
              <div className="flex items-center gap-2">
                <CheckCircle className="h-5 w-5 text-green-500" />
                <span className="text-sm font-medium text-green-400">Research Complete</span>
                {job.cached && !isViewingHistorical && (
                  <Badge variant="outline" className="text-xs">
                    Cached
                  </Badge>
                )}
                {isFollowUpInProgress && !isViewingHistorical && (
                  <Badge variant="outline" className="text-xs text-primary border-primary/50">
                    Updating...
                  </Badge>
                )}
              </div>
              <ResearchDocument
                report={displayReport}
                isStreaming={isFollowUpInProgress && !isViewingHistorical}
                streamingContent={isViewingHistorical ? null : streamingContent}
                platform={platform}
                marketId={marketId}
                selectedVersion={selectedVersion}
                onVersionChange={handleVersionChange}
                isViewingHistorical={isViewingHistorical}
              />
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
