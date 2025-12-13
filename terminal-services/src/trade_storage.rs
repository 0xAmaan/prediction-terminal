//! Trade Storage Service
//!
//! SQLite-based storage for historical trades, enabling price history generation.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use rust_decimal::Decimal;
use std::path::Path;
use std::sync::Mutex;
use terminal_core::{Platform, Trade, TradeOutcome, TradeSide};

/// Trade storage service using SQLite
pub struct TradeStorage {
    conn: Mutex<Connection>,
}

impl TradeStorage {
    /// Create a new TradeStorage instance
    ///
    /// Creates the database file and tables if they don't exist.
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, TradeStorageError> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.as_ref().parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                TradeStorageError::Io(format!("Failed to create database directory: {}", e))
            })?;
        }

        let conn = Connection::open(db_path).map_err(TradeStorageError::Database)?;

        let storage = Self {
            conn: Mutex::new(conn),
        };
        storage.init_schema()?;

        Ok(storage)
    }

    /// Create an in-memory TradeStorage (useful for testing)
    pub fn new_in_memory() -> Result<Self, TradeStorageError> {
        let conn = Connection::open_in_memory().map_err(TradeStorageError::Database)?;

        let storage = Self {
            conn: Mutex::new(conn),
        };
        storage.init_schema()?;

        Ok(storage)
    }

    /// Initialize the database schema
    fn init_schema(&self) -> Result<(), TradeStorageError> {
        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        conn.execute_batch(
            r#"
            -- Trades table (existing)
            CREATE TABLE IF NOT EXISTS trades (
                id TEXT PRIMARY KEY,
                platform TEXT NOT NULL,
                market_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                price REAL NOT NULL,
                quantity REAL NOT NULL,
                outcome TEXT NOT NULL,
                side TEXT,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            );

            CREATE INDEX IF NOT EXISTS idx_trades_market
            ON trades(platform, market_id, timestamp);

            CREATE INDEX IF NOT EXISTS idx_trades_timestamp
            ON trades(timestamp);

            -- Prices table (current prices for fast lookup)
            CREATE TABLE IF NOT EXISTS prices (
                platform TEXT NOT NULL,
                market_id TEXT NOT NULL,
                yes_price REAL,
                no_price REAL,
                spread REAL,
                updated_at INTEGER NOT NULL,
                PRIMARY KEY (platform, market_id)
            );

            -- Orderbook snapshots table (for historical depth analysis)
            CREATE TABLE IF NOT EXISTS orderbook_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                platform TEXT NOT NULL,
                market_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                yes_bids TEXT,
                yes_asks TEXT,
                no_bids TEXT,
                no_asks TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_ob_market
            ON orderbook_snapshots(platform, market_id, timestamp);

            -- Pre-computed candles table
            CREATE TABLE IF NOT EXISTS candles (
                platform TEXT NOT NULL,
                market_id TEXT NOT NULL,
                interval TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                open REAL,
                high REAL,
                low REAL,
                close REAL,
                volume REAL,
                trade_count INTEGER,
                PRIMARY KEY (platform, market_id, interval, timestamp)
            );

            -- Price snapshots table (for historical price change calculation)
            CREATE TABLE IF NOT EXISTS price_snapshots (
                platform TEXT NOT NULL,
                market_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                yes_price REAL NOT NULL,
                no_price REAL,
                PRIMARY KEY (platform, market_id, timestamp)
            );

            CREATE INDEX IF NOT EXISTS idx_price_snapshots_lookup
            ON price_snapshots(platform, market_id, timestamp DESC);
            "#,
        )
        .map_err(TradeStorageError::Database)?;

        Ok(())
    }

    /// Store a single trade
    pub fn store_trade(&self, trade: &Trade) -> Result<(), TradeStorageError> {
        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        let platform_str = match trade.platform {
            Platform::Kalshi => "kalshi",
            Platform::Polymarket => "polymarket",
        };

        let outcome_str = match trade.outcome {
            TradeOutcome::Yes => "yes",
            TradeOutcome::No => "no",
        };

        let side_str = trade.side.as_ref().map(|s| match s {
            TradeSide::Buy => "buy",
            TradeSide::Sell => "sell",
        });

        let timestamp = trade.timestamp.timestamp();
        let price: f64 = trade
            .price
            .try_into()
            .unwrap_or_else(|_| trade.price.to_string().parse().unwrap_or(0.0));
        let quantity: f64 = trade
            .quantity
            .try_into()
            .unwrap_or_else(|_| trade.quantity.to_string().parse().unwrap_or(0.0));

        conn.execute(
            r#"
            INSERT OR REPLACE INTO trades (id, platform, market_id, timestamp, price, quantity, outcome, side)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                trade.id,
                platform_str,
                trade.market_id,
                timestamp,
                price,
                quantity,
                outcome_str,
                side_str,
            ],
        )
        .map_err(TradeStorageError::Database)?;

        Ok(())
    }

    /// Store multiple trades in a batch
    pub fn store_trades(&self, trades: &[Trade]) -> Result<usize, TradeStorageError> {
        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        let mut stored = 0;
        for trade in trades {
            let platform_str = match trade.platform {
                Platform::Kalshi => "kalshi",
                Platform::Polymarket => "polymarket",
            };

            let outcome_str = match trade.outcome {
                TradeOutcome::Yes => "yes",
                TradeOutcome::No => "no",
            };

            let side_str = trade.side.as_ref().map(|s| match s {
                TradeSide::Buy => "buy",
                TradeSide::Sell => "sell",
            });

            let timestamp = trade.timestamp.timestamp();
            let price: f64 = trade
                .price
                .try_into()
                .unwrap_or_else(|_| trade.price.to_string().parse().unwrap_or(0.0));
            let quantity: f64 = trade
                .quantity
                .try_into()
                .unwrap_or_else(|_| trade.quantity.to_string().parse().unwrap_or(0.0));

            let result = conn.execute(
                r#"
                INSERT OR IGNORE INTO trades (id, platform, market_id, timestamp, price, quantity, outcome, side)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                "#,
                params![
                    trade.id,
                    platform_str,
                    trade.market_id,
                    timestamp,
                    price,
                    quantity,
                    outcome_str,
                    side_str,
                ],
            );

            if result.is_ok() {
                stored += 1;
            }
        }

        Ok(stored)
    }

    /// Get trades for a market within a time range
    pub fn get_trades(
        &self,
        platform: Platform,
        market_id: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Trade>, TradeStorageError> {
        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        let platform_str = match platform {
            Platform::Kalshi => "kalshi",
            Platform::Polymarket => "polymarket",
        };

        let from_ts = from.timestamp();
        let to_ts = to.timestamp();

        let mut stmt = conn
            .prepare(
                r#"
            SELECT id, platform, market_id, timestamp, price, quantity, outcome, side
            FROM trades
            WHERE platform = ?1 AND market_id = ?2 AND timestamp >= ?3 AND timestamp <= ?4
            ORDER BY timestamp ASC
            "#,
            )
            .map_err(TradeStorageError::Database)?;

        let trades = stmt
            .query_map(params![platform_str, market_id, from_ts, to_ts], |row| {
                let id: String = row.get(0)?;
                let platform_str: String = row.get(1)?;
                let market_id: String = row.get(2)?;
                let timestamp: i64 = row.get(3)?;
                let price: f64 = row.get(4)?;
                let quantity: f64 = row.get(5)?;
                let outcome_str: String = row.get(6)?;
                let side_str: Option<String> = row.get(7)?;

                Ok((
                    id,
                    platform_str,
                    market_id,
                    timestamp,
                    price,
                    quantity,
                    outcome_str,
                    side_str,
                ))
            })
            .map_err(TradeStorageError::Database)?
            .filter_map(|r| r.ok())
            .map(
                |(id, platform_str, market_id, timestamp, price, quantity, outcome_str, side_str)| {
                    Trade {
                        id,
                        market_id,
                        platform: if platform_str == "kalshi" {
                            Platform::Kalshi
                        } else {
                            Platform::Polymarket
                        },
                        timestamp: DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now),
                        price: Decimal::try_from(price).unwrap_or_default(),
                        quantity: Decimal::try_from(quantity).unwrap_or_default(),
                        outcome: if outcome_str == "yes" {
                            TradeOutcome::Yes
                        } else {
                            TradeOutcome::No
                        },
                        side: side_str.map(|s| {
                            if s == "buy" {
                                TradeSide::Buy
                            } else {
                                TradeSide::Sell
                            }
                        }),
                        transaction_hash: None, // Not stored in local DB
                    }
                },
            )
            .collect();

        Ok(trades)
    }

    /// Get the latest trade for a market
    pub fn get_latest_trade(
        &self,
        platform: Platform,
        market_id: &str,
    ) -> Result<Option<Trade>, TradeStorageError> {
        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        let platform_str = match platform {
            Platform::Kalshi => "kalshi",
            Platform::Polymarket => "polymarket",
        };

        let mut stmt = conn
            .prepare(
                r#"
            SELECT id, platform, market_id, timestamp, price, quantity, outcome, side
            FROM trades
            WHERE platform = ?1 AND market_id = ?2
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
            )
            .map_err(TradeStorageError::Database)?;

        let result = stmt
            .query_row(params![platform_str, market_id], |row| {
                let id: String = row.get(0)?;
                let platform_str: String = row.get(1)?;
                let market_id: String = row.get(2)?;
                let timestamp: i64 = row.get(3)?;
                let price: f64 = row.get(4)?;
                let quantity: f64 = row.get(5)?;
                let outcome_str: String = row.get(6)?;
                let side_str: Option<String> = row.get(7)?;

                Ok((
                    id,
                    platform_str,
                    market_id,
                    timestamp,
                    price,
                    quantity,
                    outcome_str,
                    side_str,
                ))
            })
            .optional()
            .map_err(TradeStorageError::Database)?;

        Ok(result.map(
            |(id, platform_str, market_id, timestamp, price, quantity, outcome_str, side_str)| {
                Trade {
                    id,
                    market_id,
                    platform: if platform_str == "kalshi" {
                        Platform::Kalshi
                    } else {
                        Platform::Polymarket
                    },
                    timestamp: DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now),
                    price: Decimal::try_from(price).unwrap_or_default(),
                    quantity: Decimal::try_from(quantity).unwrap_or_default(),
                    outcome: if outcome_str == "yes" {
                        TradeOutcome::Yes
                    } else {
                        TradeOutcome::No
                    },
                    side: side_str.map(|s| {
                        if s == "buy" {
                            TradeSide::Buy
                        } else {
                            TradeSide::Sell
                        }
                    }),
                    transaction_hash: None, // Not stored in local DB
                }
            },
        ))
    }

    /// Get the count of trades for a market
    pub fn get_trade_count(
        &self,
        platform: Platform,
        market_id: &str,
    ) -> Result<usize, TradeStorageError> {
        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        let platform_str = match platform {
            Platform::Kalshi => "kalshi",
            Platform::Polymarket => "polymarket",
        };

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM trades WHERE platform = ?1 AND market_id = ?2",
                params![platform_str, market_id],
                |row| row.get(0),
            )
            .map_err(TradeStorageError::Database)?;

        Ok(count as usize)
    }

    /// Check if a trade exists
    pub fn trade_exists(&self, trade_id: &str) -> Result<bool, TradeStorageError> {
        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        let exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM trades WHERE id = ?1)",
                params![trade_id],
                |row| row.get(0),
            )
            .map_err(TradeStorageError::Database)?;

        Ok(exists)
    }

    // =========================================================================
    // Trade Aggregation Methods (for market stats)
    // =========================================================================

    /// Get total volume (sum of price * quantity) for a market in a time range
    pub fn get_volume_in_range(
        &self,
        platform: Platform,
        market_id: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<f64, TradeStorageError> {
        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        let platform_str = match platform {
            Platform::Kalshi => "kalshi",
            Platform::Polymarket => "polymarket",
        };

        let from_ts = from.timestamp();
        let to_ts = to.timestamp();

        let volume: f64 = conn
            .query_row(
                r#"
                SELECT COALESCE(SUM(price * quantity), 0.0)
                FROM trades
                WHERE platform = ?1 AND market_id = ?2 AND timestamp >= ?3 AND timestamp <= ?4
                "#,
                params![platform_str, market_id, from_ts, to_ts],
                |row| row.get(0),
            )
            .map_err(TradeStorageError::Database)?;

        Ok(volume)
    }

    /// Get transaction counts by outcome (yes/no) for a market in a time range
    pub fn get_txn_counts_in_range(
        &self,
        platform: Platform,
        market_id: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<TxnCounts, TradeStorageError> {
        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        let platform_str = match platform {
            Platform::Kalshi => "kalshi",
            Platform::Polymarket => "polymarket",
        };

        let from_ts = from.timestamp();
        let to_ts = to.timestamp();

        let yes_count: i64 = conn
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM trades
                WHERE platform = ?1 AND market_id = ?2 AND timestamp >= ?3 AND timestamp <= ?4 AND outcome = 'yes'
                "#,
                params![platform_str, market_id, from_ts, to_ts],
                |row| row.get(0),
            )
            .map_err(TradeStorageError::Database)?;

        let no_count: i64 = conn
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM trades
                WHERE platform = ?1 AND market_id = ?2 AND timestamp >= ?3 AND timestamp <= ?4 AND outcome = 'no'
                "#,
                params![platform_str, market_id, from_ts, to_ts],
                |row| row.get(0),
            )
            .map_err(TradeStorageError::Database)?;

        Ok(TxnCounts {
            yes_count: yes_count as u32,
            no_count: no_count as u32,
        })
    }

    /// Get the earliest trade price in a time range (for computing price change)
    pub fn get_earliest_price_in_range(
        &self,
        platform: Platform,
        market_id: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Option<f64>, TradeStorageError> {
        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        let platform_str = match platform {
            Platform::Kalshi => "kalshi",
            Platform::Polymarket => "polymarket",
        };

        let from_ts = from.timestamp();
        let to_ts = to.timestamp();

        let price: Option<f64> = conn
            .query_row(
                r#"
                SELECT price
                FROM trades
                WHERE platform = ?1 AND market_id = ?2 AND timestamp >= ?3 AND timestamp <= ?4
                ORDER BY timestamp ASC
                LIMIT 1
                "#,
                params![platform_str, market_id, from_ts, to_ts],
                |row| row.get(0),
            )
            .optional()
            .map_err(TradeStorageError::Database)?;

        Ok(price)
    }

    /// Get aggregated stats for multiple markets efficiently
    pub fn get_bulk_stats_in_range(
        &self,
        platform: Platform,
        market_ids: &[String],
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<MarketTradeStats>, TradeStorageError> {
        if market_ids.is_empty() {
            return Ok(Vec::new());
        }

        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        let platform_str = match platform {
            Platform::Kalshi => "kalshi",
            Platform::Polymarket => "polymarket",
        };

        let from_ts = from.timestamp();
        let to_ts = to.timestamp();

        // Build query with placeholders for all market IDs
        let placeholders: String = market_ids.iter().enumerate()
            .map(|(i, _)| format!("?{}", i + 5))
            .collect::<Vec<_>>()
            .join(", ");

        let query = format!(
            r#"
            SELECT
                market_id,
                COALESCE(SUM(price * quantity), 0.0) as volume,
                COUNT(CASE WHEN outcome = 'yes' THEN 1 END) as yes_count,
                COUNT(CASE WHEN outcome = 'no' THEN 1 END) as no_count,
                (SELECT price FROM trades t2
                 WHERE t2.platform = ?1 AND t2.market_id = trades.market_id
                 AND t2.timestamp >= ?3 AND t2.timestamp <= ?4
                 ORDER BY t2.timestamp ASC LIMIT 1) as earliest_price
            FROM trades
            WHERE platform = ?1 AND market_id IN ({}) AND timestamp >= ?3 AND timestamp <= ?4
            GROUP BY market_id
            "#,
            placeholders
        );

        let mut stmt = conn.prepare(&query).map_err(TradeStorageError::Database)?;

        // Build params: platform, unused, from, to, then all market_ids
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![
            Box::new(platform_str.to_string()),
            Box::new(platform_str.to_string()), // Placeholder for subquery
            Box::new(from_ts),
            Box::new(to_ts),
        ];
        for id in market_ids {
            params_vec.push(Box::new(id.clone()));
        }

        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();

        let stats = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok(MarketTradeStats {
                    market_id: row.get(0)?,
                    volume: row.get(1)?,
                    yes_count: row.get::<_, i64>(2)? as u32,
                    no_count: row.get::<_, i64>(3)? as u32,
                    earliest_price: row.get(4)?,
                })
            })
            .map_err(TradeStorageError::Database)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(stats)
    }

    // =========================================================================
    // Price Storage Methods
    // =========================================================================

    /// Store or update current prices for a market
    pub fn store_price(
        &self,
        platform: Platform,
        market_id: &str,
        yes_price: Option<f64>,
        no_price: Option<f64>,
    ) -> Result<(), TradeStorageError> {
        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        let platform_str = match platform {
            Platform::Kalshi => "kalshi",
            Platform::Polymarket => "polymarket",
        };

        let spread = match (yes_price, no_price) {
            (Some(yes), Some(no)) => Some((yes - no).abs()),
            _ => None,
        };

        let now = chrono::Utc::now().timestamp();

        conn.execute(
            r#"
            INSERT OR REPLACE INTO prices (platform, market_id, yes_price, no_price, spread, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![platform_str, market_id, yes_price, no_price, spread, now],
        )
        .map_err(TradeStorageError::Database)?;

        Ok(())
    }

    /// Get current price for a market
    pub fn get_price(
        &self,
        platform: Platform,
        market_id: &str,
    ) -> Result<Option<StoredPrice>, TradeStorageError> {
        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        let platform_str = match platform {
            Platform::Kalshi => "kalshi",
            Platform::Polymarket => "polymarket",
        };

        let result = conn
            .query_row(
                "SELECT yes_price, no_price, spread, updated_at FROM prices WHERE platform = ?1 AND market_id = ?2",
                params![platform_str, market_id],
                |row| {
                    Ok(StoredPrice {
                        yes_price: row.get(0)?,
                        no_price: row.get(1)?,
                        spread: row.get(2)?,
                        updated_at: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(TradeStorageError::Database)?;

        Ok(result)
    }

    // =========================================================================
    // Orderbook Snapshot Methods
    // =========================================================================

    /// Store an orderbook snapshot
    pub fn store_orderbook_snapshot(
        &self,
        platform: Platform,
        market_id: &str,
        yes_bids: &str,
        yes_asks: &str,
        no_bids: &str,
        no_asks: &str,
    ) -> Result<(), TradeStorageError> {
        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        let platform_str = match platform {
            Platform::Kalshi => "kalshi",
            Platform::Polymarket => "polymarket",
        };

        let now = chrono::Utc::now().timestamp();

        conn.execute(
            r#"
            INSERT INTO orderbook_snapshots (platform, market_id, timestamp, yes_bids, yes_asks, no_bids, no_asks)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![platform_str, market_id, now, yes_bids, yes_asks, no_bids, no_asks],
        )
        .map_err(TradeStorageError::Database)?;

        Ok(())
    }

    /// Get orderbook snapshots for a market within a time range
    pub fn get_orderbook_snapshots(
        &self,
        platform: Platform,
        market_id: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        limit: Option<usize>,
    ) -> Result<Vec<OrderbookSnapshot>, TradeStorageError> {
        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        let platform_str = match platform {
            Platform::Kalshi => "kalshi",
            Platform::Polymarket => "polymarket",
        };

        let from_ts = from.timestamp();
        let to_ts = to.timestamp();
        let limit = limit.unwrap_or(1000) as i64;

        let mut stmt = conn
            .prepare(
                r#"
                SELECT timestamp, yes_bids, yes_asks, no_bids, no_asks
                FROM orderbook_snapshots
                WHERE platform = ?1 AND market_id = ?2 AND timestamp >= ?3 AND timestamp <= ?4
                ORDER BY timestamp DESC
                LIMIT ?5
                "#,
            )
            .map_err(TradeStorageError::Database)?;

        let snapshots = stmt
            .query_map(params![platform_str, market_id, from_ts, to_ts, limit], |row| {
                Ok(OrderbookSnapshot {
                    timestamp: row.get(0)?,
                    yes_bids: row.get(1)?,
                    yes_asks: row.get(2)?,
                    no_bids: row.get(3)?,
                    no_asks: row.get(4)?,
                })
            })
            .map_err(TradeStorageError::Database)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(snapshots)
    }

    /// Prune old orderbook snapshots
    pub fn prune_orderbook_snapshots(&self, older_than_days: u64) -> Result<usize, TradeStorageError> {
        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        let cutoff = chrono::Utc::now().timestamp() - (older_than_days as i64 * 86400);

        let deleted = conn
            .execute(
                "DELETE FROM orderbook_snapshots WHERE timestamp < ?1",
                params![cutoff],
            )
            .map_err(TradeStorageError::Database)?;

        Ok(deleted)
    }

    // =========================================================================
    // Candle Storage Methods
    // =========================================================================

    /// Store or update a candle
    pub fn store_candle(
        &self,
        platform: Platform,
        market_id: &str,
        interval: &str,
        timestamp: i64,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
        trade_count: i64,
    ) -> Result<(), TradeStorageError> {
        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        let platform_str = match platform {
            Platform::Kalshi => "kalshi",
            Platform::Polymarket => "polymarket",
        };

        conn.execute(
            r#"
            INSERT OR REPLACE INTO candles (platform, market_id, interval, timestamp, open, high, low, close, volume, trade_count)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![platform_str, market_id, interval, timestamp, open, high, low, close, volume, trade_count],
        )
        .map_err(TradeStorageError::Database)?;

        Ok(())
    }

    /// Get candles for a market
    pub fn get_candles(
        &self,
        platform: Platform,
        market_id: &str,
        interval: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<StoredCandle>, TradeStorageError> {
        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        let platform_str = match platform {
            Platform::Kalshi => "kalshi",
            Platform::Polymarket => "polymarket",
        };

        let from_ts = from.timestamp();
        let to_ts = to.timestamp();

        let mut stmt = conn
            .prepare(
                r#"
                SELECT timestamp, open, high, low, close, volume, trade_count
                FROM candles
                WHERE platform = ?1 AND market_id = ?2 AND interval = ?3 AND timestamp >= ?4 AND timestamp <= ?5
                ORDER BY timestamp ASC
                "#,
            )
            .map_err(TradeStorageError::Database)?;

        let candles = stmt
            .query_map(params![platform_str, market_id, interval, from_ts, to_ts], |row| {
                Ok(StoredCandle {
                    timestamp: row.get(0)?,
                    open: row.get(1)?,
                    high: row.get(2)?,
                    low: row.get(3)?,
                    close: row.get(4)?,
                    volume: row.get(5)?,
                    trade_count: row.get(6)?,
                })
            })
            .map_err(TradeStorageError::Database)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(candles)
    }

    // =========================================================================
    // Price Snapshot Methods (for historical price change calculation)
    // =========================================================================

    /// Store a price snapshot for a market
    pub fn store_price_snapshot(
        &self,
        platform: Platform,
        market_id: &str,
        yes_price: f64,
        no_price: Option<f64>,
    ) -> Result<(), TradeStorageError> {
        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        let platform_str = match platform {
            Platform::Kalshi => "kalshi",
            Platform::Polymarket => "polymarket",
        };

        let now = chrono::Utc::now().timestamp();

        conn.execute(
            r#"
            INSERT OR REPLACE INTO price_snapshots (platform, market_id, timestamp, yes_price, no_price)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![platform_str, market_id, now, yes_price, no_price],
        )
        .map_err(TradeStorageError::Database)?;

        Ok(())
    }

    /// Store multiple price snapshots in batch
    pub fn store_price_snapshots_batch(
        &self,
        snapshots: &[(Platform, String, f64, Option<f64>)],
    ) -> Result<usize, TradeStorageError> {
        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        let now = chrono::Utc::now().timestamp();
        let mut stored = 0;

        for (platform, market_id, yes_price, no_price) in snapshots {
            let platform_str = match platform {
                Platform::Kalshi => "kalshi",
                Platform::Polymarket => "polymarket",
            };

            if conn
                .execute(
                    r#"
                    INSERT OR REPLACE INTO price_snapshots (platform, market_id, timestamp, yes_price, no_price)
                    VALUES (?1, ?2, ?3, ?4, ?5)
                    "#,
                    params![platform_str, market_id, now, yes_price, no_price],
                )
                .is_ok()
            {
                stored += 1;
            }
        }

        Ok(stored)
    }

    /// Get the price snapshot closest to a specific timestamp (for historical comparison)
    pub fn get_price_at_time(
        &self,
        platform: Platform,
        market_id: &str,
        target_time: DateTime<Utc>,
    ) -> Result<Option<PriceSnapshot>, TradeStorageError> {
        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        let platform_str = match platform {
            Platform::Kalshi => "kalshi",
            Platform::Polymarket => "polymarket",
        };

        let target_ts = target_time.timestamp();

        // Get the snapshot closest to target time (prefer older if exactly between two)
        let result = conn
            .query_row(
                r#"
                SELECT timestamp, yes_price, no_price
                FROM price_snapshots
                WHERE platform = ?1 AND market_id = ?2 AND timestamp <= ?3
                ORDER BY timestamp DESC
                LIMIT 1
                "#,
                params![platform_str, market_id, target_ts],
                |row| {
                    Ok(PriceSnapshot {
                        timestamp: row.get(0)?,
                        yes_price: row.get(1)?,
                        no_price: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(TradeStorageError::Database)?;

        Ok(result)
    }

    /// Get price snapshots for multiple markets at a specific time (batch operation)
    pub fn get_prices_at_time_batch(
        &self,
        platform: Platform,
        market_ids: &[String],
        target_time: DateTime<Utc>,
    ) -> Result<Vec<(String, PriceSnapshot)>, TradeStorageError> {
        if market_ids.is_empty() {
            return Ok(Vec::new());
        }

        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        let platform_str = match platform {
            Platform::Kalshi => "kalshi",
            Platform::Polymarket => "polymarket",
        };

        let target_ts = target_time.timestamp();
        let mut results = Vec::new();

        // Query each market (SQLite doesn't support LATERAL JOIN easily)
        for market_id in market_ids {
            if let Ok(Some(snapshot)) = conn
                .query_row(
                    r#"
                    SELECT timestamp, yes_price, no_price
                    FROM price_snapshots
                    WHERE platform = ?1 AND market_id = ?2 AND timestamp <= ?3
                    ORDER BY timestamp DESC
                    LIMIT 1
                    "#,
                    params![platform_str, market_id, target_ts],
                    |row| {
                        Ok(PriceSnapshot {
                            timestamp: row.get(0)?,
                            yes_price: row.get(1)?,
                            no_price: row.get(2)?,
                        })
                    },
                )
                .optional()
            {
                results.push((market_id.clone(), snapshot));
            }
        }

        Ok(results)
    }

    /// Prune old price snapshots (keep only last N days)
    pub fn prune_price_snapshots(&self, older_than_days: u64) -> Result<usize, TradeStorageError> {
        let conn = self.conn.lock().map_err(|_| TradeStorageError::LockError)?;

        let cutoff = chrono::Utc::now().timestamp() - (older_than_days as i64 * 86400);

        let deleted = conn
            .execute(
                "DELETE FROM price_snapshots WHERE timestamp < ?1",
                params![cutoff],
            )
            .map_err(TradeStorageError::Database)?;

        Ok(deleted)
    }
}

/// Price snapshot data
#[derive(Debug, Clone)]
pub struct PriceSnapshot {
    pub timestamp: i64,
    pub yes_price: f64,
    pub no_price: Option<f64>,
}

/// Stored price data
#[derive(Debug, Clone)]
pub struct StoredPrice {
    pub yes_price: Option<f64>,
    pub no_price: Option<f64>,
    pub spread: Option<f64>,
    pub updated_at: i64,
}

/// Stored orderbook snapshot
#[derive(Debug, Clone)]
pub struct OrderbookSnapshot {
    pub timestamp: i64,
    pub yes_bids: Option<String>,
    pub yes_asks: Option<String>,
    pub no_bids: Option<String>,
    pub no_asks: Option<String>,
}

/// Stored candle data
#[derive(Debug, Clone)]
pub struct StoredCandle {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub trade_count: i64,
}

/// Transaction counts by outcome
#[derive(Debug, Clone)]
pub struct TxnCounts {
    pub yes_count: u32,
    pub no_count: u32,
}

/// Aggregated trade stats for a market
#[derive(Debug, Clone)]
pub struct MarketTradeStats {
    pub market_id: String,
    pub volume: f64,
    pub yes_count: u32,
    pub no_count: u32,
    pub earliest_price: Option<f64>,
}

/// Errors that can occur during trade storage operations
#[derive(Debug, thiserror::Error)]
pub enum TradeStorageError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Failed to acquire lock")]
    LockError,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_trade(id: &str, market_id: &str, price: f64, timestamp_offset: i64) -> Trade {
        Trade {
            id: id.to_string(),
            market_id: market_id.to_string(),
            platform: Platform::Kalshi,
            timestamp: Utc::now() + chrono::Duration::seconds(timestamp_offset),
            price: Decimal::try_from(price).unwrap(),
            quantity: dec!(100),
            outcome: TradeOutcome::Yes,
            side: Some(TradeSide::Buy),
            transaction_hash: None,
        }
    }

    #[test]
    fn test_store_and_retrieve_trade() {
        let storage = TradeStorage::new_in_memory().unwrap();

        let trade = create_test_trade("trade1", "market1", 0.55, 0);
        storage.store_trade(&trade).unwrap();

        let latest = storage
            .get_latest_trade(Platform::Kalshi, "market1")
            .unwrap();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().id, "trade1");
    }

    #[test]
    fn test_store_batch_trades() {
        let storage = TradeStorage::new_in_memory().unwrap();

        let trades = vec![
            create_test_trade("trade1", "market1", 0.50, -100),
            create_test_trade("trade2", "market1", 0.55, -50),
            create_test_trade("trade3", "market1", 0.60, 0),
        ];

        let stored = storage.store_trades(&trades).unwrap();
        assert_eq!(stored, 3);

        let count = storage.get_trade_count(Platform::Kalshi, "market1").unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_trade_exists() {
        let storage = TradeStorage::new_in_memory().unwrap();

        let trade = create_test_trade("trade1", "market1", 0.55, 0);
        storage.store_trade(&trade).unwrap();

        assert!(storage.trade_exists("trade1").unwrap());
        assert!(!storage.trade_exists("trade_nonexistent").unwrap());
    }
}
