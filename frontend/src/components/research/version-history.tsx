"use client";

import { useState, useEffect, useRef } from "react";
import { ChevronDown, Clock, Loader2 } from "lucide-react";
import { api } from "@/lib/api";
import { cn } from "@/lib/utils";
import type { ResearchVersion } from "@/lib/types";

interface VersionHistoryProps {
  platform: string;
  marketId: string;
  selectedVersion: string | null; // null = current
  onVersionChange: (versionKey: string | null) => void;
  disabled?: boolean;
}

export function VersionHistory({
  platform,
  marketId,
  selectedVersion,
  onVersionChange,
  disabled = false,
}: VersionHistoryProps) {
  const [versions, setVersions] = useState<ResearchVersion[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isOpen, setIsOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const fetchVersions = async () => {
      try {
        setIsLoading(true);
        setError(null);
        const data = await api.getVersions(platform, marketId);
        setVersions(data.versions);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load versions");
      } finally {
        setIsLoading(false);
      }
    };

    fetchVersions();
  }, [platform, marketId]);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
      hour: "numeric",
      minute: "2-digit",
      hour12: true,
    });
  };

  const getSelectedLabel = () => {
    if (!selectedVersion) return "Current";
    const version = versions.find((v) => v.key === selectedVersion);
    return version ? formatDate(version.created_at) : "Loading...";
  };

  if (isLoading) {
    return (
      <div className="flex items-center gap-2 text-sm text-muted-foreground">
        <Loader2 className="h-4 w-4 animate-spin" />
        <span>Loading versions...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="text-sm text-muted-foreground">
        Version history unavailable
      </div>
    );
  }

  if (versions.length === 0) {
    return null; // No versions to show
  }

  return (
    <div className="relative" ref={dropdownRef}>
      <button
        type="button"
        onClick={() => !disabled && setIsOpen(!isOpen)}
        disabled={disabled}
        className={cn(
          "flex items-center gap-2 px-3 py-1.5 text-sm rounded-md border border-border/50 bg-background/50 hover:bg-secondary/50 transition-colors",
          disabled && "opacity-50 cursor-not-allowed",
          isOpen && "bg-secondary/50"
        )}
      >
        <Clock className="h-4 w-4 text-muted-foreground" />
        <span>Version: {getSelectedLabel()}</span>
        <ChevronDown
          className={cn(
            "h-4 w-4 text-muted-foreground transition-transform",
            isOpen && "rotate-180"
          )}
        />
      </button>

      {isOpen && (
        <div className="absolute top-full left-0 mt-1 z-50 min-w-[200px] max-h-[300px] overflow-y-auto rounded-md border border-border/50 bg-card shadow-lg">
          {/* Current version option */}
          <button
            type="button"
            onClick={() => {
              onVersionChange(null);
              setIsOpen(false);
            }}
            className={cn(
              "w-full px-3 py-2 text-left text-sm hover:bg-secondary/50 transition-colors flex items-center justify-between",
              !selectedVersion && "bg-secondary/30 font-medium"
            )}
          >
            <span>Current</span>
            {!selectedVersion && (
              <span className="text-xs text-primary">Active</span>
            )}
          </button>

          {/* Divider */}
          {versions.length > 0 && (
            <div className="border-t border-border/30 my-1" />
          )}

          {/* Historical versions */}
          {versions.map((version) => (
            <button
              key={version.key}
              type="button"
              onClick={() => {
                onVersionChange(version.key);
                setIsOpen(false);
              }}
              className={cn(
                "w-full px-3 py-2 text-left text-sm hover:bg-secondary/50 transition-colors",
                selectedVersion === version.key && "bg-secondary/30 font-medium"
              )}
            >
              <div className="flex items-center justify-between">
                <span>{formatDate(version.created_at)}</span>
                {selectedVersion === version.key && (
                  <span className="text-xs text-muted-foreground">Viewing</span>
                )}
              </div>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
