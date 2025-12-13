"use client";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import { WebSocketProvider } from "@/providers/websocket-provider";

export const Providers = ({ children }: { children: ReactNode }) => {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            // Backend cache TTL is 5 minutes, so data is fresh for that long
            staleTime: 5 * 60 * 1000, // 5 minutes
            // No automatic refetch - rely on backend cache freshness
            // Real-time data comes via WebSocket anyway
            refetchInterval: false,
            // Refetch on window focus for stale data
            refetchOnWindowFocus: "always",
          },
        },
      })
  );

  return (
    <QueryClientProvider client={queryClient}>
      <WebSocketProvider>{children}</WebSocketProvider>
    </QueryClientProvider>
  );
};
