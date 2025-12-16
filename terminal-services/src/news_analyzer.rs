//! AI-powered news analysis for market trading signals
//!
//! This module analyzes news articles and matches them to relevant prediction
//! markets, generating trading signals based on how the news might affect
//! market prices.

use std::sync::Arc;

use terminal_core::{
    MatchedMarket, NewsItem, Platform, PredictionMarket, PriceSignal, SuggestedAction,
    TerminalError,
};
use terminal_research::OpenAIClient;
use tracing::{debug, instrument};

use crate::MarketCache;

/// Configuration for the news analyzer
#[derive(Debug, Clone)]
pub struct NewsAnalyzerConfig {
    /// Maximum number of markets to consider for matching
    pub max_markets_to_consider: usize,
    /// Minimum relevance threshold for market matching (0.0 - 1.0)
    pub min_relevance_threshold: f64,
}

impl Default for NewsAnalyzerConfig {
    fn default() -> Self {
        Self {
            max_markets_to_consider: 50,
            min_relevance_threshold: 0.3,
        }
    }
}

/// AI-powered news analyzer that matches news to markets and generates trading signals
pub struct NewsAnalyzer {
    openai_client: OpenAIClient,
    market_cache: Arc<MarketCache>,
    config: NewsAnalyzerConfig,
}

impl NewsAnalyzer {
    /// Create a new news analyzer
    pub fn new(market_cache: Arc<MarketCache>) -> Result<Self, TerminalError> {
        let openai_client = OpenAIClient::new()?;
        Ok(Self {
            openai_client,
            market_cache,
            config: NewsAnalyzerConfig::default(),
        })
    }

    /// Create a new news analyzer with custom config
    pub fn with_config(
        market_cache: Arc<MarketCache>,
        config: NewsAnalyzerConfig,
    ) -> Result<Self, TerminalError> {
        let openai_client = OpenAIClient::new()?;
        Ok(Self {
            openai_client,
            market_cache,
            config,
        })
    }

    /// Analyze a news item and enrich it with market matching and trading signals
    #[instrument(skip(self, news_item), fields(news_title = %news_item.title))]
    pub async fn analyze_news(&self, mut news_item: NewsItem) -> Result<NewsItem, TerminalError> {
        // Get top markets by volume from cache
        let markets = self.market_cache.get_markets(Some(Platform::Polymarket));

        // Take only the top N markets
        let markets: Vec<_> = markets
            .into_iter()
            .take(self.config.max_markets_to_consider)
            .collect();

        if markets.is_empty() {
            debug!("No markets available for matching");
            return Ok(news_item);
        }

        // Build prompt for AI analysis
        let analysis = self
            .analyze_with_ai(&news_item, &markets)
            .await?;

        // Apply the analysis results to the news item
        if let Some(matched) = analysis.matched_market {
            news_item.matched_market = Some(matched);
        }
        news_item.price_signal = analysis.price_signal;
        news_item.suggested_action = analysis.suggested_action;
        news_item.signal_reasoning = analysis.signal_reasoning;

        Ok(news_item)
    }

    /// Use AI to analyze the news and match it to a market
    async fn analyze_with_ai(
        &self,
        news_item: &NewsItem,
        markets: &[PredictionMarket],
    ) -> Result<NewsAnalysis, TerminalError> {
        // Build market list for the prompt (top 20 by volume)
        let market_list: String = markets
            .iter()
            .take(20)
            .enumerate()
            .map(|(i, m)| {
                let price_pct = m.yes_price.to_string().parse::<f64>().unwrap_or(0.0) * 100.0;
                format!(
                    "{}. [{}] {} ({}% YES, ${:.0}K volume)",
                    i + 1,
                    m.id,
                    m.title,
                    price_pct,
                    m.volume.to_string().parse::<f64>().unwrap_or(0.0) / 1000.0
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let current_date = chrono::Utc::now().format("%Y-%m-%d").to_string();

        let system_prompt = format!(r#"You are a prediction market trading analyst. Today's date is {}.

Your job is to analyze news articles and determine:
1. Which prediction market (if any) the news is most relevant to
2. For multi-outcome markets, which specific outcome is affected
3. Whether the news suggests the market is mispriced (underpriced or overpriced)
4. What trading action (if any) the news suggests

Be conservative - only suggest a signal if the news has a clear, direct impact on a market's probability.

Respond with valid JSON in this exact format:
{{
  "matched_market_id": "the market ID from the list, or null if no good match",
  "specific_outcome": "for multi-outcome markets, the specific person/option affected, or null for binary markets",
  "relevance_explanation": "why this market matches, or why no market matches",
  "price_signal": "underpriced|overpriced|neutral",
  "suggested_action": "buy|sell|hold",
  "signal_reasoning": "1-2 sentence explanation mentioning the specific outcome if applicable"
}}

Guidelines:
- Only match to a market if the news directly affects its outcome
- For multi-outcome markets, identify the SPECIFIC outcome affected by the news
- Consider eligibility constraints (term limits, age requirements, legal restrictions, etc.)
- If someone is ineligible for a position, news about them should be marked "neutral" for that market
- "underpriced" means the news suggests probability should be HIGHER than current price
- "overpriced" means the news suggests probability should be LOWER than current price
- "neutral" means the news doesn't clearly indicate mispricing OR there are eligibility issues
- Be skeptical - most news is noise, not signal
- Consider the current market price when assessing mispricing"#, current_date);

        let user_prompt = format!(
            r#"## News Article
Title: {}
Summary: {}
Source: {}
Published: {}

## Available Markets
{}

Analyze this news and determine if it's relevant to any of these markets."#,
            news_item.title,
            news_item.summary,
            news_item.source.name,
            news_item.published_at.format("%Y-%m-%d %H:%M UTC"),
            market_list
        );

        // Call OpenAI
        let response = self
            .openai_client
            .simple_chat(&system_prompt, &user_prompt)
            .await?;

        // Parse the response
        let analysis: RawNewsAnalysis = serde_json::from_str(&extract_json(&response)?)
            .map_err(|e| TerminalError::parse(format!("Failed to parse AI response: {}", e)))?;

        // Convert to our internal format
        let matched_market = if let Some(market_id) = analysis.matched_market_id {
            markets.iter().find(|m| m.id == market_id).map(|m| {
                MatchedMarket {
                    platform: m.platform,
                    market_id: m.id.clone(),
                    title: m.title.clone(),
                    current_price: m.yes_price.to_string().parse().unwrap_or(0.0),
                    url: m.url.clone(),
                    outcome: analysis.specific_outcome.clone(),
                }
            })
        } else {
            None
        };

        let price_signal = match analysis.price_signal.as_str() {
            "underpriced" => Some(PriceSignal::Underpriced),
            "overpriced" => Some(PriceSignal::Overpriced),
            _ => Some(PriceSignal::Neutral),
        };

        let suggested_action = match analysis.suggested_action.as_str() {
            "buy" => Some(SuggestedAction::Buy),
            "sell" => Some(SuggestedAction::Sell),
            _ => Some(SuggestedAction::Hold),
        };

        Ok(NewsAnalysis {
            matched_market,
            price_signal,
            suggested_action,
            signal_reasoning: if analysis.signal_reasoning.is_empty() {
                None
            } else {
                Some(analysis.signal_reasoning)
            },
        })
    }
}

/// Internal analysis result
struct NewsAnalysis {
    matched_market: Option<MatchedMarket>,
    price_signal: Option<PriceSignal>,
    suggested_action: Option<SuggestedAction>,
    signal_reasoning: Option<String>,
}

/// Raw JSON response from AI
#[derive(serde::Deserialize)]
struct RawNewsAnalysis {
    matched_market_id: Option<String>,
    specific_outcome: Option<String>,
    #[allow(dead_code)]
    relevance_explanation: String,
    price_signal: String,
    suggested_action: String,
    signal_reasoning: String,
}

/// Extract JSON from a response that might contain markdown code blocks
fn extract_json(content: &str) -> Result<String, TerminalError> {
    // Try to find JSON in code blocks first
    if let Some(start) = content.find("```json") {
        let start = start + 7;
        if let Some(end) = content[start..].find("```") {
            return Ok(content[start..start + end].trim().to_string());
        }
    }

    // Try plain code blocks
    if let Some(start) = content.find("```") {
        let start = start + 3;
        // Skip language identifier if present
        let start = content[start..]
            .find('\n')
            .map(|n| start + n + 1)
            .unwrap_or(start);
        if let Some(end) = content[start..].find("```") {
            return Ok(content[start..start + end].trim().to_string());
        }
    }

    // Try to find raw JSON
    if let Some(start) = content.find('{') {
        if let Some(end) = content.rfind('}') {
            return Ok(content[start..=end].to_string());
        }
    }

    Err(TerminalError::parse("No JSON found in response"))
}
