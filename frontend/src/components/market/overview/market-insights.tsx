"use client";

import { Sparkles } from "lucide-react";

// Fey color tokens
const fey = {
  bg300: "#131419",
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  purple: "#6166DC",
  border: "rgba(255, 255, 255, 0.06)",
};

interface MarketInsightsProps {
  className?: string;
}

export const MarketInsights = ({ className = "" }: MarketInsightsProps) => {
  return (
    <div
      className={`rounded-lg ${className}`}
      style={{
        backgroundColor: fey.bg300,
        border: `1px solid ${fey.border}`,
      }}
    >
      <div className="p-5 pb-3">
        <div className="flex items-center gap-2">
          <div
            className="p-1.5 rounded"
            style={{ backgroundColor: "rgba(97, 102, 220, 0.1)" }}
          >
            <Sparkles className="h-4 w-4" style={{ color: fey.purple }} />
          </div>
          <span
            className="text-sm font-semibold"
            style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
          >
            Market Insights
          </span>
        </div>
      </div>
      <div className="px-5 pb-5">
        <div className="space-y-3">
          <p
            className="text-sm leading-relaxed"
            style={{ color: fey.grey500 }}
          >
            AI-powered market analysis coming soon. This section will provide
            contextual insights about market movements, key events, and
            sentiment analysis.
          </p>
          <div className="flex items-center gap-2 text-xs">
            <div
              className="h-1.5 w-1.5 rounded-full animate-pulse"
              style={{ backgroundColor: "rgba(97, 102, 220, 0.5)" }}
            />
            <span style={{ color: "rgba(125, 139, 150, 0.6)" }}>
              Feature in development
            </span>
          </div>
        </div>
      </div>
    </div>
  );
};
