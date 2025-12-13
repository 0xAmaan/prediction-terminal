"use client";

import { useState, createContext, useContext, type ReactNode } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { ChevronLeft, ChevronRight } from "lucide-react";

// Fey color tokens
const fey = {
  bg100: "#070709",
  bg300: "#131419",
  bg400: "#16181C",
  grey100: "#EEF0F1",
  grey500: "#7D8B96",
  border: "rgba(255, 255, 255, 0.06)",
};

// ============================================================================
// Context for Pro Mode
// ============================================================================

interface ProModeContextType {
  proMode: boolean;
  setProMode: (value: boolean) => void;
}

const ProModeContext = createContext<ProModeContextType>({
  proMode: false,
  setProMode: () => {},
});

export const useProMode = () => useContext(ProModeContext);

export const ProModeProvider = ({ children }: { children: ReactNode }) => {
  const [proMode, setProMode] = useState(() => {
    if (typeof window !== "undefined") {
      return localStorage.getItem("proMode") === "true";
    }
    return false;
  });

  const handleSetProMode = (value: boolean) => {
    setProMode(value);
    if (typeof window !== "undefined") {
      localStorage.setItem("proMode", String(value));
    }
  };

  return (
    <ProModeContext.Provider value={{ proMode, setProMode: handleSetProMode }}>
      {children}
    </ProModeContext.Provider>
  );
};

// ============================================================================
// Types
// ============================================================================

interface CollapsibleRailProps {
  children: ReactNode;
  side: "left" | "right";
  defaultCollapsed?: boolean;
  collapsedWidth?: number;
  expandedWidth?: number | string;
  className?: string;
}

// ============================================================================
// Animation Variants
// ============================================================================

const railVariants = {
  expanded: (expandedWidth: number | string) => ({
    width: expandedWidth,
    transition: {
      type: "spring" as const,
      stiffness: 300,
      damping: 30,
    },
  }),
  collapsed: (collapsedWidth: number) => ({
    width: collapsedWidth,
    transition: {
      type: "spring" as const,
      stiffness: 300,
      damping: 30,
    },
  }),
};

const contentVariants = {
  visible: {
    opacity: 1,
    transition: { delay: 0.1, duration: 0.2 },
  },
  hidden: {
    opacity: 0,
    transition: { duration: 0.1 },
  },
};

// ============================================================================
// Collapsible Rail Component
// ============================================================================

export const CollapsibleRail = ({
  children,
  side,
  defaultCollapsed = false,
  collapsedWidth = 48,
  expandedWidth = 320,
  className = "",
}: CollapsibleRailProps) => {
  const [isCollapsed, setIsCollapsed] = useState(defaultCollapsed);

  const toggleCollapse = () => setIsCollapsed(!isCollapsed);

  const Icon = side === "left" ? (isCollapsed ? ChevronRight : ChevronLeft) : (isCollapsed ? ChevronLeft : ChevronRight);

  return (
    <motion.div
      className={`relative flex-shrink-0 ${className}`}
      variants={railVariants}
      initial={isCollapsed ? "collapsed" : "expanded"}
      animate={isCollapsed ? "collapsed" : "expanded"}
      custom={isCollapsed ? collapsedWidth : expandedWidth}
    >
      {/* Toggle Button */}
      <button
        onClick={toggleCollapse}
        className={`absolute top-4 z-10 flex items-center justify-center w-6 h-6 rounded-full transition-colors ${
          side === "left" ? "-right-3" : "-left-3"
        }`}
        style={{
          backgroundColor: fey.bg400,
          border: `1px solid ${fey.border}`,
          color: fey.grey500,
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.backgroundColor = fey.bg300;
          e.currentTarget.style.color = fey.grey100;
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.backgroundColor = fey.bg400;
          e.currentTarget.style.color = fey.grey500;
        }}
      >
        <Icon className="h-3.5 w-3.5" />
      </button>

      {/* Content */}
      <AnimatePresence mode="wait">
        {!isCollapsed && (
          <motion.div
            className="h-full overflow-hidden"
            variants={contentVariants}
            initial="hidden"
            animate="visible"
            exit="hidden"
          >
            {children}
          </motion.div>
        )}
      </AnimatePresence>

      {/* Collapsed State Indicator */}
      {isCollapsed && (
        <motion.div
          className="h-full flex flex-col items-center pt-16 gap-4"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.2 }}
        >
          {/* Visual indicators when collapsed */}
          <div
            className="w-1 h-8 rounded-full"
            style={{ backgroundColor: fey.border }}
          />
          <div
            className="w-1 h-4 rounded-full"
            style={{ backgroundColor: fey.border }}
          />
          <div
            className="w-1 h-2 rounded-full"
            style={{ backgroundColor: fey.border }}
          />
        </motion.div>
      )}
    </motion.div>
  );
};

// ============================================================================
// Panel Component (for use within rails)
// ============================================================================

interface PanelProps {
  children: ReactNode;
  title?: string;
  icon?: ReactNode;
  collapsible?: boolean;
  defaultExpanded?: boolean;
  className?: string;
}

export const Panel = ({
  children,
  title,
  icon,
  collapsible = false,
  defaultExpanded = true,
  className = "",
}: PanelProps) => {
  const [isExpanded, setIsExpanded] = useState(defaultExpanded);

  return (
    <div
      className={`rounded-lg overflow-hidden ${className}`}
      style={{
        backgroundColor: fey.bg300,
        border: `1px solid ${fey.border}`,
      }}
    >
      {/* Header */}
      {title && (
        <div
          className={`px-4 py-3 flex items-center justify-between ${
            collapsible ? "cursor-pointer" : ""
          }`}
          style={{ borderBottom: isExpanded ? `1px solid ${fey.border}` : "none" }}
          onClick={collapsible ? () => setIsExpanded(!isExpanded) : undefined}
        >
          <div className="flex items-center gap-2">
            {icon && (
              <div
                className="p-1 rounded"
                style={{ backgroundColor: "rgba(84, 187, 247, 0.1)" }}
              >
                {icon}
              </div>
            )}
            <span
              className="text-sm font-semibold"
              style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
            >
              {title}
            </span>
          </div>
          {collapsible && (
            <motion.div
              animate={{ rotate: isExpanded ? 0 : -90 }}
              transition={{ duration: 0.2 }}
            >
              <ChevronLeft className="h-4 w-4" style={{ color: fey.grey500 }} />
            </motion.div>
          )}
        </div>
      )}

      {/* Content */}
      <AnimatePresence>
        {isExpanded && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            transition={{ duration: 0.2 }}
          >
            <div className="p-4">{children}</div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
};

// ============================================================================
// Workspace Layout Component
// ============================================================================

interface WorkspaceLayoutProps {
  leftRail?: ReactNode;
  center: ReactNode;
  rightRail?: ReactNode;
  footer?: ReactNode;
  className?: string;
}

export const WorkspaceLayout = ({
  leftRail,
  center,
  rightRail,
  footer,
  className = "",
}: WorkspaceLayoutProps) => {
  return (
    <div
      className={`flex flex-col min-h-screen ${className}`}
      style={{ backgroundColor: fey.bg100 }}
    >
      {/* Main Content Area */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left Rail */}
        {leftRail && (
          <CollapsibleRail side="left" expandedWidth={280} collapsedWidth={48}>
            <div className="p-4 space-y-4 h-full overflow-y-auto">
              {leftRail}
            </div>
          </CollapsibleRail>
        )}

        {/* Center Content */}
        <div className="flex-1 overflow-y-auto p-6">
          {center}
        </div>

        {/* Right Rail */}
        {rightRail && (
          <CollapsibleRail side="right" expandedWidth={360} collapsedWidth={48}>
            <div className="p-4 space-y-4 h-full overflow-y-auto">
              {rightRail}
            </div>
          </CollapsibleRail>
        )}
      </div>

      {/* Footer (Market Bar) */}
      {footer}
    </div>
  );
};

export default CollapsibleRail;
