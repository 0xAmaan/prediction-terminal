"use client";

import { ReactNode } from "react";
import { SourceInfo } from "./types";
import { InlineCitation } from "@/components/research/inline-citation";

// Regex to match [cite:1,2,3:Label Text]
const CITATION_REGEX = /\[cite:([\d,]+):([^\]]+)\]/g;

interface ParsedSegment {
  type: "text" | "citation";
  content: string;
  sourceIds?: number[];
  label?: string;
}

/**
 * Parse text content and extract citation markers
 */
export function parseCitations(text: string): ParsedSegment[] {
  const segments: ParsedSegment[] = [];
  let lastIndex = 0;

  // Reset regex state
  CITATION_REGEX.lastIndex = 0;

  let match;
  while ((match = CITATION_REGEX.exec(text)) !== null) {
    // Add text before citation
    if (match.index > lastIndex) {
      segments.push({
        type: "text",
        content: text.slice(lastIndex, match.index),
      });
    }

    // Parse citation
    const sourceIds = match[1].split(",").map((id) => parseInt(id.trim(), 10));
    const label = match[2].trim();

    segments.push({
      type: "citation",
      content: match[0],
      sourceIds,
      label,
    });

    lastIndex = match.index + match[0].length;
  }

  // Add remaining text
  if (lastIndex < text.length) {
    segments.push({
      type: "text",
      content: text.slice(lastIndex),
    });
  }

  return segments;
}

/**
 * Check if text contains any citation markers
 */
export function hasCitations(text: string): boolean {
  CITATION_REGEX.lastIndex = 0;
  return CITATION_REGEX.test(text);
}

/**
 * Render text with inline citations as React nodes
 */
export function renderWithCitations(
  text: string,
  sources: SourceInfo[]
): ReactNode[] {
  const segments = parseCitations(text);

  return segments.map((segment, idx) => {
    if (segment.type === "citation" && segment.sourceIds && segment.label) {
      return (
        <InlineCitation
          key={`cite-${idx}`}
          sourceIds={segment.sourceIds}
          label={segment.label}
          sources={sources}
        />
      );
    }
    return <span key={`text-${idx}`}>{segment.content}</span>;
  });
}
