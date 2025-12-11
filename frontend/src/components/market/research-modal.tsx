"use client";

import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Loader2,
  CheckCircle,
  XCircle,
  TrendingUp,
  TrendingDown,
  Minus,
  ExternalLink,
} from "lucide-react";
import type { ResearchJob, KeyFactor } from "@/lib/types";
import ReactMarkdown from "react-markdown";

interface ResearchModalProps {
  isOpen: boolean;
  onClose: () => void;
  job: ResearchJob | null;
  marketTitle: string;
  error?: string | null;
}

export function ResearchModal({
  isOpen,
  onClose,
  job,
  marketTitle,
  error: externalError,
}: ResearchModalProps) {
  const progressPercent =
    job && job.progress.total_steps > 0
      ? (job.progress.completed_steps / job.progress.total_steps) * 100
      : 0;

  const isComplete = job?.status === "completed";
  const isFailed = job?.status === "failed";
  const isRunning = job && !isComplete && !isFailed;

  // Show loading state while waiting for job
  const showLoading = !job && !externalError;

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="max-w-4xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            {showLoading && <Loader2 className="h-5 w-5 animate-spin" />}
            {isRunning && <Loader2 className="h-5 w-5 animate-spin" />}
            {isComplete && <CheckCircle className="h-5 w-5 text-green-500" />}
            {isFailed && <XCircle className="h-5 w-5 text-red-500" />}
            Research: {marketTitle}
          </DialogTitle>
        </DialogHeader>

        {/* Loading State */}
        {showLoading && (
          <div className="flex items-center justify-center py-8">
            <div className="text-center">
              <Loader2 className="h-8 w-8 animate-spin mx-auto mb-4 text-muted-foreground" />
              <p className="text-muted-foreground">Starting research...</p>
            </div>
          </div>
        )}

        {/* External Error */}
        {externalError && !job && (
          <div className="p-4 bg-red-500/10 border border-red-500/30 rounded-lg">
            <p className="text-red-400">{externalError}</p>
          </div>
        )}

        {/* Progress Section */}
        {isRunning && (
          <div className="space-y-4">
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
            <p className="text-red-400">{job?.error || "Research failed"}</p>
          </div>
        )}

        {/* Report Display */}
        {isComplete && job?.report && (
          <div className="space-y-6">
            {/* Executive Summary */}
            <Card className="border-border/30">
              <CardHeader className="pb-2">
                <CardTitle className="text-base">Executive Summary</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-muted-foreground leading-relaxed whitespace-pre-wrap">
                  {job.report.executive_summary}
                </p>
              </CardContent>
            </Card>

            {/* Key Factors */}
            <Card className="border-border/30">
              <CardHeader className="pb-2">
                <CardTitle className="text-base">Key Factors</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="grid gap-3">
                  {job.report.key_factors.map((factor, i) => (
                    <KeyFactorBadge key={i} factor={factor} />
                  ))}
                </div>
              </CardContent>
            </Card>

            {/* Sections */}
            {job.report.sections.map((section, i) => (
              <Card key={i} className="border-border/30">
                <CardHeader className="pb-2">
                  <CardTitle className="text-base">{section.heading}</CardTitle>
                </CardHeader>
                <CardContent className="prose prose-invert prose-sm max-w-none prose-p:text-muted-foreground prose-headings:text-foreground prose-strong:text-foreground prose-a:text-primary">
                  <ReactMarkdown>{section.content}</ReactMarkdown>
                </CardContent>
              </Card>
            ))}

            {/* Confidence Assessment */}
            <Card className="border-border/30">
              <CardHeader className="pb-2">
                <CardTitle className="text-base">
                  Confidence Assessment
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-muted-foreground">
                  {job.report.confidence_assessment}
                </p>
              </CardContent>
            </Card>

            {/* Sources */}
            <Card className="border-border/30">
              <CardHeader className="pb-2">
                <CardTitle className="text-base">
                  Sources ({job.report.sources.length})
                </CardTitle>
              </CardHeader>
              <CardContent>
                <ul className="space-y-2 text-sm">
                  {job.report.sources.map((source, i) => (
                    <li key={i} className="flex items-start gap-2">
                      <ExternalLink className="h-4 w-4 mt-0.5 flex-shrink-0 text-muted-foreground" />
                      <a
                        href={source}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-primary hover:underline break-all"
                      >
                        {source}
                      </a>
                    </li>
                  ))}
                </ul>
              </CardContent>
            </Card>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}

function KeyFactorBadge({ factor }: { factor: KeyFactor }) {
  const impactIcon = {
    bullish: <TrendingUp className="h-4 w-4 text-green-500" />,
    bearish: <TrendingDown className="h-4 w-4 text-red-500" />,
    neutral: <Minus className="h-4 w-4 text-gray-500" />,
  }[factor.impact];

  const confidenceColor = {
    high: "bg-green-500/20 text-green-400 border-green-500/30",
    medium: "bg-yellow-500/20 text-yellow-400 border-yellow-500/30",
    low: "bg-gray-500/20 text-gray-400 border-gray-500/30",
  }[factor.confidence];

  return (
    <div className="flex items-center justify-between p-3 bg-muted/50 rounded-lg">
      <div className="flex items-center gap-2">
        {impactIcon}
        <span className="text-sm">{factor.factor}</span>
      </div>
      <Badge variant="outline" className={confidenceColor}>
        {factor.confidence}
      </Badge>
    </div>
  );
}
