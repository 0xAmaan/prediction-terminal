"use client";

import {
  TrendingUp,
  DollarSign,
  Clock,
  Calendar,
  Activity,
  Users,
} from "lucide-react";

// Fey color tokens
const fey = {
  bg300: "#131419",
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  skyBlue: "#54BBF7",
  border: "rgba(255, 255, 255, 0.06)",
};

interface StatItem {
  label: string;
  value: string;
  icon: React.ElementType;
  subtext?: string;
}

interface KeyStatsGridProps {
  volume: string;
  liquidity?: string;
  timeRemaining: string;
  closeDate: string;
  ticker?: string;
  openInterest?: string;
  className?: string;
}

const formatVolume = (volume: string): string => {
  const num = parseFloat(volume);
  if (isNaN(num) || num === 0) return "—";
  if (num >= 1_000_000) return `$${(num / 1_000_000).toFixed(2)}M`;
  if (num >= 1_000) return `$${(num / 1_000).toFixed(1)}K`;
  return `$${num.toFixed(0)}`;
};

const formatTicker = (ticker: string): string => {
  if (ticker.startsWith("0x") && ticker.length > 20) {
    return `${ticker.slice(0, 8)}...${ticker.slice(-4)}`;
  }
  return ticker;
};

export const KeyStatsGrid = ({
  volume,
  liquidity,
  timeRemaining,
  closeDate,
  ticker,
  openInterest,
  className = "",
}: KeyStatsGridProps) => {
  const stats: StatItem[] = [
    {
      label: "Volume",
      value: formatVolume(volume),
      icon: TrendingUp,
    },
    {
      label: "Liquidity",
      value: liquidity ? formatVolume(liquidity) : "—",
      icon: DollarSign,
    },
    {
      label: "Time Left",
      value: timeRemaining,
      icon: Clock,
    },
    {
      label: "Close Date",
      value: closeDate,
      icon: Calendar,
    },
  ];

  // Add optional stats
  if (openInterest) {
    stats.push({
      label: "Open Interest",
      value: formatVolume(openInterest),
      icon: Users,
    });
  }

  if (ticker) {
    stats.push({
      label: "Ticker",
      value: formatTicker(ticker),
      icon: Activity,
    });
  }

  return (
    <div
      className={`rounded-lg ${className}`}
      style={{
        backgroundColor: fey.bg300,
        border: `1px solid ${fey.border}`,
      }}
    >
      <div className="p-6">
        <div className="grid grid-cols-2 md:grid-cols-3 gap-6">
          {stats.map((stat) => (
            <div key={stat.label} className="space-y-1.5">
              <div className="flex items-center gap-2">
                <div
                  className="p-1.5 rounded"
                  style={{ backgroundColor: "rgba(84, 187, 247, 0.1)" }}
                >
                  <stat.icon className="h-3 w-3" style={{ color: fey.skyBlue }} />
                </div>
                <span
                  className="text-[10px] uppercase tracking-wider font-medium"
                  style={{ color: fey.grey500 }}
                >
                  {stat.label}
                </span>
              </div>
              <div
                className="text-lg font-semibold"
                style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
              >
                {stat.value}
              </div>
              {stat.subtext && (
                <div className="text-xs" style={{ color: fey.grey500 }}>
                  {stat.subtext}
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
