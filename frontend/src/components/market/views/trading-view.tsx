"use client";

import { useMemo } from "react";
import type { PredictionMarket, Trade } from "@/lib/types";

// Components
import { PriceChart } from "@/components/market/price-chart";
import { OrderBookV2 } from "@/components/market/orderbook/order-book-v2";
import { TradeExecution } from "@/components/market/trade-execution";
import { LiveTradesTable } from "@/components/market/live-trades-table";
import { MarketInfoPanel } from "@/components/market/market-info-panel";
import { TradingHeader } from "@/components/market/layout/trading-header";

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

interface OrderBookData {
  yes_bids: Array<{ price: string; quantity: string; order_count: number | null }>;
  yes_asks: Array<{ price: string; quantity: string; order_count: number | null }>;
  no_bids: Array<{ price: string; quantity: string; order_count: number | null }>;
  no_asks: Array<{ price: string; quantity: string; order_count: number | null }>;
}

export interface TradingViewProps {
  market: PredictionMarket;
  orderBook: OrderBookData | null;
  orderBookLoading: boolean;
  trades: Trade[];
  relatedMarkets: PredictionMarket[];
  relatedMarketsLoading: boolean;
  livePrices: { yesPrice: string; noPrice: string } | null;
  priceHistory: number[];
}

export const TradingView = ({
  market,
  orderBook,
  orderBookLoading,
  trades,
  livePrices,
}: TradingViewProps) => {
  // Use live prices from WebSocket if available, fallback to REST data
  const currentYesPrice = livePrices?.yesPrice ?? market.yes_price;
  const currentNoPrice = livePrices?.noPrice ?? market.no_price;

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
      {/* Header with market info and stats */}
      <TradingHeader market={market} yesPrice={currentYesPrice} />

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
              <PriceChart
                platform={market.platform}
                marketId={market.id}
                currentPrice={parseFloat(currentYesPrice)}
                title="Price History"
              />
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
              yesPrice={currentYesPrice}
              noPrice={currentNoPrice}
              trades={trades}
              tokenId={market.clob_token_id}
              marketTitle={market.title}
              negRisk={false}
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
              yesPrice={currentYesPrice}
              noPrice={currentNoPrice}
              spread={spread}
            />
          </div>
        </div>
      </div>
    </div>
  );
};

export default TradingView;
