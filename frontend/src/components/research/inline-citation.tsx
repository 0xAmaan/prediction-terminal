"use client";

import { useState, useRef } from "react";
import { SourceInfo } from "@/lib/types";
import { cn } from "@/lib/utils";

interface InlineCitationProps {
  sourceIds: number[];
  label: string;
  sources: SourceInfo[];
}

export function InlineCitation({ sourceIds, label, sources }: InlineCitationProps) {
  const [isOpen, setIsOpen] = useState(false);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Get the sources for this citation
  const citedSources = sourceIds
    .map((id) => sources.find((s) => s.id === id))
    .filter((s): s is SourceInfo => s !== undefined);

  const count = citedSources.length;
  const displayLabel = count > 1 ? `${label} +${count - 1}` : label;

  const handleMouseEnter = () => {
    if (timeoutRef.current) clearTimeout(timeoutRef.current);
    setIsOpen(true);
  };

  const handleMouseLeave = () => {
    timeoutRef.current = setTimeout(() => setIsOpen(false), 200);
  };

  const handleClick = () => {
    // Open first source in new tab
    if (citedSources[0]) {
      window.open(citedSources[0].url, "_blank", "noopener,noreferrer");
    }
  };

  if (citedSources.length === 0) {
    // Fallback if no sources found - just show the label
    return (
      <span className="inline-flex items-center gap-1 px-1.5 py-0.5 text-xs rounded bg-muted/30 text-muted-foreground">
        {label}
      </span>
    );
  }

  return (
    <span className="relative inline-block align-baseline">
      <button
        onClick={handleClick}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
        className={cn(
          "inline-flex items-center gap-1 px-1.5 py-0.5 text-xs rounded-full",
          "bg-primary/10 hover:bg-primary/20 text-primary/80 hover:text-primary",
          "border border-primary/20 hover:border-primary/40",
          "transition-all duration-150 cursor-pointer",
          "font-medium"
        )}
      >
        {displayLabel}
      </button>

      {isOpen && (
        <span
          onMouseEnter={handleMouseEnter}
          onMouseLeave={handleMouseLeave}
          className={cn(
            "absolute z-50 top-full left-0 mt-1.5 w-80 block",
            "bg-popover border border-border rounded-lg shadow-xl",
            "animate-in fade-in-0 zoom-in-95 duration-150"
          )}
        >
          <span className="block p-1.5 max-h-64 overflow-y-auto">
            {citedSources.map((source, idx) => (
              <a
                key={source.id}
                href={source.url}
                target="_blank"
                rel="noopener noreferrer"
                className={cn(
                  "flex items-start gap-2.5 p-2 rounded-md",
                  "hover:bg-muted/50 transition-colors",
                  idx < citedSources.length - 1 && "mb-0.5"
                )}
              >
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
                  <span className="w-4 h-4 mt-0.5 rounded-sm bg-muted/50 flex-shrink-0 flex items-center justify-center text-[10px] text-muted-foreground">
                    {source.site_name?.[0]?.toUpperCase() || "?"}
                  </span>
                )}
                <span className="flex-1 min-w-0">
                  <span className="block text-sm font-medium text-foreground line-clamp-2 leading-tight">
                    {source.title || source.url}
                  </span>
                  <span className="block text-xs text-muted-foreground mt-0.5 truncate">
                    {source.site_name || new URL(source.url).hostname.replace("www.", "")}
                  </span>
                </span>
              </a>
            ))}
          </span>
        </span>
      )}
    </span>
  );
}
