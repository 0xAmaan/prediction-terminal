"use client";

// Fey color tokens
const fey = {
  bg300: "#131419",
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  teal: "#4DBE95",
  red: "#D84F68",
  border: "rgba(255, 255, 255, 0.06)",
};

interface QuickStatsProps {
  yesPrice: string;
  noPrice: string;
  volume24h?: string;
  className?: string;
}

const formatPrice = (price: string): string => {
  const num = parseFloat(price);
  if (isNaN(num)) return "—";
  return `${(num * 100).toFixed(1)}¢`;
};

const formatVolume = (volume: string): string => {
  const num = parseFloat(volume);
  if (isNaN(num) || num === 0) return "—";
  if (num >= 1_000_000) return `$${(num / 1_000_000).toFixed(2)}M`;
  if (num >= 1_000) return `$${(num / 1_000).toFixed(1)}K`;
  return `$${num.toFixed(0)}`;
};

export const QuickStats = ({
  yesPrice,
  noPrice,
  volume24h,
  className = "",
}: QuickStatsProps) => {
  const yesNum = parseFloat(yesPrice);
  const noNum = parseFloat(noPrice);
  const spread = Math.abs(1 - yesNum - noNum) * 100;

  return (
    <div
      className={`rounded-lg ${className}`}
      style={{
        backgroundColor: fey.bg300,
        border: `1px solid ${fey.border}`,
      }}
    >
      <div className="p-4 space-y-4">
        {/* YES/NO Prices */}
        <div className="flex items-center justify-between">
          <div>
            <div
              className="text-[10px] uppercase tracking-wider font-medium mb-1"
              style={{ color: fey.grey500 }}
            >
              YES
            </div>
            <div
              className="text-2xl font-mono font-bold"
              style={{
                color: yesNum >= 0.5 ? fey.teal : fey.grey100,
                letterSpacing: "-0.02em",
              }}
            >
              {formatPrice(yesPrice)}
            </div>
          </div>
          <div className="text-right">
            <div
              className="text-[10px] uppercase tracking-wider font-medium mb-1"
              style={{ color: fey.grey500 }}
            >
              NO
            </div>
            <div
              className="text-2xl font-mono font-bold"
              style={{
                color: noNum >= 0.5 ? fey.red : fey.grey100,
                letterSpacing: "-0.02em",
              }}
            >
              {formatPrice(noPrice)}
            </div>
          </div>
        </div>

        {/* Divider */}
        <div className="h-px" style={{ backgroundColor: fey.border }} />

        {/* Secondary stats */}
        <div className="flex items-center justify-between text-sm">
          <span style={{ color: fey.grey500 }}>Spread</span>
          <span className="font-mono" style={{ color: fey.grey100 }}>
            {spread.toFixed(2)}%
          </span>
        </div>

        {volume24h && (
          <div className="flex items-center justify-between text-sm">
            <span style={{ color: fey.grey500 }}>24h Volume</span>
            <span className="font-mono" style={{ color: fey.grey100 }}>
              {formatVolume(volume24h)}
            </span>
          </div>
        )}
      </div>
    </div>
  );
};
