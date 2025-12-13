//! Candle Service
//!
//! Aggregates trades into OHLCV (Open, High, Low, Close, Volume) candles for price history.

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use std::collections::BTreeMap;
use std::sync::Arc;
use terminal_core::{Platform, PriceCandle, PriceHistory, PriceInterval, Trade};

use crate::trade_storage::{TradeStorage, TradeStorageError};

/// Service for building price candles from stored trades
pub struct CandleService {
    storage: Arc<TradeStorage>,
}

impl CandleService {
    /// Create a new CandleService
    pub fn new(storage: Arc<TradeStorage>) -> Self {
        Self { storage }
    }

    /// Build candles for a market over a time range
    pub fn build_candles(
        &self,
        platform: Platform,
        market_id: &str,
        interval: PriceInterval,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<PriceHistory, CandleServiceError> {
        // Fetch trades for the time range
        let trades = self.storage.get_trades(platform, market_id, from, to)?;

        if trades.is_empty() {
            return Ok(PriceHistory {
                market_id: market_id.to_string(),
                platform,
                interval,
                candles: vec![],
            });
        }

        // Group trades by candle bucket
        let interval_secs = interval.to_seconds() as i64;
        let mut buckets: BTreeMap<i64, Vec<&Trade>> = BTreeMap::new();

        for trade in &trades {
            let bucket = (trade.timestamp.timestamp() / interval_secs) * interval_secs;
            buckets.entry(bucket).or_default().push(trade);
        }

        // Build candles from buckets
        let candles: Vec<PriceCandle> = buckets
            .into_iter()
            .map(|(bucket_ts, bucket_trades)| {
                self.build_candle_from_trades(bucket_ts, &bucket_trades)
            })
            .collect();

        Ok(PriceHistory {
            market_id: market_id.to_string(),
            platform,
            interval,
            candles,
        })
    }

    /// Build a single candle from a set of trades
    fn build_candle_from_trades(&self, timestamp: i64, trades: &[&Trade]) -> PriceCandle {
        // Sort trades by timestamp for accurate open/close
        let mut sorted_trades: Vec<_> = trades.iter().collect();
        sorted_trades.sort_by_key(|t| t.timestamp);

        let open = sorted_trades.first().map(|t| t.price).unwrap_or_default();
        let close = sorted_trades.last().map(|t| t.price).unwrap_or_default();

        let high = sorted_trades
            .iter()
            .map(|t| t.price)
            .max()
            .unwrap_or_default();

        let low = sorted_trades
            .iter()
            .map(|t| t.price)
            .min()
            .unwrap_or_default();

        let volume = sorted_trades.iter().map(|t| t.quantity).sum();

        PriceCandle {
            timestamp: DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now),
            open,
            high,
            low,
            close,
            volume,
        }
    }

    /// Get candles for a specific timeframe preset
    ///
    /// Convenient method that calculates the appropriate time range based on the timeframe.
    pub fn get_candles_for_timeframe(
        &self,
        platform: Platform,
        market_id: &str,
        timeframe: &str,
    ) -> Result<PriceHistory, CandleServiceError> {
        let now = Utc::now();

        let (from, interval) = match timeframe.to_uppercase().as_str() {
            "1H" => (now - Duration::hours(1), PriceInterval::OneMinute),
            "24H" => (now - Duration::hours(24), PriceInterval::FifteenMinutes),
            "7D" => (now - Duration::days(7), PriceInterval::OneHour),
            "30D" => (now - Duration::days(30), PriceInterval::FourHours),
            "ALL" | _ => (now - Duration::days(90), PriceInterval::OneDay),
        };

        self.build_candles(platform, market_id, interval, from, now)
    }

    /// Fill gaps in candle data with synthetic candles
    ///
    /// Creates candles with no change (close = open of previous) for periods with no trades.
    pub fn fill_gaps(&self, history: &mut PriceHistory) {
        if history.candles.len() < 2 {
            return;
        }

        let interval_secs = history.interval.to_seconds() as i64;
        let mut filled_candles: Vec<PriceCandle> = Vec::new();
        let mut prev_close = history.candles[0].close;

        for candle in &history.candles {
            // Check if there's a gap from the last candle
            if let Some(last_filled) = filled_candles.last() {
                let expected_ts = last_filled.timestamp.timestamp() + interval_secs;
                let actual_ts = candle.timestamp.timestamp();

                // Fill gaps
                let mut gap_ts = expected_ts;
                while gap_ts < actual_ts {
                    filled_candles.push(PriceCandle {
                        timestamp: DateTime::from_timestamp(gap_ts, 0).unwrap_or_else(Utc::now),
                        open: prev_close,
                        high: prev_close,
                        low: prev_close,
                        close: prev_close,
                        volume: Decimal::ZERO,
                    });
                    gap_ts += interval_secs;
                }
            }

            filled_candles.push(candle.clone());
            prev_close = candle.close;
        }

        history.candles = filled_candles;
    }
}

/// Errors that can occur during candle operations
#[derive(Debug, thiserror::Error)]
pub enum CandleServiceError {
    #[error("Storage error: {0}")]
    Storage(#[from] TradeStorageError),

    #[error("Invalid time range")]
    InvalidTimeRange,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use terminal_core::{TradeOutcome, TradeSide};

    fn create_test_trade(
        id: &str,
        market_id: &str,
        price: Decimal,
        timestamp: DateTime<Utc>,
    ) -> Trade {
        Trade {
            id: id.to_string(),
            market_id: market_id.to_string(),
            platform: Platform::Kalshi,
            timestamp,
            price,
            quantity: dec!(100),
            outcome: TradeOutcome::Yes,
            side: Some(TradeSide::Buy),
        }
    }

    #[test]
    fn test_build_candles() {
        let storage = Arc::new(TradeStorage::new_in_memory().unwrap());
        let service = CandleService::new(storage.clone());

        let base_time = Utc::now() - Duration::hours(1);

        // Create trades at different prices
        let trades = vec![
            create_test_trade("t1", "market1", dec!(0.50), base_time),
            create_test_trade(
                "t2",
                "market1",
                dec!(0.55),
                base_time + Duration::minutes(5),
            ),
            create_test_trade(
                "t3",
                "market1",
                dec!(0.45),
                base_time + Duration::minutes(10),
            ),
            create_test_trade(
                "t4",
                "market1",
                dec!(0.52),
                base_time + Duration::minutes(15),
            ),
        ];

        for trade in &trades {
            storage.store_trade(trade).unwrap();
        }

        let history = service
            .build_candles(
                Platform::Kalshi,
                "market1",
                PriceInterval::OneHour,
                base_time - Duration::minutes(1),
                Utc::now(),
            )
            .unwrap();

        assert_eq!(history.candles.len(), 1);

        let candle = &history.candles[0];
        assert_eq!(candle.open, dec!(0.50));
        assert_eq!(candle.high, dec!(0.55));
        assert_eq!(candle.low, dec!(0.45));
        assert_eq!(candle.close, dec!(0.52));
        assert_eq!(candle.volume, dec!(400)); // 4 trades * 100 quantity
    }

    #[test]
    fn test_empty_trades() {
        let storage = Arc::new(TradeStorage::new_in_memory().unwrap());
        let service = CandleService::new(storage);

        let history = service
            .build_candles(
                Platform::Kalshi,
                "nonexistent",
                PriceInterval::OneHour,
                Utc::now() - Duration::hours(1),
                Utc::now(),
            )
            .unwrap();

        assert!(history.candles.is_empty());
    }
}
