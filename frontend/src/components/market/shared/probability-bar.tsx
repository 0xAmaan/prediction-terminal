"use client";

import { cn } from "@/lib/utils";

// Fey color tokens
const fey = {
  bg400: "#16181C",
  grey500: "#7D8B96",
  teal: "#4DBE95",
  tealDark: "#3DA07A",
};

interface ProbabilityBarProps {
  probability: number; // 0-100
  className?: string;
  showLabels?: boolean;
  size?: "sm" | "md" | "lg";
}

export const ProbabilityBar = ({
  probability,
  className = "",
  showLabels = false,
  size = "md",
}: ProbabilityBarProps) => {
  const clampedProbability = Math.min(100, Math.max(0, probability));

  const heightClasses = {
    sm: "h-1",
    md: "h-2",
    lg: "h-3",
  };

  return (
    <div className={cn("w-full", className)}>
      {showLabels && (
        <div className="flex justify-between mb-1.5">
          <span className="text-xs" style={{ color: fey.grey500 }}>YES</span>
          <span className="text-xs" style={{ color: fey.grey500 }}>NO</span>
        </div>
      )}
      <div
        className={cn(
          "w-full rounded-full overflow-hidden",
          heightClasses[size]
        )}
        style={{ backgroundColor: fey.bg400 }}
      >
        <div
          className="h-full rounded-full transition-all duration-500 ease-out"
          style={{
            width: `${clampedProbability}%`,
            background: `linear-gradient(to right, ${fey.teal}, ${fey.tealDark})`,
          }}
        />
      </div>
    </div>
  );
};
