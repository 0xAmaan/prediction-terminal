"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import { useWebSocketContext } from "@/providers/websocket-provider";
import { api } from "@/lib/api";
import type {
  ResearchJob,
  ResearchUpdate,
  ResearchStatus,
  SynthesizedReport,
} from "@/lib/types";
import type { ServerMessage } from "@/hooks/use-websocket";

export function useResearch() {
  const [job, setJob] = useState<ResearchJob | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isFollowUpInProgress, setIsFollowUpInProgress] = useState(false);
  const [streamingContent, setStreamingContent] = useState<string | null>(null);
  const pollingRef = useRef<NodeJS.Timeout | null>(null);

  const { onMessage, connectionState } = useWebSocketContext();
  const isConnected = connectionState === "connected";

  // Poll for updates as fallback (WebSocket may not send research updates yet)
  useEffect(() => {
    if (!job?.id) return;

    // Don't poll if job is in a terminal state
    if (job.status === "completed" || job.status === "failed") {
      if (pollingRef.current) {
        clearInterval(pollingRef.current);
        pollingRef.current = null;
      }
      return;
    }

    // Poll every 2 seconds
    pollingRef.current = setInterval(async () => {
      try {
        const updated = await api.getResearchJob(job.id);
        setJob(updated);

        // Stop polling if completed or failed
        if (updated.status === "completed" || updated.status === "failed") {
          if (pollingRef.current) {
            clearInterval(pollingRef.current);
            pollingRef.current = null;
          }
        }
      } catch (e) {
        console.error("Failed to poll job:", e);
      }
    }, 2000);

    return () => {
      if (pollingRef.current) {
        clearInterval(pollingRef.current);
        pollingRef.current = null;
      }
    };
  }, [job?.id, job?.status]);

  // Listen for WebSocket updates (faster than polling when available)
  useEffect(() => {
    if (!job?.id) return;

    const unsubscribe = onMessage((msg: ServerMessage) => {
      // Check if this is a research update message
      // The backend sends: { type: "ResearchUpdate", ... } or it could be nested
      const messageAny = msg as unknown as Record<string, unknown>;

      // Handle different message formats the backend might send
      let update: ResearchUpdate | null = null;

      if (messageAny.type === "ResearchUpdate" && messageAny.ResearchUpdate) {
        update = messageAny.ResearchUpdate as ResearchUpdate;
      } else if (
        messageAny.type === "research_update" ||
        (messageAny.type &&
          typeof messageAny.type === "string" &&
          ["status_changed", "progress_update", "completed", "failed"].includes(
            messageAny.type,
          ))
      ) {
        update = messageAny as unknown as ResearchUpdate;
      }

      if (!update || update.job_id !== job.id) return;

      switch (update!.type) {
        case "followup_started":
          setIsFollowUpInProgress(true);
          setStreamingContent(null);
          break;
        case "document_editing":
          if (update!.content_chunk) {
            setStreamingContent((prev) =>
              prev ? prev + update!.content_chunk : update!.content_chunk!
            );
          }
          break;
        case "followup_completed":
          setIsFollowUpInProgress(false);
          setStreamingContent(null);
          setJob((prev) => {
            if (!prev) return prev;
            return {
              ...prev,
              report: update!.report,
            };
          });
          break;
        default:
          setJob((prev) => {
            if (!prev) return prev;

            switch (update!.type) {
              case "status_changed":
                return { ...prev, status: update!.status! };
              case "progress_update":
                return { ...prev, progress: update!.progress! };
              case "completed":
                return {
                  ...prev,
                  status: "completed" as ResearchStatus,
                  report: update!.report,
                };
              case "failed":
                return {
                  ...prev,
                  status: "failed" as ResearchStatus,
                  error: update!.error,
                };
              default:
                return prev;
            }
          });
      }
    });

    return unsubscribe;
  }, [job?.id, onMessage]);

  const startResearch = useCallback(async (platform: string, marketId: string) => {
    setIsLoading(true);
    setError(null);

    try {
      const response = await api.startResearch(platform, marketId);
      const fullJob = await api.getResearchJob(response.job_id);
      setJob(fullJob);
      return fullJob;
    } catch (e) {
      const errorMessage =
        e instanceof Error ? e.message : "Failed to start research";
      setError(errorMessage);
      throw e;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const refreshJob = useCallback(async () => {
    if (!job?.id) return;
    try {
      const updated = await api.getResearchJob(job.id);
      setJob(updated);
    } catch (e) {
      console.error("Failed to refresh job:", e);
    }
  }, [job?.id]);

  // Refresh by platform/marketId - fetches from S3 cache
  // Use this after follow-up research since backend creates new job IDs
  const refreshByMarket = useCallback(async (platform: string, marketId: string) => {
    try {
      const updated = await api.getResearchByMarket(platform, marketId);
      if (updated) {
        setJob(updated);
      }
    } catch (e) {
      console.error("Failed to refresh by market:", e);
    }
  }, []);

  const reset = useCallback(() => {
    setJob(null);
    setError(null);
    setIsLoading(false);
  }, []);

  // Update job report when WebSocket receives followup_completed
  const updateReport = useCallback((report: SynthesizedReport) => {
    setJob((prev) => {
      if (!prev) return prev;
      return { ...prev, report };
    });
  }, []);

  return {
    job,
    isLoading,
    error,
    isConnected,
    isFollowUpInProgress,
    streamingContent,
    startResearch,
    refreshJob,
    refreshByMarket,
    reset,
    updateReport,
  };
}
