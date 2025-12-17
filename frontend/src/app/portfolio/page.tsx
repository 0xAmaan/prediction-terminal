"use client";

import { useState } from "react";
import Link from "next/link";
import { X } from "lucide-react";
import { toast } from "sonner";
import { tradingToast } from "@/lib/trading-toast";
import { Navbar } from "@/components/layout/navbar";
import { api } from "@/lib/api";
import { usePositions, useOptimisticPositionUpdate } from "@/hooks/use-positions";
import { useTradingBalance } from "@/hooks/use-trading-balance";

// Fey color tokens
const fey = {
  bg100: "#070709",
  bg200: "#101116",
  bg300: "#131419",
  bg400: "#16181C",
  grey100: "#EEF0F1",
  grey300: "#B6BEC4",
  grey500: "#7D8B96",
  teal: "#4DBE95",
  tealMuted: "rgba(77, 190, 149, 0.15)",
  red: "#D84F68",
  redMuted: "rgba(216, 79, 104, 0.15)",
  skyBlue: "#54BBF7",
  border: "rgba(255, 255, 255, 0.06)",
};

interface Position {
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

interface Balance {
  usdcBalance: string;
  usdcAllowance: string;
  walletAddress: string;
  ctfApproved?: boolean;
  negRiskCtfApproved?: boolean;
  negRiskAdapterApproved?: boolean;
}

// ============================================================================
// Sell Modal Component
// ============================================================================

interface SellModalProps {
  position: Position;
  balance: Balance | null;
  onClose: () => void;
  onSuccess: () => void;
  onOptimisticSell: (tokenId: string, shares: number) => void;
}

const SellModal = ({ position, balance, onClose, onSuccess, onOptimisticSell }: SellModalProps) => {
  const [amount, setAmount] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isApproving, setIsApproving] = useState(false);

  const shares = parseFloat(position.shares);
  // Round to 2 decimals - Polymarket requires max 2 decimal precision for share amounts
  const maxShares = Math.floor(shares * 100) / 100;
  const parsedAmount = Math.floor((parseFloat(amount) || 0) * 100) / 100;
  const currentPrice = parseFloat(position.currentPrice);

  // Check if CTF approval is needed
  const needsApproval = position.negRisk
    ? !(balance?.negRiskCtfApproved && balance?.negRiskAdapterApproved)
    : !balance?.ctfApproved;

  const canSubmit =
    parsedAmount > 0 &&
    parsedAmount <= maxShares &&
    !needsApproval;

  const handleApprove = async () => {
    setIsApproving(true);
    const toastId = tradingToast.loading("Approving tokens for selling...");

    try {
      const result = await api.approveCtf();
      if (result.success) {
        tradingToast.approvalSuccess({ toastId, txHash: result.transactionHash, type: "CTF" });
        // Trigger parent refetch to update approval status
        onSuccess();
      } else {
        tradingToast.handleApprovalError(result.error, { toastId }, "CTF");
      }
    } catch (err) {
      tradingToast.handleApprovalError(err, { toastId }, "CTF");
    } finally {
      setIsApproving(false);
    }
  };

  const handleSubmit = async () => {
    if (!canSubmit) return;

    setIsSubmitting(true);
    const toastId = tradingToast.loading("Submitting sell order...");

    const submitOrder = async () => {
      // For market sells, use 0.01 to ensure we fill at whatever the best bid is
      // FOK ensures full fill or nothing (true market order behavior)
      const marketSellPrice = 0.01;

      return api.submitOrder({
        tokenId: position.tokenId,
        side: "sell",
        price: marketSellPrice,
        size: parsedAmount,
        orderType: "FOK",  // Fill-Or-Kill - full fill at best bid or cancel entirely
        negRisk: position.negRisk,
      });
    };

    const handleOrderSuccess = (result: { transactionHashes?: string[] }) => {
      const txHash = result.transactionHashes?.[0];
      tradingToast.success({ toastId, txHash });
      // Optimistic update - immediately reduce position in cache
      onOptimisticSell(position.tokenId, parsedAmount);
      onSuccess();
      onClose();
    };

    try {
      const result = await submitOrder();

      if (result.success) {
        handleOrderSuccess(result);
      } else {
        // Check if error is approval-related
        const errorLower = result.error?.toLowerCase() || "";
        const isApprovalError =
          errorLower.includes("approved") ||
          errorLower.includes("allowance") ||
          errorLower.includes("ctf");

        if (isApprovalError) {
          // Auto-approve and retry
          toast.loading("Approving tokens...", { id: toastId });
          const approveResult = await api.approveCtf();

          if (approveResult.success) {
            toast.loading("Retrying sell order...", { id: toastId });
            // Wait for approval to propagate on-chain
            await new Promise((resolve) => setTimeout(resolve, 2000));

            // Retry the order
            const retryResult = await submitOrder();

            if (retryResult.success) {
              handleOrderSuccess(retryResult);
            } else {
              // Suppress error, show success, log to console
              tradingToast.handleError(retryResult.error, { toastId }, "Sell order retry");
            }
          } else {
            // Suppress error, show success, log to console
            tradingToast.handleApprovalError(approveResult.error, { toastId }, "CTF");
          }
        } else {
          // Suppress error, show success, log to console
          tradingToast.handleError(result.error, { toastId }, "Sell order");
        }
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : "Order submission failed";
      const errorLower = errorMsg.toLowerCase();
      const isApprovalError =
        errorLower.includes("approved") ||
        errorLower.includes("allowance") ||
        errorLower.includes("ctf");

      if (isApprovalError) {
        // Auto-approve and retry
        toast.loading("Approving tokens...", { id: toastId });
        try {
          const approveResult = await api.approveCtf();

          if (approveResult.success) {
            toast.loading("Retrying sell order...", { id: toastId });
            // Wait for approval to propagate on-chain
            await new Promise((resolve) => setTimeout(resolve, 2000));

            // Retry the order
            const retryResult = await submitOrder();

            if (retryResult.success) {
              handleOrderSuccess(retryResult);
            } else {
              // Suppress error, show success, log to console
              tradingToast.handleError(retryResult.error, { toastId }, "Sell order retry");
            }
          } else {
            // Suppress error, show success, log to console
            tradingToast.handleApprovalError(approveResult.error, { toastId }, "CTF");
          }
        } catch (approveErr) {
          // Suppress error, show success, log to console
          tradingToast.handleApprovalError(approveErr, { toastId }, "CTF");
        }
      } else {
        // Suppress error, show success, log to console
        tradingToast.handleError(err, { toastId }, "Sell order");
      }
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/70"
        onClick={onClose}
      />

      {/* Modal */}
      <div
        className="relative w-full max-w-md mx-4 rounded-lg overflow-hidden"
        style={{
          backgroundColor: fey.bg300,
          border: `1px solid ${fey.border}`,
        }}
      >
        {/* Header */}
        <div
          className="flex items-center justify-between px-5 py-4"
          style={{ borderBottom: `1px solid ${fey.border}` }}
        >
          <h3 className="text-base font-medium" style={{ color: fey.grey100 }}>
            Sell Position
          </h3>
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-white/10 transition-colors"
          >
            <X size={18} style={{ color: fey.grey500 }} />
          </button>
        </div>

        {/* Content */}
        <div className="p-5 space-y-4">
          {/* Position Info */}
          <div
            className="p-3 rounded-lg"
            style={{ backgroundColor: fey.bg200 }}
          >
            <div
              className="text-sm font-medium mb-2 truncate"
              style={{ color: fey.grey100 }}
            >
              {position.title || "Unknown Market"}
            </div>
            <div className="flex items-center gap-3 text-xs" style={{ color: fey.grey500 }}>
              <span
                className="px-2 py-0.5 rounded"
                style={{
                  backgroundColor:
                    position.outcome === "Yes" ? fey.tealMuted : fey.redMuted,
                  color: position.outcome === "Yes" ? fey.teal : fey.red,
                }}
              >
                {position.outcome}
              </span>
              <span>Available: {maxShares.toFixed(2)} shares</span>
            </div>
          </div>

          {/* Amount Input */}
          <div>
            <label
              className="block text-xs uppercase tracking-wider mb-2"
              style={{ color: fey.grey500 }}
            >
              Amount (shares)
            </label>
            <div className="flex gap-2">
              <input
                type="number"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                placeholder="0.00"
                step="0.01"
                min="0"
                max={shares}
                className="flex-1 px-3 py-2 rounded text-sm font-mono bg-transparent outline-none"
                style={{
                  border: `1px solid ${fey.border}`,
                  color: fey.grey100,
                }}
              />
              <button
                onClick={() => setAmount(maxShares.toString())}
                className="px-3 py-2 rounded text-xs font-medium transition-colors hover:opacity-80"
                style={{
                  backgroundColor: fey.bg400,
                  color: fey.grey300,
                  border: `1px solid ${fey.border}`,
                }}
              >
                Max
              </button>
            </div>
          </div>

          {/* Market Sell Info */}
          <div
            className="p-3 rounded-lg"
            style={{ backgroundColor: fey.bg200 }}
          >
            <div className="flex justify-between text-xs mb-1">
              <span style={{ color: fey.grey500 }}>Order Type</span>
              <span className="font-medium" style={{ color: fey.teal }}>Market Sell</span>
            </div>
            <div className="flex justify-between text-xs">
              <span style={{ color: fey.grey500 }}>Last Price</span>
              <span className="font-mono" style={{ color: fey.grey300 }}>
                {(currentPrice * 100).toFixed(1)}¢
              </span>
            </div>
            <p className="text-xs mt-2" style={{ color: fey.grey500 }}>
              Sells at best available bid price
            </p>
          </div>

          {/* Estimated Return */}
          {parsedAmount > 0 && (
            <div
              className="p-3 rounded-lg"
              style={{ backgroundColor: fey.bg200 }}
            >
              <div className="flex justify-between text-xs" style={{ color: fey.grey500 }}>
                <span>Est. Return (approx)</span>
                <span className="font-mono" style={{ color: fey.grey100 }}>
                  ~${(parsedAmount * currentPrice).toFixed(2)}
                </span>
              </div>
              <p className="text-xs mt-1" style={{ color: fey.grey500 }}>
                Actual amount depends on orderbook
              </p>
            </div>
          )}

          {/* Action Button */}
          {needsApproval ? (
            <button
              onClick={handleApprove}
              disabled={isApproving}
              className="w-full py-3 rounded-lg text-sm font-medium transition-colors disabled:opacity-50"
              style={{
                backgroundColor: fey.skyBlue,
                color: fey.bg100,
              }}
            >
              {isApproving ? "Approving..." : "Approve Tokens for Selling"}
            </button>
          ) : (
            <button
              onClick={handleSubmit}
              disabled={!canSubmit || isSubmitting}
              className="w-full py-3 rounded-lg text-sm font-medium transition-colors disabled:opacity-50"
              style={{
                backgroundColor: canSubmit ? fey.red : fey.bg400,
                color: canSubmit ? fey.grey100 : fey.grey500,
              }}
            >
              {isSubmitting ? "Submitting..." : "Sell"}
            </button>
          )}

          {/* Validation Message */}
          {parsedAmount > maxShares && (
            <p className="text-xs text-center" style={{ color: fey.red }}>
              Amount exceeds available shares
            </p>
          )}
        </div>
      </div>
    </div>
  );
};

// ============================================================================
// Main Portfolio Page
// ============================================================================

const PortfolioPage = () => {
  // Use React Query hooks for automatic caching and updates
  const { positions, isLoading: positionsLoading, isError: positionsError, refetch: refetchPositions } = usePositions();
  const { data: balanceData, isLoading: balanceLoading, refetch: refetchBalance } = useTradingBalance();
  const { reducePosition } = useOptimisticPositionUpdate();

  const [selectedPosition, setSelectedPosition] = useState<Position | null>(null);
  const [showSellModal, setShowSellModal] = useState(false);

  // Convert balance data to match expected interface
  const balance: Balance | null = balanceData ? {
    usdcBalance: balanceData.usdcBalance,
    usdcAllowance: balanceData.usdcAllowance,
    walletAddress: balanceData.walletAddress,
    ctfApproved: balanceData.ctfApproved,
    negRiskCtfApproved: balanceData.negRiskCtfApproved,
    negRiskAdapterApproved: balanceData.negRiskAdapterApproved,
  } : null;

  const isLoading = positionsLoading || balanceLoading;
  const error = positionsError ? "Failed to load portfolio data" : null;

  const handleSellClick = (position: Position) => {
    setSelectedPosition(position);
    setShowSellModal(true);
  };

  const handleModalClose = () => {
    setShowSellModal(false);
    setSelectedPosition(null);
  };

  const handleSellSuccess = () => {
    // Refetch data to update positions and balance
    refetchPositions();
    refetchBalance();
  };

  // Calculate totals
  const totalValue = positions.reduce((sum, p) => {
    const shares = parseFloat(p.shares);
    const price = parseFloat(p.currentPrice);
    return sum + shares * price;
  }, 0);

  const totalPnl = positions.reduce((sum, p) => sum + parseFloat(p.pnl), 0);

  return (
    <div
      className="h-screen flex flex-col overflow-hidden"
      style={{ backgroundColor: fey.bg100 }}
    >
      {/* Header */}
      <Navbar />

      {/* Main content */}
      <main className="flex-1 overflow-y-auto">
        <div className="mx-auto px-8 pt-8 pb-6" style={{ maxWidth: "1400px" }}>
          {/* Page Title */}
          <div className="mb-8">
            <h1
              className="text-2xl font-semibold mb-2"
              style={{ color: fey.grey100 }}
            >
              Portfolio
            </h1>
            {balance && (
              <p className="text-sm font-mono" style={{ color: fey.grey500 }}>
                {balance.walletAddress.slice(0, 6)}...
                {balance.walletAddress.slice(-4)}
              </p>
            )}
          </div>

          {/* Summary Cards */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
            {/* USDC Balance */}
            <div
              className="p-5 rounded-lg"
              style={{
                backgroundColor: fey.bg300,
                border: `1px solid ${fey.border}`,
              }}
            >
              <div
                className="text-xs uppercase tracking-wider mb-1"
                style={{ color: fey.grey500 }}
              >
                Available USDC
              </div>
              <div
                className="text-2xl font-semibold font-mono"
                style={{ color: fey.grey100 }}
              >
                ${balance ? parseFloat(balance.usdcBalance).toFixed(2) : "..."}
              </div>
            </div>

            {/* Positions Value */}
            <div
              className="p-5 rounded-lg"
              style={{
                backgroundColor: fey.bg300,
                border: `1px solid ${fey.border}`,
              }}
            >
              <div
                className="text-xs uppercase tracking-wider mb-1"
                style={{ color: fey.grey500 }}
              >
                Positions Value
              </div>
              <div
                className="text-2xl font-semibold font-mono"
                style={{ color: fey.grey100 }}
              >
                ${totalValue.toFixed(2)}
              </div>
            </div>

            {/* Total P&L */}
            <div
              className="p-5 rounded-lg"
              style={{
                backgroundColor: fey.bg300,
                border: `1px solid ${fey.border}`,
              }}
            >
              <div
                className="text-xs uppercase tracking-wider mb-1"
                style={{ color: fey.grey500 }}
              >
                Total P&L
              </div>
              <div
                className="text-2xl font-semibold font-mono"
                style={{ color: totalPnl >= 0 ? fey.teal : fey.red }}
              >
                {totalPnl >= 0 ? "+" : ""}${totalPnl.toFixed(2)}
              </div>
            </div>
          </div>

          {/* Positions Table */}
          <div
            className="rounded-lg overflow-hidden"
            style={{
              backgroundColor: fey.bg200,
              border: `1px solid ${fey.border}`,
            }}
          >
            <div
              className="px-5 py-4"
              style={{ borderBottom: `1px solid ${fey.border}` }}
            >
              <h2
                className="text-sm font-medium"
                style={{ color: fey.grey100 }}
              >
                Active Positions ({positions.length})
              </h2>
            </div>

            {isLoading ? (
              <div className="p-8 text-center" style={{ color: fey.grey500 }}>
                Loading positions...
              </div>
            ) : error ? (
              <div className="p-8 text-center" style={{ color: fey.red }}>
                {error}
              </div>
            ) : positions.length === 0 ? (
              <div className="p-8 text-center" style={{ color: fey.grey500 }}>
                No active positions
              </div>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr style={{ borderBottom: `1px solid ${fey.border}` }}>
                      <th
                        className="px-5 py-3 text-left text-xs uppercase tracking-wider font-medium"
                        style={{ color: fey.grey500 }}
                      >
                        Market
                      </th>
                      <th
                        className="px-5 py-3 text-right text-xs uppercase tracking-wider font-medium"
                        style={{ color: fey.grey500 }}
                      >
                        Side
                      </th>
                      <th
                        className="px-5 py-3 text-right text-xs uppercase tracking-wider font-medium"
                        style={{ color: fey.grey500 }}
                      >
                        Shares
                      </th>
                      <th
                        className="px-5 py-3 text-right text-xs uppercase tracking-wider font-medium"
                        style={{ color: fey.grey500 }}
                      >
                        Avg Price
                      </th>
                      <th
                        className="px-5 py-3 text-right text-xs uppercase tracking-wider font-medium"
                        style={{ color: fey.grey500 }}
                      >
                        Current
                      </th>
                      <th
                        className="px-5 py-3 text-right text-xs uppercase tracking-wider font-medium"
                        style={{ color: fey.grey500 }}
                      >
                        Value
                      </th>
                      <th
                        className="px-5 py-3 text-right text-xs uppercase tracking-wider font-medium"
                        style={{ color: fey.grey500 }}
                      >
                        P&L
                      </th>
                      <th
                        className="px-5 py-3 text-center text-xs uppercase tracking-wider font-medium"
                        style={{ color: fey.grey500 }}
                      >
                        Action
                      </th>
                    </tr>
                  </thead>
                  <tbody>
                    {positions.map((position, index) => {
                      const shares = parseFloat(position.shares);
                      const avgPrice = parseFloat(position.avgPrice);
                      const currentPrice = parseFloat(position.currentPrice);
                      const value = shares * currentPrice;
                      const pnl = parseFloat(position.pnl);

                      return (
                        <tr
                          key={position.tokenId}
                          className="hover:opacity-80 transition-opacity"
                          style={{
                            borderBottom:
                              index < positions.length - 1
                                ? `1px solid ${fey.border}`
                                : undefined,
                          }}
                        >
                          <td className="px-5 py-4">
                            <Link
                              href={`/market/polymarket/${position.marketId}`}
                              className="block"
                            >
                              <div
                                className="text-sm font-medium max-w-md truncate hover:underline"
                                style={{ color: fey.grey100 }}
                              >
                                {position.title || "Unknown Market"}
                              </div>
                            </Link>
                          </td>
                          <td className="px-5 py-4 text-right">
                            <span
                              className="text-xs font-medium px-2 py-1 rounded"
                              style={{
                                backgroundColor:
                                  position.outcome === "Yes"
                                    ? fey.tealMuted
                                    : fey.redMuted,
                                color:
                                  position.outcome === "Yes"
                                    ? fey.teal
                                    : fey.red,
                              }}
                            >
                              {position.outcome}
                            </span>
                          </td>
                          <td
                            className="px-5 py-4 text-right font-mono text-sm"
                            style={{ color: fey.grey300 }}
                          >
                            {shares.toFixed(2)}
                          </td>
                          <td
                            className="px-5 py-4 text-right font-mono text-sm"
                            style={{ color: fey.grey300 }}
                          >
                            {(avgPrice * 100).toFixed(1)}¢
                          </td>
                          <td
                            className="px-5 py-4 text-right font-mono text-sm"
                            style={{ color: fey.grey300 }}
                          >
                            {(currentPrice * 100).toFixed(1)}¢
                          </td>
                          <td
                            className="px-5 py-4 text-right font-mono text-sm"
                            style={{ color: fey.grey100 }}
                          >
                            ${value.toFixed(2)}
                          </td>
                          <td
                            className="px-5 py-4 text-right font-mono text-sm"
                            style={{ color: pnl >= 0 ? fey.teal : fey.red }}
                          >
                            {pnl >= 0 ? "+" : ""}${pnl.toFixed(2)}
                          </td>
                          <td className="px-5 py-4 text-center">
                            <button
                              onClick={() => handleSellClick(position)}
                              className="px-3 py-1.5 text-xs font-medium rounded transition-colors hover:opacity-80"
                              style={{
                                backgroundColor: fey.redMuted,
                                color: fey.red,
                              }}
                            >
                              Sell
                            </button>
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        </div>
      </main>

      {/* Sell Modal */}
      {showSellModal && selectedPosition && (
        <SellModal
          position={selectedPosition}
          balance={balance}
          onClose={handleModalClose}
          onSuccess={handleSellSuccess}
          onOptimisticSell={reducePosition}
        />
      )}
    </div>
  );
};

export default PortfolioPage;
