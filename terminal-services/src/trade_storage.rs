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
