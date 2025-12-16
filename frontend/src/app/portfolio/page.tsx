"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
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
}

const PortfolioPage = () => {
  const [positions, setPositions] = useState<Position[]>([]);
  const [balance, setBalance] = useState<Balance | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
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
    };

    fetchData();
  }, []);

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
                          className="hover:opacity-80 transition-opacity cursor-pointer"
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
                                className="text-sm font-medium max-w-md truncate"
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
    </div>
  );
};

export default PortfolioPage;
