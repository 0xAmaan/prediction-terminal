"use client";

import { useEffect, useState, useCallback, useRef } from "react";
import { Button } from "@/components/ui/button";
import { Progress } from "@/components/ui/progress";
import { Loader2, CheckCircle, XCircle, RefreshCw, AlertCircle, Search } from "lucide-react";
import { useResearch } from "@/hooks/use-research";
import { ResearchDocument } from "@/components/research/research-document";
import { ResearchChat } from "@/components/research/research-chat";
import { api } from "@/lib/api";
import type { PredictionMarket, SynthesizedReport } from "@/lib/types";

// Fey color tokens
const fey = {
  bg100: "#070709",
  bg200: "#101116",
  bg300: "#131419",
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  border: "rgba(255, 255, 255, 0.06)",
};

interface ResearchViewProps {
  platform: string;
  marketId: string;
  market?: PredictionMarket;
}

export function ResearchView({ platform, marketId, market }: ResearchViewProps) {
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
    const pollForUpdate = async () => {
      let attempts = 0;
      const maxAttempts = 30;
      const pollInterval = 1000;

      while (attempts < maxAttempts) {
        await new Promise((resolve) => setTimeout(resolve, pollInterval));
        try {
          await refreshByMarket(platform, marketId);
          setVersionRefreshKey((prev) => prev + 1);
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
      setHistoricalReport(null);
      return;
    }

    setIsLoadingVersion(true);
    try {
      const versionData = await api.getVersion(platform, marketId, versionKey);
      setHistoricalReport(versionData.report ?? null);
    } catch (e) {
      console.error("Failed to load version:", e);
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
    <div className="flex-1 flex min-h-0 overflow-hidden">
      {/* Chat Panel - Left Side */}
      <div
        className="w-2/5 flex flex-col min-h-0"
        style={{ borderRight: `1px solid ${fey.border}` }}
      >
        <ResearchChat
          platform={platform}
          marketId={marketId}
          isFollowUpInProgress={isFollowUpInProgress}
          disabled={isViewingHistorical}
          onResearchTriggered={handleResearchTriggered}
        />
      </div>

      {/* Document Panel - Right Side */}
      <div
        ref={documentPanelRef}
        className="w-3/5 flex flex-col min-h-0 overflow-y-auto p-6"
        style={{ backgroundColor: fey.bg100 }}
      >
        {/* Start Research Button - shown for new markets */}
        {showStartButton && (
          <div className="flex flex-col items-center justify-center py-20">
            <div className="text-center max-w-md">
              <div
                className="w-16 h-16 mx-auto mb-6 rounded-full flex items-center justify-center"
                style={{ backgroundColor: "rgba(77, 190, 149, 0.1)" }}
              >
                <Search className="h-8 w-8" style={{ color: "#4DBE95" }} />
              </div>
              <h3 className="text-xl font-semibold mb-3" style={{ color: fey.grey100 }}>
                Start Deep Research
              </h3>
              <p className="mb-6" style={{ color: fey.grey500 }}>
                Get AI-powered analysis of this market including news, data, and key factors that could influence the outcome.
              </p>
              <Button
                size="lg"
                className="gap-2"
                onClick={handleStartResearch}
                style={{ backgroundColor: "#4DBE95", color: fey.bg100 }}
              >
                <Search className="h-4 w-4" />
                Start Research
              </Button>
            </div>
          </div>
        )}

        {/* Loading State */}
        {showLoading && (
          <div className="flex items-center justify-center py-20">
            <div className="flex items-center gap-3">
              <Loader2 className="h-5 w-5 animate-spin" style={{ color: fey.grey500 }} />
              <span className="text-sm" style={{ color: fey.grey500 }}>
                Checking for existing research...
              </span>
            </div>
          </div>
        )}

        {/* External Error */}
        {error && !job && (
          <div
            className="p-6 rounded-lg"
            style={{
              backgroundColor: "rgba(216, 79, 104, 0.1)",
              border: "1px solid rgba(216, 79, 104, 0.3)",
            }}
          >
            <div className="flex items-center gap-2 mb-3">
              <AlertCircle className="h-5 w-5" style={{ color: "#D84F68" }} />
              <span className="font-medium" style={{ color: "#D84F68" }}>
                Failed to Start Research
              </span>
            </div>
            <p className="text-sm mb-4" style={{ color: "#D84F68" }}>{error}</p>
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
          <div
            className="rounded-lg p-6 space-y-4"
            style={{
              backgroundColor: "rgba(77, 190, 149, 0.05)",
              border: "1px solid rgba(77, 190, 149, 0.3)",
            }}
          >
            <div className="flex items-center gap-2">
              <Loader2 className="h-5 w-5 animate-spin" style={{ color: "#4DBE95" }} />
              <span className="text-sm font-medium" style={{ color: "#4DBE95" }}>
                Research in progress
              </span>
            </div>
            <Progress value={progressPercent} className="h-2" />
            <div className="flex justify-between text-sm">
              <span className="font-medium" style={{ color: fey.grey100 }}>
                {job.progress.current_step || "Initializing..."}
              </span>
              <span style={{ color: fey.grey500 }}>
                Step {job.progress.completed_steps} of {job.progress.total_steps}
              </span>
            </div>
            {job.progress.current_query && (
              <div
                className="flex items-center gap-2 text-sm rounded px-3 py-2"
                style={{ backgroundColor: "rgba(255, 255, 255, 0.05)" }}
              >
                <span className="text-xs uppercase tracking-wide" style={{ color: fey.grey500 }}>
                  Searching:
                </span>
                <span className="italic" style={{ color: fey.grey500 }}>
                  &quot;{job.progress.current_query}&quot;
                </span>
              </div>
            )}
            {job.progress.searches_total > 0 && (
              <div className="text-sm" style={{ color: fey.grey500 }}>
                Web searches: {job.progress.searches_completed} of {job.progress.searches_total} complete
              </div>
            )}
          </div>
        )}

        {/* Error Display */}
        {isFailed && (
          <div
            className="p-6 rounded-lg"
            style={{
              backgroundColor: "rgba(216, 79, 104, 0.1)",
              border: "1px solid rgba(216, 79, 104, 0.3)",
            }}
          >
            <div className="flex items-center gap-2 mb-3">
              <XCircle className="h-5 w-5" style={{ color: "#D84F68" }} />
              <span className="font-medium" style={{ color: "#D84F68" }}>
                Research Failed
              </span>
            </div>
            <p className="text-sm mb-4" style={{ color: "#D84F68" }}>
              {job?.error || "An error occurred during research"}
            </p>
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
              <Loader2 className="h-8 w-8 animate-spin mx-auto mb-4" style={{ color: fey.grey500 }} />
              <p style={{ color: fey.grey500 }}>Loading version...</p>
            </div>
          </div>
        )}

        {/* Report Display */}
        {isComplete && displayReport && !isLoadingVersion && (
          <div className="space-y-4">
            <div className="flex items-center gap-2">
              <CheckCircle className="h-5 w-5" style={{ color: "#4DBE95" }} />
              <span className="text-sm font-medium" style={{ color: "#4DBE95" }}>
                Research Complete
              </span>
              {job.cached && !isViewingHistorical && (
                <span
                  className="text-xs px-2 py-0.5 rounded"
                  style={{
                    backgroundColor: "rgba(255, 255, 255, 0.05)",
                    border: `1px solid ${fey.border}`,
                    color: fey.grey500,
                  }}
                >
                  Cached
                </span>
              )}
              {isFollowUpInProgress && !isViewingHistorical && (
                <span
                  className="text-xs px-2 py-0.5 rounded"
                  style={{
                    backgroundColor: "rgba(77, 190, 149, 0.1)",
                    border: "1px solid rgba(77, 190, 149, 0.3)",
                    color: "#4DBE95",
                  }}
                >
                  Updating...
                </span>
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
    </div>
  );
}
