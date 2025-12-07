"use client";

type PlatformOption = "all" | "kalshi" | "polymarket";

interface PlatformFilterProps {
  value: PlatformOption;
  onChange: (value: PlatformOption) => void;
}

const getTextStyles = (option: PlatformOption, isActive: boolean) => {
  if (!isActive) {
    return "text-muted-foreground hover:text-foreground";
  }

  switch (option) {
    case "kalshi":
      return "text-[#22c55e]";
    case "polymarket":
      return "text-[#3b82f6]";
    default:
      return "text-foreground";
  }
};

export const PlatformFilter = ({ value, onChange }: PlatformFilterProps) => {
  const options: { label: string; value: PlatformOption }[] = [
    { label: "Trending", value: "all" },
    { label: "Kalshi", value: "kalshi" },
    { label: "Polymarket", value: "polymarket" },
  ];

  return (
    <div className="flex gap-8">
      {options.map((option) => (
        <button
          key={option.value}
          onClick={() => onChange(option.value)}
          className={`text-2xl font-semibold transition-colors duration-200 cursor-pointer ${getTextStyles(option.value, value === option.value)}`}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
};
