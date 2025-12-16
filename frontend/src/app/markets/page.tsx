"use client";

import { useState } from "react";
import { MarketsGrid } from "@/components/markets-grid";
import { NewsFeed } from "@/components/news";
import { Navbar } from "@/components/layout/navbar";

// Fey color tokens
const fey = {
  bg100: "#070709",
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  skyBlue: "#54BBF7",
  teal: "#4DBE95",
  border: "rgba(255, 255, 255, 0.06)",
};

const MarketsPage = () => {
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const [activeTab, setActiveTab] = useState<"markets" | "news">("markets");

  const handleSearchChange = (value: string) => {
    setSearch(value);
    setTimeout(() => setDebouncedSearch(value), 300);
  };

  return (
    <div className="h-screen flex flex-col overflow-hidden" style={{ backgroundColor: fey.bg100 }}>
      {/* Header */}
      <Navbar search={search} onSearchChange={handleSearchChange} />

      {/* Tab Navigation */}
      <div
        className="shrink-0"
        style={{
          backgroundColor: fey.bg100,
          borderBottom: `1px solid ${fey.border}`,
        }}
      >
        <div className="mx-auto px-8" style={{ maxWidth: "1800px" }}>
          <div className="flex gap-1">
            <button
              onClick={() => setActiveTab("markets")}
              className="px-6 py-3 text-sm font-medium transition-colors relative"
              style={{
                color: activeTab === "markets" ? fey.grey100 : fey.grey500,
              }}
            >
              Markets
              {activeTab === "markets" && (
                <div
                  className="absolute bottom-0 left-0 right-0 h-0.5"
                  style={{ backgroundColor: fey.skyBlue }}
                />
              )}
            </button>
            <button
              onClick={() => setActiveTab("news")}
              className="px-6 py-3 text-sm font-medium transition-colors relative"
              style={{
                color: activeTab === "news" ? fey.grey100 : fey.grey500,
              }}
            >
              News Feed
              {activeTab === "news" && (
                <div
                  className="absolute bottom-0 left-0 right-0 h-0.5"
                  style={{ backgroundColor: fey.skyBlue }}
                />
              )}
            </button>
          </div>
        </div>
      </div>

      {/* Main content - scrollable */}
      <main className="flex-1 overflow-y-auto">
        <div className="mx-auto px-8 pt-8 pb-6" style={{ maxWidth: "1800px" }}>
          {activeTab === "markets" ? (
            <MarketsGrid search={debouncedSearch} />
          ) : (
            <div>
              <h2
                className="text-lg font-semibold mb-6"
                style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
              >
                Market News
              </h2>
              <NewsFeed />
            </div>
          )}
        </div>
      </main>

      {/* Footer - Fey style */}
      <footer
        className="shrink-0 py-3"
        style={{
          backgroundColor: fey.bg100,
          borderTop: `1px solid ${fey.border}`,
        }}
      >
        <div className="px-6">
          <div className="flex items-center justify-between">
            <p className="text-sm" style={{ color: fey.grey500 }}>
              {/* KALSHI_DISABLED: was "Data from Kalshi and Polymarket" */}
              Data from{" "}
              <a
                href="https://polymarket.com"
                target="_blank"
                rel="noopener noreferrer"
                className="hover:underline font-medium"
                style={{ color: fey.skyBlue }}
              >
                Polymarket
              </a>
            </p>
            <div className="flex items-center gap-2 text-sm" style={{ color: fey.grey500 }}>
              <span
                className="h-2 w-2 rounded-full"
                style={{ backgroundColor: fey.teal }}
              />
              <span>Connection stable</span>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
};

export default MarketsPage;
