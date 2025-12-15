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
  ArrowLeft,
} from "lucide-react";
import Link from "next/link";
import type { ResearchJob, ResearchStatus } from "@/lib/types";

export default function ResearchPage() {
  const [search, setSearch] = useState("");

  const {
    data: jobs,
    isLoading,
  } = useQuery({
    queryKey: ["research-jobs"],
    queryFn: api.listResearchJobs,
    refetchInterval: 5000, // Poll every 5 seconds
  });

  const filteredJobs = jobs?.filter((job) =>
    job.market_title.toLowerCase().includes(search.toLowerCase()),
  );

  return (
    <div className="min-h-screen bg-background">
      <header className="border-b border-border/50 bg-card/50 backdrop-blur-xl sticky top-0 z-50">
        <div className="px-8 py-4 flex items-center justify-between">
          <div className="flex items-center gap-4">
            <Link
              href="/"
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
          <div className="flex items-center gap-4">
            <div className="relative w-64">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Search reports..."
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                className="pl-10"
              />
            </div>
          </div>
        </div>
      </header>

      <main className="px-8 py-6">
        {isLoading ? (
          <div className="flex justify-center py-20">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        ) : !filteredJobs || filteredJobs.length === 0 ? (
          <div className="text-center py-20">
            <FileText className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
            <h2 className="text-xl font-semibold mb-2">
              No research reports yet
            </h2>
            <p className="text-muted-foreground mb-4">
              Start researching markets to generate comprehensive analysis
              reports.
            </p>
            <Link href="/">
              <Button>Browse Markets</Button>
            </Link>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {filteredJobs.map((job) => (
              <ResearchJobCard key={job.id} job={job} />
            ))}
          </div>
        )}
      </main>
    </div>
  );
}

function ResearchJobCard({ job }: { job: ResearchJob }) {
  const statusConfig: Record<
    ResearchStatus,
    { icon: React.ReactNode; color: string }
  > = {
    pending: {
      icon: <Clock className="h-4 w-4" />,
      color: "bg-gray-500/20 text-gray-400",
    },
    decomposing: {
      icon: <Loader2 className="h-4 w-4 animate-spin" />,
      color: "bg-blue-500/20 text-blue-400",
    },
    searching: {
      icon: <Loader2 className="h-4 w-4 animate-spin" />,
      color: "bg-blue-500/20 text-blue-400",
    },
    analyzing: {
      icon: <Loader2 className="h-4 w-4 animate-spin" />,
      color: "bg-blue-500/20 text-blue-400",
    },
    synthesizing: {
      icon: <Loader2 className="h-4 w-4 animate-spin" />,
      color: "bg-blue-500/20 text-blue-400",
    },
    completed: {
      icon: <CheckCircle className="h-4 w-4" />,
      color: "bg-green-500/20 text-green-400",
    },
    failed: {
      icon: <XCircle className="h-4 w-4" />,
      color: "bg-red-500/20 text-red-400",
    },
  };

  const { icon, color } = statusConfig[job.status];

  return (
    <Card className="border-border/30 hover:shadow-lg transition-shadow">
      <CardHeader className="pb-2">
        <div className="flex items-start justify-between">
          <Badge variant="outline" className="mb-2 capitalize">
            {job.platform}
          </Badge>
          <Badge className={`${color} border-0`}>
            <span className="flex items-center gap-1">
              {icon}
              {job.status}
            </span>
          </Badge>
        </div>
        <CardTitle className="text-lg line-clamp-2">{job.market_title}</CardTitle>
      </CardHeader>
      <CardContent>
        {job.status !== "completed" && job.status !== "failed" && (
          <p className="text-sm text-muted-foreground">
            {job.progress.current_step || "Starting..."}
          </p>
        )}
        {job.status === "completed" && job.report && (
          <p className="text-sm text-muted-foreground line-clamp-3">
            {job.report.executive_summary}
          </p>
        )}
        {job.status === "failed" && (
          <p className="text-sm text-red-400">{job.error}</p>
        )}
        <div className="mt-4 flex justify-between items-center text-xs text-muted-foreground">
          <span>{new Date(job.created_at).toLocaleDateString()}</span>
          <Link href={`/market/${job.platform}/${job.market_id}?tab=research`}>
            <Button variant="ghost" size="sm">
              View Research
            </Button>
          </Link>
        </div>
      </CardContent>
    </Card>
  );
}
