# Build stage
FROM rust:1.83-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./

# Copy all workspace members
COPY barter ./barter
COPY barter-data ./barter-data
COPY barter-execution ./barter-execution
COPY barter-instrument ./barter-instrument
COPY barter-integration ./barter-integration
COPY barter-macro ./barter-macro
COPY terminal-core ./terminal-core
COPY terminal-kalshi ./terminal-kalshi
COPY terminal-polymarket ./terminal-polymarket
COPY terminal-services ./terminal-services
COPY terminal-api ./terminal-api

# Build release binary
RUN cargo build --release -p terminal-api

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create data directory for SQLite
RUN mkdir -p /app/data

# Copy binary from builder
COPY --from=builder /app/target/release/terminal-api /app/terminal-api

# Set environment defaults
ENV SERVER_PORT=3001
ENV TRADES_DB_PATH=/app/data/trades.db
ENV RUST_LOG=info

EXPOSE 3001

CMD ["./terminal-api"]
