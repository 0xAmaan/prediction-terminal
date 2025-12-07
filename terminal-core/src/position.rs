//! Position and portfolio tracking structures

use crate::platform::Platform;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Outcome type for a prediction market position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    Yes,
    No,
}

/// A position in a prediction market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Market identifier on the platform
    pub market_id: String,

    /// Human-readable market title
    pub market_title: String,

    /// Which platform this position is on
    pub platform: Platform,

    /// Which outcome we hold (YES or NO)
    pub outcome: Outcome,

    /// Number of contracts/shares held
    pub quantity: Decimal,

    /// Average price paid per contract
    pub avg_price: Decimal,

    /// Current market price for this outcome
    pub current_price: Decimal,

    /// Unrealized profit/loss
    pub unrealized_pnl: Decimal,
}

impl Position {
    /// Calculate the current value of this position
    pub fn current_value(&self) -> Decimal {
        self.quantity * self.current_price
    }

    /// Calculate the cost basis
    pub fn cost_basis(&self) -> Decimal {
        self.quantity * self.avg_price
    }

    /// Calculate unrealized P&L
    pub fn calculate_pnl(&self) -> Decimal {
        self.current_value() - self.cost_basis()
    }

    /// Calculate P&L as a percentage
    pub fn pnl_percentage(&self) -> Decimal {
        if self.cost_basis().is_zero() {
            Decimal::ZERO
        } else {
            (self.unrealized_pnl / self.cost_basis()) * Decimal::from(100)
        }
    }
}

/// Account balance on a platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    /// Which platform this balance is on
    pub platform: Platform,

    /// Available balance for trading
    pub available: Decimal,

    /// Balance locked in open orders
    pub locked: Decimal,

    /// Total balance (available + locked)
    pub total: Decimal,

    /// Currency/token symbol (USD for Kalshi, USDC for Polymarket)
    pub currency: String,
}

impl Balance {
    /// Create a new balance
    pub fn new(platform: Platform, available: Decimal, locked: Decimal, currency: &str) -> Self {
        Self {
            platform,
            available,
            locked,
            total: available + locked,
            currency: currency.to_string(),
        }
    }
}

/// Aggregated portfolio across platforms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    /// All positions across platforms
    pub positions: Vec<Position>,

    /// Balances on each platform
    pub balances: Vec<Balance>,

    /// Total portfolio value (positions + available balance)
    pub total_value: Decimal,

    /// Total unrealized P&L
    pub total_pnl: Decimal,
}

impl Portfolio {
    /// Create an empty portfolio
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            balances: Vec::new(),
            total_value: Decimal::ZERO,
            total_pnl: Decimal::ZERO,
        }
    }

    /// Calculate totals from positions and balances
    pub fn calculate_totals(&mut self) {
        // Sum up position values
        let positions_value: Decimal = self.positions.iter().map(|p| p.current_value()).sum();

        // Sum up available balances
        let balances_total: Decimal = self.balances.iter().map(|b| b.available).sum();

        // Sum up P&L
        self.total_pnl = self.positions.iter().map(|p| p.unrealized_pnl).sum();

        self.total_value = positions_value + balances_total;
    }

    /// Get positions filtered by platform
    pub fn positions_for_platform(&self, platform: Platform) -> Vec<&Position> {
        self.positions
            .iter()
            .filter(|p| p.platform == platform)
            .collect()
    }

    /// Get balance for a specific platform
    pub fn balance_for_platform(&self, platform: Platform) -> Option<&Balance> {
        self.balances.iter().find(|b| b.platform == platform)
    }
}

impl Default for Portfolio {
    fn default() -> Self {
        Self::new()
    }
}
