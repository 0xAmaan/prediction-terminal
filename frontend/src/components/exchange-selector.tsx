"use client";

// Fey color tokens
const fey = {
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  grey900: "#202427",
  bg500: "#1A1B20",
  accent: "#54BBF7",
};

export type Exchange = "polymarket";

interface ExchangeSelectorProps {
  value: Exchange;
  onChange: (value: Exchange) => void;
}

export const ExchangeSelector = ({ value, onChange }: ExchangeSelectorProps) => {
  // Since we only support Polymarket now, display a simple badge
  // Component structure kept for future multi-platform support

  return (
    <div className="inline-flex gap-0.5 p-0.5 rounded-lg" style={{ backgroundColor: fey.bg500 }}>
      <div
        className="px-4 py-2 text-sm font-medium rounded-md"
        style={{
          backgroundColor: fey.grey900,
          color: fey.grey100,
          letterSpacing: "-0.01em",
        }}
      >
        Polymarket
      </div>
    </div>
  );
};
