"use client";

import { motion } from "framer-motion";

// Fey color tokens
const fey = {
  bg100: "#070709",
  bg300: "#131419",
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  grey700: "#4E5860",
  border: "rgba(255, 255, 255, 0.06)",
};

export type MarketTab = "overview" | "trading" | "research";

interface MarketTabsProps {
  activeTab: MarketTab;
  onTabChange: (tab: MarketTab) => void;
  showKeyboardHints?: boolean;
}

export const MarketTabs = ({
  activeTab,
  onTabChange,
  showKeyboardHints = true,
}: MarketTabsProps) => {
  const tabs: { id: MarketTab; label: string; shortcut: string }[] = [
    { id: "overview", label: "Overview", shortcut: "O" },
    { id: "trading", label: "Trading", shortcut: "T" },
    { id: "research", label: "Research", shortcut: "R" },
  ];

  return (
    <div
      className="sticky top-[57px] z-40"
      style={{
        backgroundColor: fey.bg100,
        borderBottom: `1px solid ${fey.border}`,
      }}
    >
      <div className="px-6 lg:px-8">
        <nav className="flex justify-center gap-8" aria-label="Market tabs">
          {tabs.map((tab) => {
            const isActive = activeTab === tab.id;
            return (
              <button
                key={tab.id}
                onClick={() => onTabChange(tab.id)}
                className="relative pb-3 pt-3 text-sm font-semibold transition-colors duration-200 cursor-pointer group flex items-center gap-2"
                style={{
                  color: isActive ? fey.grey100 : fey.grey500,
                  letterSpacing: "-0.02em",
                }}
              >
                {tab.label}
                {/* Keyboard shortcut hint */}
                {showKeyboardHints && (
                  <span
                    className="hidden lg:inline-flex items-center justify-center h-5 w-5 rounded text-[10px] font-mono opacity-0 group-hover:opacity-100 transition-opacity"
                    style={{
                      backgroundColor: fey.bg300,
                      color: fey.grey500,
                      border: `1px solid ${fey.border}`,
                    }}
                  >
                    {tab.shortcut}
                  </span>
                )}
                {/* Active indicator with animation */}
                {isActive && (
                  <motion.div
                    layoutId="activeTabIndicator"
                    className="absolute bottom-0 left-0 right-0 h-0.5 rounded-full"
                    style={{ backgroundColor: fey.grey100 }}
                    transition={{ type: "spring", stiffness: 500, damping: 30 }}
                  />
                )}
              </button>
            );
          })}
        </nav>
      </div>
    </div>
  );
};
