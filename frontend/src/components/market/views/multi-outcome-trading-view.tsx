"use client";

import { useMemo, useEffect } from "react";
import { useQuery } from "@tanstack/react-query";
import type { PredictionMarket, MarketOption } from "@/lib/types";
import { api } from "@/lib/api";

// Components
import { PriceChart } from "@/components/market/price-chart";
import { OrderBookV2 } from "@/components/market/orderbook/order-book-v2";
import { TradeExecution } from "@/components/market/trade-execution";
import { LiveTradesTable } from "@/components/market/live-trades-table";
import { MarketInfoPanel } from "@/components/market/market-info-panel";
import { MultiOutcomeTradingHeader } from "@/components/market/layout/multi-outcome-trading-header";

// Fey color tokens
const fey = {
  bg100: "#070709",
  bg200: "#101116",
  bg300: "#131419",
  bg400: "#16181C",
  grey100: "#EEF0F1",
  grey300: "#B6BEC4",
  grey500: "#7D8B96",
  teal: "#4DBE95",
  red: "#D84F68",
  skyBlue: "#54BBF7",
  border: "rgba(255, 255, 255, 0.06)",
};

// ============================================================================
// Types
// ============================================================================

export interface MultiOutcomeTradingViewProps {
  market: PredictionMarket;
  options: MarketOption[];
  selectedOutcome: MarketOption | null;
  onOutcomeSelect: (outcome: MarketOption) => void;
  relatedMarkets: PredictionMarket[];
  relatedMarketsLoading: boolean;
}

// ============================================================================
// Main Component
// ============================================================================

export const MultiOutcomeTradingView = ({
  market,
  options,
  selectedOutcome,
  onOutcomeSelect,
}: MultiOutcomeTradingViewProps) => {
  // Get leading outcome for display
  const leadingOutcome = useMemo(() => {
    if (options.length === 0) return null;
    return [...options].sort(
      (a, b) => parseFloat(b.yes_price) - parseFloat(a.yes_price)
    )[0];
  }, [options]);

  // Initialize selected outcome to leading if not set
  useEffect(() => {
    if (!selectedOutcome && leadingOutcome) {
      onOutcomeSelect(leadingOutcome);
    }
  }, [selectedOutcome, leadingOutcome, onOutcomeSelect]);

  // The outcome ID to use for queries (clob_token_id for Polymarket)
  const outcomeId = selectedOutcome?.clob_token_id ?? selectedOutcome?.market_id ?? "";
  const isLeading = selectedOutcome?.market_id === leadingOutcome?.market_id;

  // Fetch orderbook for selected outcome
  const { data: orderBook, isLoading: orderBookLoading } = useQuery({
    queryKey: ["outcome-orderbook", market.platform, market.id, outcomeId],
    queryFn: () => api.getOutcomeOrderBook(market.platform, market.id, outcomeId),
    enabled: !!outcomeId,
    refetchInterval: 5000,
  });

  // Fetch trades for selected outcome
  const { data: tradesData } = useQuery({
    queryKey: ["outcome-trades", market.platform, market.id, outcomeId],
    queryFn: () => api.getOutcomeTrades(market.platform, market.id, outcomeId, 50),
    enabled: !!outcomeId,
    refetchInterval: 10000,
  });

  const trades = tradesData?.trades ?? [];

  // Current price from selected outcome
  const currentPrice = selectedOutcome ? selectedOutcome.yes_price : "0";
  const noPrice = selectedOutcome ? (1 - parseFloat(selectedOutcome.yes_price)).toString() : "0";

  // Calculate spread from order book
  const spread = useMemo(() => {
    if (!orderBook || orderBook.yes_bids.length === 0 || orderBook.yes_asks.length === 0) {
      return null;
    }
    const bestBid = parseFloat(orderBook.yes_bids[0].price);
    const bestAsk = parseFloat(orderBook.yes_asks[0].price);
    return bestAsk - bestBid;
  }, [orderBook]);

  return (
    <div
      className="min-h-full flex flex-col"
      style={{ backgroundColor: fey.bg100 }}
    >
      {/* Header with outcome selector and stats */}
      <MultiOutcomeTradingHeader
        market={market}
        options={options}
        selectedOutcome={selectedOutcome}
        onOutcomeSelect={onOutcomeSelect}
        isLeading={isLeading}
      />

      {/* Main Grid Layout - 3 columns, right column spans full height */}
      <div
        className="flex-1 grid grid-cols-[2fr_1fr_1fr] gap-px"
        style={{ backgroundColor: fey.border }}
      >
        {/* Left 2 columns container */}
        <div className="col-span-2 flex flex-col gap-px" style={{ backgroundColor: fey.border }}>
          {/* Row 1: Chart + Order Book */}
          <div className="grid grid-cols-[2fr_1fr] gap-px h-[50vh]" style={{ backgroundColor: fey.border }}>
            {/* Price Chart */}
            <div
              className="overflow-hidden p-2"
              style={{ backgroundColor: fey.bg100 }}
            >
              {selectedOutcome && (
                <PriceChart
                  platform={market.platform}
                  marketId={outcomeId}
                  currentPrice={parseFloat(currentPrice)}
                  title={`${selectedOutcome.name} - Price History`}
                />
              )}
            </div>

            {/* Order Book */}
            <div
              className="overflow-hidden p-2"
              style={{ backgroundColor: fey.bg100 }}
            >
              <OrderBookV2
                yesBids={orderBook?.yes_bids ?? []}
                yesAsks={orderBook?.yes_asks ?? []}
                isLoading={orderBookLoading}
                maxLevels={10}
                showHeatmap={true}
                showImbalance={true}
                showWalls={true}
                proMode={true}
              />
            </div>
          </div>

          {/* Row 2: Live Trades Table */}
          <div
            className="p-2 h-[75vh]"
            style={{ backgroundColor: fey.bg100 }}
          >
            <LiveTradesTable trades={trades} className="h-full" />
          </div>
        </div>

        {/* Right column: Trade Execution + Market Info (stacked, natural heights) */}
        <div className="flex flex-col gap-px" style={{ backgroundColor: fey.border }}>
          {/* Trade Execution */}
          <div
            className="p-2"
            style={{ backgroundColor: fey.bg100 }}
          >
            <TradeExecution
              yesPrice={currentPrice}
              noPrice={noPrice}
              trades={trades}
              tokenId={selectedOutcome?.clob_token_id}
              marketId={market.id}
              outcome={selectedOutcome?.name || "Yes"}
              marketTitle={selectedOutcome ? `${market.title} - ${selectedOutcome.name}` : market.title}
              negRisk={true}
              bestAsk={orderBook?.yes_asks?.[0] ? parseFloat(orderBook.yes_asks[0].price) : undefined}
              bestBid={orderBook?.yes_bids?.[0] ? parseFloat(orderBook.yes_bids[0].price) : undefined}
            />
          </div>

          {/* Market Info Panel */}
          <div
            className="p-2"
            style={{ backgroundColor: fey.bg100 }}
          >
            <MarketInfoPanel
              market={market}
              yesPrice={currentPrice}
              noPrice={noPrice}
              spread={spread}
              outcomeName={selectedOutcome?.name}
              isLeading={isLeading}
              outcomeCount={options.length}
            />
          </div>
        </div>
      </div>
    </div>
  );
};

export default MultiOutcomeTradingView;
