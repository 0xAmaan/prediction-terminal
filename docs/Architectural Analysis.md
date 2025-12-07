# Architectural Analysis of Real-Time Trading Platforms: Axiom Trade and Recommendations for Next.js/barter-rs

The challenge of efficiently handling and fanning out real-time market data to a large number of users is central to any high-performance trading platform. This report analyzes the likely mechanics employed by platforms like Axiom Trade and provides a concrete architectural recommendation for a similar project using Next.js and the `barter-rs` framework.

## 1. The Mechanics of Real-Time Data in Trading Platforms

The core question regarding how a platform can simultaneously display live data for a general markets page and a detailed individual market page without overwhelming the client is addressed by a multi-tiered architecture known as a **Market Data Fan-Out Architecture** [2].

The answer to whether a client subscribes to 50 different WebSockets is **no, but the server does**. The complexity and volume of real-time data are managed by shifting the burden of aggregation and filtering from the client's browser to a dedicated, powerful backend service.

### A. The Server-Side: Market Data Aggregator and Broker

A high-performance platform employs a robust backend service that acts as a central hub for all market data. This service is responsible for managing all external connections and preparing the data for efficient client delivery.

| Component | Function | Architectural Role |
| :--- | :--- | :--- |
| **Exchange Connectors** | Maintains persistent WebSocket connections to all external exchanges (e.g., Polymarket, Kalshi). | Handles the high volume of raw, noisy data and manages reconnections and error handling for external APIs. |
| **Normalization Layer** | Converts the disparate data formats from each exchange into a single, unified internal data structure. | Ensures the frontend only has to deal with one data format, regardless of the source, simplifying client-side logic. |
| **Pub/Sub Broker** | A central messaging system (e.g., Redis Pub/Sub, Kafka, or an in-memory channel) that manages data streams. | Decouples data ingestion from delivery. Data is categorized into "channels" (e.g., `market_summary`, `order_book:BTC-USD`). |
| **Client WebSocket Server** | Accepts connections from client browsers and manages the fan-out logic. | Manages a single, persistent connection per client and handles the logic of which data to send to which user based on their subscriptions. |

### B. The Client-Side: Efficient Subscription Management

The client-side is designed for efficiency and minimal connections, establishing **only one** persistent WebSocket connection to the platform's dedicated WebSocket Server.

1.  **Subscription/Unsubscription Protocol:** The client sends simple text messages over its single connection to request or stop receiving data for specific channels.
    *   **General Markets Page:** The client sends a single subscription message for a broad channel, such as `SUBSCRIBE market_summary`. The server then streams a small, aggregated data payload (last price, 24h change) for all markets.
    *   **Individual Market Page:** The client sends a targeted subscription message, such as `SUBSCRIBE order_book:KALSHI-EVENT-X`. Crucially, it also sends an **unsubscription** message for the `market_summary` channel to conserve bandwidth, ensuring the client only receives the data it needs for the current view.

This hybrid approach, utilizing **REST for static data** (historical charts, market descriptions) and **WebSockets for all real-time, high-time-sensitive data** (prices, trades, order book), is the industry standard for high-performance trading applications [3].

## 2. Architectural Guidance for Next.js and `barter-rs`

Your choice of Next.js and `barter-rs` is well-suited for this architecture, as it naturally separates the high-performance data aggregation (Rust) from the user interface (Next.js).

### A. Rust Backend with `barter-rs` (The Market Data Aggregator)

The Rust application, leveraging the performance of the language, should be the core of the Market Data Aggregator.

| Layer | Technology/Implementation | Role in Your Project |
| :--- | :--- | :--- |
| **Exchange Connectivity** | `barter-rs` | Use the framework to handle the low-level WebSocket connections, re-connections, and data parsing from Polymarket and Kalshi. |
| **Data Normalization** | Custom Rust structs/logic | Implement logic to convert `barter-rs`'s generic `MarketEvent` into a unified `InternalMarketData` struct that your frontend expects. |
| **Pub/Sub Broker** | `tokio::sync::broadcast` (Simple) or Redis (Scalable) | **Recommendation:** Start with `tokio::sync::broadcast` channels for simplicity and low latency within a single process. For production-level scalability, integrate a separate Redis instance to allow for multiple Rust worker processes. |
| **Client WebSocket Server** | `warp`, `axum`, or `actix-web` with WebSocket support | This service will listen for incoming client connections and subscription messages (e.g., "subscribe to market X"). It then subscribes to the corresponding internal Pub/Sub channel and forwards the data to the client. |

### B. Next.js Frontend (The Client)

The Next.js application will focus on managing the single connection and rendering the data efficiently.

| Layer | Technology/Implementation | Role in Your Project |
| :--- | :--- | :--- |
| **Initial Data Load** | Next.js Server Components / SSR | Use Next.js's data fetching capabilities to load the initial, non-real-time state of the market (e.g., historical data, market metadata) to prevent a blank screen and improve SEO. |
| **Client WebSocket Manager** | React Context or Custom Hook (`useWebSocket`) | This critical component manages the single connection to your Rust backend. It handles the logic for sending `SUBSCRIBE` and `UNSUBSCRIBE` messages based on which components are currently mounted (i.e., which page the user is on). |
| **Data Rendering** | React State Management (e.g., Zustand, Redux) | The WebSocket Manager feeds the live data into your global state. Components (like the market list or the individual market chart) then subscribe to the relevant slice of state to re-render only when their specific data changes, ensuring UI performance [4]. |

By adopting this **Fan-Out Architecture**, you will successfully replicate the performance and efficiency of platforms like Axiom Trade, ensuring your Next.js frontend remains responsive while your Rust backend handles the heavy lifting of data aggregation.

***

### References

[1] Axiom.trade. *Axiom: The Gateway to DeFi*. [https://axiom.trade/](https://axiom.trade/)
[2] Ably. *WebSocket architecture best practices to design robust realtime systems*. [https://ably.com/topic/websocket-architecture-best-practices](https://ably.com/topic/websocket-architecture-best-practices)
[3] CoinAPI. *Why WebSocket Multiple Updates Beat REST APIs for Real-Time Crypto Trading*. [https://www.coinapi.io/blog/why-websocket-multiple-updates-beat-rest-apis-for-real-time-crypto-trading](https://www.coinapi.io/blog/why-websocket-multiple-updates-beat-rest-apis-for-real-time-crypto-trading)
[4] InfluxData. *How to Store and Analyze Real-Time Stock Trading Data with Next.js and InfluxDB*. [https://www.influxdata.com/blog/real-time-stock-trading-data-next.js-influxdb/](https://www.influxdata.com/blog/real-time-stock-trading-data-next.js-influxdb/)
