//! Candle Service
//!
//! Aggregates trades into OHLCV (Open, High, Low, Close, Volume) candles for price history.
//!
//! Supports two modes:
//! 1. **Trade-based**: Build candles from stored trades (limited by backfill depth)
//! 2. **Hybrid**: Combine native price API data with trade volume data (complete coverage)

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use std::collections::BTreeMap;
use std::sync::Arc;
use terminal_core::{Platform, PriceCandle, PriceHistory, PriceInterval, Trade, TradeSide};
use terminal_polymarket::PriceHistoryPoint;

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

        // Aggregate volumes by trade side
        let buy_volume: Decimal = sorted_trades
            .iter()
            .filter(|t| t.side == Some(TradeSide::Buy))
            .map(|t| t.quantity)
            .sum();

        let sell_volume: Decimal = sorted_trades
            .iter()
            .filter(|t| t.side == Some(TradeSide::Sell))
            .map(|t| t.quantity)
            .sum();

        // Total volume includes trades with unknown side
        let volume = sorted_trades.iter().map(|t| t.quantity).sum();

        PriceCandle {
            timestamp: DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now),
            open,
            high,
            low,
            close,
            volume,
            buy_volume,
            sell_volume,
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
                        buy_volume: Decimal::ZERO,
                        sell_volume: Decimal::ZERO,
                    });
                    gap_ts += interval_secs;
                }
            }

            filled_candles.push(candle.clone());
            prev_close = candle.close;
        }

        history.candles = filled_candles;
    }

    // ========================================================================
    // Hybrid Candle Building (Native Prices + Trade Volumes)
    // ========================================================================

    /// Build candles from native price history with trade volume data
    ///
    /// Combines price data from the platform's native API (complete coverage)
    /// with buy/sell volume data from stored trades (where available).
    ///
    /// # Arguments
    /// * `platform` - The trading platform
    /// * `market_id` - Market identifier
    /// * `prices` - Price history points from native API
    /// * `interval` - Candle interval (1m, 15m, 1h, 4h, 1d)
    /// * `from_filter` - Optional: filter to only include prices after this timestamp
    pub fn build_hybrid_candles(
        &self,
        platform: Platform,
        market_id: &str,
        prices: Vec<PriceHistoryPoint>,
        interval: PriceInterval,
        from_filter: Option<DateTime<Utc>>,
    ) -> Result<PriceHistory, CandleServiceError> {
        // Apply time filter if provided
        let filtered_prices: Vec<PriceHistoryPoint> = if let Some(from_time) = from_filter {
            let from_ts = from_time.timestamp();
            prices.into_iter().filter(|p| p.t >= from_ts).collect()
        } else {
            prices
        };

        if filtered_prices.is_empty() {
            return Ok(PriceHistory {
                market_id: market_id.to_string(),
                platform,
                interval,
                candles: vec![],
            });
        }

        let interval_secs = interval.to_seconds() as i64;

        // Get time range from filtered price data
        let from_ts = filtered_prices.iter().map(|p| p.t).min().unwrap_or(0);
        let to_ts = filtered_prices.iter().map(|p| p.t).max().unwrap_or(0);
        let from = DateTime::from_timestamp(from_ts, 0).unwrap_or_else(Utc::now);
        let to = DateTime::from_timestamp(to_ts, 0).unwrap_or_else(Utc::now) + Duration::hours(1);

        // Fetch trades for volume data
        let trades = self.storage.get_trades(platform, market_id, from, to).unwrap_or_default();

        // Group trades by candle bucket
        let mut trade_buckets: BTreeMap<i64, Vec<&Trade>> = BTreeMap::new();
        for trade in &trades {
            let bucket = (trade.timestamp.timestamp() / interval_secs) * interval_secs;
            trade_buckets.entry(bucket).or_default().push(trade);
        }

        // Group prices by candle bucket
        let mut price_buckets: BTreeMap<i64, Vec<&PriceHistoryPoint>> = BTreeMap::new();
        for price in &filtered_prices {
            let bucket = (price.t / interval_secs) * interval_secs;
            price_buckets.entry(bucket).or_default().push(price);
        }

        // Build candles from price buckets, enriched with trade volume
        let candles: Vec<PriceCandle> = price_buckets
            .into_iter()
            .map(|(bucket_ts, bucket_prices)| {
                let trades_for_bucket = trade_buckets.get(&bucket_ts);
                self.build_candle_from_prices_and_trades(bucket_ts, &bucket_prices, trades_for_bucket)
            })
            .collect();

        Ok(PriceHistory {
            market_id: market_id.to_string(),
            platform,
            interval,
            candles,
        })
    }

    /// Build a single candle from price points and optional trade data
    fn build_candle_from_prices_and_trades(
        &self,
        timestamp: i64,
        prices: &[&PriceHistoryPoint],
        trades: Option<&Vec<&Trade>>,
    ) -> PriceCandle {
        let mut sorted_prices: Vec<_> = prices.iter().collect();
        sorted_prices.sort_by_key(|p| p.t);

        let open = Decimal::try_from(sorted_prices.first().map(|p| p.p).unwrap_or(0.0))
            .unwrap_or_default();
        let close = Decimal::try_from(sorted_prices.last().map(|p| p.p).unwrap_or(0.0))
            .unwrap_or_default();
        let high = Decimal::try_from(
            sorted_prices.iter().map(|p| p.p).fold(f64::MIN, f64::max),
        ).unwrap_or_default();
        let low = Decimal::try_from(
            sorted_prices.iter().map(|p| p.p).fold(f64::MAX, f64::min),
        ).unwrap_or_default();

        // Get volume from trades if available
        let (volume, buy_volume, sell_volume) = if let Some(bucket_trades) = trades {
            let buy_vol: Decimal = bucket_trades
                .iter()
                .filter(|t| t.side == Some(TradeSide::Buy))
                .map(|t| t.quantity)
                .sum();
            let sell_vol: Decimal = bucket_trades
                .iter()
                .filter(|t| t.side == Some(TradeSide::Sell))
                .map(|t| t.quantity)
                .sum();
            let total_vol: Decimal = bucket_trades.iter().map(|t| t.quantity).sum();
            (total_vol, buy_vol, sell_vol)
        } else {
            (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO)
        };

        PriceCandle {
            timestamp: DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now),
            open,
            high,
            low,
            close,
            volume,
            buy_volume,
            sell_volume,
        }
    }

    /// Get hybrid candles for a specific timeframe preset
    ///
    /// # Arguments
    /// * `platform` - The trading platform
    /// * `market_id` - Market identifier
    /// * `prices` - Price history points from native API
    /// * `timeframe` - Timeframe preset ("1H", "24H", "7D", "30D", "ALL")
    /// * `from_filter` - Optional: filter to only include prices after this timestamp
    pub fn get_hybrid_candles_for_timeframe(
        &self,
        platform: Platform,
        market_id: &str,
        prices: Vec<PriceHistoryPoint>,
        timeframe: &str,
        from_filter: Option<DateTime<Utc>>,
    ) -> Result<PriceHistory, CandleServiceError> {
        let interval = match timeframe.to_uppercase().as_str() {
            "1H" => PriceInterval::OneMinute,
            "24H" => PriceInterval::FifteenMinutes,
            "7D" => PriceInterval::OneHour,
            "30D" => PriceInterval::FourHours,
            "ALL" | _ => PriceInterval::OneDay,
        };
        self.build_hybrid_candles(platform, market_id, prices, interval, from_filter)
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
        side: TradeSide,
    ) -> Trade {
        Trade {
            id: id.to_string(),
            market_id: market_id.to_string(),
            platform: Platform::Kalshi,
            timestamp,
            price,
            quantity: dec!(100),
            outcome: TradeOutcome::Yes,
            side: Some(side),
            transaction_hash: None,
        }
    }

    #[test]
    fn test_build_candles() {
        let storage = Arc::new(TradeStorage::new_in_memory().unwrap());
        let service = CandleService::new(storage.clone());

        let base_time = Utc::now() - Duration::hours(1);

        // Create trades with mixed buy/sell sides
        let trades = vec![
            create_test_trade("t1", "market1", dec!(0.50), base_time, TradeSide::Buy),
            create_test_trade("t2", "market1", dec!(0.55), base_time + Duration::minutes(5), TradeSide::Buy),
            create_test_trade("t3", "market1", dec!(0.45), base_time + Duration::minutes(10), TradeSide::Sell),
            create_test_trade("t4", "market1", dec!(0.52), base_time + Duration::minutes(15), TradeSide::Buy),
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
        assert_eq!(candle.buy_volume, dec!(300)); // 3 buy trades * 100 quantity
        assert_eq!(candle.sell_volume, dec!(100)); // 1 sell trade * 100 quantity
        assert!(candle.is_buy_pressure()); // More buys than sells
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
