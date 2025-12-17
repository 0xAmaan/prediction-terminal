"use client";

import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Loader2,
  Search,
  FileText,
  Clock,
  CheckCircle,
  XCircle,
  RefreshCw,
} from "lucide-react";
import Link from "next/link";
import type {
  ResearchJob,
  ResearchJobSummary,
  ResearchStatus,
} from "@/lib/types";

// Union type for merged reports - can be either full job or summary
type MergedReport = ResearchJob | ResearchJobSummary;

// Helper to get executive summary from either type
const getExecutiveSummary = (job: MergedReport): string | undefined => {
  // Check if it's a full ResearchJob (has 'report' field)
  if ("report" in job && job.report) {
    return job.report.executive_summary;
  }
  // Otherwise it's a ResearchJobSummary (has 'executive_summary' directly)
  if ("executive_summary" in job) {
    return job.executive_summary;
  }
  return undefined;
};

// Fey color tokens
const fey = {
  bg100: "#070709",
  bg300: "#0F1012",
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  skyBlue: "#54BBF7",
  teal: "#4DBE95",
  border: "rgba(255, 255, 255, 0.06)",
};

export function ResearchReports() {
  const [search, setSearch] = useState("");

  // Fetch saved reports from S3 (persisted)
  const {
    data: savedReports,
    isLoading: isLoadingSaved,
    refetch: refetchSaved,
  } = useQuery({
    queryKey: ["saved-research-reports"],
    queryFn: api.listSavedReports,
    staleTime: 60000, // Consider stale after 1 minute
  });

  // Also fetch in-progress jobs (in-memory, for real-time status)
  // Only poll when there are active (non-completed/failed) jobs
  const { data: activeJobs } = useQuery({
    queryKey: ["research-jobs"],
    queryFn: api.listResearchJobs,
    refetchInterval: (query) => {
      const jobs = query.state.data;
      const hasActiveJobs = jobs?.some(
        (j) => j.status !== "completed" && j.status !== "failed",
      );
      return hasActiveJobs ? 5000 : false;
    },
  });

  // Merge: active jobs that are not completed + saved reports
  // Active jobs take precedence if they exist for the same market
  const mergedReports = (() => {
    const savedMap = new Map(
      (savedReports || []).map((r) => [`${r.platform}:${r.market_id}`, r]),
    );
    const activeMap = new Map(
      (activeJobs || []).map((j) => [`${j.platform}:${j.market_id}`, j]),
    );

    // Start with saved reports
    const result = new Map(savedMap);

    // Override with active jobs (they have more recent status)
    activeMap.forEach((job, key) => {
      result.set(key, job);
    });

    return Array.from(result.values()).sort(
      (a, b) =>
        new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime(),
    );
  })();

  const filteredReports = mergedReports.filter((job) =>
    job.market_title.toLowerCase().includes(search.toLowerCase()),
  );

  const isLoading = isLoadingSaved;

  return (
    <div>
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <h2
          className="text-lg font-semibold"
          style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
        >
          Research Reports
        </h2>
        <div className="flex items-center gap-3">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => refetchSaved()}
            title="Refresh reports"
            className="h-8 w-8"
            style={{ color: fey.grey500 }}
          >
            <RefreshCw className="h-4 w-4" />
          </Button>
          <div className="relative w-64">
            <Search
              className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4"
              style={{ color: fey.grey500 }}
            />
            <Input
              placeholder="Search reports..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="pl-10 h-9 text-sm"
              style={{
                backgroundColor: fey.bg300,
                borderColor: fey.border,
                color: fey.grey100,
              }}
            />
          </div>
        </div>
      </div>

      {/* Content */}
      {isLoading ? (
        <div className="flex justify-center py-20">
          <Loader2 className="h-8 w-8 animate-spin" style={{ color: fey.grey500 }} />
        </div>
      ) : filteredReports.length === 0 ? (
        <div className="text-center py-20">
          <FileText className="h-12 w-12 mx-auto mb-4" style={{ color: fey.grey500 }} />
          <h3 className="text-lg font-semibold mb-2" style={{ color: fey.grey100 }}>
            No research reports yet
          </h3>
          <p className="mb-4" style={{ color: fey.grey500 }}>
            Start researching markets to generate comprehensive analysis reports.
          </p>
          <Link href="/markets">
            <Button
              style={{
                backgroundColor: fey.skyBlue,
                color: fey.bg100,
              }}
            >
              Browse Markets
            </Button>
          </Link>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {filteredReports.map((job) => (
            <ResearchJobCard key={`${job.platform}:${job.market_id}`} job={job} />
          ))}
        </div>
      )}
    </div>
  );
}

function ResearchJobCard({ job }: { job: MergedReport }) {
  const statusConfig: Record<
    ResearchStatus,
    { icon: React.ReactNode; color: string; bgColor: string }
  > = {
    pending: {
      icon: <Clock className="h-3.5 w-3.5" />,
      color: fey.grey500,
      bgColor: "rgba(125, 139, 150, 0.15)",
    },
    decomposing: {
      icon: <Loader2 className="h-3.5 w-3.5 animate-spin" />,
      color: fey.skyBlue,
      bgColor: "rgba(84, 187, 247, 0.15)",
    },
    searching: {
      icon: <Loader2 className="h-3.5 w-3.5 animate-spin" />,
      color: fey.skyBlue,
      bgColor: "rgba(84, 187, 247, 0.15)",
    },
    analyzing: {
      icon: <Loader2 className="h-3.5 w-3.5 animate-spin" />,
      color: fey.skyBlue,
      bgColor: "rgba(84, 187, 247, 0.15)",
    },
    synthesizing: {
      icon: <Loader2 className="h-3.5 w-3.5 animate-spin" />,
      color: fey.skyBlue,
      bgColor: "rgba(84, 187, 247, 0.15)",
    },
    completed: {
      icon: <CheckCircle className="h-3.5 w-3.5" />,
      color: fey.teal,
      bgColor: "rgba(77, 190, 149, 0.15)",
    },
    failed: {
      icon: <XCircle className="h-3.5 w-3.5" />,
      color: "#F87171",
      bgColor: "rgba(248, 113, 113, 0.15)",
    },
  };

  const { icon, color, bgColor } = statusConfig[job.status];

  return (
    <Card
      className="border transition-all hover:border-opacity-30"
      style={{
        backgroundColor: fey.bg300,
        borderColor: fey.border,
      }}
    >
      <CardHeader className="pb-2">
        <div className="flex items-start justify-between">
          <Badge
            variant="outline"
            className="capitalize text-xs"
            style={{
              borderColor: fey.border,
              color: fey.grey500,
            }}
          >
            {job.platform}
          </Badge>
          <span
            className="flex items-center gap-1.5 text-xs px-2 py-1 rounded-full"
            style={{ backgroundColor: bgColor, color }}
          >
            {icon}
            {job.status}
          </span>
        </div>
        <CardTitle
          className="text-base font-medium line-clamp-2 mt-2"
          style={{ color: fey.grey100 }}
        >
          {job.market_title}
        </CardTitle>
      </CardHeader>
      <CardContent>
        {job.status !== "completed" && job.status !== "failed" && (
          <p className="text-sm line-clamp-2" style={{ color: fey.grey500 }}>
            {job.progress.current_step || "Starting..."}
          </p>
        )}
        {job.status === "completed" && getExecutiveSummary(job) && (
          <p className="text-sm line-clamp-3" style={{ color: fey.grey500 }}>
            {getExecutiveSummary(job)}
          </p>
        )}
        {job.status === "failed" && (
          <p className="text-sm" style={{ color: "#F87171" }}>
            {job.error}
          </p>
        )}
        <div
          className="mt-4 flex justify-between items-center text-xs"
          style={{ color: fey.grey500 }}
        >
          <span>{new Date(job.created_at).toLocaleDateString()}</span>
          <Link href={`/market/${job.platform}/${job.market_id}?tab=research`}>
            <Button
              variant="ghost"
              size="sm"
              className="h-7 text-xs"
              style={{ color: fey.skyBlue }}
            >
              View Report
            </Button>
          </Link>
        </div>
      </CardContent>
    </Card>
  );
}
