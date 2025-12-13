"use client";

import { useState, useEffect } from "react";

// ============================================================================
// Pro Mode Hook (simplified - always returns true)
// ============================================================================

interface UseProModeReturn {
  proMode: boolean;
}

export const useProMode = (): UseProModeReturn => {
  // Pro mode is now always enabled
  return { proMode: true };
};

// ============================================================================
// Pro Mode Features
// ============================================================================

export interface ProModeFeatures {
  // Order Book
  showHeatmap: boolean;
  showImbalanceMeter: boolean;
  showWallDetection: boolean;
  showOrderBookDepth: number; // 3 for casual, 10 for pro

  // Trade Flow
  showMomentumGauge: boolean;
  showPressureBar: boolean;
  showBubbleTimeline: boolean;

  // Intelligence
  showSentimentGauge: boolean;
  showNewsFeed: boolean;
  showAIInsights: boolean;

  // UI
  expandedPanels: boolean;
  keyboardShortcuts: boolean;
}

export const getProModeFeatures = (proMode: boolean): ProModeFeatures => {
  if (proMode) {
    return {
      // Order Book - Full features
      showHeatmap: true,
      showImbalanceMeter: true,
      showWallDetection: true,
      showOrderBookDepth: 10,

      // Trade Flow - Full features
      showMomentumGauge: true,
      showPressureBar: true,
      showBubbleTimeline: true,

      // Intelligence - Full features
      showSentimentGauge: true,
      showNewsFeed: true,
      showAIInsights: true,

      // UI
      expandedPanels: true,
      keyboardShortcuts: true,
    };
  }

  // Casual mode - simplified view
  return {
    // Order Book - Basic
    showHeatmap: false,
    showImbalanceMeter: false,
    showWallDetection: false,
    showOrderBookDepth: 3,

    // Trade Flow - Hidden
    showMomentumGauge: false,
    showPressureBar: false,
    showBubbleTimeline: false,

    // Intelligence - Basic
    showSentimentGauge: false,
    showNewsFeed: false,
    showAIInsights: false,

    // UI
    expandedPanels: false,
    keyboardShortcuts: false,
  };
};

// ============================================================================
// Keyboard Shortcuts Hook
// ============================================================================

interface KeyboardShortcut {
  key: string;
  description: string;
  action: () => void;
}

export const useKeyboardShortcuts = (
  shortcuts: KeyboardShortcut[],
  enabled: boolean = true,
) => {
  const [showHelp, setShowHelp] = useState(false);

  useEffect(() => {
    if (!enabled) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      // Don't trigger if user is typing in an input
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement
      ) {
        return;
      }

      // Show help with ?
      if (e.key === "?" && !e.metaKey && !e.ctrlKey) {
        e.preventDefault();
        setShowHelp((prev) => !prev);
        return;
      }

      // Close help with Escape
      if (e.key === "Escape" && showHelp) {
        setShowHelp(false);
        return;
      }

      // Find matching shortcut
      const shortcut = shortcuts.find(
        (s) => s.key.toLowerCase() === e.key.toLowerCase(),
      );

      if (shortcut && !e.metaKey && !e.ctrlKey && !e.altKey) {
        e.preventDefault();
        shortcut.action();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [enabled, shortcuts, showHelp]);

  return { showHelp, setShowHelp };
};

export default useProMode;
