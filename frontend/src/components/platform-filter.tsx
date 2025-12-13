"use client";

// Fey color tokens
const fey = {
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  polymarket: "#54BBF7",  // Fey sky blue
  accent: "#54BBF7",
};

export type FilterOption =
  | "all"
  | "trending"
  | "expiring"
  | "new"
  | "crypto"
  | "politics"
  | "sports";

interface PlatformFilterProps {
  value: FilterOption;
  onChange: (value: FilterOption) => void;
}

export const PlatformFilter = ({ value, onChange }: PlatformFilterProps) => {
  const options: { label: string; value: FilterOption }[] = [
    { label: "All", value: "all" },
    { label: "Trending", value: "trending" },
    { label: "Expiring Soon", value: "expiring" },
    { label: "New", value: "new" },
    { label: "Crypto", value: "crypto" },
    { label: "Politics", value: "politics" },
    { label: "Sports", value: "sports" },
  ];

  return (
    <div className="flex gap-6 overflow-x-auto pb-1">
      {options.map((option) => {
        const isActive = value === option.value;

        return (
          <button
            key={option.value}
            onClick={() => onChange(option.value)}
            className="relative pb-2 text-base font-medium transition-colors duration-200 cursor-pointer whitespace-nowrap"
            style={{
              color: isActive ? fey.accent : fey.grey500,
              letterSpacing: "-0.01em",
            }}
          >
            {option.label}
            {/* Fey-style underline indicator */}
            {isActive && (
              <div
                className="absolute bottom-0 left-0 right-0 h-0.5 rounded-full animate-tab-indicator"
                style={{ backgroundColor: fey.accent }}
              />
            )}
          </button>
        );
      })}
    </div>
  );
};
