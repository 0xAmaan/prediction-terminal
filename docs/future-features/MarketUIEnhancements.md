# Comprehensive Plan for Individual Market UI Improvement (Polymarket Focus)

This document outlines a robust strategy for significantly improving the individual market user interface, specifically addressing the requirements for displaying detailed price graphs and individual market order books, with a focus on the complexities of multi-outcome markets on Polymarket.

## Part 1: Data Acquisition Strategy (Rust Backend)

The foundation of this improvement lies in a reliable and efficient data pipeline. Our research confirms that the necessary data is available via the Polymarket Central Limit Order Book (CLOB) API.

### A. Data Endpoints and Token-Centric Model

The key to handling multi-outcome markets is recognizing that **each outcome is represented by a unique token ID**. For a market like "TIME 2025 Person of the Year," the outcomes ("Artificial Intelligence," "Jensen Huang," etc.) are distinct tokens, and data must be fetched for each one.

| Data Type | Polymarket CLOB API Endpoint | Required Parameter | Multi-Outcome Strategy |
| :--- | :--- | :--- | :--- |
| **Historical Price Data (Graphs)** | `GET /prices-history` | `market` (Token ID) | Fetch the price history for **each outcome token ID** in the market. |
| **Order Book Data** | `GET /book` | `token_id` | Fetch the order book for the **"Yes" token ID** of the currently selected outcome. |

### B. Backend Data Aggregation and Caching

Given the need to fetch multiple data streams (one for each outcome) for a single market view, a caching and aggregation layer in the **Rust backend** is essential for performance and efficiency.

1.  **Scheduled Fetching:** Implement a service in the Rust backend to periodically fetch and store the historical price data for all active market tokens. This prevents the Next.js frontend from making multiple, slow API calls directly to Polymarket.
2.  **Real-Time Order Book:** The order book data is highly dynamic. The Rust backend should act as a proxy, or ideally, connect to a WebSocket stream (if available and feasible) to maintain a near-real-time view of the order book for the currently viewed market. When the Next.js frontend requests a market, the Rust backend serves the latest snapshot.
3.  **Data Structure for Frontend:** The backend should aggregate the data into a single, clean JSON object for the frontend, structured by outcome:

```json
{
  "market_id": "...",
  "outcomes": [
    {
      "token_id": "...",
      "name": "Artificial Intelligence",
      "current_price": 0.43,
      "price_history": [
        {"t": 1697875200, "p": 0.43},
        // ...
      ],
      "order_book": {
        "bids": [
          {"price": 0.42, "size": 100},
          // ...
        ],
        "asks": [
          {"price": 0.44, "size": 50},
          // ...
        ]
      }
    }
    // ... other outcomes
  ]
}
```

## Part 2: Frontend Implementation (Next.js UI)

The Next.js frontend will consume the aggregated data from the Rust backend to create a sophisticated and interactive UI.

### A. Price Graph Visualization

The goal is to move beyond the current combined graph to offer a more detailed analysis.

| Feature | Implementation Detail | Recommended Library Option |
| :--- | :--- | :--- |
| **Combined Overview Graph** | Display all major outcome price lines on a single chart for a quick comparison of probabilities. This should be the default view. | **TanStack Charts** or **Highcharts** (known for handling large time-series datasets efficiently). |
| **Individual Outcome Graph** | Allow users to select an outcome (e.g., by clicking on its name or a dedicated button) to view a dedicated, larger chart showing only that outcome's price history. This chart should support different time intervals (1H, 1D, 1W, ALL) and a candlestick/OHLC representation if trade data is available, or a simple line chart of the last traded price. | The same library used for the overview graph to maintain consistency and reduce bundle size. |
| **Price Change Display** | Display the 24-hour price change and percentage change prominently next to the current price for each outcome. | Standard Next.js component using data calculated by the Rust backend. |

### B. Order Book Visualization

The most significant improvement will be the introduction of a visual order book, which is currently missing from the main Polymarket UI.

1.  **Market Depth Chart (Visual):**
    *   This is a cumulative graph of all open buy (bid) and sell (ask) orders, showing the total volume available at each price level.
    *   It provides an immediate, intuitive sense of market liquidity and resistance levels.
    *   **Implementation:** A dedicated chart component that plots the cumulative size of bids (green, left side) and asks (red, right side) against the price.
    *   **Recommendation:** Use a specialized component or a flexible charting library (e.g., **SciChart** or **Highcharts**) that supports this specific financial visualization.

2.  **Order Book Table (Detailed):**
    *   A traditional, tabular display of the top 10-20 price levels for bids and asks.
    *   **Bids:** Price (descending) and Size (volume).
    *   **Asks:** Price (ascending) and Size (volume).
    *   **Implementation:** A simple, high-performance virtualized table component to handle rapid updates.

### C. Multi-Outcome UI Handling

The UI must clearly manage the context switch between outcomes.

*   **Outcome Selector:** The primary market page should feature a prominent list of all outcomes. When a user clicks on an outcome:
    *   The **Trading Widget** (Buy/Sell) updates to reflect the selected outcome's token.
    *   The **Order Book/Depth Chart** section dynamically updates to display the order book for the selected outcome's "Yes" token.
    *   The **Individual Outcome Graph** view (if separate from the combined graph) updates to show the history for the selected outcome.

## Part 3: Future-Proofing and Kalshi Integration

The proposed architecture is designed to be **robust** and easily extensible to other platforms like Kalshi.

*   **Abstraction Layer:** The Rust backend should implement a data abstraction layer (e.g., a `MarketDataProvider` trait or interface) that defines the required methods (`get_historical_prices(market_id)`, `get_order_book(token_id)`).
*   **Kalshi Integration:** When integrating Kalshi, a new module will be created in the Rust backend that implements the `MarketDataProvider` interface using the Kalshi API. The Next.js frontend will remain largely unchanged, as it will continue to consume the standardized data structure defined in Part 1. This separation of concerns ensures that the UI improvements are reusable across different prediction market platforms.

This plan provides a clear, two-part approach: a high-performance, data-centric Rust backend and a modern, interactive Next.js frontend, ensuring the final product is both fast and feature-rich.

---
**References**

[1] Polymarket Documentation. *Historical Timeseries Data*.
[2] Polymarket Documentation. *Get Book*.
[3] embeddable.com. *8 Best React Chart Libraries for Visualizing Data in 2025*.
[4] Highcharts. *Order book chart Demo*.
[5] SciChart. *React Market Depth Chart*.
