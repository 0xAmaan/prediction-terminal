"use client";

import { useState } from "react";
import { MarketsGrid } from "@/components/markets-grid";
import { NewsFeed } from "@/components/news";
import { Activity, Search } from "lucide-react";
import { Input } from "@/components/ui/input";

// Fey color tokens
const fey = {
  bg100: "#070709",
  bg200: "#101116",
  bg300: "#131419",
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  skyBlue: "#54BBF7",
  teal: "#4DBE95",
  border: "rgba(255, 255, 255, 0.06)",
};

const HomePage = () => {
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");
  const [activeTab, setActiveTab] = useState<"markets" | "news">("markets");

  const handleSearchChange = (value: string) => {
    setSearch(value);
    setTimeout(() => setDebouncedSearch(value), 300);
  };

  return (
    <div className="h-screen flex flex-col overflow-hidden" style={{ backgroundColor: fey.bg100 }}>
      {/* Header - Fey style */}
      <header
        className="shrink-0 z-50"
        style={{
          backgroundColor: fey.bg100,
          borderBottom: `1px solid ${fey.border}`,
        }}
      >
        <div className="mx-auto px-8 py-4" style={{ maxWidth: "1600px" }}>
          <div className="flex items-center justify-between gap-8">
            {/* Left: Logo + Title */}
            <div className="flex items-center gap-3 shrink-0">
              <div
                className="p-2.5 rounded-lg"
                style={{ backgroundColor: "rgba(84, 187, 247, 0.1)" }}
              >
                <Activity className="h-6 w-6" style={{ color: fey.skyBlue }} />
              </div>
              <h1
                className="text-xl font-semibold"
                style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
              >
                Prediction Terminal
              </h1>
            </div>

            {/* Center: Search - Fey style */}
            <div className="flex-1 max-w-2xl">
              <div className="relative">
                <Search
                  className="absolute left-4 top-1/2 -translate-y-1/2 h-5 w-5"
                  style={{ color: fey.grey500 }}
                />
                <Input
                  placeholder="Search markets..."
                  value={search}
                  onChange={(e) => handleSearchChange(e.target.value)}
                  className="search-input h-11 pl-12 pr-4 text-base rounded-lg focus-visible:ring-0 focus-visible:border-transparent"
                  style={{
                    backgroundColor: fey.bg200,
                    border: `1px solid ${fey.border}`,
                    color: fey.grey100,
                  }}
                />
              </div>
            </div>

            {/* Right: Account */}
            <div className="shrink-0">
              <button className="gradient-orb h-9 w-9 rounded-full" aria-label="Account" />
            </div>
          </div>
        </div>
      </header>

      {/* Tab Navigation */}
      <div
        className="shrink-0"
        style={{
          backgroundColor: fey.bg100,
          borderBottom: `1px solid ${fey.border}`,
        }}
      >
        <div className="mx-auto px-8" style={{ maxWidth: "1600px" }}>
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
        <div className="mx-auto px-8 pt-8 pb-6" style={{ maxWidth: "1600px" }}>
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

export default HomePage;
