//! Position calculation from trade history
//!
//! Derives current holdings by aggregating trades for each token.

use crate::types::{Position, UserTrade};
use std::collections::HashMap;
use tracing::debug;

/// Internal structure for accumulating position data
#[derive(Debug, Default)]
struct PositionAccumulator {
    market_id: String,
    token_id: String,
    total_bought: f64,
    total_sold: f64,
    total_buy_cost: f64,
    total_sell_revenue: f64,
}

impl PositionAccumulator {
    fn new(market_id: String, token_id: String) -> Self {
        Self {
            market_id,
            token_id,
            ..Default::default()
        }
    }

    fn add_trade(&mut self, side: &str, size: f64, price: f64) {
        match side.to_uppercase().as_str() {
            "BUY" => {
                self.total_bought += size;
                self.total_buy_cost += size * price;
            }
            "SELL" => {
                self.total_sold += size;
                self.total_sell_revenue += size * price;
            }
            _ => {
                debug!("Unknown trade side: {}", side);
            }
        }
    }

    fn net_shares(&self) -> f64 {
        self.total_bought - self.total_sold
    }

    fn avg_entry_price(&self) -> f64 {
        if self.total_bought > 0.0 {
            self.total_buy_cost / self.total_bought
        } else {
            0.0
        }
    }

    fn realized_pnl(&self) -> f64 {
        // PnL from shares that were sold
        if self.total_sold > 0.0 {
            let avg_buy_price = self.avg_entry_price();
            self.total_sell_revenue - (self.total_sold * avg_buy_price)
        } else {
            0.0
        }
    }

    fn to_position(&self) -> Position {
        let shares = self.net_shares();
        let avg_price = self.avg_entry_price();
        let pnl = self.realized_pnl();

        // Determine outcome from token_id
        // In Polymarket, token IDs are typically the same as the condition ID
        // The outcome is usually encoded in the market structure, not the token
        // For now, we'll leave it empty and let the frontend fill it in
        let outcome = String::new();

        Position {
            market_id: self.market_id.clone(),
            token_id: self.token_id.clone(),
            outcome,
            shares: format!("{:.6}", shares),
            avg_price: format!("{:.4}", avg_price),
            current_price: "0.00".to_string(), // Would need orderbook data to fill
            pnl: format!("{:.2}", pnl),
            title: String::new(), // Not available from trade-based calculation
            neg_risk: false,      // Default to false, would need market metadata
        }
    }
}

/// Calculate positions from a list of trades
///
/// Aggregates trades by token_id to compute:
/// - Net shares held
/// - Average entry price (volume-weighted)
/// - Realized PnL (from closed positions)
///
/// Only returns positions with non-zero shares.
pub fn calculate_positions(trades: &[UserTrade]) -> Vec<Position> {
    let mut accumulators: HashMap<String, PositionAccumulator> = HashMap::new();

    for trade in trades {
        // Skip trades that aren't matched/filled
        if trade.status.to_lowercase() != "matched" && trade.status.to_lowercase() != "filled" {
            continue;
        }

        let size: f64 = trade.size.parse().unwrap_or(0.0);
        let price: f64 = trade.price.parse().unwrap_or(0.0);

        if size <= 0.0 {
            continue;
        }

        let accumulator = accumulators
            .entry(trade.asset_id.clone())
            .or_insert_with(|| {
                PositionAccumulator::new(trade.market.clone(), trade.asset_id.clone())
            });

        accumulator.add_trade(&trade.side, size, price);
    }

    // Convert to positions, filtering out zero holdings
    accumulators
        .values()
        .filter(|acc| acc.net_shares().abs() > 0.0001) // Small threshold for float comparison
        .map(|acc| acc.to_position())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_trade(
        asset_id: &str,
        market: &str,
        side: &str,
        size: &str,
        price: &str,
    ) -> UserTrade {
        UserTrade {
            id: "test".to_string(),
            market: market.to_string(),
            asset_id: asset_id.to_string(),
            side: side.to_string(),
            size: size.to_string(),
            price: price.to_string(),
            status: "matched".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            match_time: None,
            transaction_hash: None,
        }
    }

    #[test]
    fn test_single_buy() {
        let trades = vec![make_trade("token1", "market1", "BUY", "10", "0.5")];

        let positions = calculate_positions(&trades);
        assert_eq!(positions.len(), 1);

        let pos = &positions[0];
        assert_eq!(pos.token_id, "token1");
        assert_eq!(pos.shares, "10.000000");
        assert_eq!(pos.avg_price, "0.5000");
    }

    #[test]
    fn test_buy_and_sell() {
        let trades = vec![
            make_trade("token1", "market1", "BUY", "10", "0.5"),
            make_trade("token1", "market1", "SELL", "5", "0.7"),
        ];

        let positions = calculate_positions(&trades);
        assert_eq!(positions.len(), 1);

        let pos = &positions[0];
        assert_eq!(pos.shares, "5.000000"); // 10 - 5 = 5
        assert_eq!(pos.avg_price, "0.5000"); // avg buy price
    }

    #[test]
    fn test_fully_closed_position() {
        let trades = vec![
            make_trade("token1", "market1", "BUY", "10", "0.5"),
            make_trade("token1", "market1", "SELL", "10", "0.7"),
        ];

        let positions = calculate_positions(&trades);
        // Position should not appear since shares are 0
        assert_eq!(positions.len(), 0);
    }

    #[test]
    fn test_multiple_tokens() {
        let trades = vec![
            make_trade("token1", "market1", "BUY", "10", "0.5"),
            make_trade("token2", "market2", "BUY", "20", "0.3"),
        ];

        let positions = calculate_positions(&trades);
        assert_eq!(positions.len(), 2);
    }

    #[test]
    fn test_pnl_calculation() {
        // Buy 10 @ 0.50, sell 5 @ 0.70
        // Realized PnL = 5 * (0.70 - 0.50) = 1.00
        let trades = vec![
            make_trade("token1", "market1", "BUY", "10", "0.5"),
            make_trade("token1", "market1", "SELL", "5", "0.7"),
        ];

        let positions = calculate_positions(&trades);
        let pos = &positions[0];
        assert_eq!(pos.pnl, "1.00");
    }
}
