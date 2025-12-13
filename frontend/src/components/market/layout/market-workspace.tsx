"use client";

import { type ReactNode } from "react";
import { motion } from "framer-motion";
import { staggerContainer, staggerItem, slideFromLeft, slideFromRight } from "@/lib/motion";

// Fey color tokens
const fey = {
  bg100: "#070709",
  bg200: "#101116",
  bg300: "#131419",
  border: "rgba(255, 255, 255, 0.06)",
};

// ============================================================================
// Types
// ============================================================================

interface MarketWorkspaceProps {
  header: ReactNode;
  leftRail?: ReactNode;
  center: ReactNode;
  rightRail?: ReactNode;
  footer?: ReactNode;
  className?: string;
}

// ============================================================================
// Main Workspace Component
// ============================================================================

export const MarketWorkspace = ({
  header,
  leftRail,
  center,
  rightRail,
  footer,
  className = "",
}: MarketWorkspaceProps) => {
  return (
    <div
      className={`min-h-screen flex flex-col ${className}`}
      style={{ backgroundColor: fey.bg100 }}
    >
      {/* Header - Sticky */}
      {header}

      {/* Main Content Area */}
      <motion.main
        className="flex-1 px-6 lg:px-12 xl:px-16 py-6 pb-20" // pb-20 for market bar space
        variants={staggerContainer}
        initial="hidden"
        animate="visible"
      >
        <div className="max-w-[1800px] mx-auto">
          <div className="grid grid-cols-12 gap-6">
            {/* Left Rail - 3 cols on xl, hidden on smaller */}
            {leftRail && (
              <motion.aside
                className="hidden xl:block xl:col-span-3 space-y-4"
                variants={slideFromLeft}
              >
                {leftRail}
              </motion.aside>
            )}

            {/* Center - Main Content */}
            <motion.div
              className={`col-span-12 ${
                leftRail && rightRail
                  ? "xl:col-span-5"
                  : leftRail || rightRail
                    ? "xl:col-span-8"
                    : "xl:col-span-12"
              } space-y-6`}
              variants={staggerItem}
            >
              {center}
            </motion.div>

            {/* Right Rail - 4 cols on xl */}
            {rightRail && (
              <motion.aside
                className="col-span-12 xl:col-span-4 space-y-4"
                variants={slideFromRight}
              >
                {rightRail}
              </motion.aside>
            )}
          </div>
        </div>
      </motion.main>

      {/* Footer - Fixed Market Bar */}
      {footer}
    </div>
  );
};

// ============================================================================
// Section Component (for grouping related panels)
// ============================================================================

interface SectionProps {
  children: ReactNode;
  title?: string;
  className?: string;
}

export const Section = ({ children, title, className = "" }: SectionProps) => (
  <motion.section className={`space-y-4 ${className}`} variants={staggerItem}>
    {title && (
      <h2
        className="text-xs uppercase tracking-wider font-medium px-1"
        style={{ color: "#7D8B96" }}
      >
        {title}
      </h2>
    )}
    {children}
  </motion.section>
);

// ============================================================================
// Card Component (standard panel styling)
// ============================================================================

interface CardProps {
  children: ReactNode;
  title?: string;
  icon?: ReactNode;
  action?: ReactNode;
  noPadding?: boolean;
  className?: string;
}

export const Card = ({
  children,
  title,
  icon,
  action,
  noPadding = false,
  className = "",
}: CardProps) => (
  <motion.div
    className={`rounded-lg overflow-hidden ${className}`}
    style={{
      backgroundColor: fey.bg300,
      border: `1px solid ${fey.border}`,
    }}
    variants={staggerItem}
    whileHover={{ borderColor: "rgba(255, 255, 255, 0.10)" }}
    transition={{ duration: 0.2 }}
  >
    {title && (
      <div
        className="px-5 py-4 flex items-center justify-between"
        style={{ borderBottom: `1px solid ${fey.border}` }}
      >
        <div className="flex items-center gap-2">
          {icon && (
            <div
              className="p-1.5 rounded"
              style={{ backgroundColor: "rgba(84, 187, 247, 0.1)" }}
            >
              {icon}
            </div>
          )}
          <span
            className="text-sm font-semibold"
            style={{ color: "#EEF0F1", letterSpacing: "-0.02em" }}
          >
            {title}
          </span>
        </div>
        {action}
      </div>
    )}
    <div className={noPadding ? "" : "p-5"}>{children}</div>
  </motion.div>
);

// ============================================================================
// Stat Display Component (Fey-style horizontal row)
// ============================================================================

interface StatDisplayProps {
  label: string;
  value: string | ReactNode;
  subValue?: string;
  color?: string;
  icon?: ReactNode;
  showBorder?: boolean;
}

export const StatDisplay = ({
  label,
  value,
  subValue,
  color = "#EEF0F1",
  icon,
  showBorder = true,
}: StatDisplayProps) => {
  return (
    <div
      className="flex items-center justify-between py-3"
      style={showBorder ? { borderBottom: `1px solid ${fey.border}` } : undefined}
    >
      <span
        className="text-xs uppercase tracking-wider font-medium"
        style={{ color: "#7D8B96" }}
      >
        {label}
      </span>
      <div className="flex items-center gap-2">
        {icon && (
          <span style={{ color: "#7D8B96" }}>{icon}</span>
        )}
        <span
          className="text-base font-mono font-semibold tabular-nums"
          style={{ color, letterSpacing: "-0.02em" }}
        >
          {value}
        </span>
        {subValue && (
          <span
            className="text-xs"
            style={{ color: "#7D8B96" }}
          >
            {subValue}
          </span>
        )}
      </div>
    </div>
  );
};

// ============================================================================
// Divider Component
// ============================================================================

export const Divider = ({ className = "" }: { className?: string }) => (
  <div
    className={`h-px w-full ${className}`}
    style={{ backgroundColor: fey.border }}
  />
);

export default MarketWorkspace;
