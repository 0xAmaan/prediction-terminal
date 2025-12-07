//! Platform definitions for prediction markets

use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported prediction market platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    /// Kalshi - US regulated prediction market
    Kalshi,
    /// Polymarket - Crypto-based prediction market
    Polymarket,
}

impl Platform {
    /// Get a short identifier for the platform (for display)
    pub fn short_name(&self) -> &'static str {
        match self {
            Platform::Kalshi => "K",
            Platform::Polymarket => "P",
        }
    }

    /// Get the full display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Platform::Kalshi => "Kalshi",
            Platform::Polymarket => "Polymarket",
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::str::FromStr for Platform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "kalshi" | "k" => Ok(Platform::Kalshi),
            "polymarket" | "poly" | "p" => Ok(Platform::Polymarket),
            _ => Err(format!("Unknown platform: {}", s)),
        }
    }
}
