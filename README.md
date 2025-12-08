# Prediction Market Terminal

A unified trading terminal that aggregates prediction markets from **Kalshi** and **Polymarket** into a single interface.

Built on top of [barter-rs](https://github.com/barter-rs/barter-rs), a Rust algorithmic trading framework.

## Features

- **Multi-Platform Aggregation**: View markets from Kalshi and Polymarket in one place
- **Real-Time Data**: Live orderbook updates, trades, and price changes via WebSocket
- **Price Charts**: Interactive charts with multiple timeframes (1H, 24H, 7D, 30D, ALL)
- **Order Book Visualization**: YES/NO sides with bid/ask depth
- **Trade History**: Recent trades with real-time updates
- **Multi-Outcome Markets**: Support for markets with multiple outcomes (not just Yes/No)
- **Platform Filtering**: Toggle between platforms or view all markets

## Screenshots

<!-- Add screenshots here -->

## Quick Start

### Prerequisites

- Rust 1.70+
- Bun (for frontend)
- SQLite

### Backend

```bash
# Clone the repository
git clone <repo-url>
cd prediction-terminal

# Create environment file
cp .env.example .env.local
# Edit .env.local with your API credentials (optional for read-only)

# Run the API server
cargo run -p terminal-api

# Or with hot reload
cargo watch -w terminal-api -w terminal-core -w terminal-services -w terminal-kalshi -w terminal-polymarket -x 'run -p terminal-api'
```

The API server runs on `http://localhost:3001` by default.

### Frontend

```bash
cd frontend

# Install dependencies
bun install

# Start development server
bun run dev --turbopack

# Or run both frontend and backend with hot reload
bun run dev:backend  # Runs cargo watch for the Rust backend
```

The frontend runs on `http://localhost:3000` by default.

### Environment Variables

```bash
# Backend (.env.local at repository root)
TRADES_DB_PATH=data/trades.db    # SQLite database path
SERVER_PORT=3001                  # API server port

# Optional: Kalshi API (for authenticated endpoints)
KALSHI_API_KEY=your_key
KALSHI_PRIVATE_KEY_PATH=/path/to/private.pem

# Optional: Polymarket API (for authenticated endpoints)
POLY_API_KEY=your_key
POLY_SECRET=your_secret
POLY_PASSPHRASE=your_passphrase

# Frontend (frontend/.env.local)
NEXT_PUBLIC_API_URL=http://localhost:3001
```

## Tech Stack

### Backend (Rust)
- **Axum**: HTTP server and WebSocket handling
- **Tokio**: Async runtime
- **SQLite**: Trade history persistence
- **reqwest**: HTTP client for exchange APIs

### Frontend (TypeScript)
- **Next.js 15**: React framework with App Router
- **React 19**: UI library
- **TanStack Query**: Data fetching and caching
- **TradingView Lightweight Charts**: Price charts
- **TailwindCSS**: Styling

## Architecture

```
┌─────────────────┐     ┌─────────────────┐
│  Kalshi API     │     │ Polymarket API  │
└────────┬────────┘     └────────┬────────┘
         │                       │
         └───────────┬───────────┘
                     │
         ┌───────────▼───────────┐
         │   Rust Backend        │
         │   (terminal-api)      │
         │                       │
         │  - MarketService      │
         │  - Aggregator         │
         │  - TradeStorage       │
         └───────────┬───────────┘
                     │
              REST + WebSocket
                     │
         ┌───────────▼───────────┐
         │   Next.js Frontend    │
         │                       │
         │  - Markets Grid       │
         │  - Market Detail      │
         │  - Real-time Updates  │
         └───────────────────────┘
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed architecture documentation.

## API Endpoints

### REST API

| Endpoint | Description |
|----------|-------------|
| `GET /api/markets` | List markets (query: platform, search, limit) |
| `GET /api/markets/:platform/:id` | Get single market |
| `GET /api/markets/:platform/:id/orderbook` | Get order book |
| `GET /api/markets/:platform/:id/trades` | Get trade history |
| `GET /api/markets/:platform/:id/history` | Get price candles |
| `GET /api/health` | Health check |

### WebSocket

Connect to `ws://localhost:3001/ws` and send subscription messages:

```json
{"type": "Subscribe", "channel": {"type": "Orderbook", "platform": "kalshi", "market_id": "..."}}
{"type": "Subscribe", "channel": {"type": "Trades", "platform": "polymarket", "market_id": "..."}}
{"type": "Unsubscribe", "channel": {"type": "Orderbook", "platform": "kalshi", "market_id": "..."}}
```

## Project Structure

```
├── barter/                 # Original barter-rs trading engine
├── barter-data/            # Market data streaming (crypto exchanges)
├── barter-execution/       # Order execution
├── barter-instrument/      # Instrument definitions
├── barter-integration/     # REST/WebSocket framework
│
├── terminal-core/          # Prediction market types (NEW)
├── terminal-kalshi/        # Kalshi API client (NEW)
├── terminal-polymarket/    # Polymarket API client (NEW)
├── terminal-services/      # Business logic layer (NEW)
├── terminal-api/           # HTTP/WebSocket server (NEW)
│
├── frontend/               # Next.js application (NEW)
│   ├── src/app/            # Pages
│   ├── src/components/     # React components
│   ├── src/hooks/          # Custom hooks
│   └── src/lib/            # Utilities
│
└── docs/                   # Documentation
    ├── ARCHITECTURE.md     # Technical architecture
    ├── BRAINLIFT.md        # Project summary
    └── DEMO_TOPICS.md      # Demo talking points
```

## Development

### Building

```bash
# Build all Rust crates
cargo build

# Build just the API server
cargo build -p terminal-api

# Build frontend
cd frontend && bun run build
```

### Testing

```bash
# Run Rust tests
cargo test

# Test a single crate
cargo test -p terminal-core

# Run frontend linting
cd frontend && bun run lint
```

### Formatting

```bash
# Format Rust code (uses rustfmt.toml: edition 2024, crate-level imports)
cargo fmt
```

## About This Project

This project was built as part of the Gauntlet "Uncharted Territory" challenge - a brownfield development exercise where participants fork an existing open-source repository in an unfamiliar programming language and extend it with new functionality.

**Original Repository**: [barter-rs/barter-rs](https://github.com/barter-rs/barter-rs) - A Rust algorithmic trading framework for crypto exchanges.

**What Was Added**: Complete prediction market support including:
- 5 new Rust crates (~9,400 lines) for Kalshi and Polymarket integration
- Full Next.js frontend (~7,500 lines) with real-time data
- WebSocket fan-out architecture for efficient real-time updates
- SQLite persistence for trade history and candle generation

See [docs/BRAINLIFT.md](docs/BRAINLIFT.md) for the full project summary.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Original Barter-rs Documentation

<details>
<summary>Click to expand original barter-rs README</summary>

### Barter Overview

Barter is an algorithmic trading ecosystem of Rust libraries for building high-performance live-trading, paper-trading and back-testing systems.

* **Fast**: Written in native Rust. Minimal allocations. Data-oriented state management system with direct index lookups.
* **Robust**: Strongly typed. Thread safe. Extensive test coverage.
* **Customisable**: Plug and play Strategy and RiskManager components that facilitates most trading strategies.
* **Scalable**: Multithreaded architecture with modular design. Leverages Tokio for I/O.

### Barter Libraries

* **Barter**: Algorithmic trading Engine with feature rich state management system.
* **Barter-Instrument**: Exchange, Instrument and Asset data structures and utilities.
* **Barter-Data**: Stream public market data from financial venues.
* **Barter-Execution**: Stream private account data and execute orders.
* **Barter-Integration**: Low-level frameworks for flexible REST/WebSocket integrations.

For more information, see the [original barter-rs repository](https://github.com/barter-rs/barter-rs).

</details>
