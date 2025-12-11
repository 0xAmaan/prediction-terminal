use reqwest::Client;
use serde::{Deserialize, Serialize};
use terminal_core::TerminalError;
use tracing::instrument;

const EXA_API_BASE: &str = "https://api.exa.ai";

#[derive(Debug, Clone)]
pub struct ExaClient {
    client: Client,
    api_key: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExaSearchRequest {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_results: Option<u32>,
    #[serde(rename = "type")]
    pub search_type: String, // "auto", "neural", "fast"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>, // "news", "research paper", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_published_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_domains: Option<Vec<String>>,
    pub contents: ExaContentsOptions,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExaContentsOptions {
    pub text: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlights: Option<ExaHighlightOptions>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExaHighlightOptions {
    pub num_sentences: u32,
    pub highlights_per_url: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExaSearchResponse {
    pub results: Vec<ExaSearchResult>,
    #[serde(default)]
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExaSearchResult {
    pub url: String,
    pub title: Option<String>,
    pub id: String,
    pub published_date: Option<String>,
    pub author: Option<String>,
    pub text: Option<String>,
    pub highlights: Option<Vec<String>>,
    pub highlight_scores: Option<Vec<f64>>,
}

impl ExaClient {
    pub fn new() -> Result<Self, TerminalError> {
        let api_key = std::env::var("EXA_API_KEY")
            .map_err(|_| TerminalError::config("EXA_API_KEY environment variable not set"))?;

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| TerminalError::network(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, api_key })
    }

    #[instrument(skip(self))]
    pub async fn search(&self, request: ExaSearchRequest) -> Result<ExaSearchResponse, TerminalError> {
        let url = format!("{}/search", EXA_API_BASE);

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| TerminalError::network(format!("Exa API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TerminalError::api(format!("Exa API error ({}): {}", status, body)));
        }

        response
            .json()
            .await
            .map_err(|e| TerminalError::parse(format!("Failed to parse Exa response: {}", e)))
    }

    /// Convenience method for news search with recent date filter
    pub async fn search_news(
        &self,
        query: &str,
        days_back: u32,
        num_results: u32,
    ) -> Result<ExaSearchResponse, TerminalError> {
        let start_date = chrono::Utc::now()
            .checked_sub_signed(chrono::Duration::days(days_back as i64))
            .map(|d| d.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string());

        let request = ExaSearchRequest {
            query: query.to_string(),
            num_results: Some(num_results),
            search_type: "auto".to_string(),
            category: Some("news".to_string()),
            start_published_date: start_date,
            include_domains: None,
            exclude_domains: None,
            contents: ExaContentsOptions {
                text: true,
                highlights: Some(ExaHighlightOptions {
                    num_sentences: 3,
                    highlights_per_url: 2,
                    query: Some(query.to_string()),
                }),
            },
        };

        self.search(request).await
    }

    /// Convenience method for research/analysis search
    pub async fn search_research(
        &self,
        query: &str,
        num_results: u32,
    ) -> Result<ExaSearchResponse, TerminalError> {
        let request = ExaSearchRequest {
            query: query.to_string(),
            num_results: Some(num_results),
            search_type: "neural".to_string(), // Semantic search for research
            category: None,
            start_published_date: None,
            include_domains: None,
            exclude_domains: None,
            contents: ExaContentsOptions {
                text: true,
                highlights: Some(ExaHighlightOptions {
                    num_sentences: 5,
                    highlights_per_url: 3,
                    query: Some(query.to_string()),
                }),
            },
        };

        self.search(request).await
    }
}
