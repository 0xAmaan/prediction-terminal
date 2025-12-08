"use client";

import { useState } from "react";
import { MarketsGrid } from "@/components/markets-grid";
import { Activity, Search } from "lucide-react";
import { Input } from "@/components/ui/input";

const HomePage = () => {
  const [search, setSearch] = useState("");
  const [debouncedSearch, setDebouncedSearch] = useState("");

  const handleSearchChange = (value: string) => {
    setSearch(value);
    setTimeout(() => setDebouncedSearch(value), 300);
  };

  return (
    <div className="h-screen bg-background flex flex-col overflow-hidden">
      {/* Header */}
      <header className="shrink-0 border-b border-border/50 bg-card/50 backdrop-blur-xl z-50">
        <div className="mx-auto px-8 py-4" style={{ maxWidth: "1600px" }}>
          <div className="flex items-center justify-between gap-8">
            {/* Left: Logo + Title */}
            <div className="flex items-center gap-3 shrink-0">
              <div className="p-2.5 rounded-xl bg-primary/10">
                <Activity className="h-7 w-7 text-primary" />
              </div>
              <h1 className="text-2xl font-semibold tracking-tight">Prediction Terminal</h1>
            </div>

            {/* Center: Search */}
            <div className="flex-1 max-w-2xl">
              <div className="relative">
                <Search className="absolute left-4 top-1/2 -translate-y-1/2 h-5 w-5 text-muted-foreground" />
                <Input
                  placeholder="Search markets..."
                  value={search}
                  onChange={(e) => handleSearchChange(e.target.value)}
                  className="search-input h-12 pl-12 pr-4 text-base bg-secondary/80 border border-border/30 rounded-xl focus-visible:ring-0 focus-visible:border-transparent placeholder:text-muted-foreground/60"
                />
              </div>
            </div>

            {/* Right: Account */}
            <div className="shrink-0">
              <button className="gradient-orb h-10 w-10 rounded-full" aria-label="Account" />
            </div>
          </div>
        </div>
      </header>

      {/* Main content - scrollable */}
      <main className="flex-1 overflow-hidden">
        <div className="h-full mx-auto px-8 pt-10 pb-6" style={{ maxWidth: "1600px" }}>
          <MarketsGrid search={debouncedSearch} />
        </div>
      </main>

      {/* Footer - always visible */}
      <footer className="shrink-0 border-t border-border/50 py-3 bg-card/30">
        <div className="px-6">
          <div className="flex items-center justify-between">
            <p className="text-base text-muted-foreground">
              Data from{" "}
              <a
                href="https://kalshi.com"
                target="_blank"
                rel="noopener noreferrer"
                className="text-[#22c55e] hover:underline font-medium"
              >
                Kalshi
              </a>{" "}
              and{" "}
              <a
                href="https://polymarket.com"
                target="_blank"
                rel="noopener noreferrer"
                className="text-[#3b82f6] hover:underline font-medium"
              >
                Polymarket
              </a>
            </p>
            <div className="flex items-center gap-2 text-base text-muted-foreground">
              <span className="h-2.5 w-2.5 rounded-full bg-[#22c55e]"></span>
              <span>Connection stable</span>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
};

export default HomePage;
