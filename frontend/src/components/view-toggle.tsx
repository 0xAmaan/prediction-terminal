"use client";

type ViewMode = "grid" | "table";

interface ViewToggleProps {
  value: ViewMode;
  onChange: (value: ViewMode) => void;
}

export const ViewToggle = ({ value, onChange }: ViewToggleProps) => {
  const options: { label: string; value: ViewMode }[] = [
    { label: "Grid", value: "grid" },
    { label: "Table", value: "table" },
  ];

  return (
    <div
      className="flex items-center gap-1 p-0.5 rounded-lg"
      style={{
        background: "rgba(255, 255, 255, 0.05)",
        border: "1px solid rgba(255, 255, 255, 0.05)",
        boxShadow: "-0.5px 1.5px 1.5px rgba(0, 0, 0, 0.25)",
      }}
    >
      {options.map((option) => {
        const isSelected = value === option.value;

        return (
          <button
            key={option.value}
            onClick={() => onChange(option.value)}
            className="relative px-5 py-1.5 rounded-md text-sm font-medium transition-all duration-200 cursor-pointer"
            style={{
              background: isSelected ? "rgba(255, 255, 255, 0.08)" : "transparent",
              color: isSelected ? "#FFFFFF" : "rgba(255, 255, 255, 0.6)",
              boxShadow: isSelected ? "-0.5px 1px 1.5px rgba(0, 0, 0, 0.25)" : "none",
            }}
          >
            {option.label}
          </button>
        );
      })}
    </div>
  );
};
