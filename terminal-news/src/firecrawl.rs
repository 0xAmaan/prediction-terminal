//! Firecrawl API client for article scraping

use reqwest::Client;
use tracing::{debug, instrument};

use crate::error::NewsError;
use crate::types::{ArticleContent, FirecrawlScrapeRequest, FirecrawlScrapeResponse};

/// Firecrawl API client
pub struct FirecrawlClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl FirecrawlClient {
    /// Create a new Firecrawl client
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.firecrawl.dev".to_string(),
        }
    }

    /// Scrape an article URL and return its content
    #[instrument(skip(self))]
    pub async fn scrape_article(&self, url: &str) -> Result<ArticleContent, NewsError> {
        let request = FirecrawlScrapeRequest {
            url: url.to_string(),
            formats: vec!["markdown".to_string()],
            only_main_content: true,
        };

        debug!("Scraping article: {}", url);

        let response = self
            .client
            .post(format!("{}/v1/scrape", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| NewsError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(NewsError::ApiError {
                status: status.as_u16(),
                message: body,
            });
        }

        let firecrawl_response: FirecrawlScrapeResponse = response
            .json()
            .await
            .map_err(|e| NewsError::ParseError(e.to_string()))?;

        if !firecrawl_response.success {
            return Err(NewsError::ScrapeFailed(
                firecrawl_response
                    .error
                    .unwrap_or_else(|| "Unknown scrape error".to_string()),
            ));
        }

        let data = firecrawl_response
            .data
            .ok_or_else(|| NewsError::ScrapeFailed("No data in response".to_string()))?;

        let markdown = data
            .markdown
            .ok_or_else(|| NewsError::ScrapeFailed("No markdown content".to_string()))?;

        debug!("Successfully scraped {} chars", markdown.len());

        Ok(ArticleContent {
            markdown,
            title: data.metadata.as_ref().and_then(|m| m.title.clone()),
            description: data.metadata.as_ref().and_then(|m| m.description.clone()),
            image_url: data.metadata.as_ref().and_then(|m| m.og_image.clone()),
            site_name: data.metadata.as_ref().and_then(|m| m.site_name.clone()),
        })
    }

    /// Check if the client is configured (has an API key)
    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }
}
