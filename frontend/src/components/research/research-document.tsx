"use client";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { TrendingUp, TrendingDown, Minus, ExternalLink, Loader2, Lock } from "lucide-react";
import ReactMarkdown from "react-markdown";
import { cn } from "@/lib/utils";
import { VersionHistory } from "./version-history";
import type { SynthesizedReport, KeyFactor } from "@/lib/types";

interface ResearchDocumentProps {
  report: SynthesizedReport;
  isStreaming?: boolean;
  streamingContent?: string | null;
  platform?: string;
  marketId?: string;
  selectedVersion?: string | null;
  onVersionChange?: (versionKey: string | null) => void;
  isViewingHistorical?: boolean;
}

export function ResearchDocument({
  report,
  isStreaming = false,
  streamingContent,
  platform,
  marketId,
  selectedVersion,
  onVersionChange,
  isViewingHistorical = false,
}: ResearchDocumentProps) {
  const showVersionHistory = platform && marketId && onVersionChange;

  return (
    <div className={cn("space-y-6", isStreaming && "animate-pulse-subtle")}>
      {/* Document Header with Version History */}
      {showVersionHistory && (
        <div className="flex items-center justify-between">
          <VersionHistory
            platform={platform}
            marketId={marketId}
            selectedVersion={selectedVersion ?? null}
            onVersionChange={onVersionChange}
            disabled={isStreaming}
          />
          {isViewingHistorical && (
            <Badge variant="outline" className="flex items-center gap-1.5 text-amber-400 border-amber-500/30 bg-amber-500/10">
              <Lock className="h-3 w-3" />
              Read Only
            </Badge>
          )}
        </div>
      )}
      {/* Streaming indicator */}
      {isStreaming && (
        <div className="flex items-center gap-2 p-3 bg-primary/10 border border-primary/30 rounded-lg">
          <Loader2 className="h-4 w-4 animate-spin text-primary" />
          <span className="text-sm text-primary">Updating research with new findings...</span>
        </div>
      )}

      {/* Streaming content preview */}
      {streamingContent && (
        <Card className="border-primary/30 bg-primary/5">
          <CardHeader className="pb-2">
            <CardTitle className="text-base flex items-center gap-2">
              <Loader2 className="h-4 w-4 animate-spin" />
              New Research Content
            </CardTitle>
          </CardHeader>
          <CardContent className="prose prose-invert prose-sm max-w-none">
            <ReactMarkdown>{streamingContent}</ReactMarkdown>
          </CardContent>
        </Card>
      )}

      {/* Executive Summary */}
      <Card className="border-border/30">
        <CardHeader className="pb-2">
          <CardTitle className="text-base">Executive Summary</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground leading-relaxed whitespace-pre-wrap">
            {report.executive_summary}
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
            {report.key_factors.map((factor, i) => (
              <KeyFactorBadge key={i} factor={factor} />
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Sections */}
      {report.sections.map((section, i) => (
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
          <CardTitle className="text-base">Confidence Assessment</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground">{report.confidence_assessment}</p>
        </CardContent>
      </Card>

      {/* Sources */}
      <Card className="border-border/30">
        <CardHeader className="pb-2">
          <CardTitle className="text-base">Sources ({report.sources.length})</CardTitle>
        </CardHeader>
        <CardContent>
          <ul className="space-y-2 text-sm">
            {report.sources.map((source, i) => (
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
