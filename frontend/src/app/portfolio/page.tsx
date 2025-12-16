"use client";

import { useState, useEffect, useCallback } from "react";
import Link from "next/link";
import { X } from "lucide-react";
import { toast } from "sonner";
import { Navbar } from "@/components/layout/navbar";
import { api } from "@/lib/api";

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
}

const SellModal = ({ position, balance, onClose, onSuccess }: SellModalProps) => {
  const [amount, setAmount] = useState("");
  const [price, setPrice] = useState(
    (parseFloat(position.currentPrice) * 100).toFixed(0)
  );
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isApproving, setIsApproving] = useState(false);

  const shares = parseFloat(position.shares);
  const parsedAmount = parseFloat(amount) || 0;
  const parsedPrice = parseFloat(price) / 100; // Convert cents to decimal

  // Check if CTF approval is needed
  const needsApproval = position.negRisk
    ? !(balance?.negRiskCtfApproved && balance?.negRiskAdapterApproved)
    : !balance?.ctfApproved;

  const canSubmit =
    parsedAmount > 0 &&
    parsedAmount <= shares &&
    parsedPrice > 0 &&
    parsedPrice < 1 &&
    !needsApproval;

  const handleApprove = async () => {
    setIsApproving(true);
    const toastId = toast.loading("Approving tokens for selling...");

    try {
      const result = await api.approveCtf();
      if (result.success) {
        toast.success(
          result.transactionHash ? (
            <span>
              Approved!{" "}
              <a
                href={`https://polygonscan.com/tx/${result.transactionHash}`}
                target="_blank"
                rel="noopener noreferrer"
                className="underline text-blue-400"
              >
                Tx: {result.transactionHash.slice(0, 10)}...
              </a>
            </span>
          ) : (
            "Tokens approved for selling!"
          ),
          { id: toastId }
        );
        // Trigger parent refetch to update approval status
        onSuccess();
      } else {
        toast.error(result.error || "Approval failed", { id: toastId });
      }
    } catch (err) {
      toast.error(
        err instanceof Error ? err.message : "Approval failed",
        { id: toastId }
      );
    } finally {
      setIsApproving(false);
    }
  };

  const handleSubmit = async () => {
    if (!canSubmit) return;

    setIsSubmitting(true);
    const toastId = toast.loading("Submitting sell order...");

    const submitOrder = async () => {
      return api.submitOrder({
        tokenId: position.tokenId,
        side: "sell",
        price: parsedPrice,
        size: parsedAmount,
        orderType: "GTC",
        negRisk: position.negRisk,
      });
    };

    const handleSuccess = (result: { transactionHashes?: string[] }) => {
      const txHash = result.transactionHashes?.[0];
      toast.success(
        txHash ? (
          <span>
            Sell order placed!{" "}
            <a
              href={`https://polygonscan.com/tx/${txHash}`}
              target="_blank"
              rel="noopener noreferrer"
              className="underline text-blue-400 hover:text-blue-300"
            >
              Tx: {txHash.slice(0, 10)}...
            </a>
          </span>
        ) : (
          "Sell order placed!"
        ),
        { id: toastId }
      );
      onSuccess();
      onClose();
    };

    try {
      const result = await submitOrder();

      if (result.success) {
        handleSuccess(result);
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
              handleSuccess(retryResult);
            } else {
              toast.error(retryResult.error || "Order failed after approval", {
                id: toastId,
              });
            }
          } else {
            toast.error(approveResult.error || "Approval failed", {
              id: toastId,
            });
          }
        } else {
          toast.error(result.error || "Order failed", { id: toastId });
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
              handleSuccess(retryResult);
            } else {
              toast.error(retryResult.error || "Order failed after approval", {
                id: toastId,
              });
            }
          } else {
            toast.error(approveResult.error || "Approval failed", {
              id: toastId,
            });
          }
        } catch (approveErr) {
          toast.error(
            approveErr instanceof Error ? approveErr.message : "Approval failed",
            { id: toastId }
          );
        }
      } else {
        toast.error(errorMsg, { id: toastId });
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
              <span>Available: {shares.toFixed(2)} shares</span>
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
                onClick={() => setAmount(shares.toString())}
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

          {/* Price Input */}
          <div>
            <label
              className="block text-xs uppercase tracking-wider mb-2"
              style={{ color: fey.grey500 }}
            >
              Price (cents)
            </label>
            <input
              type="number"
              value={price}
              onChange={(e) => setPrice(e.target.value)}
              placeholder="50"
              step="1"
              min="1"
              max="99"
              className="w-full px-3 py-2 rounded text-sm font-mono bg-transparent outline-none"
              style={{
                border: `1px solid ${fey.border}`,
                color: fey.grey100,
              }}
            />
            <p className="text-xs mt-1" style={{ color: fey.grey500 }}>
              Current: {(parseFloat(position.currentPrice) * 100).toFixed(1)}¢
            </p>
          </div>

          {/* Estimated Return */}
          {parsedAmount > 0 && parsedPrice > 0 && (
            <div
              className="p-3 rounded-lg"
              style={{ backgroundColor: fey.bg200 }}
            >
              <div className="flex justify-between text-xs" style={{ color: fey.grey500 }}>
                <span>Est. Return</span>
                <span className="font-mono" style={{ color: fey.grey100 }}>
                  ${(parsedAmount * parsedPrice).toFixed(2)}
                </span>
              </div>
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
          {parsedAmount > shares && (
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
  const [positions, setPositions] = useState<Position[]>([]);
  const [balance, setBalance] = useState<Balance | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedPosition, setSelectedPosition] = useState<Position | null>(null);
  const [showSellModal, setShowSellModal] = useState(false);

  const fetchData = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const [positionsData, balanceData] = await Promise.all([
        api.getPositions(),
        api.getTradingBalance(),
      ]);
      setPositions(positionsData);
      setBalance(balanceData);
    } catch (err) {
      console.error("Failed to fetch portfolio data:", err);
      setError("Failed to load portfolio data");
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

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
    fetchData();
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
        />
      )}
    </div>
  );
};

export default PortfolioPage;
