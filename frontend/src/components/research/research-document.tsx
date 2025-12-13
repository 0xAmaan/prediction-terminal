"use client";

import React, { useEffect, useRef, useState, useMemo, ReactNode } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { TrendingUp, TrendingDown, Minus, ExternalLink, Loader2, Lock, FileText } from "lucide-react";
import ReactMarkdown, { Components } from "react-markdown";
import remarkGfm from "remark-gfm";
import remarkBreaks from "remark-breaks";
import { cn } from "@/lib/utils";
import { VersionHistory } from "./version-history";
import { TradingAnalysisPanel } from "./trading-analysis";
import type { SynthesizedReport, KeyFactor, SourceInfo } from "@/lib/types";
import { hasCitations, renderWithCitations } from "@/lib/citation-parser";

// Helper to process children and render citations
function processChildren(children: ReactNode, sources: SourceInfo[]): ReactNode {
  return React.Children.map(children, (child) => {
    if (typeof child === "string") {
      // Check if string contains citation markers
      if (hasCitations(child)) {
        return <>{renderWithCitations(child, sources)}</>;
      }
    }
    return child;
  });
}

// Create markdown components with citation support
const createMarkdownComponents = (sources: SourceInfo[]): Components => ({
  // Paragraphs with proper spacing and citation support
  p: ({ children }) => {
    const processedChildren = processChildren(children, sources);
    return <p className="mb-3 last:mb-0 leading-relaxed">{processedChildren}</p>;
  },
  // Headings with proper spacing
  h2: ({ children }) => (
    <h2 className="text-foreground font-semibold text-base mt-6 mb-3 first:mt-0">{children}</h2>
  ),
  h3: ({ children }) => (
    <h3 className="text-foreground font-semibold text-sm mt-5 mb-2 first:mt-0">{children}</h3>
  ),
  h4: ({ children }) => (
    <h4 className="text-foreground font-semibold text-sm mt-4 mb-2 first:mt-0">{children}</h4>
  ),
  // Bold text styling with citation support
  strong: ({ children }) => {
    const processedChildren = processChildren(children, sources);
    return <strong className="font-semibold text-foreground">{processedChildren}</strong>;
  },
  // Lists with proper spacing
  ul: ({ children }) => (
    <ul className="list-disc pl-5 mb-3 space-y-1">{children}</ul>
  ),
  ol: ({ children }) => (
    <ol className="list-decimal pl-5 mb-3 space-y-1">{children}</ol>
  ),
  // List items with citation support
  li: ({ children }) => {
    const processedChildren = processChildren(children, sources);
    return <li className="pl-1">{processedChildren}</li>;
  },
  // Links
  a: ({ children, href }) => (
    <a href={href} className="text-primary hover:underline" target="_blank" rel="noopener noreferrer">{children}</a>
  ),
});

interface ResearchDocumentProps {
  report: SynthesizedReport;
  isStreaming?: boolean;
  streamingContent?: string | null;
  platform?: string;
  marketId?: string;
  selectedVersion?: string | null;
  onVersionChange?: (versionKey: string | null) => void;
  isViewingHistorical?: boolean;
  onScrollToTop?: () => void;
  versionRefreshKey?: number;
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
  onScrollToTop,
  versionRefreshKey = 0,
}: ResearchDocumentProps) {
  const showVersionHistory = platform && marketId && onVersionChange;
  const streamingRef = useRef<HTMLDivElement>(null);
  const executiveSummaryRef = useRef<HTMLDivElement>(null);
  const [flashKey, setFlashKey] = useState(0);
  const [contentFlash, setContentFlash] = useState(false);

  // Auto-scroll to streaming content when it appears and flash it
  useEffect(() => {
    if (streamingContent && streamingRef.current) {
      streamingRef.current.scrollIntoView({ behavior: "smooth", block: "start" });
      setFlashKey((prev) => prev + 1);
    }
  }, [streamingContent]);

  // Flash and scroll to top when streaming starts
  useEffect(() => {
    if (isStreaming && onScrollToTop) {
      onScrollToTop();
    }
  }, [isStreaming, onScrollToTop]);

  // Flash content when version refresh key changes (after chat triggers research)
  useEffect(() => {
    if (versionRefreshKey > 0) {
      setContentFlash(true);
      // Remove flash class after animation completes
      const timer = setTimeout(() => setContentFlash(false), 1500);
      // Scroll to executive summary
      if (executiveSummaryRef.current) {
        executiveSummaryRef.current.scrollIntoView({ behavior: "smooth", block: "start" });
      }
      return () => clearTimeout(timer);
    }
  }, [versionRefreshKey]);

  // Memoize markdown components with citation support
  const markdownComponents = useMemo(
    () => createMarkdownComponents(report.sources),
    [report.sources]
  );

  // Render executive summary with citation support
  const executiveSummaryWithCitations = useMemo(() => {
    if (hasCitations(report.executive_summary)) {
      return renderWithCitations(report.executive_summary, report.sources);
    }
    return report.executive_summary;
  }, [report.executive_summary, report.sources]);

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
            refreshKey={versionRefreshKey}
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
        <div ref={streamingRef} key={flashKey} className="animate-content-flash">
          <Card className="border-primary/30 bg-primary/5">
            <CardHeader className="pb-2">
              <CardTitle className="text-base flex items-center gap-2">
                <Loader2 className="h-4 w-4 animate-spin" />
                New Research Content
              </CardTitle>
            </CardHeader>
            <CardContent className="text-sm text-muted-foreground">
              <ReactMarkdown
                remarkPlugins={[remarkGfm, remarkBreaks]}
                components={markdownComponents}
              >
                {streamingContent}
              </ReactMarkdown>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Executive Summary */}
      <div ref={executiveSummaryRef} className={cn(contentFlash && "animate-content-flash")}>
        <Card className="border-border/30">
          <CardHeader className="pb-2">
            <CardTitle className="text-base">Executive Summary</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-muted-foreground leading-relaxed whitespace-pre-wrap">
              {executiveSummaryWithCitations}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Trading Analysis */}
      {report.trading_analysis && (
        <div className={cn(contentFlash && "animate-content-flash")}>
          <TradingAnalysisPanel analysis={report.trading_analysis} />
        </div>
      )}

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
          <CardContent className="text-sm text-muted-foreground">
            <ReactMarkdown
              remarkPlugins={[remarkGfm, remarkBreaks]}
              components={markdownComponents}
            >
              {section.content}
            </ReactMarkdown>
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
        <CardHeader className="pb-3">
          <CardTitle className="text-base flex items-center gap-2">
            <FileText className="h-4 w-4" />
            Sources ({report.sources.length + (report.general_sources?.length || 0)})
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Cited Sources */}
          {report.sources.length > 0 && (
            <div>
              <h4 className="text-sm font-medium text-muted-foreground mb-3">
                Cited Sources
              </h4>
              <ul className="space-y-2">
                {report.sources.map((source) => (
                  <li key={source.id} className="flex items-start gap-2.5 p-2 rounded-md hover:bg-muted/30 transition-colors">
                    {source.favicon_url ? (
                      <img
                        src={source.favicon_url}
                        alt=""
                        className="w-4 h-4 mt-0.5 rounded-sm flex-shrink-0"
                        onError={(e) => {
                          (e.target as HTMLImageElement).style.display = "none";
                        }}
                      />
                    ) : (
                      <div className="w-4 h-4 mt-0.5 rounded-sm bg-muted/50 flex-shrink-0 flex items-center justify-center text-[10px] text-muted-foreground">
                        {source.site_name?.[0]?.toUpperCase() || "?"}
                      </div>
                    )}
                    <div className="flex-1 min-w-0">
                      <a
                        href={source.url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-sm text-primary hover:underline block leading-tight"
                      >
                        {source.title || source.url}
                      </a>
                      <span className="text-xs text-muted-foreground mt-0.5 flex items-center gap-1.5">
                        <span className="bg-muted px-1.5 py-0.5 rounded text-[10px] font-medium">{source.id}</span>
                        {source.site_name}
                      </span>
                    </div>
                  </li>
                ))}
              </ul>
            </div>
          )}

          {/* General Sources */}
          {report.general_sources && report.general_sources.length > 0 && (
            <div>
              <h4 className="text-sm font-medium text-muted-foreground mb-3">
                Additional Sources
              </h4>
              <ul className="space-y-2 text-sm">
                {report.general_sources.map((url, i) => (
                  <li key={i} className="flex items-start gap-2">
                    <ExternalLink className="h-4 w-4 mt-0.5 flex-shrink-0 text-muted-foreground" />
                    <a
                      href={url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-primary hover:underline break-all"
                    >
                      {url}
                    </a>
                  </li>
                ))}
              </ul>
            </div>
          )}
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
