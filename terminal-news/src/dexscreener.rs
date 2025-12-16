//! DexScreener API client for trending token signals
//!
//! Fetches trending/boosted tokens from DexScreener as early market signals.
//! Free API, no authentication required.

use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::time::Duration;
use tracing::{debug, error, info, warn};

use terminal_core::{NewsItem, NewsSource};

const DEXSCREENER_API_BASE: &str = "https://api.dexscreener.com";

/// DexScreener API client
pub struct DexScreenerClient {
    http: Client,
}

/// Token boost response from DexScreener
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenBoost {
    url: String,
    chain_id: String,
    token_address: String,
    icon: Option<String>,
    header: Option<String>,
    description: Option<String>,
    links: Option<Vec<TokenLink>>,
    total_amount: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TokenLink {
    #[serde(rename = "type")]
    link_type: Option<String>,
    label: Option<String>,
    url: Option<String>,
}

/// Token profile response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct TokenProfile {
    url: String,
    chain_id: String,
    token_address: String,
    icon: Option<String>,
    header: Option<String>,
    description: Option<String>,
    links: Option<Vec<TokenLink>>,
}

impl DexScreenerClient {
    /// Create a new DexScreener client
    pub fn new() -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("PredictionTerminal/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { http }
    }

    /// Fetch latest boosted tokens (paid promotions - early signal)
    pub async fn fetch_latest_boosts(&self) -> Result<Vec<NewsItem>, DexScreenerError> {
        let url = format!("{}/token-boosts/latest/v1", DEXSCREENER_API_BASE);
        debug!("[DexScreener] Fetching latest boosts from: {}", url);

        let response = self.http.get(&url).send().await
            .map_err(|e| DexScreenerError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DexScreenerError::HttpError(response.status().as_u16()));
        }

        let boosts: Vec<TokenBoost> = response.json().await
            .map_err(|e| DexScreenerError::ParseError(e.to_string()))?;

        info!("[DexScreener] Fetched {} latest boosted tokens", boosts.len());

        Ok(boosts.into_iter().map(|b| self.boost_to_news_item(b, "Latest Boost")).collect())
    }

    /// Fetch top boosted tokens (most actively promoted)
    pub async fn fetch_top_boosts(&self) -> Result<Vec<NewsItem>, DexScreenerError> {
        let url = format!("{}/token-boosts/top/v1", DEXSCREENER_API_BASE);
        debug!("[DexScreener] Fetching top boosts from: {}", url);

        let response = self.http.get(&url).send().await
            .map_err(|e| DexScreenerError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DexScreenerError::HttpError(response.status().as_u16()));
        }

        let boosts: Vec<TokenBoost> = response.json().await
            .map_err(|e| DexScreenerError::ParseError(e.to_string()))?;

        info!("[DexScreener] Fetched {} top boosted tokens", boosts.len());

        Ok(boosts.into_iter().map(|b| self.boost_to_news_item(b, "Top Boost")).collect())
    }

    /// Fetch latest token profiles (new tokens)
    pub async fn fetch_latest_profiles(&self) -> Result<Vec<NewsItem>, DexScreenerError> {
        let url = format!("{}/token-profiles/latest/v1", DEXSCREENER_API_BASE);
        debug!("[DexScreener] Fetching latest profiles from: {}", url);

        let response = self.http.get(&url).send().await
            .map_err(|e| DexScreenerError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DexScreenerError::HttpError(response.status().as_u16()));
        }

        let profiles: Vec<TokenProfile> = response.json().await
            .map_err(|e| DexScreenerError::ParseError(e.to_string()))?;

        info!("[DexScreener] Fetched {} latest token profiles", profiles.len());

        Ok(profiles.into_iter().map(|p| self.profile_to_news_item(p)).collect())
    }

    /// Fetch all trending signals (boosts + profiles)
    pub async fn fetch_trending_signals(&self, limit: usize) -> Vec<NewsItem> {
        let mut all_items = Vec::new();

        // Fetch latest boosts (highest priority - someone is paying to promote)
        match self.fetch_latest_boosts().await {
            Ok(items) => all_items.extend(items),
            Err(e) => warn!("[DexScreener] Failed to fetch latest boosts: {}", e),
        }

        // Fetch top boosts
        match self.fetch_top_boosts().await {
            Ok(items) => all_items.extend(items),
            Err(e) => warn!("[DexScreener] Failed to fetch top boosts: {}", e),
        }

        // Fetch latest profiles
        match self.fetch_latest_profiles().await {
            Ok(items) => all_items.extend(items),
            Err(e) => warn!("[DexScreener] Failed to fetch latest profiles: {}", e),
        }

        // Deduplicate by token address (keep first occurrence)
        let mut seen = std::collections::HashSet::new();
        all_items.retain(|item| {
            let key = item.url.clone();
            seen.insert(key)
        });

        // Sort by relevance and take top N
        all_items.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        all_items.truncate(limit);

        all_items
    }

    /// Convert a token boost to NewsItem
    fn boost_to_news_item(&self, boost: TokenBoost, boost_type: &str) -> NewsItem {
        // Get token name - header field sometimes contains URLs, so filter those out
        let fallback_name = truncate_address(&boost.token_address);
        let token_name = boost.header
            .as_deref()
            .filter(|h| !h.starts_with("http"))
            .unwrap_or(&fallback_name);

        let title = format!(
            "🚀 {} on {}: {}",
            boost_type,
            format_chain(&boost.chain_id),
            token_name
        );

        let summary = boost.description.clone().unwrap_or_else(|| {
            format!(
                "Token {} on {} received a boost{}",
                truncate_address(&boost.token_address),
                format_chain(&boost.chain_id),
                boost.total_amount.map(|a| format!(" (${:.0})", a)).unwrap_or_default()
            )
        });

        // Higher relevance for tokens with more boost amount
        let relevance = match boost.total_amount {
            Some(amount) if amount >= 1000.0 => 0.9,
            Some(amount) if amount >= 500.0 => 0.8,
            Some(amount) if amount >= 100.0 => 0.7,
            Some(_) => 0.6,
            None => 0.5,
        };

        // Find Twitter link if available
        let twitter_url = boost.links.as_ref().and_then(|links| {
            links.iter()
                .find(|l| l.link_type.as_deref() == Some("twitter"))
                .and_then(|l| l.url.clone())
        });

        NewsItem {
            id: generate_id(&boost.url),
            title,
            url: boost.url,
            published_at: Utc::now(), // DexScreener doesn't provide timestamps
            source: NewsSource {
                name: format!("DexScreener • {}", format_chain(&boost.chain_id)),
                url: "https://dexscreener.com".to_string(),
                favicon_url: Some("https://dexscreener.com/favicon.ico".to_string()),
            },
            summary,
            content: None,
            image_url: boost.icon,
            relevance_score: relevance,
            related_market_ids: vec![],
            search_query: twitter_url, // Store Twitter URL in search_query for reference
            // AI-enriched fields (set later by NewsAnalyzer)
            matched_market: None,
            price_signal: None,
            suggested_action: None,
            signal_reasoning: None,
        }
    }

    /// Convert a token profile to NewsItem
    fn profile_to_news_item(&self, profile: TokenProfile) -> NewsItem {
        // Get token name - header field sometimes contains URLs, so filter those out
        let fallback_name = truncate_address(&profile.token_address);
        let token_name = profile.header
            .as_deref()
            .filter(|h| !h.starts_with("http"))
            .unwrap_or(&fallback_name);

        let title = format!(
            "📊 New Token on {}: {}",
            format_chain(&profile.chain_id),
            token_name
        );

        let summary = profile.description.clone().unwrap_or_else(|| {
            format!(
                "New token profile: {} on {}",
                truncate_address(&profile.token_address),
                format_chain(&profile.chain_id)
            )
        });

        NewsItem {
            id: generate_id(&profile.url),
            title,
            url: profile.url,
            published_at: Utc::now(),
            source: NewsSource {
                name: format!("DexScreener • {}", format_chain(&profile.chain_id)),
                url: "https://dexscreener.com".to_string(),
                favicon_url: Some("https://dexscreener.com/favicon.ico".to_string()),
            },
            summary,
            content: None,
            image_url: profile.icon,
            relevance_score: 0.5,
            related_market_ids: vec![],
            search_query: None,
            // AI-enriched fields (set later by NewsAnalyzer)
            matched_market: None,
            price_signal: None,
            suggested_action: None,
            signal_reasoning: None,
        }
    }
}

impl Default for DexScreenerClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Format chain ID to human readable
fn format_chain(chain_id: &str) -> &str {
    match chain_id {
        "solana" => "Solana",
        "ethereum" => "Ethereum",
        "bsc" => "BSC",
        "polygon" => "Polygon",
        "arbitrum" => "Arbitrum",
        "base" => "Base",
        "avalanche" => "Avalanche",
        "optimism" => "Optimism",
        _ => chain_id,
    }
}

/// Truncate token address for display
fn truncate_address(addr: &str) -> String {
    if addr.len() > 12 {
        format!("{}...{}", &addr[..6], &addr[addr.len()-4..])
    } else {
        addr.to_string()
    }
}

/// Generate stable ID
fn generate_id(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let result = hasher.finalize();
    format!("dex_{}", hex::encode(&result[..8]))
}

/// DexScreener API errors
#[derive(Debug, thiserror::Error)]
pub enum DexScreenerError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("HTTP error: {0}")]
    HttpError(u16),

    #[error("Parse error: {0}")]
    ParseError(String),
}
