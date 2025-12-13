//! Market Stats Service
//!
//! Provides computed statistics for markets including price changes,
//! volume, and transaction counts over configurable timeframes.

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use terminal_core::Platform;
use tracing::{debug, warn};

use crate::trade_storage::TradeStorage;

/// Timeframe for stats calculation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Timeframe {
    #[serde(rename = "1h")]
    OneHour,
    #[serde(rename = "24h")]
    TwentyFourHours,
    #[serde(rename = "7d")]
    SevenDays,
    #[serde(rename = "30d")]
    ThirtyDays,
}

impl Timeframe {
    /// Get the duration for this timeframe
    pub fn duration(&self) -> Duration {
        match self {
            Timeframe::OneHour => Duration::hours(1),
            Timeframe::TwentyFourHours => Duration::hours(24),
            Timeframe::SevenDays => Duration::days(7),
            Timeframe::ThirtyDays => Duration::days(30),
        }
    }

    /// Get the start time for this timeframe (from now)
    pub fn start_time(&self) -> DateTime<Utc> {
        Utc::now() - self.duration()
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "1h" => Some(Timeframe::OneHour),
            "24h" => Some(Timeframe::TwentyFourHours),
            "7d" => Some(Timeframe::SevenDays),
            "30d" => Some(Timeframe::ThirtyDays),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Timeframe::OneHour => "1h",
            Timeframe::TwentyFourHours => "24h",
            Timeframe::SevenDays => "7d",
            Timeframe::ThirtyDays => "30d",
        }
    }
}

impl std::fmt::Display for Timeframe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Market statistics for a specific timeframe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStats {
    /// Market ID
    pub market_id: String,
    /// Platform
    pub platform: Platform,
    /// Current YES price
    pub yes_price: Decimal,
    /// Current NO price
    pub no_price: Decimal,
    /// Absolute price change in the timeframe (in cents, e.g., 0.81)
    pub price_change: Decimal,
    /// Percentage price change (e.g., 0.97 for +0.97%)
    pub price_change_percent: Decimal,
    /// Trading volume in the timeframe (price * quantity sum)
    pub volume: Decimal,
    /// Number of YES trades in the timeframe
    pub yes_txn_count: u32,
    /// Number of NO trades in the timeframe
    pub no_txn_count: u32,
    /// The timeframe these stats cover
    pub timeframe: Timeframe,
}

/// Service for computing market statistics
pub struct MarketStatsService {
    trade_storage: Arc<TradeStorage>,
}

impl MarketStatsService {
    /// Create a new MarketStatsService
    pub fn new(trade_storage: Arc<TradeStorage>) -> Self {
        Self { trade_storage }
    }

    /// Get stats for a single market
    pub fn get_market_stats(
        &self,
        platform: Platform,
        market_id: &str,
        current_yes_price: Decimal,
        current_no_price: Decimal,
        timeframe: Timeframe,
    ) -> MarketStats {
        let now = Utc::now();
        let from = timeframe.start_time();

        // Get volume and transaction counts from trade storage
        let volume = self
            .trade_storage
            .get_volume_in_range(platform, market_id, from, now)
            .unwrap_or(0.0);

        let txn_counts = self
            .trade_storage
            .get_txn_counts_in_range(platform, market_id, from, now)
            .unwrap_or(crate::trade_storage::TxnCounts {
                yes_count: 0,
                no_count: 0,
            });

        // Get historical price for change calculation
        let (price_change, price_change_percent) = self
            .trade_storage
            .get_price_at_time(platform, market_id, from)
            .ok()
            .flatten()
            .map(|snapshot| {
                let old_price = Decimal::try_from(snapshot.yes_price).unwrap_or(current_yes_price);
                let change = current_yes_price - old_price;
                let percent = if old_price > Decimal::ZERO {
                    (change / old_price) * Decimal::from(100)
                } else {
                    Decimal::ZERO
                };
                (change, percent)
            })
            .unwrap_or((Decimal::ZERO, Decimal::ZERO));

        MarketStats {
            market_id: market_id.to_string(),
            platform,
            yes_price: current_yes_price,
            no_price: current_no_price,
            price_change,
            price_change_percent,
            volume: Decimal::try_from(volume).unwrap_or(Decimal::ZERO),
            yes_txn_count: txn_counts.yes_count,
            no_txn_count: txn_counts.no_count,
            timeframe,
        }
    }

    /// Get stats for multiple markets efficiently
    pub fn get_bulk_market_stats(
        &self,
        markets: &[(Platform, String, Decimal, Decimal)], // (platform, market_id, yes_price, no_price)
        timeframe: Timeframe,
    ) -> Vec<MarketStats> {
        if markets.is_empty() {
            return Vec::new();
        }

        let now = Utc::now();
        let from = timeframe.start_time();

        // Group markets by platform for efficient batch queries
        let mut by_platform: HashMap<Platform, Vec<(String, Decimal, Decimal)>> = HashMap::new();
        for (platform, market_id, yes_price, no_price) in markets {
            by_platform
                .entry(*platform)
                .or_default()
                .push((market_id.clone(), *yes_price, *no_price));
        }

        let mut results = Vec::new();

        for (platform, market_data) in by_platform {
            let market_ids: Vec<String> = market_data.iter().map(|(id, _, _)| id.clone()).collect();

            // Get trade stats in batch
            let trade_stats = self
                .trade_storage
                .get_bulk_stats_in_range(platform, &market_ids, from, now)
                .unwrap_or_default();

            // Get historical prices in batch
            let historical_prices = self
                .trade_storage
                .get_prices_at_time_batch(platform, &market_ids, from)
                .unwrap_or_default();

            // Build lookup maps
            let trade_stats_map: HashMap<String, _> = trade_stats
                .into_iter()
                .map(|s| (s.market_id.clone(), s))
                .collect();

            let historical_prices_map: HashMap<String, _> = historical_prices
                .into_iter()
                .collect();

            // Build stats for each market
            for (market_id, yes_price, no_price) in market_data {
                let (volume, yes_count, no_count) = trade_stats_map
                    .get(&market_id)
                    .map(|s| (s.volume, s.yes_count, s.no_count))
                    .unwrap_or((0.0, 0, 0));

                let (price_change, price_change_percent) = historical_prices_map
                    .get(&market_id)
                    .map(|snapshot| {
                        let old_price = Decimal::try_from(snapshot.yes_price).unwrap_or(yes_price);
                        let change = yes_price - old_price;
                        let percent = if old_price > Decimal::ZERO {
                            (change / old_price) * Decimal::from(100)
                        } else {
                            Decimal::ZERO
                        };
                        (change, percent)
                    })
                    .unwrap_or((Decimal::ZERO, Decimal::ZERO));

                results.push(MarketStats {
                    market_id,
                    platform,
                    yes_price,
                    no_price,
                    price_change,
                    price_change_percent,
                    volume: Decimal::try_from(volume).unwrap_or(Decimal::ZERO),
                    yes_txn_count: yes_count,
                    no_txn_count: no_count,
                    timeframe,
                });
            }
        }

        results
    }

    /// Snapshot current prices for all provided markets
    /// Call this periodically (e.g., every 5 minutes) to enable price change calculation
    pub fn snapshot_prices(
        &self,
        markets: &[(Platform, String, f64, Option<f64>)], // (platform, market_id, yes_price, no_price)
    ) -> usize {
        match self.trade_storage.store_price_snapshots_batch(markets) {
            Ok(count) => {
                debug!("Stored {} price snapshots", count);
                count
            }
            Err(e) => {
                warn!("Failed to store price snapshots: {}", e);
                0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeframe_parsing() {
        assert_eq!(Timeframe::from_str("1h"), Some(Timeframe::OneHour));
        assert_eq!(Timeframe::from_str("24h"), Some(Timeframe::TwentyFourHours));
        assert_eq!(Timeframe::from_str("7d"), Some(Timeframe::SevenDays));
        assert_eq!(Timeframe::from_str("30d"), Some(Timeframe::ThirtyDays));
        assert_eq!(Timeframe::from_str("invalid"), None);
    }

    #[test]
    fn test_timeframe_duration() {
        assert_eq!(Timeframe::OneHour.duration(), Duration::hours(1));
        assert_eq!(Timeframe::TwentyFourHours.duration(), Duration::hours(24));
        assert_eq!(Timeframe::SevenDays.duration(), Duration::days(7));
        assert_eq!(Timeframe::ThirtyDays.duration(), Duration::days(30));
    }
}
