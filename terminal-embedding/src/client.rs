//! OpenAI embedding client

use async_openai::{
    config::OpenAIConfig,
    types::embeddings::{CreateEmbeddingRequest, EmbeddingInput},
    Client,
};
use tracing::{debug, info, instrument};

use crate::{error::Result, types::EmbeddingVector};

/// OpenAI embedding client
pub struct EmbeddingClient {
    client: Client<OpenAIConfig>,
    model: String,
    dimension: usize,
}

impl EmbeddingClient {
    /// Create a new embedding client
    ///
    /// Uses text-embedding-3-small model (1536 dimensions)
    /// Cost: $0.00002 per 1K tokens
    pub fn new(api_key: String) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        Self {
            client: Client::with_config(config),
            model: "text-embedding-3-small".to_string(),
            dimension: 1536,
        }
    }

    /// Generate embedding for a prediction market
    ///
    /// Builds rich context from:
    /// - Market title (most important)
    /// - Market description (optional)
    /// - Outcome titles (optional, for context)
    #[instrument(skip(self, description, outcomes))]
    pub async fn embed_market(
        &self,
        title: &str,
        description: Option<&str>,
        outcomes: Option<&Vec<String>>,
    ) -> Result<EmbeddingVector> {
        let mut text = format!("Prediction Market: {}", title);

        if let Some(desc) = description {
            text.push_str(&format!("\nDescription: {}", desc));
        }

        if let Some(opts) = outcomes {
            if !opts.is_empty() {
                let outcomes_text = opts
                    .iter()
                    .take(10) // Limit to top 10 outcomes
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                text.push_str(&format!("\nPossible Outcomes: {}", outcomes_text));
            }
        }

        debug!("Embedding market with text length: {} chars", text.len());
        self.generate_embedding(&text).await
    }

    /// Generate embedding for a news article
    ///
    /// Weights title more heavily by repeating it 3x
    /// This ensures title keywords dominate the embedding
    #[instrument(skip(self))]
    pub async fn embed_news(&self, title: &str, summary: &str) -> Result<EmbeddingVector> {
        // Weight title 3x more than summary
        let text = format!("{}\n{}\n{}\n{}", title, title, title, summary);

        debug!("Embedding news with text length: {} chars", text.len());
        self.generate_embedding(&text).await
    }

    /// Generate embedding for arbitrary text
    pub async fn embed_text(&self, text: &str) -> Result<EmbeddingVector> {
        self.generate_embedding(text).await
    }

    /// Low-level embedding generation
    async fn generate_embedding(&self, text: &str) -> Result<EmbeddingVector> {
        let request = CreateEmbeddingRequest {
            model: self.model.clone(),
            input: EmbeddingInput::String(text.to_string()),
            encoding_format: None,
            dimensions: None,
            user: None,
        };

        let response = self.client.embeddings().create(request).await?;

        if response.data.is_empty() {
            return Err(crate::error::EmbeddingError::Config(
                "No embeddings returned from API".to_string(),
            ));
        }

        let embedding = response.data[0].embedding.clone();

        // Validate dimension
        if embedding.len() != self.dimension {
            return Err(crate::error::EmbeddingError::InvalidDimension {
                expected: self.dimension,
                actual: embedding.len(),
            });
        }

        info!(
            "Generated embedding: dimension={}, model={}",
            embedding.len(),
            self.model
        );

        Ok(embedding)
    }

    /// Get the embedding model name
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get the embedding dimension
    pub fn dimension(&self) -> usize {
        self.dimension
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires API key
    async fn test_embed_market() {
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        let client = EmbeddingClient::new(api_key);

        let embedding = client
            .embed_market(
                "Will Bitcoin hit $100k by end of 2025?",
                Some("Prediction market for BTC price target"),
                Some(&vec!["Yes".to_string(), "No".to_string()]),
            )
            .await
            .expect("Failed to generate embedding");

        assert_eq!(embedding.len(), 1536);
    }

    #[tokio::test]
    #[ignore] // Requires API key
    async fn test_embed_news() {
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        let client = EmbeddingClient::new(api_key);

        let embedding = client
            .embed_news(
                "Bitcoin surges past $95k on ETF optimism",
                "Bitcoin reached a new high as institutional investors pour into crypto ETFs",
            )
            .await
            .expect("Failed to generate embedding");

        assert_eq!(embedding.len(), 1536);
    }
}
