"use client";

import { cn } from "@/lib/utils";

type ConnectionState = "connected" | "connecting" | "disconnected" | "reconnecting";

interface ConnectionIndicatorProps {
  state: ConnectionState;
  latency?: number | null;
  className?: string;
  showLabel?: boolean;
}

const stateConfig: Record<
  ConnectionState,
  { color: string; pulseColor: string; label: string }
> = {
  connected: {
    color: "bg-green-500",
    pulseColor: "bg-green-400",
    label: "Live",
  },
  connecting: {
    color: "bg-yellow-500",
    pulseColor: "bg-yellow-400",
    label: "Connecting",
  },
  reconnecting: {
    color: "bg-yellow-500",
    pulseColor: "bg-yellow-400",
    label: "Reconnecting",
  },
  disconnected: {
    color: "bg-red-500",
    pulseColor: "bg-red-400",
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
            className={cn(
              "absolute inline-flex h-full w-full animate-ping rounded-full opacity-75",
              config.pulseColor
            )}
          />
        )}
        <span
          className={cn(
            "relative inline-flex h-2 w-2 rounded-full",
            config.color
          )}
        />
      </div>
      {showLabel && (
        <span className="text-xs text-muted-foreground">
          {config.label}
          {state === "connected" && latency !== null && latency !== undefined && (
            <span className="ml-1 opacity-60">({latency}ms)</span>
          )}
        </span>
      )}
    </div>
  );
};
