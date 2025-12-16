"use client";

import { useMemo, useState, useEffect } from "react";
import { ExternalLink } from "lucide-react";
import type { Trade } from "@/lib/types";

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
  tealMuted: "rgba(77, 190, 149, 0.08)",
  red: "#D84F68",
  redMuted: "rgba(216, 79, 104, 0.08)",
  border: "rgba(255, 255, 255, 0.06)",
};

interface LiveTradesTableProps {
  trades: Trade[];
  className?: string;
}

const formatTimeAgo = (timestamp: string): string => {
  const now = new Date();
  const tradeTime = new Date(timestamp);
  const diffMs = now.getTime() - tradeTime.getTime();
  const diffSeconds = Math.floor(diffMs / 1000);

  if (diffSeconds < 60) return `${diffSeconds}s`;
  if (diffSeconds < 3600) return `${Math.floor(diffSeconds / 60)}m`;
  if (diffSeconds < 86400) return `${Math.floor(diffSeconds / 3600)}h`;
  return `${Math.floor(diffSeconds / 86400)}d`;
};

const truncateAddress = (hash?: string): string => {
  if (!hash) return "—";
  if (hash.length <= 10) return hash;
  return `${hash.slice(0, 4)}...${hash.slice(-4)}`;
};

export const LiveTradesTable = ({
  trades,
  className = "",
}: LiveTradesTableProps) => {
  // Tick every second to update relative timestamps
  const [, setTick] = useState(0);
  useEffect(() => {
    const interval = setInterval(() => setTick((t) => t + 1), 1000);
    return () => clearInterval(interval);
  }, []);

  // Sort trades by timestamp, newest first
  const sortedTrades = useMemo(() => {
    return [...trades].sort(
      (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
    );
  }, [trades]);

  return (
    <div
      className={`rounded-lg overflow-hidden flex flex-col h-full ${className}`}
      style={{
        backgroundColor: fey.bg300,
        border: `1px solid ${fey.border}`,
      }}
    >
      {/* Header */}
      <div
        className="px-4 py-3 flex items-center justify-between"
        style={{ borderBottom: `1px solid ${fey.border}` }}
      >
        <span
          className="text-sm font-semibold"
          style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
        >
          Trades
        </span>
        <span className="text-xs font-mono" style={{ color: fey.grey500 }}>
          {trades.length} recent
        </span>
      </div>

      {/* Table Header */}
      <div
        className="grid grid-cols-6 px-4 py-2 text-[10px] font-medium uppercase tracking-wider"
        style={{
          color: fey.grey500,
          backgroundColor: fey.bg400,
          borderBottom: `1px solid ${fey.border}`,
        }}
      >
        <div>Age</div>
        <div>Type</div>
        <div className="text-right">Price</div>
        <div className="text-right">Amount</div>
        <div className="text-right">Total</div>
        <div className="text-right">Trader</div>
      </div>

      {/* Table Body */}
      <div className="flex-1 overflow-y-auto">
        {sortedTrades.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <p className="text-sm" style={{ color: fey.grey500 }}>
              No trades yet
            </p>
          </div>
        ) : (
          sortedTrades.map((trade, index) => {
            const isBuy =
              trade.side?.toLowerCase() === "buy" ||
              (!trade.side && trade.outcome?.toLowerCase() === "yes");
            const price = parseFloat(trade.price);
            const quantity = parseFloat(trade.quantity);
            const total = price * quantity;
            const polygonscanUrl = trade.transaction_hash
              ? `https://polygonscan.com/tx/${trade.transaction_hash}`
              : null;

            return (
              <div
                key={trade.id || index}
                className="grid grid-cols-6 px-4 py-2.5 text-xs transition-colors hover:bg-white/[0.02]"
                style={{
                  borderBottom: `1px solid ${fey.border}`,
                }}
              >
                {/* Age */}
                <div className="font-mono" style={{ color: fey.grey500 }}>
                  {formatTimeAgo(trade.timestamp)}
                </div>

                {/* Type */}
                <div
                  className="font-bold"
                  style={{ color: isBuy ? fey.teal : fey.red }}
                >
                  {isBuy ? "Buy" : "Sell"}
                </div>

                {/* Price */}
                <div
                  className="text-right font-mono"
                  style={{ color: fey.grey100 }}
                >
                  {(price * 100).toFixed(1)}¢
                </div>

                {/* Amount */}
                <div
                  className="text-right font-mono"
                  style={{ color: fey.grey300 }}
                >
                  {quantity >= 1000
                    ? `${(quantity / 1000).toFixed(1)}K`
                    : quantity.toFixed(0)}
                </div>

                {/* Total */}
                <div
                  className="text-right font-mono"
                  style={{ color: isBuy ? fey.teal : fey.red }}
                >
                  ${total >= 1000 ? `${(total / 1000).toFixed(2)}K` : total.toFixed(2)}
                </div>

                {/* Trader/Tx */}
                <div
                  className="text-right font-mono flex items-center justify-end gap-1"
                  style={{ color: fey.grey500 }}
                >
                  {truncateAddress(trade.transaction_hash)}
                  {polygonscanUrl && (
                    <a
                      href={polygonscanUrl}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="hover:opacity-80 transition-opacity"
                      onClick={(e) => e.stopPropagation()}
                    >
                      <ExternalLink className="h-3 w-3" style={{ color: fey.grey500 }} />
                    </a>
                  )}
                </div>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};

export default LiveTradesTable;
