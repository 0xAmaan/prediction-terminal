"use client";

import { useEffect, useState, useCallback, useRef } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Loader2, ArrowLeft, CheckCircle, XCircle, RefreshCw, AlertCircle, Search } from "lucide-react";
import { Skeleton } from "@/components/ui/skeleton";
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
    refreshByMarket,
  } = useResearch();

  // Version state
  const [selectedVersion, setSelectedVersion] = useState<string | null>(null);
  const [historicalReport, setHistoricalReport] = useState<SynthesizedReport | null>(null);
  const [isLoadingVersion, setIsLoadingVersion] = useState(false);

  // Track if we've checked for existing research and if user wants to start new research
  const [hasCheckedCache, setHasCheckedCache] = useState(false);
  const [userWantsResearch, setUserWantsResearch] = useState(false);

  // Ref for document panel scrolling
  const documentPanelRef = useRef<HTMLDivElement>(null);

  // Scroll document panel to top
  const handleScrollToTop = useCallback(() => {
    documentPanelRef.current?.scrollTo({ top: 0, behavior: "smooth" });
  }, []);

  // State to track version refresh
  const [versionRefreshKey, setVersionRefreshKey] = useState(0);

  // Handle when chat triggers research - refresh the job to get updated report
  const handleResearchTriggered = useCallback(async () => {
    // Poll for the updated report using platform/marketId
    // (backend creates new job IDs for follow-up research, so we can't use job.id)
    const pollForUpdate = async () => {
      let attempts = 0;
      const maxAttempts = 30; // 30 seconds max wait
      const pollInterval = 1000; // 1 second

      while (attempts < maxAttempts) {
        await new Promise((resolve) => setTimeout(resolve, pollInterval));
        try {
          await refreshByMarket(platform, marketId);
          // Refresh version history
          setVersionRefreshKey((prev) => prev + 1);
          // Scroll to top to show updated content
          handleScrollToTop();
          return;
        } catch (e) {
          console.error("Failed to poll for update:", e);
        }
        attempts++;
      }
    };

    pollForUpdate();
  }, [platform, marketId, refreshByMarket, handleScrollToTop]);

  // On mount, check for existing cached research
  useEffect(() => {
    if (hasCheckedCache) return;

    const checkCache = async () => {
      try {
        const cached = await api.getResearchByMarket(platform, marketId);
        if (cached) {
          // Found cached research - load it automatically
          startResearch(platform, marketId);
        }
        // No cached research - user must click to start
      } catch (e) {
        console.error("Failed to check cache:", e);
      } finally {
        setHasCheckedCache(true);
      }
    };

    checkCache();
  }, [platform, marketId, hasCheckedCache, startResearch]);

  // Start research when user explicitly requests it
  useEffect(() => {
    if (userWantsResearch && !job && !isLoading) {
      startResearch(platform, marketId);
      setUserWantsResearch(false);
    }
  }, [userWantsResearch, job, isLoading, platform, marketId, startResearch]);

  // Handler for starting research manually
  const handleStartResearch = useCallback(() => {
    setUserWantsResearch(true);
  }, []);

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
  const showLoading = isLoading || (!hasCheckedCache && !job && !error);
  const showStartButton = hasCheckedCache && !job && !isLoading && !error;

  // Determine which report to display
  const isViewingHistorical = selectedVersion !== null;
  const displayReport = isViewingHistorical ? historicalReport : job?.report;

  return (
    <div className="h-screen bg-background flex flex-col overflow-hidden">
      {/* Header */}
      <header className="flex-shrink-0 border-b border-border/50 bg-card/50 backdrop-blur-xl z-50">
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
      <div className="flex-shrink-0 px-8 py-4 border-b border-border/30">
        <h2 className="text-xl font-semibold">{market?.title || job?.market_title || "Loading..."}</h2>
      </div>

      {/* Main Content */}
      <main className="flex-1 flex min-h-0">
        {/* Chat Panel */}
        <div className="w-2/5 border-r border-border/30 flex flex-col min-h-0">
          <ResearchChat
            platform={platform}
            marketId={marketId}
            isFollowUpInProgress={isFollowUpInProgress}
            disabled={isViewingHistorical}
            onResearchTriggered={handleResearchTriggered}
          />
        </div>

        {/* Document Panel */}
        <div ref={documentPanelRef} className="w-3/5 flex flex-col min-h-0 overflow-y-auto p-6">
          {/* Start Research Button - shown for new markets */}
          {showStartButton && (
            <div className="flex flex-col items-center justify-center py-20">
              <div className="text-center max-w-md">
                <div className="w-16 h-16 mx-auto mb-6 rounded-full bg-primary/10 flex items-center justify-center">
                  <Search className="h-8 w-8 text-primary" />
                </div>
                <h3 className="text-xl font-semibold mb-3">Start Deep Research</h3>
                <p className="text-muted-foreground mb-6">
                  Get AI-powered analysis of this market including news, data, and key factors that could influence the outcome.
                </p>
                <Button
                  size="lg"
                  className="gap-2"
                  onClick={handleStartResearch}
                >
                  <Search className="h-4 w-4" />
                  Start Research
                </Button>
              </div>
            </div>
          )}

          {/* Loading State - Skeleton UI */}
          {showLoading && (
            <div className="space-y-6">
              <div className="flex items-center gap-3">
                <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
                <span className="text-sm text-muted-foreground">Checking for existing research...</span>
              </div>
              {/* Skeleton cards */}
              <div className="space-y-4">
                <div className="border border-border/30 rounded-lg p-4">
                  <Skeleton className="h-4 w-32 mb-3" />
                  <Skeleton className="h-3 w-full mb-2" />
                  <Skeleton className="h-3 w-4/5 mb-2" />
                  <Skeleton className="h-3 w-3/4" />
                </div>
                <div className="border border-border/30 rounded-lg p-4">
                  <Skeleton className="h-4 w-24 mb-3" />
                  <div className="space-y-2">
                    <Skeleton className="h-10 w-full" />
                    <Skeleton className="h-10 w-full" />
                    <Skeleton className="h-10 w-full" />
                  </div>
                </div>
                <div className="border border-border/30 rounded-lg p-4">
                  <Skeleton className="h-4 w-40 mb-3" />
                  <Skeleton className="h-3 w-full mb-2" />
                  <Skeleton className="h-3 w-5/6 mb-2" />
                  <Skeleton className="h-3 w-full mb-2" />
                  <Skeleton className="h-3 w-2/3" />
                </div>
              </div>
            </div>
          )}

          {/* External Error */}
          {error && !job && (
            <div className="p-6 bg-red-500/10 border border-red-500/30 rounded-lg">
              <div className="flex items-center gap-2 mb-3">
                <AlertCircle className="h-5 w-5 text-red-500" />
                <span className="font-medium text-red-400">Failed to Start Research</span>
              </div>
              <p className="text-red-400 text-sm mb-4">{error}</p>
              <Button
                variant="outline"
                size="sm"
                className="gap-2"
                onClick={() => startResearch(platform, marketId)}
              >
                <RefreshCw className="h-4 w-4" />
                Try Again
              </Button>
            </div>
          )}

          {/* Progress Section */}
          {isRunning && (
            <div className="border border-primary/30 bg-primary/5 rounded-lg p-6 space-y-4">
              <div className="flex items-center gap-2">
                <Loader2 className="h-5 w-5 animate-spin text-primary" />
                <span className="text-sm font-medium text-primary">Research in progress</span>
              </div>
              <Progress value={progressPercent} className="h-2" />
              <div className="flex justify-between text-sm">
                <span className="text-foreground font-medium">{job.progress.current_step || "Initializing..."}</span>
                <span className="text-muted-foreground">
                  Step {job.progress.completed_steps} of {job.progress.total_steps}
                </span>
              </div>
              {job.progress.current_query && (
                <div className="flex items-center gap-2 text-sm text-muted-foreground bg-muted/50 rounded px-3 py-2">
                  <span className="text-xs uppercase tracking-wide">Searching:</span>
                  <span className="italic">&quot;{job.progress.current_query}&quot;</span>
                </div>
              )}
              {job.progress.searches_total > 0 && (
                <div className="text-sm text-muted-foreground">
                  Web searches: {job.progress.searches_completed} of {job.progress.searches_total} complete
                </div>
              )}
            </div>
          )}

          {/* Error Display */}
          {isFailed && (
            <div className="p-6 bg-red-500/10 border border-red-500/30 rounded-lg">
              <div className="flex items-center gap-2 mb-3">
                <XCircle className="h-5 w-5 text-red-500" />
                <span className="font-medium text-red-400">Research Failed</span>
              </div>
              <p className="text-red-400 text-sm mb-4">{job?.error || "An error occurred during research"}</p>
              <Button
                variant="outline"
                size="sm"
                className="gap-2"
                onClick={() => startResearch(platform, marketId)}
              >
                <RefreshCw className="h-4 w-4" />
                Retry Research
              </Button>
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
                onScrollToTop={handleScrollToTop}
                versionRefreshKey={versionRefreshKey}
              />
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
