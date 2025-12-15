"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState, useCallback, useEffect, useRef } from "react";
import { Activity, Search } from "lucide-react";
import { Input } from "@/components/ui/input";

// Fey color tokens
const fey = {
  bg100: "#070709",
  bg200: "#101116",
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  skyBlue: "#54BBF7",
  border: "rgba(255, 255, 255, 0.06)",
};

interface NavbarProps {
  search?: string;
  onSearchChange?: (value: string) => void;
}

export const Navbar = ({ search, onSearchChange }: NavbarProps) => {
  const router = useRouter();
  const [localSearch, setLocalSearch] = useState("");
  const searchInputRef = useRef<HTMLInputElement>(null);

  // Use controlled search if provided, otherwise use local state
  const searchValue = search ?? localSearch;

  // Focus search input on "/" key press
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Don't trigger if user is typing in an input or textarea
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement ||
        (e.target as HTMLElement).isContentEditable
      ) {
        return;
      }

      if (e.key === "/") {
        e.preventDefault();
        searchInputRef.current?.focus();
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, []);

  const handleSearchChange = useCallback(
    (value: string) => {
      if (onSearchChange) {
        // Controlled mode: parent handles search
        onSearchChange(value);
      } else {
        // Uncontrolled mode: update local state
        setLocalSearch(value);
      }
    },
    [onSearchChange]
  );

  const handleSearchSubmit = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === "Enter" && !onSearchChange) {
        // Navigate to home with search query when not in controlled mode
        router.push(`/?search=${encodeURIComponent(searchValue)}`);
      }
    },
    [router, searchValue, onSearchChange]
  );

  return (
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
          <Link href="/" className="flex items-center gap-3 shrink-0">
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
              Premonition
            </h1>
          </Link>

          {/* Center: Search - Fey style */}
          <div className="flex-1 max-w-2xl">
            <div className="relative">
              <Search
                className="absolute left-4 top-1/2 -translate-y-1/2 h-5 w-5"
                style={{ color: fey.grey500 }}
              />
              <Input
                ref={searchInputRef}
                placeholder="Search markets..."
                value={searchValue}
                onChange={(e) => handleSearchChange(e.target.value)}
                onKeyDown={handleSearchSubmit}
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
  );
};

export default Navbar;
