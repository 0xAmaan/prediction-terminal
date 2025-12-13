"use client";

import { useState, useMemo } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  Brain,
  Newspaper,
  TrendingUp,
  Sparkles,
  ExternalLink,
  Clock,
  AlertTriangle,
  Info,
} from "lucide-react";
import { SentimentGauge, MiniSentiment } from "./sentiment-gauge";
import type { MarketSentiment } from "@/hooks/use-market-sentiment";

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
  skyBlueMuted: "rgba(84, 187, 247, 0.15)",
  amber: "#F5A524",
  amberMuted: "rgba(245, 165, 36, 0.15)",
  border: "rgba(255, 255, 255, 0.06)",
};

// ============================================================================
// Types
// ============================================================================

type TabType = "sentiment" | "news" | "ai";

interface IntelligencePanelProps {
  sentiment: MarketSentiment;
  marketTitle?: string;
  platform?: string;
  showDetails?: boolean;
  className?: string;
}

interface NewsItem {
  id: string;
  title: string;
  source: string;
  timestamp: Date;
  url?: string;
  sentiment?: "bullish" | "bearish" | "neutral";
}

interface AIInsight {
  id: string;
  type: "summary" | "anomaly" | "correlation";
  title: string;
  description: string;
  confidence: number;
  timestamp: Date;
}

// ============================================================================
// Tab Button Component
// ============================================================================

interface TabButtonProps {
  tab: TabType;
  activeTab: TabType;
  icon: React.ElementType;
  label: string;
  onClick: () => void;
  badge?: number;
}

const TabButton = ({
  tab,
  activeTab,
  icon: Icon,
  label,
  onClick,
  badge,
}: TabButtonProps) => {
  const isActive = tab === activeTab;

  return (
    <button
      onClick={onClick}
      className="relative flex items-center gap-1.5 px-3 py-2 rounded-md transition-colors"
      style={{
        backgroundColor: isActive ? fey.bg400 : "transparent",
        color: isActive ? fey.grey100 : fey.grey500,
      }}
    >
      <Icon className="h-3.5 w-3.5" />
      <span className="text-xs font-medium">{label}</span>
      {badge !== undefined && badge > 0 && (
        <span
          className="ml-1 px-1.5 py-0.5 rounded-full text-[9px] font-medium"
          style={{
            backgroundColor: fey.skyBlueMuted,
            color: fey.skyBlue,
          }}
        >
          {badge}
        </span>
      )}
      {isActive && (
        <motion.div
          className="absolute bottom-0 left-0 right-0 h-0.5 rounded-full"
          style={{ backgroundColor: fey.skyBlue }}
          layoutId="activeTab"
          transition={{ type: "spring", stiffness: 300, damping: 30 }}
        />
      )}
    </button>
  );
};

// ============================================================================
// News Feed Tab Content
// ============================================================================

const NewsFeedContent = ({ marketTitle }: { marketTitle?: string }) => {
  // Placeholder news items - in production, these would come from an API
  const placeholderNews: NewsItem[] = useMemo(
    () => [
      {
        id: "1",
        title: "Market activity picking up ahead of deadline",
        source: "Platform",
        timestamp: new Date(Date.now() - 1000 * 60 * 30),
        sentiment: "bullish",
      },
      {
        id: "2",
        title: "New liquidity providers entering market",
        source: "Analytics",
        timestamp: new Date(Date.now() - 1000 * 60 * 60 * 2),
        sentiment: "neutral",
      },
    ],
    []
  );

  return (
    <div className="p-4">
      {/* Coming soon state with some placeholder items */}
      <div className="space-y-3">
        {placeholderNews.map((item, i) => (
          <motion.div
            key={item.id}
            className="p-3 rounded-lg"
            style={{ backgroundColor: fey.bg400 }}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.1 }}
          >
            <div className="flex items-start justify-between gap-2">
              <div className="flex-1">
                <h4
                  className="text-sm font-medium leading-tight"
                  style={{ color: fey.grey100 }}
                >
                  {item.title}
                </h4>
                <div
                  className="flex items-center gap-2 mt-1 text-[10px]"
                  style={{ color: fey.grey500 }}
                >
                  <span>{item.source}</span>
                  <span>•</span>
                  <span className="flex items-center gap-1">
                    <Clock className="h-3 w-3" />
                    {formatTimeAgo(item.timestamp)}
                  </span>
                </div>
              </div>
              {item.sentiment && (
                <div
                  className="px-1.5 py-0.5 rounded text-[9px] uppercase font-medium"
                  style={{
                    backgroundColor:
                      item.sentiment === "bullish"
                        ? fey.tealMuted
                        : item.sentiment === "bearish"
                          ? fey.redMuted
                          : "rgba(125, 139, 150, 0.15)",
                    color:
                      item.sentiment === "bullish"
                        ? fey.teal
                        : item.sentiment === "bearish"
                          ? fey.red
                          : fey.grey500,
                  }}
                >
                  {item.sentiment}
                </div>
              )}
            </div>
          </motion.div>
        ))}

        {/* Coming soon message */}
        <div
          className="mt-4 p-4 rounded-lg text-center"
          style={{
            backgroundColor: fey.bg400,
            border: `1px dashed ${fey.border}`,
          }}
        >
          <Newspaper
            className="h-8 w-8 mx-auto mb-2 opacity-30"
            style={{ color: fey.grey500 }}
          />
          <p className="text-sm font-medium" style={{ color: fey.grey300 }}>
            More news sources coming soon
          </p>
          <p className="text-xs mt-1" style={{ color: fey.grey500 }}>
            Real-time headlines and market-moving events
          </p>
        </div>
      </div>
    </div>
  );
};

// ============================================================================
// AI Insights Tab Content
// ============================================================================

const AIInsightsContent = ({ sentiment }: { sentiment: MarketSentiment }) => {
  // Generate dynamic insights based on sentiment data
  const insights: AIInsight[] = useMemo(() => {
    const result: AIInsight[] = [];

    // Market summary insight
    result.push({
      id: "summary",
      type: "summary",
      title: "Market Overview",
      description: `Market sentiment is ${sentiment.label.toLowerCase()} with ${(sentiment.confidence * 100).toFixed(0)}% confidence. ${
        sentiment.score > 20
          ? "Buyers are showing strong interest."
          : sentiment.score < -20
            ? "Sellers are dominating the market."
            : "Trading activity is balanced."
      }`,
      confidence: sentiment.confidence,
      timestamp: new Date(),
    });

    // Anomaly detection (if signals suggest unusual activity)
    if (sentiment.signals.some((s) => s.source === "Whale Activity")) {
      result.push({
        id: "whale",
        type: "anomaly",
        title: "Large Order Detected",
        description:
          "A significant order has been placed that may indicate institutional interest or informed trading.",
        confidence: 0.75,
        timestamp: new Date(),
      });
    }

    // Correlation insight
    if (Math.abs(sentiment.components.orderBookImbalance) > 30) {
      result.push({
        id: "correlation",
        type: "correlation",
        title: "Order Book Signal",
        description: `${sentiment.components.orderBookImbalance > 0 ? "Strong bid" : "Strong ask"}-side liquidity suggests ${sentiment.components.orderBookImbalance > 0 ? "buying" : "selling"} pressure may continue.`,
        confidence: 0.65,
        timestamp: new Date(),
      });
    }

    return result;
  }, [sentiment]);

  const getInsightIcon = (type: AIInsight["type"]) => {
    switch (type) {
      case "summary":
        return Info;
      case "anomaly":
        return AlertTriangle;
      case "correlation":
        return TrendingUp;
    }
  };

  const getInsightColor = (type: AIInsight["type"]) => {
    switch (type) {
      case "summary":
        return fey.skyBlue;
      case "anomaly":
        return fey.amber;
      case "correlation":
        return fey.teal;
    }
  };

  return (
    <div className="p-4 space-y-3">
      {insights.map((insight, i) => {
        const Icon = getInsightIcon(insight.type);
        const color = getInsightColor(insight.type);

        return (
          <motion.div
            key={insight.id}
            className="p-3 rounded-lg"
            style={{ backgroundColor: fey.bg400 }}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.1 }}
          >
            <div className="flex items-start gap-3">
              <div
                className="p-1.5 rounded"
                style={{ backgroundColor: `${color}15` }}
              >
                <Icon className="h-4 w-4" style={{ color }} />
              </div>
              <div className="flex-1">
                <div className="flex items-center justify-between">
                  <h4
                    className="text-sm font-medium"
                    style={{ color: fey.grey100 }}
                  >
                    {insight.title}
                  </h4>
                  <span
                    className="text-[10px] font-mono"
                    style={{ color: fey.grey500 }}
                  >
                    {(insight.confidence * 100).toFixed(0)}%
                  </span>
                </div>
                <p
                  className="text-xs mt-1 leading-relaxed"
                  style={{ color: fey.grey300 }}
                >
                  {insight.description}
                </p>
              </div>
            </div>
          </motion.div>
        );
      })}

      {/* AI disclaimer */}
      <div
        className="flex items-start gap-2 p-3 rounded-lg text-[10px]"
        style={{
          backgroundColor: fey.amberMuted,
          color: fey.amber,
        }}
      >
        <Sparkles className="h-3.5 w-3.5 flex-shrink-0 mt-0.5" />
        <span>
          AI-generated insights based on market data. Not financial advice.
          Always do your own research.
        </span>
      </div>
    </div>
  );
};

// ============================================================================
// Helpers
// ============================================================================

const formatTimeAgo = (date: Date): string => {
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);

  if (minutes < 1) return "just now";
  if (minutes < 60) return `${minutes}m ago`;
  if (hours < 24) return `${hours}h ago`;
  return `${Math.floor(hours / 24)}d ago`;
};

// ============================================================================
// Main Intelligence Panel Component
// ============================================================================

export const IntelligencePanel = ({
  sentiment,
  marketTitle,
  platform,
  showDetails = true,
  className = "",
}: IntelligencePanelProps) => {
  const [activeTab, setActiveTab] = useState<TabType>("sentiment");

  return (
    <div
      className={`rounded-lg overflow-hidden ${className}`}
      style={{
        backgroundColor: fey.bg300,
        border: `1px solid ${fey.border}`,
      }}
    >
      {/* Header */}
      <div
        className="px-4 py-3 flex items-center justify-between"
        style={{ borderBottom: `1px solid ${fey.border}` }}
      >
        <div className="flex items-center gap-2">
          <div
            className="p-1.5 rounded"
            style={{ backgroundColor: fey.skyBlueMuted }}
          >
            <Brain className="h-4 w-4" style={{ color: fey.skyBlue }} />
          </div>
          <span
            className="text-sm font-semibold"
            style={{ color: fey.grey100, letterSpacing: "-0.02em" }}
          >
            Intelligence
          </span>
        </div>

        {/* Mini sentiment badge in header */}
        <MiniSentiment score={sentiment.score} showLabel={false} />
      </div>

      {/* Tabs */}
      <div
        className="px-2 py-2 flex items-center gap-1"
        style={{ borderBottom: `1px solid ${fey.border}` }}
      >
        <TabButton
          tab="sentiment"
          activeTab={activeTab}
          icon={TrendingUp}
          label="Sentiment"
          onClick={() => setActiveTab("sentiment")}
        />
        <TabButton
          tab="news"
          activeTab={activeTab}
          icon={Newspaper}
          label="News"
          onClick={() => setActiveTab("news")}
          badge={2}
        />
        <TabButton
          tab="ai"
          activeTab={activeTab}
          icon={Sparkles}
          label="AI"
          onClick={() => setActiveTab("ai")}
        />
      </div>

      {/* Tab Content */}
      <AnimatePresence mode="wait">
        <motion.div
          key={activeTab}
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -10 }}
          transition={{ duration: 0.15 }}
        >
          {activeTab === "sentiment" && (
            <div className="p-4">
              <SentimentGauge
                sentiment={sentiment}
                showDetails={showDetails}
                className="border-0"
              />
            </div>
          )}
          {activeTab === "news" && <NewsFeedContent marketTitle={marketTitle} />}
          {activeTab === "ai" && <AIInsightsContent sentiment={sentiment} />}
        </motion.div>
      </AnimatePresence>
    </div>
  );
};

// ============================================================================
// Compact Intelligence Strip (for minimal displays)
// ============================================================================

interface IntelligenceStripProps {
  sentiment: MarketSentiment;
  className?: string;
}

export const IntelligenceStrip = ({
  sentiment,
  className = "",
}: IntelligenceStripProps) => {
  return (
    <div
      className={`flex items-center gap-4 px-4 py-2 rounded-lg ${className}`}
      style={{ backgroundColor: fey.bg400 }}
    >
      <div className="flex items-center gap-2">
        <Brain className="h-4 w-4" style={{ color: fey.skyBlue }} />
        <span className="text-xs font-medium" style={{ color: fey.grey100 }}>
          Sentiment
        </span>
      </div>
      <MiniSentiment score={sentiment.score} />
      <div className="flex-1" />
      <span className="text-[10px]" style={{ color: fey.grey500 }}>
        {(sentiment.confidence * 100).toFixed(0)}% confidence
      </span>
    </div>
  );
};

export default IntelligencePanel;
