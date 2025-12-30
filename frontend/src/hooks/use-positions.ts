"use client";

import { useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";

export interface Position {
  marketId: string;
  tokenId: string;
  outcome: string;
  shares: string;
  avgPrice: string;
  currentPrice: string;
  pnl: string;
  title: string;
  negRisk: boolean;
}

export const POSITIONS_QUERY_KEY = ["trading-positions"];

/**
 * Hook to fetch and poll trading positions with React Query.
 * Allows cache invalidation from other components (e.g., after placing a trade).
 *
 * @param enabled - Whether to enable the query (default: true)
 * @param pollInterval - Polling interval in ms (default: 30000 = 30s)
 */
export const usePositions = (
  enabled: boolean = true,
  pollInterval: number = 60000, // 60s to avoid overwriting optimistic updates during demos
) => {
  const query = useQuery({
    queryKey: POSITIONS_QUERY_KEY,
    queryFn: async (): Promise<Position[]> => {
      return api.getPositions();
    },
    enabled,
    refetchInterval: pollInterval,
    staleTime: 10000, // Consider data stale after 10s
    retry: 2,
  });

  return {
    positions: query.data ?? [],
    isLoading: query.isLoading,
    isError: query.isError,
    error: query.error,
    refetch: query.refetch,
  };
};

/**
 * Hook to get the query client for invalidating positions cache.
 * Use this in components that modify positions (e.g., trade-execution).
 */
export const useInvalidatePositions = () => {
  const queryClient = useQueryClient();

  return () => {
    queryClient.invalidateQueries({ queryKey: POSITIONS_QUERY_KEY });
  };
};

/**
 * Hook for optimistic position updates.
 * Updates the cache immediately before API confirms, for instant UI feedback.
 */
export const useOptimisticPositionUpdate = () => {
  const queryClient = useQueryClient();

  return {
    /**
     * Add a new position or increase shares of existing position (for BUY orders)
     */
    addPosition: (position: Partial<Position> & { tokenId: string }) => {
      queryClient.setQueryData<Position[]>(POSITIONS_QUERY_KEY, (old = []) => {
        const existing = old.find((p) => p.tokenId === position.tokenId);
        if (existing) {
          // Increase shares of existing position
          const newShares =
            parseFloat(existing.shares) + parseFloat(position.shares || "0");
          return old.map((p) =>
            p.tokenId === position.tokenId
              ? { ...p, shares: newShares.toString() }
              : p,
          );
        }
        // Add new position with defaults for missing fields
        const newPosition: Position = {
          marketId: position.marketId || "",
          tokenId: position.tokenId,
          outcome: position.outcome || "",
          shares: position.shares || "0",
          avgPrice: position.avgPrice || "0",
          currentPrice: position.currentPrice || position.avgPrice || "0",
          pnl: position.pnl || "0",
          title: position.title || "",
          negRisk: position.negRisk ?? false,
        };
        return [...old, newPosition];
      });
    },

    /**
     * Reduce shares or remove position entirely (for SELL orders)
     */
    reducePosition: (tokenId: string, sharesToSell: number) => {
      queryClient.setQueryData<Position[]>(POSITIONS_QUERY_KEY, (old = []) => {
        return old
          .map((p) => {
            if (p.tokenId !== tokenId) return p;
            const remaining = parseFloat(p.shares) - sharesToSell;
            if (remaining <= 0.0001) return null; // Remove if essentially zero
            return { ...p, shares: remaining.toString() };
          })
          .filter((p): p is Position => p !== null);
      });
    },
  };
};

export default usePositions;
