"use client";

import { useState, useMemo } from "react";
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
  tealMuted: "rgba(77, 190, 149, 0.15)",
  red: "#D84F68",
  redMuted: "rgba(216, 79, 104, 0.15)",
  border: "rgba(255, 255, 255, 0.06)",
};

type Side = "buy" | "sell";
type OrderType = "market" | "limit";

interface TradeExecutionProps {
  yesPrice: string;
  noPrice: string;
  trades?: Trade[];
  className?: string;
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
}: TradeExecutionProps) => {
  const [side, setSide] = useState<Side>("buy");
  const [orderType, setOrderType] = useState<OrderType>("market");
  const [amount, setAmount] = useState("");
  const [limitPrice, setLimitPrice] = useState("");

  const isBuy = side === "buy";
  const accentColor = isBuy ? fey.teal : fey.red;

  const quickAmounts = ["0.01", "0.1", "1", "10"];

  const currentPrice = parseFloat(yesPrice);
  const estimatedCost = amount ? parseFloat(amount) * currentPrice : 0;

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

  return (
    <div
      className={`rounded-lg overflow-hidden flex flex-col h-full ${className}`}
      style={{
        backgroundColor: fey.bg300,
        border: `1px solid ${fey.border}`,
      }}
    >
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

      {/* Amount Input */}
      <div className="px-4 pt-4 flex-1">
        <div className="flex items-center justify-between mb-2">
          <label className="text-sm font-medium" style={{ color: fey.grey500 }}>
            AMOUNT
          </label>
          <span className="text-sm font-mono" style={{ color: fey.grey500 }}>
            0.0
          </span>
        </div>
        <input
          type="text"
          value={amount}
          onChange={(e) => setAmount(e.target.value)}
          placeholder="0.00"
          className="w-full px-3 py-3 rounded-md text-right text-xl font-mono outline-none transition-colors"
          style={{
            backgroundColor: fey.bg400,
            border: `1px solid ${fey.border}`,
            color: fey.grey100,
          }}
        />

        {/* Quick Amount Buttons */}
        <div className="grid grid-cols-4 gap-2 mt-2">
          {quickAmounts.map((amt) => (
            <button
              key={amt}
              onClick={() => setAmount(amt)}
              className="py-1.5 rounded text-sm font-medium transition-colors hover:opacity-80"
              style={{
                backgroundColor: fey.bg400,
                color: fey.grey300,
                border: `1px solid ${fey.border}`,
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
              className="w-full px-3 py-3 rounded-md text-right text-xl font-mono outline-none"
              style={{
                backgroundColor: fey.bg400,
                border: `1px solid ${fey.border}`,
                color: fey.grey100,
              }}
            />
            <span className="text-xs mt-1 block text-right" style={{ color: fey.grey500 }}>
              cents
            </span>
          </div>
        )}

        {/* Estimated Cost */}
        {amount && (
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
      </div>

      {/* Submit Button */}
      <div className="p-4">
        <button
          disabled
          className="w-full py-3.5 rounded-lg text-base font-semibold transition-all cursor-not-allowed opacity-60"
          style={{
            backgroundColor: accentColor,
            color: fey.bg100,
          }}
        >
          {isBuy ? "Buy" : "Sell"} YES
        </button>
        <p className="text-xs text-center mt-2" style={{ color: fey.grey500 }}>
          Trading not yet available
        </p>
      </div>
    </div>
  );
};

export default TradeExecution;
