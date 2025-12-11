"use client";

import { useState } from "react";
import Link from "next/link";
import { Activity, Search, Newspaper, ArrowLeft } from "lucide-react";
import { Input } from "@/components/ui/input";
import { NewsFeed } from "@/components/news";

const NewsPage = () => {
  const [search, setSearch] = useState("");

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="sticky top-0 z-50 border-b border-border/50 bg-card/50 backdrop-blur-xl">
        <div className="mx-auto px-8 py-4" style={{ maxWidth: "1200px" }}>
          <div className="flex items-center justify-between gap-8">
            {/* Left: Logo + Title */}
            <Link href="/" className="flex items-center gap-3 shrink-0 group">
              <div className="p-2.5 rounded-xl bg-primary/10 group-hover:bg-primary/20 transition-colors">
                <Activity className="h-7 w-7 text-primary" />
              </div>
              <h1 className="text-2xl font-semibold tracking-tight">
                Prediction Terminal
              </h1>
            </Link>

            {/* Center: Search */}
            <div className="flex-1 max-w-xl">
              <div className="relative">
                <Search className="absolute left-4 top-1/2 -translate-y-1/2 h-5 w-5 text-muted-foreground" />
                <Input
                  placeholder="Search news..."
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                  className="h-12 pl-12 pr-4 text-base bg-secondary/80 border border-border/30 rounded-xl focus-visible:ring-0 focus-visible:border-transparent"
                />
              </div>
            </div>

            {/* Right: Placeholder */}
            <div className="shrink-0 w-10" />
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="mx-auto px-8 py-8" style={{ maxWidth: "1200px" }}>
        {/* Back link */}
        <Link
          href="/"
          className="inline-flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground mb-6 transition-colors"
        >
          <ArrowLeft className="h-4 w-4" />
          Back to Markets
        </Link>

        {/* Page title */}
        <div className="flex items-center gap-3 mb-8">
          <div className="p-2 rounded-lg bg-primary/10">
            <Newspaper className="h-5 w-5 text-primary" />
          </div>
          <h2 className="text-xl font-semibold">Prediction Market News</h2>
        </div>

        {/* News feed */}
        <div className="max-w-2xl">
          <NewsFeed limit={20} />
        </div>
      </main>

      {/* Footer */}
      <footer className="border-t border-border/50 py-4 mt-12 bg-card/30">
        <div className="mx-auto px-8" style={{ maxWidth: "1200px" }}>
          <p className="text-sm text-muted-foreground text-center">
            News aggregated from various sources using{" "}
            <a
              href="https://exa.ai"
              target="_blank"
              rel="noopener noreferrer"
              className="text-primary hover:underline"
            >
              Exa.ai
            </a>
          </p>
        </div>
      </footer>
    </div>
  );
};

export default NewsPage;
