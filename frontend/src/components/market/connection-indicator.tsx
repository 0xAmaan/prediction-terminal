"use client";

import { cn } from "@/lib/utils";

// Fey color tokens
const fey = {
  grey500: "#7D8B96",
  teal: "#4DBE95",
  red: "#D84F68",
  amber: "#D4A853", // Warm amber for connecting/reconnecting states
};

type ConnectionState = "connected" | "connecting" | "disconnected" | "reconnecting";

interface ConnectionIndicatorProps {
  state: ConnectionState;
  latency?: number | null;
  className?: string;
  showLabel?: boolean;
}

const stateConfig: Record<
  ConnectionState,
  { color: string; label: string }
> = {
  connected: {
    color: fey.teal,
    label: "Live",
  },
  connecting: {
    color: fey.amber,
    label: "Connecting",
  },
  reconnecting: {
    color: fey.amber,
    label: "Reconnecting",
  },
  disconnected: {
    color: fey.red,
    label: "Disconnected",
  },
};

export const ConnectionIndicator = ({
  state,
  latency,
  className,
  showLabel = true,
}: ConnectionIndicatorProps) => {
  const config = stateConfig[state];
  const isActive = state === "connected" || state === "connecting" || state === "reconnecting";

  return (
    <div className={cn("flex items-center gap-2", className)}>
      <div className="relative flex h-2 w-2">
        {isActive && (
          <span
            className="absolute inline-flex h-full w-full animate-ping rounded-full opacity-75"
            style={{ backgroundColor: config.color }}
          />
        )}
        <span
          className="relative inline-flex h-2 w-2 rounded-full"
          style={{ backgroundColor: config.color }}
        />
      </div>
      {showLabel && (
        <span className="text-xs" style={{ color: fey.grey500 }}>
          {config.label}
          {state === "connected" && latency !== null && latency !== undefined && (
            <span className="ml-1 opacity-60">({latency}ms)</span>
          )}
        </span>
      )}
    </div>
  );
};
