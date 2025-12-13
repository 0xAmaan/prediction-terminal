"use client";

import { motion } from "framer-motion";
import { Zap, Eye, EyeOff, Keyboard } from "lucide-react";

// Fey color tokens
const fey = {
  bg300: "#131419",
  bg400: "#16181C",
  grey100: "#EEF0F1",
  grey300: "#B6BEC4",
  grey500: "#7D8B96",
  teal: "#4DBE95",
  tealMuted: "rgba(77, 190, 149, 0.15)",
  skyBlue: "#54BBF7",
  skyBlueMuted: "rgba(84, 187, 247, 0.15)",
  amber: "#F5A524",
  amberMuted: "rgba(245, 165, 36, 0.15)",
  border: "rgba(255, 255, 255, 0.06)",
};

// ============================================================================
// Types
// ============================================================================

interface ProModeToggleProps {
  proMode: boolean;
  onToggle: () => void;
  showLabel?: boolean;
  size?: "sm" | "md" | "lg";
  className?: string;
}

// ============================================================================
// Main Toggle Component
// ============================================================================

export const ProModeToggle = ({
  proMode,
  onToggle,
  showLabel = true,
  size = "md",
  className = "",
}: ProModeToggleProps) => {
  const sizes = {
    sm: {
      toggle: "w-8 h-4",
      dot: "w-3 h-3",
      translate: "translate-x-4",
      icon: "h-3 w-3",
      text: "text-[10px]",
      padding: "px-2 py-1",
    },
    md: {
      toggle: "w-10 h-5",
      dot: "w-4 h-4",
      translate: "translate-x-5",
      icon: "h-3.5 w-3.5",
      text: "text-xs",
      padding: "px-3 py-1.5",
    },
    lg: {
      toggle: "w-12 h-6",
      dot: "w-5 h-5",
      translate: "translate-x-6",
      icon: "h-4 w-4",
      text: "text-sm",
      padding: "px-4 py-2",
    },
  };

  const s = sizes[size];

  return (
    <button
      onClick={onToggle}
      className={`flex items-center gap-2 rounded-lg transition-colors ${s.padding} ${className}`}
      style={{
        backgroundColor: proMode ? fey.amberMuted : fey.bg400,
        border: `1px solid ${proMode ? "rgba(245, 165, 36, 0.3)" : fey.border}`,
      }}
      title={proMode ? "Switch to Simple View (P)" : "Switch to Pro View (P)"}
    >
      {/* Icon */}
      <Zap
        className={s.icon}
        style={{ color: proMode ? fey.amber : fey.grey500 }}
      />

      {/* Label */}
      {showLabel && (
        <span
          className={`font-medium ${s.text}`}
          style={{ color: proMode ? fey.amber : fey.grey500 }}
        >
          {proMode ? "Pro" : "Simple"}
        </span>
      )}

      {/* Toggle Switch */}
      <div
        className={`relative rounded-full ${s.toggle}`}
        style={{
          backgroundColor: proMode
            ? "rgba(245, 165, 36, 0.3)"
            : "rgba(125, 139, 150, 0.2)",
        }}
      >
        <motion.div
          className={`absolute top-0.5 left-0.5 rounded-full ${s.dot}`}
          style={{
            backgroundColor: proMode ? fey.amber : fey.grey500,
          }}
          animate={{
            x: proMode ? parseInt(s.translate.split("-x-")[1]) : 0,
          }}
          transition={{ type: "spring", stiffness: 500, damping: 30 }}
        />
      </div>
    </button>
  );
};

// ============================================================================
// Static Pro Badge (non-interactive)
// ============================================================================

interface ProBadgeProps {
  size?: "sm" | "md" | "lg";
  className?: string;
}

export const ProBadge = ({
  size = "sm",
  className = "",
}: ProBadgeProps) => {
  const sizes = {
    sm: {
      icon: "h-3 w-3",
      text: "text-[10px]",
      padding: "px-2 py-1",
    },
    md: {
      icon: "h-3.5 w-3.5",
      text: "text-xs",
      padding: "px-3 py-1.5",
    },
    lg: {
      icon: "h-4 w-4",
      text: "text-sm",
      padding: "px-4 py-2",
    },
  };

  const s = sizes[size];

  return (
    <div
      className={`flex items-center gap-1.5 rounded-lg ${s.padding} ${className}`}
      style={{
        backgroundColor: fey.amberMuted,
        border: "1px solid rgba(245, 165, 36, 0.3)",
      }}
    >
      <Zap className={s.icon} style={{ color: fey.amber }} />
      <span className={`font-medium ${s.text}`} style={{ color: fey.amber }}>
        Pro
      </span>
    </div>
  );
};

// ============================================================================
// Minimal Toggle (just the switch)
// ============================================================================

interface MinimalToggleProps {
  proMode: boolean;
  onToggle: () => void;
  className?: string;
}

export const MinimalProToggle = ({
  proMode,
  onToggle,
  className = "",
}: MinimalToggleProps) => {
  return (
    <button
      onClick={onToggle}
      className={`relative w-10 h-5 rounded-full transition-colors ${className}`}
      style={{
        backgroundColor: proMode
          ? "rgba(245, 165, 36, 0.3)"
          : "rgba(125, 139, 150, 0.2)",
      }}
      title={`${proMode ? "Pro" : "Simple"} View (P)`}
    >
      <motion.div
        className="absolute top-0.5 left-0.5 w-4 h-4 rounded-full"
        style={{
          backgroundColor: proMode ? fey.amber : fey.grey500,
        }}
        animate={{ x: proMode ? 20 : 0 }}
        transition={{ type: "spring", stiffness: 500, damping: 30 }}
      />
    </button>
  );
};

// ============================================================================
// Keyboard Shortcuts Help Modal
// ============================================================================

interface ShortcutItem {
  key: string;
  description: string;
}

interface KeyboardShortcutsHelpProps {
  isOpen: boolean;
  onClose: () => void;
  shortcuts?: ShortcutItem[];
}

export const KeyboardShortcutsHelp = ({
  isOpen,
  onClose,
  shortcuts = [
    { key: "1", description: "1 hour timeframe" },
    { key: "2", description: "4 hour timeframe" },
    { key: "3", description: "1 day timeframe" },
    { key: "O", description: "Toggle order book panel" },
    { key: "T", description: "Toggle trade flow panel" },
    { key: "?", description: "Show/hide this help" },
    { key: "Esc", description: "Close dialogs" },
  ],
}: KeyboardShortcutsHelpProps) => {
  if (!isOpen) return null;

  return (
    <motion.div
      className="fixed inset-0 z-50 flex items-center justify-center p-4"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      onClick={onClose}
    >
      {/* Backdrop */}
      <div
        className="absolute inset-0"
        style={{ backgroundColor: "rgba(0, 0, 0, 0.7)" }}
      />

      {/* Modal */}
      <motion.div
        className="relative rounded-xl overflow-hidden max-w-sm w-full"
        style={{
          backgroundColor: fey.bg300,
          border: `1px solid ${fey.border}`,
        }}
        initial={{ scale: 0.95, y: 20 }}
        animate={{ scale: 1, y: 0 }}
        exit={{ scale: 0.95, y: 20 }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div
          className="px-4 py-3 flex items-center gap-2"
          style={{ borderBottom: `1px solid ${fey.border}` }}
        >
          <div
            className="p-1.5 rounded"
            style={{ backgroundColor: fey.skyBlueMuted }}
          >
            <Keyboard className="h-4 w-4" style={{ color: fey.skyBlue }} />
          </div>
          <span
            className="text-sm font-semibold"
            style={{ color: fey.grey100 }}
          >
            Keyboard Shortcuts
          </span>
        </div>

        {/* Shortcuts List */}
        <div className="p-4 space-y-2">
          {shortcuts.map((shortcut, i) => (
            <motion.div
              key={shortcut.key}
              className="flex items-center justify-between"
              initial={{ opacity: 0, x: -10 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: i * 0.03 }}
            >
              <span className="text-sm" style={{ color: fey.grey300 }}>
                {shortcut.description}
              </span>
              <kbd
                className="px-2 py-1 rounded text-xs font-mono font-medium"
                style={{
                  backgroundColor: fey.bg400,
                  color: fey.grey100,
                  border: `1px solid ${fey.border}`,
                }}
              >
                {shortcut.key}
              </kbd>
            </motion.div>
          ))}
        </div>

        {/* Footer */}
        <div
          className="px-4 py-3 text-center"
          style={{
            backgroundColor: fey.bg400,
            borderTop: `1px solid ${fey.border}`,
          }}
        >
          <span className="text-xs" style={{ color: fey.grey500 }}>
            Press <kbd className="px-1.5 py-0.5 rounded text-[10px] font-mono" style={{ backgroundColor: fey.bg300, color: fey.grey300 }}>?</kbd> to toggle • <kbd className="px-1.5 py-0.5 rounded text-[10px] font-mono" style={{ backgroundColor: fey.bg300, color: fey.grey300 }}>Esc</kbd> to close
          </span>
        </div>
      </motion.div>
    </motion.div>
  );
};

// ============================================================================
// View Mode Indicator (shows current mode in corner)
// ============================================================================

interface ViewModeIndicatorProps {
  proMode: boolean;
  className?: string;
}

export const ViewModeIndicator = ({
  proMode,
  className = "",
}: ViewModeIndicatorProps) => {
  return (
    <motion.div
      className={`fixed bottom-4 right-4 z-40 ${className}`}
      initial={{ opacity: 0, scale: 0.8, y: 20 }}
      animate={{ opacity: 1, scale: 1, y: 0 }}
      transition={{ delay: 0.5 }}
    >
      <div
        className="flex items-center gap-2 px-3 py-2 rounded-full shadow-lg"
        style={{
          backgroundColor: proMode ? fey.amberMuted : fey.bg400,
          border: `1px solid ${proMode ? "rgba(245, 165, 36, 0.3)" : fey.border}`,
        }}
      >
        {proMode ? (
          <Eye className="h-4 w-4" style={{ color: fey.amber }} />
        ) : (
          <EyeOff className="h-4 w-4" style={{ color: fey.grey500 }} />
        )}
        <span
          className="text-xs font-medium"
          style={{ color: proMode ? fey.amber : fey.grey500 }}
        >
          {proMode ? "Pro View" : "Simple View"}
        </span>
        <kbd
          className="px-1.5 py-0.5 rounded text-[10px] font-mono"
          style={{
            backgroundColor: proMode ? "rgba(245, 165, 36, 0.2)" : fey.bg300,
            color: proMode ? fey.amber : fey.grey500,
          }}
        >
          P
        </kbd>
      </div>
    </motion.div>
  );
};

export default ProModeToggle;
