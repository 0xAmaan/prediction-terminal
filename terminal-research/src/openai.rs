use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use serde::{Deserialize, Serialize};
use terminal_core::TerminalError;
use tracing::instrument;

use crate::exa::ExaSearchResult;

#[derive(Debug, Clone)]
pub struct OpenAIClient {
    client: Client<OpenAIConfig>,
    model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecomposedQuestions {
    pub main_question: String,
    pub sub_questions: Vec<SubQuestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubQuestion {
    pub question: String,
    pub category: String, // "news", "analysis", "historical", "expert_opinion"
    pub search_query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesizedReport {
    pub title: String,
    pub executive_summary: String,
    pub sections: Vec<ReportSection>,
    pub key_factors: Vec<KeyFactor>,
    pub confidence_assessment: String,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    pub heading: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyFactor {
    pub factor: String,
    pub impact: String,     // "bullish", "bearish", "neutral"
    pub confidence: String, // "high", "medium", "low"
}

impl OpenAIClient {
    pub fn new() -> Result<Self, TerminalError> {
        // async-openai reads OPENAI_API_KEY from env automatically
        let config = OpenAIConfig::default();
        let client = Client::with_config(config);

        Ok(Self {
            client,
            model: "gpt-4o".to_string(),
        })
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    #[instrument(skip(self))]
    pub async fn decompose_question(
        &self,
        market_title: &str,
        market_description: &str,
    ) -> Result<DecomposedQuestions, TerminalError> {
        let system_prompt = r#"You are a research analyst specializing in prediction markets.
Your task is to decompose a market question into sub-questions that will help gather comprehensive information.

For each sub-question, assign a category:
- "news": Recent news and developments
- "analysis": Expert analysis and opinions
- "historical": Historical precedents and data
- "expert_opinion": Domain expert perspectives

Respond with valid JSON in this exact format:
{
  "main_question": "The original question",
  "sub_questions": [
    {
      "question": "A specific sub-question",
      "category": "news",
      "search_query": "Optimized search query for this question"
    }
  ]
}

Generate 4-6 diverse sub-questions covering different angles."#;

        let user_prompt = format!(
            "Market Title: {}\n\nDescription: {}\n\nDecompose this into research sub-questions.",
            market_title, market_description
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(system_prompt)
                    .build()
                    .map_err(|e| TerminalError::internal(e.to_string()))?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(user_prompt)
                    .build()
                    .map_err(|e| TerminalError::internal(e.to_string()))?
                    .into(),
            ])
            .temperature(0.3)
            .build()
            .map_err(|e| TerminalError::internal(e.to_string()))?;

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| TerminalError::api(format!("OpenAI API error: {}", e)))?;

        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.as_ref())
            .ok_or_else(|| TerminalError::parse("No response from OpenAI"))?;

        // Extract JSON from response (handle markdown code blocks)
        let json_str = extract_json(content)?;

        serde_json::from_str(&json_str)
            .map_err(|e| TerminalError::parse(format!("Failed to parse decomposition: {}", e)))
    }

    #[instrument(skip(self, search_results))]
    pub async fn synthesize_report(
        &self,
        market_title: &str,
        market_description: &str,
        questions: &DecomposedQuestions,
        search_results: &[(SubQuestion, Vec<ExaSearchResult>)],
    ) -> Result<SynthesizedReport, TerminalError> {
        let system_prompt = r#"You are a research analyst creating comprehensive reports for prediction market traders.

Synthesize the provided search results into a detailed research report. Be objective and balanced.

Respond with valid JSON in this exact format:
{
  "title": "Report title",
  "executive_summary": "2-3 paragraph summary of key findings",
  "sections": [
    {
      "heading": "Section heading",
      "content": "Detailed markdown content with analysis"
    }
  ],
  "key_factors": [
    {
      "factor": "Description of the factor",
      "impact": "bullish|bearish|neutral",
      "confidence": "high|medium|low"
    }
  ],
  "confidence_assessment": "Overall assessment of information quality and gaps",
  "sources": ["url1", "url2"]
}

Include 4-6 sections covering different aspects. Use markdown formatting in content."#;

        // Build context from search results
        let mut context = String::new();
        for (question, results) in search_results {
            context.push_str(&format!("\n## {}\n", question.question));
            for result in results.iter().take(5) {
                if let Some(title) = &result.title {
                    context.push_str(&format!("\n### {}\n", title));
                }
                context.push_str(&format!("URL: {}\n", result.url));
                if let Some(date) = &result.published_date {
                    context.push_str(&format!("Date: {}\n", date));
                }
                if let Some(highlights) = &result.highlights {
                    for highlight in highlights {
                        context.push_str(&format!("- {}\n", highlight));
                    }
                }
                if let Some(text) = &result.text {
                    // Truncate text to avoid token limits
                    let truncated: String = text.chars().take(1500).collect();
                    context.push_str(&format!("\n{}\n", truncated));
                }
            }
        }

        let user_prompt = format!(
            "Market: {}\n\nDescription: {}\n\n## Research Data\n{}",
            market_title, market_description, context
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(system_prompt)
                    .build()
                    .map_err(|e| TerminalError::internal(e.to_string()))?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(user_prompt)
                    .build()
                    .map_err(|e| TerminalError::internal(e.to_string()))?
                    .into(),
            ])
            .temperature(0.4)
            .max_tokens(4000u32)
            .build()
            .map_err(|e| TerminalError::internal(e.to_string()))?;

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| TerminalError::api(format!("OpenAI API error: {}", e)))?;

        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.as_ref())
            .ok_or_else(|| TerminalError::parse("No response from OpenAI"))?;

        let json_str = extract_json(content)?;

        serde_json::from_str(&json_str)
            .map_err(|e| TerminalError::parse(format!("Failed to parse report: {}", e)))
    }
}

/// Extract JSON from a string that might contain markdown code blocks
fn extract_json(content: &str) -> Result<String, TerminalError> {
    // Try to find JSON in code blocks first
    if let Some(start) = content.find("```json") {
        let start = start + 7;
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
