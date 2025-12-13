"use client";

import { useMemo, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { BookOpen, Layers, AlertTriangle, Copy, Check } from "lucide-react";
import { ImbalanceMeter } from "./imbalance-meter";
import {
  type OrderBookLevel,
  type ProcessedLevel,
  calculateMetrics,
  processLevels,
  getHeatmapColor,
  formatPrice,
  formatQuantity,
  formatSpread,
} from "@/lib/orderbook-utils";

// Fey color tokens
const fey = {
  bg300: "#131419",
  bg400: "#16181C",
  grey100: "#EEF0F1",
  grey300: "#B6BEC4",
  grey500: "#7D8B96",
  teal: "#4DBE95",
  tealMuted: "rgba(77, 190, 149, 0.15)",
  red: "#D84F68",
  redMuted: "rgba(216, 79, 104, 0.15)",
  skyBlue: "#54BBF7",
  border: "rgba(255, 255, 255, 0.06)",
};

// ============================================================================
// Types
// ============================================================================

interface OrderBookV2Props {
  yesBids: OrderBookLevel[];
  yesAsks: OrderBookLevel[];
  isLoading?: boolean;
  maxLevels?: number;
  showHeatmap?: boolean;
  showImbalance?: boolean;
  showWalls?: boolean;
  proMode?: boolean;
}

// ============================================================================
// Heatmap Level Component
// ============================================================================

interface HeatmapLevelProps {
  level: ProcessedLevel;
  isBid: boolean;
  maxQuantity: number;
  showHeatmap: boolean;
  showWalls: boolean;
  onCopyPrice: (price: number) => void;
}

const HeatmapLevel = ({
  level,
  isBid,
  maxQuantity,
  showHeatmap,
  showWalls,
  onCopyPrice,
}: HeatmapLevelProps) => {
  const [isHovered, setIsHovered] = useState(false);
  const [copied, setCopied] = useState(false);

  const handleCopyPrice = () => {
    onCopyPrice(level.priceNum);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  // Calculate depth bar width
  const depthPercent = (level.quantityNum / maxQuantity) * 100;

  // Get colors
  const bgColor = showHeatmap
    ? getHeatmapColor(level.intensity, isBid, level.isWall)
    : isBid
      ? fey.tealMuted
      : fey.redMuted;

  const textColor = isBid ? fey.teal : fey.red;

  return (
    <motion.div
      className="relative grid grid-cols-3 gap-2 px-2 py-1 text-xs cursor-pointer"
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      onClick={handleCopyPrice}
      initial={{ opacity: 0, x: isBid ? -10 : 10 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.2 }}
      whileHover={{ backgroundColor: fey.bg400 }}
    >
      {/* Depth Bar Background */}
      <motion.div
        className="absolute inset-y-0 rounded-sm"
        style={{
          [isBid ? "right" : "left"]: 0,
          backgroundColor: bgColor,
        }}
        initial={{ width: 0 }}
        animate={{ width: `${depthPercent}%` }}
        transition={{ type: "spring", stiffness: 100, damping: 20 }}
      />

      {/* Wall Indicator */}
      {showWalls && level.isWall && (
        <motion.div
          className="absolute left-1 top-1/2 -translate-y-1/2"
          initial={{ scale: 0, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          transition={{ type: "spring", stiffness: 300, damping: 20 }}
        >
          <div
            className="p-0.5 rounded"
            style={{ backgroundColor: `${isBid ? fey.teal : fey.red}30` }}
          >
            <Layers
              className="h-2.5 w-2.5"
              style={{ color: isBid ? fey.teal : fey.red }}
            />
          </div>
        </motion.div>
      )}

      {/* Price */}
      <div
        className={`relative z-10 font-mono font-medium flex items-center gap-1 ${
          showWalls && level.isWall ? "pl-5" : ""
        }`}
        style={{ color: textColor }}
      >
        {formatPrice(level.priceNum)}
        {isHovered && (
          <motion.div
            initial={{ opacity: 0, scale: 0.8 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.8 }}
          >
            {copied ? (
              <Check className="h-3 w-3" style={{ color: fey.teal }} />
            ) : (
              <Copy className="h-3 w-3" style={{ color: fey.grey500 }} />
            )}
          </motion.div>
        )}
      </div>

      {/* Quantity */}
      <div
        className="relative z-10 text-right font-mono"
        style={{ color: fey.grey100 }}
      >
        {formatQuantity(level.quantityNum)}
      </div>

      {/* Order Count / Cumulative */}
      <div
        className="relative z-10 text-right font-mono"
        style={{ color: fey.grey500 }}
      >
        {isHovered
          ? formatQuantity(level.cumulativeQuantity)
          : level.order_count ?? "—"}
      </div>
    </motion.div>
  );
};

// ============================================================================
// Order Book Side Component
// ============================================================================

interface OrderBookSideProps {
  levels: ProcessedLevel[];
  isBid: boolean;
  maxQuantity: number;
  showHeatmap: boolean;
  showWalls: boolean;
  onCopyPrice: (price: number) => void;
}

const OrderBookSide = ({
  levels,
  isBid,
  maxQuantity,
  showHeatmap,
  showWalls,
  onCopyPrice,
}: OrderBookSideProps) => {
  return (
    <div className="space-y-0.5">
      <AnimatePresence mode="popLayout">
        {levels.map((level, i) => (
          <HeatmapLevel
            key={`${level.price}-${i}`}
            level={level}
            isBid={isBid}
            maxQuantity={maxQuantity}
            showHeatmap={showHeatmap}
            showWalls={showWalls}
            onCopyPrice={onCopyPrice}
          />
        ))}
      </AnimatePresence>
    </div>
  );
};

// ============================================================================
// Loading Skeleton
// ============================================================================

const OrderBookSkeleton = ({ levels = 5 }: { levels?: number }) => (
  <div className="space-y-1">
    {Array.from({ length: levels }).map((_, i) => (
      <div key={i} className="grid grid-cols-3 gap-2 px-2 py-1">
        <div
          className="h-4 rounded animate-pulse"
          style={{ backgroundColor: fey.bg400 }}
        />
        <div
          className="h-4 rounded animate-pulse"
          style={{ backgroundColor: fey.bg400 }}
        />
        <div
          className="h-4 rounded animate-pulse"
          style={{ backgroundColor: fey.bg400 }}
        />
      </div>
    ))}
  </div>
);

// ============================================================================
// Main Order Book V2 Component
// ============================================================================

export const OrderBookV2 = ({
  yesBids,
  yesAsks,
  isLoading = false,
  maxLevels = 10,
  showHeatmap = true,
  showImbalance = true,
  showWalls = true,
  proMode = false,
}: OrderBookV2Props) => {
  // Calculate metrics
  const metrics = useMemo(
    () => calculateMetrics(yesBids, yesAsks),
    [yesBids, yesAsks],
  );

  // Process levels with heatmap intensity
  const processedBids = useMemo(
    () => processLevels(yesBids, metrics, true).slice(0, maxLevels),
    [yesBids, metrics, maxLevels],
  );

  const processedAsks = useMemo(
    () => processLevels(yesAsks, metrics, false).slice(0, maxLevels),
    [yesAsks, metrics, maxLevels],
  );

  // Copy price to clipboard
  const handleCopyPrice = (price: number) => {
    navigator.clipboard.writeText(formatPrice(price));
  };

  // Count walls
  const wallCount =
    processedBids.filter((l) => l.isWall).length +
    processedAsks.filter((l) => l.isWall).length;

  if (isLoading) {
    return (
      <div
        className="rounded-lg overflow-hidden"
        style={{
          backgroundColor: fey.bg300,
          border: `1px solid ${fey.border}`,
        }}
      >
        <div className="p-4">
          <div className="flex items-center gap-2 mb-4">
            <BookOpen className="h-4 w-4" style={{ color: fey.skyBlue }} />
            <span
              className="text-sm font-semibold"
              style={{ color: fey.grey100 }}
            >
              Order Book
            </span>
          </div>
          <OrderBookSkeleton levels={maxLevels} />
        </div>
      </div>
    );
  }

  return (
    <div
      className="rounded-lg overflow-hidden h-full flex flex-col"
      style={{
        backgroundColor: fey.bg300,
        border: `1px solid ${fey.border}`,
      }}
    >
      {/* Header */}
      <div
        className="p-4 flex items-center justify-between shrink-0"
        style={{ borderBottom: `1px solid ${fey.border}` }}
      >
        <div className="flex items-center gap-2">
          <div
            className="p-1.5 rounded"
            style={{ backgroundColor: "rgba(84, 187, 247, 0.1)" }}
          >
            <BookOpen className="h-4 w-4" style={{ color: fey.skyBlue }} />
          </div>
          <span
            className="text-sm font-semibold"
            style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
          >
            Order Book
          </span>
        </div>

        <div className="flex items-center gap-3">
          {/* Wall indicator */}
          {showWalls && wallCount > 0 && (
            <div className="flex items-center gap-1">
              <AlertTriangle className="h-3 w-3" style={{ color: fey.grey500 }} />
              <span className="text-[10px]" style={{ color: fey.grey500 }}>
                {wallCount} wall{wallCount > 1 ? "s" : ""}
              </span>
            </div>
          )}

          {/* Spread */}
          <div
            className="px-2 py-0.5 rounded text-xs font-mono"
            style={{ backgroundColor: fey.bg400, color: fey.grey500 }}
          >
            {formatSpread(metrics.spread)} spread
          </div>
        </div>
      </div>

      {/* Imbalance Meter */}
      {showImbalance && (
        <div className="px-4 pt-3 shrink-0">
          <ImbalanceMeter
            imbalanceRatio={metrics.imbalanceRatio}
            bidQuantity={metrics.totalBidQty}
            askQuantity={metrics.totalAskQty}
            showDetails={proMode}
          />
        </div>
      )}

      {/* Order Book Content */}
      <div className="p-4 pt-3 flex-1 min-h-0 overflow-y-auto">
        {/* Column Headers */}
        <div
          className="grid grid-cols-3 gap-2 px-2 pb-2 text-[10px] uppercase tracking-wider font-medium"
          style={{ color: fey.grey500 }}
        >
          <div>Price</div>
          <div className="text-right">Quantity</div>
          <div className="text-right">Orders</div>
        </div>

        {/* Asks (reversed so lowest ask is at bottom, near spread) */}
        <div className="mb-1">
          <OrderBookSide
            levels={[...processedAsks].reverse()}
            isBid={false}
            maxQuantity={metrics.maxQuantity}
            showHeatmap={showHeatmap}
            showWalls={showWalls}
            onCopyPrice={handleCopyPrice}
          />
        </div>

        {/* Spread Divider */}
        <div
          className="flex items-center gap-2 py-2 my-1"
          style={{ borderTop: `1px solid ${fey.border}`, borderBottom: `1px solid ${fey.border}` }}
        >
          <span
            className="text-xs font-mono font-medium"
            style={{ color: fey.teal }}
          >
            {formatPrice(processedBids[0]?.priceNum ?? 0)}
          </span>
          <div className="flex-1 h-px" style={{ backgroundColor: fey.border }} />
          <span className="text-[10px]" style={{ color: fey.grey500 }}>
            Mid: {formatPrice(metrics.midPrice)}
          </span>
          <div className="flex-1 h-px" style={{ backgroundColor: fey.border }} />
          <span
            className="text-xs font-mono font-medium"
            style={{ color: fey.red }}
          >
            {formatPrice(processedAsks[0]?.priceNum ?? 1)}
          </span>
        </div>

        {/* Bids */}
        <div className="mt-1">
          <OrderBookSide
            levels={processedBids}
            isBid={true}
            maxQuantity={metrics.maxQuantity}
            showHeatmap={showHeatmap}
            showWalls={showWalls}
            onCopyPrice={handleCopyPrice}
          />
        </div>
      </div>

      {/* Footer - Quick Stats */}
      {proMode && (
        <div
          className="px-4 py-3 flex items-center justify-between text-[10px] shrink-0"
          style={{ borderTop: `1px solid ${fey.border}`, backgroundColor: fey.bg400 }}
        >
          <div className="flex items-center gap-4">
            <div>
              <span style={{ color: fey.grey500 }}>Total Bids: </span>
              <span className="font-mono" style={{ color: fey.teal }}>
                {formatQuantity(metrics.totalBidQty)}
              </span>
            </div>
            <div>
              <span style={{ color: fey.grey500 }}>Total Asks: </span>
              <span className="font-mono" style={{ color: fey.red }}>
                {formatQuantity(metrics.totalAskQty)}
              </span>
            </div>
          </div>
          <div>
            <span style={{ color: fey.grey500 }}>Ratio: </span>
            <span className="font-mono" style={{ color: fey.grey100 }}>
              {metrics.bidAskRatio.toFixed(2)}
            </span>
          </div>
        </div>
      )}
    </div>
  );
};

export default OrderBookV2;
