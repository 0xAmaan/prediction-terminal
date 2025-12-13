/**
 * Framer Motion shared animation variants and utilities
 * These define the choreographed motion language for the trading terminal
 */

import type { Variants, Transition } from "framer-motion";

// ============================================================================
// Fey Design Tokens for Motion
// ============================================================================

export const feyMotion = {
  // Timing
  fast: 0.15,
  medium: 0.25,
  slow: 0.4,

  // Spring presets
  snappy: { type: "spring", stiffness: 400, damping: 30 } as Transition,
  smooth: { type: "spring", stiffness: 300, damping: 35 } as Transition,
  bouncy: { type: "spring", stiffness: 500, damping: 25 } as Transition,
  gentle: { type: "spring", stiffness: 200, damping: 30 } as Transition,
};

// ============================================================================
// Page Load Choreography
// ============================================================================

export const staggerContainer: Variants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.05,
      delayChildren: 0.1,
    },
  },
};

// Page load choreography - orchestrated sequence
export const pageLoadChoreography: Variants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.08,
      delayChildren: 0.1,
      when: "beforeChildren",
    },
  },
};

export const staggerItem: Variants = {
  hidden: { opacity: 0, y: 12 },
  visible: {
    opacity: 1,
    y: 0,
    transition: feyMotion.smooth,
  },
};

// ============================================================================
// Panel Animations
// ============================================================================

export const fadeIn: Variants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: { duration: feyMotion.medium },
  },
  exit: {
    opacity: 0,
    transition: { duration: feyMotion.fast },
  },
};

export const slideFromLeft: Variants = {
  hidden: { opacity: 0, x: -20 },
  visible: {
    opacity: 1,
    x: 0,
    transition: feyMotion.smooth,
  },
  exit: {
    opacity: 0,
    x: -20,
    transition: { duration: feyMotion.fast },
  },
};

export const slideFromRight: Variants = {
  hidden: { opacity: 0, x: 20 },
  visible: {
    opacity: 1,
    x: 0,
    transition: feyMotion.smooth,
  },
  exit: {
    opacity: 0,
    x: 20,
    transition: { duration: feyMotion.fast },
  },
};

export const slideFromBottom: Variants = {
  hidden: { opacity: 0, y: 20 },
  visible: {
    opacity: 1,
    y: 0,
    transition: feyMotion.smooth,
  },
  exit: {
    opacity: 0,
    y: 20,
    transition: { duration: feyMotion.fast },
  },
};

export const scaleIn: Variants = {
  hidden: { opacity: 0, scale: 0.95 },
  visible: {
    opacity: 1,
    scale: 1,
    transition: feyMotion.snappy,
  },
  exit: {
    opacity: 0,
    scale: 0.95,
    transition: { duration: feyMotion.fast },
  },
};

// ============================================================================
// Market Bar Animation (slides up from bottom)
// ============================================================================

export const marketBarVariants: Variants = {
  hidden: { y: "100%", opacity: 0 },
  visible: {
    y: 0,
    opacity: 1,
    transition: {
      type: "spring",
      stiffness: 300,
      damping: 30,
      delay: 0.1,
    },
  },
};

// ============================================================================
// Real-time Data Updates
// ============================================================================

export const priceFlash: Variants = {
  initial: { backgroundColor: "rgba(0, 0, 0, 0)" },
  flash: (isPositive: boolean) => ({
    backgroundColor: isPositive
      ? ["rgba(0, 0, 0, 0)", "rgba(77, 190, 149, 0.2)", "rgba(0, 0, 0, 0)"]
      : ["rgba(0, 0, 0, 0)", "rgba(216, 79, 104, 0.2)", "rgba(0, 0, 0, 0)"],
    transition: { duration: 0.6, times: [0, 0.3, 1] },
  }),
};

export const tradeEntry: Variants = {
  hidden: { opacity: 0, scale: 0.8, x: 20 },
  visible: {
    opacity: 1,
    scale: 1,
    x: 0,
    transition: feyMotion.bouncy,
  },
  exit: {
    opacity: 0,
    scale: 0.8,
    transition: { duration: 0.2 },
  },
};

export const bubblePop: Variants = {
  hidden: { opacity: 0, scale: 0 },
  visible: {
    opacity: 1,
    scale: 1,
    transition: {
      type: "spring",
      stiffness: 500,
      damping: 20,
    },
  },
  whale: {
    opacity: 1,
    scale: [1, 1.15, 1],
    transition: { duration: 0.4 },
  },
};

// ============================================================================
// Order Book Animations
// ============================================================================

export const orderBookLevel: Variants = {
  hidden: { opacity: 0, scaleX: 0 },
  visible: {
    opacity: 1,
    scaleX: 1,
    transition: { duration: feyMotion.medium },
  },
  update: {
    transition: { duration: feyMotion.fast },
  },
};

export const wallPulse: Variants = {
  idle: { boxShadow: "0 0 0 0 rgba(77, 190, 149, 0)" },
  pulse: {
    boxShadow: [
      "0 0 0 0 rgba(77, 190, 149, 0)",
      "0 0 8px 2px rgba(77, 190, 149, 0.3)",
      "0 0 0 0 rgba(77, 190, 149, 0)",
    ],
    transition: { duration: 1.5, repeat: Infinity },
  },
};

// ============================================================================
// Interactive Feedback
// ============================================================================

export const hoverLift = {
  scale: 1.01,
  y: -1,
  transition: feyMotion.snappy,
};

export const tapShrink = {
  scale: 0.98,
  transition: { duration: 0.1 },
};

export const buttonVariants: Variants = {
  idle: { scale: 1 },
  hover: {
    scale: 1.02,
    transition: feyMotion.snappy,
  },
  tap: {
    scale: 0.98,
    transition: { duration: 0.1 },
  },
};

// ============================================================================
// Collapse/Expand Animations
// ============================================================================

export const collapseVariants: Variants = {
  expanded: {
    width: "auto",
    opacity: 1,
    transition: feyMotion.smooth,
  },
  collapsed: {
    width: 0,
    opacity: 0,
    transition: { duration: feyMotion.medium },
  },
};

export const accordionVariants: Variants = {
  collapsed: {
    height: 0,
    opacity: 0,
    transition: { duration: feyMotion.medium },
  },
  expanded: {
    height: "auto",
    opacity: 1,
    transition: {
      height: { duration: feyMotion.medium },
      opacity: { duration: feyMotion.medium, delay: 0.1 },
    },
  },
};

// ============================================================================
// Loading States
// ============================================================================

export const shimmer: Variants = {
  animate: {
    backgroundPosition: ["200% 0", "-200% 0"],
    transition: {
      duration: 1.5,
      repeat: Infinity,
      ease: "linear",
    },
  },
};

export const pulse: Variants = {
  animate: {
    opacity: [0.5, 1, 0.5],
    transition: {
      duration: 1.5,
      repeat: Infinity,
      ease: "easeInOut",
    },
  },
};

// ============================================================================
// Utility: Stagger delay calculator
// ============================================================================

export const getStaggerDelay = (index: number, baseDelay = 0.05): number => {
  return index * baseDelay;
};

// ============================================================================
// Utility: Reduced motion wrapper
// ============================================================================

export const getReducedMotionVariants = (variants: Variants): Variants => {
  // For users who prefer reduced motion, simplify to instant transitions
  const reduced: Variants = {};

  for (const [key, value] of Object.entries(variants)) {
    if (typeof value === "object" && value !== null) {
      reduced[key] = {
        ...value,
        transition: { duration: 0 },
      };
    }
  }

  return reduced;
};
