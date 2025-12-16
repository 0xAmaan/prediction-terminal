"use client";

import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";

export interface TradingBalance {
  usdcBalance: string;
  usdcAllowance: string;
  walletAddress: string;
  /** Whether CTF tokens are approved for selling (all required contracts) */
  ctfApproved: boolean;
  /** Whether CTF Exchange specifically is approved */
  ctfExchangeApproved: boolean;
  /** Whether Neg Risk CTF Exchange is approved */
  negRiskCtfApproved: boolean;
  /** Whether Neg Risk Adapter is approved (required for multi-outcome markets) */
  negRiskAdapterApproved: boolean;
}

/**
 * Hook to fetch and poll trading wallet balance
 *
 * @param enabled - Whether to enable the query (default: true)
 * @param pollInterval - Polling interval in ms (default: 30000 = 30s)
 */
export const useTradingBalance = (
  enabled: boolean = true,
  pollInterval: number = 30000,
) => {
  const query = useQuery({
    queryKey: ["trading-balance"],
    queryFn: async (): Promise<TradingBalance> => {
      return api.getTradingBalance();
    },
    enabled,
    refetchInterval: pollInterval,
    staleTime: 10000, // Consider data stale after 10s
    retry: 2,
  });

  // Parse balance values
  const balance = query.data?.usdcBalance
    ? parseFloat(query.data.usdcBalance)
    : 0;
  const allowance = query.data?.usdcAllowance
    ? parseFloat(query.data.usdcAllowance)
    : 0;

  // Flags for UI
  const hasBalance = balance > 0;
  const hasAllowance = allowance > 0;
  const needsApproval = hasBalance && !hasAllowance;

  // CTF approval status (for selling)
  const ctfApproved = query.data?.ctfApproved ?? false;
  const needsCtfApproval = !ctfApproved;

  return {
    // Raw data
    data: query.data,
    isLoading: query.isLoading,
    isError: query.isError,
    error: query.error,

    // Parsed values
    balance,
    allowance,
    walletAddress: query.data?.walletAddress ?? "",

    // Flags
    hasBalance,
    hasAllowance,
    needsApproval,

    // CTF approval for selling
    ctfApproved,
    needsCtfApproval,

    // Actions
    refetch: query.refetch,
  };
};

export default useTradingBalance;
