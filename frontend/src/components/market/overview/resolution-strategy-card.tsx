"use client";

import { Target, ExternalLink, Calendar, CheckCircle } from "lucide-react";
import type { MarketStatus } from "@/lib/types";

// Fey color tokens
const fey = {
  bg100: "#070709",
  bg200: "#101116",
  bg300: "#131419",
  bg400: "#16181C",
  grey100: "#EEF0F1",
  grey300: "#B6BEC4",
  grey500: "#7D8B96",
  grey700: "#4E5860",
  teal: "#4DBE95",
  red: "#D84F68",
  orange: "#C27C58",
  skyBlue: "#54BBF7",
  border: "rgba(255, 255, 255, 0.06)",
};

interface ResolutionStrategyCardProps {
  resolutionSource: string | null;
  description: string | null;
  closeTime: string | null;
  status: MarketStatus;
  url: string | null;
}

const formatCloseDate = (dateStr: string | null): string => {
  if (!dateStr) return "No end date";
  const date = new Date(dateStr);
  return date.toLocaleDateString("en-US", {
    weekday: "short",
    month: "short",
    day: "numeric",
    year: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });
};

const getStatusConfig = (status: MarketStatus) => {
  switch (status) {
    case "open":
      return { color: fey.teal, label: "Open", icon: "pulse" };
    case "closed":
      return { color: fey.orange, label: "Closed", icon: "check" };
    case "settled":
      return { color: fey.grey500, label: "Settled", icon: "check" };
    default:
      return { color: fey.grey500, label: status, icon: "pulse" };
  }
};

// Extract first paragraph or first 200 chars from description
const extractCriteria = (description: string | null): string => {
  if (!description) return "No resolution criteria specified.";

  // Split by double newline to get paragraphs
  const paragraphs = description.split(/\n\n+/);
  const firstParagraph = paragraphs[0]?.trim() || description;

  // If still too long, truncate
  if (firstParagraph.length > 300) {
    return firstParagraph.slice(0, 297) + "...";
  }
  return firstParagraph;
};

export const ResolutionStrategyCard = ({
  resolutionSource,
  description,
  closeTime,
  status,
  url,
}: ResolutionStrategyCardProps) => {
  const statusConfig = getStatusConfig(status);

  return (
    <div
      className="rounded-lg overflow-hidden"
      style={{
        backgroundColor: fey.bg300,
        border: `1px solid ${fey.border}`,
      }}
    >
      {/* Header */}
      <div
        className="flex items-center gap-2 px-4 py-3"
        style={{ borderBottom: `1px solid ${fey.border}` }}
      >
        <Target className="h-4 w-4" style={{ color: fey.skyBlue }} />
        <h3
          className="text-sm font-semibold"
          style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
        >
          Resolution Strategy
        </h3>
      </div>

      {/* Content */}
      <div className="p-4 space-y-4">
        {/* Resolution Criteria */}
        <div>
          <div
            className="text-[10px] uppercase tracking-wider mb-2 font-medium"
            style={{ color: fey.grey500 }}
          >
            Resolution Criteria
          </div>
          <p
            className="text-sm leading-relaxed"
            style={{ color: fey.grey300 }}
          >
            {extractCriteria(description)}
          </p>
        </div>

        {/* Data Sources */}
        {resolutionSource && (
          <div>
            <div
              className="text-[10px] uppercase tracking-wider mb-2 font-medium"
              style={{ color: fey.grey500 }}
            >
              Data Sources
            </div>
            <p
              className="text-sm leading-relaxed"
              style={{ color: fey.grey300 }}
            >
              {resolutionSource}
            </p>
          </div>
        )}

        {/* Key Dates & Status */}
        <div
          className="pt-3 space-y-3"
          style={{ borderTop: `1px solid ${fey.border}` }}
        >
          {/* Close Time */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Calendar className="h-3.5 w-3.5" style={{ color: fey.grey500 }} />
              <span className="text-xs" style={{ color: fey.grey500 }}>
                Closes
              </span>
            </div>
            <span
              className="text-sm font-medium"
              style={{ color: fey.grey100 }}
            >
              {formatCloseDate(closeTime)}
            </span>
          </div>

          {/* Status */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <CheckCircle className="h-3.5 w-3.5" style={{ color: fey.grey500 }} />
              <span className="text-xs" style={{ color: fey.grey500 }}>
                Status
              </span>
            </div>
            <span
              className="text-[10px] font-medium uppercase tracking-wider px-2 py-1 rounded flex items-center gap-1.5"
              style={{
                backgroundColor: `${statusConfig.color}15`,
                color: statusConfig.color,
              }}
            >
              {statusConfig.icon === "pulse" && (
                <span
                  className="h-1.5 w-1.5 rounded-full animate-pulse"
                  style={{ backgroundColor: statusConfig.color }}
                />
              )}
              {statusConfig.label}
            </span>
          </div>
        </div>

        {/* View Full Rules Link */}
        {url && (
          <a
            href={url}
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-2 pt-2 text-sm font-medium transition-colors hover:opacity-80"
            style={{ color: fey.skyBlue }}
          >
            View Full Rules
            <ExternalLink className="h-3.5 w-3.5" />
          </a>
        )}
      </div>
    </div>
  );
};
