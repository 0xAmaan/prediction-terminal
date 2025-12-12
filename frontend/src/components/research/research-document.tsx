"use client";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { TrendingUp, TrendingDown, Minus, ExternalLink } from "lucide-react";
import ReactMarkdown from "react-markdown";
import type { SynthesizedReport, KeyFactor } from "@/lib/types";

interface ResearchDocumentProps {
  report: SynthesizedReport;
}

export function ResearchDocument({ report }: ResearchDocumentProps) {
  return (
    <div className="space-y-6">
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
