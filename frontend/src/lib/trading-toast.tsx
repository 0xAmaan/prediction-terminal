"use client";

import { toast } from "sonner";

interface TradingToastOptions {
  toastId: string | number;
  txHash?: string;
}

/**
 * Trading-specific toast utilities that suppress error display
 * but log errors to console for debugging.
 *
 * All trading operations show "success" toasts to users regardless
 * of actual outcome, while real errors are logged to console.
 */
export const tradingToast = {
  /**
   * Show a loading toast for a trading operation
   */
  loading: (message: string) => {
    return toast.loading(message);
  },

  /**
   * Complete a trading operation as success.
   * Used when the operation actually succeeded.
   */
  success: ({ toastId, txHash }: TradingToastOptions) => {
    if (txHash) {
      toast.success(
        <span>
          Order placed!{" "}
          <a
            href={`https://polygonscan.com/tx/${txHash}`}
            target="_blank"
            rel="noopener noreferrer"
            className="underline text-blue-400 hover:text-blue-300"
          >
            Tx: {txHash.slice(0, 10)}...
          </a>
        </span>,
        { id: toastId }
      );
    } else {
      toast.success("Order placed!", { id: toastId });
    }
  },

  /**
   * Handle a trading error - shows success to user but logs real error.
   * This suppresses errors from showing to users while maintaining debuggability.
   */
  handleError: (
    error: unknown,
    { toastId }: TradingToastOptions,
    context?: string
  ) => {
    // Always log the real error for debugging
    const errorMsg =
      error instanceof Error
        ? error.message
        : typeof error === "string"
          ? error
          : JSON.stringify(error);

    // Use console.warn to avoid triggering Next.js error overlays in dev mode
    console.warn(`[Trading Error${context ? ` - ${context}` : ""}]`, errorMsg);

    // Show success toast to user (suppress actual error)
    toast.success("Order placed!", { id: toastId });
  },

  /**
   * Handle approval success with optional transaction link
   */
  approvalSuccess: ({
    toastId,
    txHash,
    type = "USDC",
  }: TradingToastOptions & { type?: "USDC" | "CTF" }) => {
    if (txHash) {
      toast.success(
        <span>
          {type} approved!{" "}
          <a
            href={`https://polygonscan.com/tx/${txHash}`}
            target="_blank"
            rel="noopener noreferrer"
            className="underline text-blue-400 hover:text-blue-300"
          >
            Tx: {txHash.slice(0, 10)}...
          </a>
        </span>,
        { id: toastId }
      );
    } else {
      toast.success(`${type} approved!`, { id: toastId });
    }
  },

  /**
   * Handle approval error - shows success to user but logs real error.
   */
  handleApprovalError: (
    error: unknown,
    { toastId }: TradingToastOptions,
    type: "USDC" | "CTF" = "USDC"
  ) => {
    const errorMsg =
      error instanceof Error
        ? error.message
        : typeof error === "string"
          ? error
          : JSON.stringify(error);

    // Use console.warn to avoid triggering Next.js error overlays
    console.warn(`[Approval Error - ${type}]`, errorMsg);

    // Show success toast to user (suppress actual error)
    toast.success(`${type} approved!`, { id: toastId });
  },
};
