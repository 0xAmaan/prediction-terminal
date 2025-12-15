"use client";

import { useState, useMemo, useCallback } from "react";
import { toast } from "sonner";
import type { Trade } from "@/lib/types";
import { api } from "@/lib/api";
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
  border: "rgba(255, 255, 255, 0.06)",
  yellow: "#F5A623",
};

type Side = "buy" | "sell";
type OrderType = "market" | "limit";

interface TradeExecutionProps {
  yesPrice: string;
  noPrice: string;
  trades?: Trade[];
  className?: string;
  /** CLOB token ID for order submission - required for trading */
  tokenId?: string;
  /** Market title for confirmation modal */
  marketTitle?: string;
  /** Callback after successful order */
  onOrderSubmitted?: () => void;
}

const formatVolume = (value: number): string => {
  if (value >= 1000) return `$${(value / 1000).toFixed(2)}K`;
  return `$${value.toFixed(2)}`;
};

export const TradeExecution = ({
  yesPrice,
  noPrice,
  trades = [],
  className = "",
  tokenId,
  marketTitle,
  onOrderSubmitted,
}: TradeExecutionProps) => {
  const [side, setSide] = useState<Side>("buy");
  const [orderType, setOrderType] = useState<OrderType>("limit");
  const [amount, setAmount] = useState("");
  const [limitPrice, setLimitPrice] = useState("");

  // Trading state
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [submitSuccess, setSubmitSuccess] = useState<string | null>(null);
  const [showConfirmModal, setShowConfirmModal] = useState(false);
  const [isApproving, setIsApproving] = useState(false);

  // Approval state
  const [approvalError, setApprovalError] = useState<string | null>(null);
  const [approvalSuccess, setApprovalSuccess] = useState<string | null>(null);

  // Fetch trading balance (only when trading is enabled)
  const {
    balance: usdcBalance,
    hasBalance,
    hasAllowance,
    needsApproval,
    isLoading: balanceLoading,
    walletAddress,
    refetch: refetchBalance,
  } = useTradingBalance(!!tokenId);

  const isBuy = side === "buy";
  const accentColor = isBuy ? fey.teal : fey.red;

  const quickAmounts = ["0.01", "0.1", "1", "10"];

  const currentPrice = parseFloat(yesPrice) || 0;
  const parsedAmount = parseFloat(amount) || 0;
  const parsedLimitPrice = limitPrice ? parseFloat(limitPrice) / 100 : currentPrice;

  // For market orders, use current price; for limit orders, use the limit price
  const orderPrice = orderType === "market" ? currentPrice : parsedLimitPrice;
  const estimatedCost = parsedAmount * orderPrice;

  // Validation
  const isValidAmount = parsedAmount > 0;
  // For market orders, we trust the market price; for limit orders, enforce 1-99 cents range
  const isValidPrice = orderType === "market"
    ? currentPrice > 0 && currentPrice <= 1
    : orderPrice >= 0.01 && orderPrice <= 0.99;
  const hasSufficientBalance = isBuy ? usdcBalance >= estimatedCost : true; // Selling doesn't require USDC
  const canTrade =
    tokenId &&
    isValidAmount &&
    isValidPrice &&
    !isSubmitting &&
    hasAllowance &&
    (isBuy ? hasSufficientBalance : true);

  // Calculate 5-minute volume stats from trades
  const volumeStats = useMemo(() => {
    const fiveMinutesAgo = Date.now() - 5 * 60 * 1000;
    const recentTrades = trades.filter(
      (t) => new Date(t.timestamp).getTime() > fiveMinutesAgo
    );

    let buyCount = 0;
    let sellCount = 0;
    let buyVolume = 0;
    let sellVolume = 0;

    for (const trade of recentTrades) {
      const isBuyTrade =
        trade.side?.toLowerCase() === "buy" ||
        (!trade.side && trade.outcome?.toLowerCase() === "yes");
      const value = parseFloat(trade.price) * parseFloat(trade.quantity);

      if (isBuyTrade) {
        buyCount++;
        buyVolume += value;
      } else {
        sellCount++;
        sellVolume += value;
      }
    }

    return {
      totalVolume: buyVolume + sellVolume,
      buyCount,
      sellCount,
      buyVolume,
      sellVolume,
      netVolume: buyVolume - sellVolume,
    };
  }, [trades]);

  const handleSubmitClick = useCallback(() => {
    setSubmitError(null);
    setSubmitSuccess(null);
    setShowConfirmModal(true);
  }, []);

  const handleConfirmOrder = useCallback(async () => {
    if (!tokenId || !canTrade) return;

    setShowConfirmModal(false);
    setIsSubmitting(true);
    setSubmitError(null);
    setSubmitSuccess(null);

    const toastId = toast.loading("Submitting order...");

    try {
      const result = await api.submitOrder({
        tokenId,
        side,
        price: orderPrice,
        size: parsedAmount,
        orderType: "GTC", // Good-til-cancelled
      });

      if (result.success) {
        toast.success(`Order placed! ID: ${result.orderId?.slice(0, 8)}...`, { id: toastId });
        setAmount("");
        setLimitPrice("");
        onOrderSubmitted?.();
        // Refetch balance after successful order
        refetchBalance();
      } else {
        toast.error(result.error || "Order failed", { id: toastId });
        setSubmitError(result.error || "Order failed");
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : "Order submission failed";
      toast.error(errorMsg, { id: toastId });
      setSubmitError(errorMsg);
    } finally {
      setIsSubmitting(false);
    }
  }, [tokenId, canTrade, side, orderPrice, parsedAmount, onOrderSubmitted, refetchBalance]);

  const handleCancelConfirm = useCallback(() => {
    setShowConfirmModal(false);
  }, []);

  const handleApproveUsdc = useCallback(async () => {
    setIsApproving(true);
    setApprovalError(null);
    setApprovalSuccess(null);

    const toastId = toast.loading("Approving USDC...");

    try {
      const result = await api.approveUsdc();

      if (result.success) {
        toast.success(`Approved! Tx: ${result.transactionHash?.slice(0, 10)}...`, { id: toastId });
        setApprovalSuccess(`Approved! Tx: ${result.transactionHash?.slice(0, 10)}...`);
        // Refetch balance to update allowance state
        setTimeout(() => refetchBalance(), 2000); // Wait for chain confirmation
      } else {
        toast.error(result.error || "Approval failed", { id: toastId });
        setApprovalError(result.error || "Approval failed");
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : "Approval failed";
      toast.error(errorMsg, { id: toastId });
      setApprovalError(errorMsg);
    } finally {
      setIsApproving(false);
    }
  }, [refetchBalance]);

  return (
    <div
      className={`rounded-lg overflow-hidden flex flex-col h-full relative ${className}`}
      style={{
        backgroundColor: fey.bg300,
        border: `1px solid ${fey.border}`,
      }}
    >
      {/* Confirmation Modal */}
      {showConfirmModal && (
        <div
          className="absolute inset-0 z-50 flex items-center justify-center"
          style={{ backgroundColor: "rgba(0, 0, 0, 0.8)" }}
        >
          <div
            className="mx-4 p-4 rounded-lg max-w-sm w-full"
            style={{ backgroundColor: fey.bg300, border: `1px solid ${fey.border}` }}
          >
            <h3 className="text-lg font-semibold mb-3" style={{ color: fey.grey100 }}>
              Confirm Order
            </h3>

            {/* Warning */}
            <div
              className="p-3 rounded-md mb-4 flex items-start gap-2"
              style={{ backgroundColor: "rgba(245, 166, 35, 0.1)", border: `1px solid ${fey.yellow}` }}
            >
              <span style={{ color: fey.yellow }}>⚠️</span>
              <p className="text-xs" style={{ color: fey.yellow }}>
                This is a real transaction using real money. Please review carefully.
              </p>
            </div>

            {/* Order Details */}
            <div className="space-y-2 mb-4">
              {marketTitle && (
                <div className="flex justify-between text-sm">
                  <span style={{ color: fey.grey500 }}>Market</span>
                  <span style={{ color: fey.grey100 }} className="text-right max-w-[200px] truncate">
                    {marketTitle}
                  </span>
                </div>
              )}
              <div className="flex justify-between text-sm">
                <span style={{ color: fey.grey500 }}>Side</span>
                <span style={{ color: isBuy ? fey.teal : fey.red }} className="font-semibold">
                  {side.toUpperCase()} YES
                </span>
              </div>
              <div className="flex justify-between text-sm">
                <span style={{ color: fey.grey500 }}>Amount</span>
                <span style={{ color: fey.grey100 }}>{parsedAmount} shares</span>
              </div>
              <div className="flex justify-between text-sm">
                <span style={{ color: fey.grey500 }}>Price</span>
                <span style={{ color: fey.grey100 }}>{(orderPrice * 100).toFixed(1)}¢</span>
              </div>
              <div
                className="flex justify-between text-sm pt-2 mt-2"
                style={{ borderTop: `1px solid ${fey.border}` }}
              >
                <span style={{ color: fey.grey500 }}>Est. {isBuy ? "Cost" : "Return"}</span>
                <span style={{ color: fey.grey100 }} className="font-semibold">
                  ${estimatedCost.toFixed(2)}
                </span>
              </div>
            </div>

            {/* Buttons */}
            <div className="flex gap-2">
              <button
                onClick={handleCancelConfirm}
                className="flex-1 py-2.5 rounded-md text-sm font-medium transition-colors"
                style={{
                  backgroundColor: fey.bg400,
                  color: fey.grey300,
                  border: `1px solid ${fey.border}`,
                }}
              >
                Cancel
              </button>
              <button
                onClick={handleConfirmOrder}
                className="flex-1 py-2.5 rounded-md text-sm font-semibold transition-colors"
                style={{
                  backgroundColor: accentColor,
                  color: fey.bg100,
                }}
              >
                Confirm {side.toUpperCase()}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 5-Minute Volume Summary */}
      <div
        className="px-3 py-2.5 grid grid-cols-4 gap-2 text-center"
        style={{ backgroundColor: fey.bg400, borderBottom: `1px solid ${fey.border}` }}
      >
        <div>
          <div className="text-[10px] uppercase tracking-wider" style={{ color: fey.grey500 }}>
            5m Vol
          </div>
          <div className="text-sm font-mono font-semibold" style={{ color: fey.grey100 }}>
            {formatVolume(volumeStats.totalVolume)}
          </div>
        </div>
        <div>
          <div className="text-[10px] uppercase tracking-wider" style={{ color: fey.grey500 }}>
            Buys
          </div>
          <div className="text-sm font-mono font-semibold" style={{ color: fey.teal }}>
            {volumeStats.buyCount} / {formatVolume(volumeStats.buyVolume)}
          </div>
        </div>
        <div>
          <div className="text-[10px] uppercase tracking-wider" style={{ color: fey.grey500 }}>
            Sells
          </div>
          <div className="text-sm font-mono font-semibold" style={{ color: fey.red }}>
            {volumeStats.sellCount} / {formatVolume(volumeStats.sellVolume)}
          </div>
        </div>
        <div>
          <div className="text-[10px] uppercase tracking-wider" style={{ color: fey.grey500 }}>
            Net Vol.
          </div>
          <div
            className="text-sm font-mono font-semibold"
            style={{ color: volumeStats.netVolume >= 0 ? fey.teal : fey.red }}
          >
            {volumeStats.netVolume >= 0 ? "+" : ""}
            {formatVolume(Math.abs(volumeStats.netVolume))}
          </div>
        </div>
      </div>

      {/* Buy/Sell Progress Bar */}
      {volumeStats.totalVolume > 0 && (
        <div className="h-1 flex">
          <div
            style={{
              width: `${(volumeStats.buyVolume / volumeStats.totalVolume) * 100}%`,
              backgroundColor: fey.teal,
            }}
          />
          <div
            style={{
              width: `${(volumeStats.sellVolume / volumeStats.totalVolume) * 100}%`,
              backgroundColor: fey.red,
            }}
          />
        </div>
      )}

      {/* Buy/Sell Toggle */}
      <div
        className="grid grid-cols-2 p-1 gap-1"
        style={{ backgroundColor: fey.bg200 }}
      >
        <button
          onClick={() => setSide("buy")}
          className="py-2.5 rounded-md text-base font-semibold transition-all"
          style={{
            backgroundColor: side === "buy" ? fey.teal : "transparent",
            color: side === "buy" ? fey.bg100 : fey.grey500,
          }}
        >
          Buy
        </button>
        <button
          onClick={() => setSide("sell")}
          className="py-2.5 rounded-md text-base font-semibold transition-all"
          style={{
            backgroundColor: side === "sell" ? fey.red : "transparent",
            color: side === "sell" ? fey.bg100 : fey.grey500,
          }}
        >
          Sell
        </button>
      </div>

      {/* Order Type Toggle */}
      <div className="px-4 pt-4">
        <div
          className="flex gap-1 p-1 rounded-md"
          style={{ backgroundColor: fey.bg400 }}
        >
          <button
            onClick={() => setOrderType("market")}
            className="flex-1 py-1.5 rounded text-sm font-medium transition-all"
            style={{
              backgroundColor: orderType === "market" ? fey.bg300 : "transparent",
              color: orderType === "market" ? fey.grey100 : fey.grey500,
            }}
          >
            Market
          </button>
          <button
            onClick={() => setOrderType("limit")}
            className="flex-1 py-1.5 rounded text-sm font-medium transition-all"
            style={{
              backgroundColor: orderType === "limit" ? fey.bg300 : "transparent",
              color: orderType === "limit" ? fey.grey100 : fey.grey500,
            }}
          >
            Limit
          </button>
          <button
            disabled
            className="flex-1 py-1.5 rounded text-sm font-medium opacity-50 cursor-not-allowed"
            style={{ color: fey.grey500 }}
          >
            Adv.
          </button>
        </div>
      </div>

      {/* Balance Display */}
      {tokenId && (
        <div
          className="mx-4 mt-3 p-2.5 rounded-md"
          style={{ backgroundColor: fey.bg400 }}
        >
          <div className="flex items-center justify-between">
            <span className="text-xs uppercase tracking-wider" style={{ color: fey.grey500 }}>
              Available
            </span>
            <span className="text-sm font-mono font-medium" style={{ color: fey.grey100 }}>
              {balanceLoading ? "..." : `$${usdcBalance.toFixed(2)}`}
            </span>
          </div>
          {walletAddress && (
            <div className="mt-1 text-xs font-mono truncate" style={{ color: fey.grey500 }}>
              {walletAddress.slice(0, 6)}...{walletAddress.slice(-4)}
            </div>
          )}
          {needsApproval && (
            <div className="mt-2 space-y-2">
              <button
                onClick={handleApproveUsdc}
                disabled={isApproving}
                className="w-full py-2 rounded text-sm font-medium transition-colors"
                style={{
                  backgroundColor: isApproving ? fey.bg400 : fey.yellow,
                  color: isApproving ? fey.grey500 : fey.bg100,
                  opacity: isApproving ? 0.7 : 1,
                }}
              >
                {isApproving ? (
                  <span className="flex items-center justify-center gap-2">
                    <span className="animate-spin">⏳</span>
                    Approving...
                  </span>
                ) : (
                  "Approve USDC for Trading"
                )}
              </button>
              <p className="text-xs text-center" style={{ color: fey.grey500 }}>
                One-time approval to let Polymarket spend your USDC
              </p>
            </div>
          )}
          {approvalError && (
            <div
              className="mt-2 p-2 rounded text-xs"
              style={{ backgroundColor: fey.redMuted, color: fey.red }}
            >
              {approvalError}
            </div>
          )}
          {approvalSuccess && (
            <div
              className="mt-2 p-2 rounded text-xs"
              style={{ backgroundColor: fey.tealMuted, color: fey.teal }}
            >
              {approvalSuccess}
            </div>
          )}
        </div>
      )}

      {/* Amount Input */}
      <div className="px-4 pt-4 flex-1 overflow-y-auto min-h-0">
        <div className="flex items-center justify-between mb-2">
          <label className="text-sm font-medium" style={{ color: fey.grey500 }}>
            SHARES
          </label>
        </div>
        <input
          type="text"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          placeholder="0.00"
          disabled={isSubmitting}
          className="w-full px-3 py-3 rounded-md text-right text-xl font-mono outline-none transition-colors"
          style={{
            backgroundColor: fey.bg400,
            border: `1px solid ${fey.border}`,
            color: fey.grey100,
            opacity: isSubmitting ? 0.5 : 1,
          }}
        />

        {/* Quick Amount Buttons */}
        <div className="grid grid-cols-4 gap-2 mt-2">
          {quickAmounts.map((amt) => (
            <button
              key={amt}
              onClick={() => setAmount(amt)}
              disabled={isSubmitting}
              className="py-1.5 rounded text-sm font-medium transition-colors hover:opacity-80"
              style={{
                backgroundColor: fey.bg400,
                color: fey.grey300,
                border: `1px solid ${fey.border}`,
                opacity: isSubmitting ? 0.5 : 1,
              }}
            >
              {amt}
            </button>
          ))}
        </div>

        {/* Limit Price Input (only for limit orders) */}
        {orderType === "limit" && (
          <div className="mt-4">
            <div className="flex items-center justify-between mb-2">
              <label className="text-sm font-medium" style={{ color: fey.grey500 }}>
                LIMIT PRICE
              </label>
            </div>
            <input
              type="text"
              value={limitPrice}
              onChange={(e) => setLimitPrice(e.target.value)}
              placeholder={(currentPrice * 100).toFixed(1)}
              disabled={isSubmitting}
              className="w-full px-3 py-3 rounded-md text-right text-xl font-mono outline-none"
              style={{
                backgroundColor: fey.bg400,
                border: `1px solid ${fey.border}`,
                color: fey.grey100,
                opacity: isSubmitting ? 0.5 : 1,
              }}
            />
            <span className="text-xs mt-1 block text-right" style={{ color: fey.grey500 }}>
              cents (1-99)
            </span>
          </div>
        )}

        {/* Estimated Cost */}
        {parsedAmount > 0 && (
          <div
            className="mt-4 p-3 rounded-md"
            style={{ backgroundColor: fey.bg400 }}
          >
            <div className="flex items-center justify-between">
              <span className="text-sm" style={{ color: fey.grey500 }}>
                Est. {isBuy ? "Cost" : "Return"}
              </span>
              <span className="text-base font-mono font-medium" style={{ color: fey.grey100 }}>
                ${estimatedCost.toFixed(2)}
              </span>
            </div>
          </div>
        )}

        {/* Error Message */}
        {submitError && (
          <div
            className="mt-3 p-2 rounded-md text-sm"
            style={{ backgroundColor: fey.redMuted, color: fey.red }}
          >
            {submitError}
          </div>
        )}

        {/* Success Message */}
        {submitSuccess && (
          <div
            className="mt-3 p-2 rounded-md text-sm"
            style={{ backgroundColor: fey.tealMuted, color: fey.teal }}
          >
            {submitSuccess}
          </div>
        )}
      </div>

      {/* Submit Button */}
      <div className="p-4 flex-shrink-0">
        <button
          onClick={handleSubmitClick}
          disabled={!canTrade}
          className={`w-full py-3.5 rounded-lg text-base font-semibold transition-all ${
            canTrade ? "cursor-pointer hover:opacity-90" : "cursor-not-allowed opacity-60"
          }`}
          style={{
            backgroundColor: accentColor,
            color: fey.bg100,
          }}
        >
          {isSubmitting ? (
            <span className="flex items-center justify-center gap-2">
              <span className="animate-spin">⏳</span>
              Submitting...
            </span>
          ) : (
            `${isBuy ? "Buy" : "Sell"} YES`
          )}
        </button>
        {!tokenId && (
          <p className="text-xs text-center mt-2" style={{ color: fey.grey500 }}>
            Trading not available for this market
          </p>
        )}
        {tokenId && hasAllowance && !hasSufficientBalance && isBuy && parsedAmount > 0 && (
          <p className="text-xs text-center mt-2" style={{ color: fey.red }}>
            Insufficient balance (need ${estimatedCost.toFixed(2)})
          </p>
        )}
        {tokenId && !isValidPrice && limitPrice && (
          <p className="text-xs text-center mt-2" style={{ color: fey.red }}>
            Price must be between 1¢ and 99¢
          </p>
        )}
      </div>
    </div>
  );
};

export default TradeExecution;
