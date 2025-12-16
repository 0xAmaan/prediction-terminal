"use client";

import Link from "next/link";

export const Footer = () => {
  return (
    <footer className="py-8 border-t border-[rgba(255,255,255,0.06)] bg-[#070709]">
      <div className="max-w-7xl mx-auto px-4 flex flex-col sm:flex-row justify-between items-center gap-4 text-sm text-[#7D8B96]">
        <p>© {new Date().getFullYear()} Premonition</p>
        <div className="flex gap-6">
          <a 
            href="https://github.com/0xAmaan/prediction-terminal" 
            target="_blank" 
            rel="noopener noreferrer" 
            className="hover:text-[#EEF0F1] transition-colors"
          >
            GitHub
          </a>
        </div>
      </div>
    </footer>
  );
};
