"use client";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  TrendingUp,
  TrendingDown,
  Minus,
  Calendar,
  AlertTriangle,
  Target,
  Scale,
} from "lucide-react";
import { cn } from "@/lib/utils";
import type { TradingAnalysis } from "@/lib/types";

interface TradingAnalysisProps {
  analysis: TradingAnalysis;
}

export function TradingAnalysisPanel({ analysis }: TradingAnalysisProps) {
  const edgePercent = (analysis.implied_edge * 100).toFixed(1);
  const isPositiveEdge = analysis.implied_edge > 0.02; // 2% threshold
  const isNegativeEdge = analysis.implied_edge < -0.02;

  return (
    <div className="space-y-4">
      {/* Fair Value Card */}
      <Card className="border-border/30 bg-card/50">
        <CardHeader className="pb-2">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Target className="h-4 w-4 text-muted-foreground" />
              <CardTitle className="text-base">Fair Value Estimate</CardTitle>
            </div>
            <Badge
              variant="outline"
              className={cn(
                "text-xs",
                analysis.estimate_confidence === "high" &&
                  "bg-green-500/10 text-green-400 border-green-500/30",
                analysis.estimate_confidence === "medium" &&
                  "bg-yellow-500/10 text-yellow-400 border-yellow-500/30",
                analysis.estimate_confidence === "low" &&
                  "bg-muted text-muted-foreground border-border"
              )}
            >
              {analysis.estimate_confidence} confidence
            </Badge>
          </div>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex items-baseline gap-4">
            <div className="text-3xl font-bold">
              {(analysis.fair_value_low * 100).toFixed(0)}–
              {(analysis.fair_value_high * 100).toFixed(0)}%
            </div>
            <div className="text-sm text-muted-foreground">
              vs market at {(analysis.current_price * 100).toFixed(1)}%
            </div>
          </div>

          {/* Edge indicator */}
          <div
            className={cn(
              "flex items-center gap-2 p-3 rounded-lg",
              isPositiveEdge && "bg-green-500/10",
              isNegativeEdge && "bg-red-500/10",
              !isPositiveEdge && !isNegativeEdge && "bg-muted/50"
            )}
          >
            {isPositiveEdge ? (
              <TrendingUp className="h-5 w-5 text-green-500" />
            ) : isNegativeEdge ? (
              <TrendingDown className="h-5 w-5 text-red-500" />
            ) : (
              <Minus className="h-5 w-5 text-muted-foreground" />
            )}
            <span
              className={cn(
                "font-medium text-sm",
                isPositiveEdge && "text-green-400",
                isNegativeEdge && "text-red-400",
                !isPositiveEdge && !isNegativeEdge && "text-muted-foreground"
              )}
            >
              {isPositiveEdge
                ? `+${edgePercent}% edge (market may be underpriced)`
                : isNegativeEdge
                ? `${edgePercent}% edge (market may be overpriced)`
                : "No significant edge detected"}
            </span>
          </div>

          <p className="text-sm text-muted-foreground leading-relaxed">
            {analysis.fair_value_reasoning}
          </p>
        </CardContent>
      </Card>

      {/* Catalysts */}
      {analysis.catalysts.length > 0 && (
        <Card className="border-border/30 bg-card/50">
          <CardHeader className="pb-2">
            <div className="flex items-center gap-2">
              <Calendar className="h-4 w-4 text-muted-foreground" />
              <CardTitle className="text-base">Upcoming Catalysts</CardTitle>
            </div>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              {analysis.catalysts.map((catalyst, i) => (
                <div
                  key={i}
                  className="flex items-start gap-3 text-sm p-2 rounded-lg bg-muted/30"
                >
                  <Badge
                    variant="outline"
                    className={cn(
                      "text-xs shrink-0 mt-0.5",
                      catalyst.expected_impact === "high" &&
                        "bg-red-500/10 text-red-400 border-red-500/30",
                      catalyst.expected_impact === "medium" &&
                        "bg-yellow-500/10 text-yellow-400 border-yellow-500/30",
                      catalyst.expected_impact === "low" &&
                        "bg-muted text-muted-foreground border-border"
                    )}
                  >
                    {catalyst.expected_impact}
                  </Badge>
                  <div className="flex-1 min-w-0">
                    <span className="text-foreground">{catalyst.event}</span>
                    {catalyst.direction_if_positive && (
                      <span
                        className={cn(
                          "ml-2 text-xs",
                          catalyst.direction_if_positive === "bullish"
                            ? "text-green-400"
                            : "text-red-400"
                        )}
                      >
                        ({catalyst.direction_if_positive} if positive)
                      </span>
                    )}
                  </div>
                  {catalyst.date && (
                    <span className="text-muted-foreground text-xs shrink-0">
                      {catalyst.date}
                    </span>
                  )}
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Resolution Analysis */}
      <Card className="border-border/30 bg-card/50">
        <CardHeader className="pb-2">
          <div className="flex items-center gap-2">
            <AlertTriangle className="h-4 w-4 text-muted-foreground" />
            <CardTitle className="text-base">Resolution Criteria</CardTitle>
          </div>
        </CardHeader>
        <CardContent className="space-y-3">
          <p className="text-sm text-muted-foreground leading-relaxed">
            {analysis.resolution_analysis.resolution_summary}
          </p>

          {analysis.resolution_analysis.resolution_source && (
            <div className="text-xs text-muted-foreground">
              <span className="font-medium">Source: </span>
              {analysis.resolution_analysis.resolution_source}
            </div>
          )}

          {analysis.resolution_analysis.ambiguity_flags.length > 0 && (
            <div className="p-3 rounded-lg bg-yellow-500/10 border border-yellow-500/20">
              <p className="text-xs font-medium text-yellow-400 mb-2">
                Potential Ambiguities
              </p>
              <ul className="space-y-1">
                {analysis.resolution_analysis.ambiguity_flags.map((flag, i) => (
                  <li key={i} className="text-xs text-yellow-300/80 flex gap-2">
                    <span className="shrink-0">•</span>
                    <span>{flag}</span>
                  </li>
                ))}
              </ul>
            </div>
          )}

          {analysis.resolution_analysis.historical_edge_cases.length > 0 && (
            <div className="text-xs text-muted-foreground">
              <span className="font-medium">Historical edge cases: </span>
              {analysis.resolution_analysis.historical_edge_cases.join("; ")}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Contrarian View */}
      <Card className="border-border/30 bg-card/50">
        <CardHeader className="pb-2">
          <div className="flex items-center gap-2">
            <Scale className="h-4 w-4 text-muted-foreground" />
            <CardTitle className="text-base">Contrarian Analysis</CardTitle>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          <div>
            <p className="text-xs text-muted-foreground mb-1 uppercase tracking-wide">
              Market Consensus
            </p>
            <p className="text-sm text-foreground">
              {analysis.contrarian_case.consensus_view}
            </p>
          </div>

          <div>
            <p className="text-xs text-muted-foreground mb-1 uppercase tracking-wide">
              The Case Against
            </p>
            <p className="text-sm text-foreground">
              {analysis.contrarian_case.contrarian_case}
            </p>
          </div>

          {analysis.contrarian_case.mispricing_reasons.length > 0 && (
            <div>
              <p className="text-xs text-muted-foreground mb-2 uppercase tracking-wide">
                Why Crowd Might Be Wrong
              </p>
              <ul className="space-y-1">
                {analysis.contrarian_case.mispricing_reasons.map(
                  (reason, i) => (
                    <li
                      key={i}
                      className="text-sm text-muted-foreground flex gap-2"
                    >
                      <span className="text-muted-foreground shrink-0">•</span>
                      <span>{reason}</span>
                    </li>
                  )
                )}
              </ul>
            </div>
          )}

          {analysis.contrarian_case.contrarian_triggers.length > 0 && (
            <div>
              <p className="text-xs text-muted-foreground mb-2 uppercase tracking-wide">
                Contrarian Triggers
              </p>
              <ul className="space-y-1">
                {analysis.contrarian_case.contrarian_triggers.map(
                  (trigger, i) => (
                    <li
                      key={i}
                      className="text-sm text-muted-foreground flex gap-2"
                    >
                      <span className="text-muted-foreground shrink-0">•</span>
                      <span>{trigger}</span>
                    </li>
                  )
                )}
              </ul>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
