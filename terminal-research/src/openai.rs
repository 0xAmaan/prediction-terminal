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

/// Response from analyzing a follow-up question
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowUpAnalysis {
    /// Whether the question can be answered from existing research
    pub can_answer_from_context: bool,
    /// If can_answer_from_context is true, this contains the answer
    pub answer: Option<String>,
    /// If can_answer_from_context is false, these are the search queries needed
    #[serde(default)]
    pub search_queries: Vec<String>,
    /// Brief explanation of why research is or isn't needed
    pub reasoning: String,
}

impl OpenAIClient {
    /// Analyze a follow-up question to determine if it can be answered from existing research
    ///
    /// Returns an analysis indicating whether the question can be answered from context
    /// or if new research is needed.
    #[instrument(skip(self, existing_report))]
    pub async fn analyze_followup(
        &self,
        question: &str,
        existing_report: &SynthesizedReport,
    ) -> Result<FollowUpAnalysis, TerminalError> {
        let system_prompt = r#"You are a research analyst helping answer follow-up questions about prediction market research.

Your task is to analyze whether a follow-up question can be answered using the existing research report, or if new web searches are needed.

Consider:
1. Does the existing report contain information directly relevant to the question?
2. Can the answer be reasonably inferred from the existing data?
3. Is the question asking for new information not covered in the report?
4. Is the question about recent developments that might not be in the report?

Respond with valid JSON in this exact format:
{
  "can_answer_from_context": true/false,
  "answer": "If can_answer_from_context is true, provide a thorough answer here. Otherwise, null.",
  "search_queries": ["If can_answer_from_context is false, provide 1-3 targeted search queries"],
  "reasoning": "Brief explanation of your decision"
}"#;

        let report_summary = format!(
            "# {}\n\n## Executive Summary\n{}\n\n## Key Factors\n{}\n\n## Sections\n{}",
            existing_report.title,
            existing_report.executive_summary,
            existing_report
                .key_factors
                .iter()
                .map(|f| format!("- {} ({}, {})", f.factor, f.impact, f.confidence))
                .collect::<Vec<_>>()
                .join("\n"),
            existing_report
                .sections
                .iter()
                .map(|s| format!("### {}\n{}", s.heading, s.content))
                .collect::<Vec<_>>()
                .join("\n\n")
        );

        let user_prompt = format!(
            "## Existing Research Report\n\n{}\n\n## Follow-up Question\n\n{}",
            report_summary, question
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

        let json_str = extract_json(content)?;

        serde_json::from_str(&json_str)
            .map_err(|e| TerminalError::parse(format!("Failed to parse follow-up analysis: {}", e)))
    }

    /// Answer a question using only the existing research context
    ///
    /// This is used when the analysis determines the question can be answered
    /// without additional research.
    #[instrument(skip(self, existing_report))]
    pub async fn answer_from_context(
        &self,
        question: &str,
        existing_report: &SynthesizedReport,
    ) -> Result<String, TerminalError> {
        let system_prompt = r#"You are a research analyst providing detailed answers based on existing research.

Your task is to answer the user's question thoroughly using only the information in the provided research report.
Be comprehensive but stay focused on what the report contains. If there are limitations to what can be answered,
acknowledge them but provide the best answer possible from the available information.

Respond in clear, well-structured prose. Use markdown formatting if helpful."#;

        let report_summary = format!(
            "# {}\n\n## Executive Summary\n{}\n\n## Key Factors\n{}\n\n## Sections\n{}\n\n## Confidence Assessment\n{}",
            existing_report.title,
            existing_report.executive_summary,
            existing_report
                .key_factors
                .iter()
                .map(|f| format!("- {} ({}, {})", f.factor, f.impact, f.confidence))
                .collect::<Vec<_>>()
                .join("\n"),
            existing_report
                .sections
                .iter()
                .map(|s| format!("### {}\n{}", s.heading, s.content))
                .collect::<Vec<_>>()
                .join("\n\n"),
            existing_report.confidence_assessment
        );

        let user_prompt = format!(
            "## Research Report\n\n{}\n\n## Question\n\n{}",
            report_summary, question
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
            .max_tokens(2000u32)
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

        Ok(content.clone())
    }

    /// Generate targeted search queries for a follow-up question
    ///
    /// Creates 1-3 search queries that will help answer the question.
    #[instrument(skip(self, existing_report))]
    pub async fn generate_search_queries(
        &self,
        question: &str,
        existing_report: &SynthesizedReport,
    ) -> Result<Vec<String>, TerminalError> {
        let system_prompt = r#"You are a research analyst generating search queries to answer follow-up questions.

Given a question and existing research, generate 1-3 targeted web search queries that will find
information to answer the question. The queries should:
1. Be specific and focused
2. Target recent/current information
3. Complement rather than duplicate existing research

Respond with valid JSON in this exact format:
{
  "queries": ["search query 1", "search query 2"]
}"#;

        let user_prompt = format!(
            "## Research Topic\n{}\n\n## Existing Key Factors\n{}\n\n## Follow-up Question\n{}",
            existing_report.title,
            existing_report
                .key_factors
                .iter()
                .map(|f| format!("- {}", f.factor))
                .collect::<Vec<_>>()
                .join("\n"),
            question
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

        let json_str = extract_json(content)?;

        #[derive(Deserialize)]
        struct QueriesResponse {
            queries: Vec<String>,
        }

        let parsed: QueriesResponse = serde_json::from_str(&json_str)
            .map_err(|e| TerminalError::parse(format!("Failed to parse queries: {}", e)))?;

        Ok(parsed.queries)
    }

    /// Update an existing report with new research findings
    ///
    /// Takes the existing report, new search results, and the user's question,
    /// and generates an updated report that incorporates the new information.
    #[instrument(skip(self, existing_report, new_findings))]
    pub async fn update_report(
        &self,
        existing_report: &SynthesizedReport,
        new_findings: &[(String, Vec<ExaSearchResult>)],
        question: &str,
    ) -> Result<SynthesizedReport, TerminalError> {
        let system_prompt = r#"You are a research analyst updating an existing report with new information.

Your task is to:
1. Review the existing report structure
2. Incorporate new research findings to answer the user's follow-up question
3. Update the report minimally - only add/modify what's needed for the new information
4. Maintain the same JSON structure as the original report

Guidelines:
- Keep existing sections if they're still accurate
- Add new sections only if the new information warrants it
- Update the executive summary if significant new findings are added
- Add any new key factors discovered
- Update confidence assessment if new information changes certainty
- Add new sources from the search results

Respond with valid JSON in the same format as the original report:
{
  "title": "Same title as original",
  "executive_summary": "Updated summary incorporating new findings",
  "sections": [...],
  "key_factors": [...],
  "confidence_assessment": "Updated assessment",
  "sources": ["all sources including new ones"]
}"#;

        // Build context from new search results
        let mut new_context = String::new();
        for (query, results) in new_findings {
            new_context.push_str(&format!("\n## Search: {}\n", query));
            for result in results.iter().take(5) {
                if let Some(title) = &result.title {
                    new_context.push_str(&format!("\n### {}\n", title));
                }
                new_context.push_str(&format!("URL: {}\n", result.url));
                if let Some(date) = &result.published_date {
                    new_context.push_str(&format!("Date: {}\n", date));
                }
                if let Some(highlights) = &result.highlights {
                    for highlight in highlights {
                        new_context.push_str(&format!("- {}\n", highlight));
                    }
                }
                if let Some(text) = &result.text {
                    let truncated: String = text.chars().take(1500).collect();
                    new_context.push_str(&format!("\n{}\n", truncated));
                }
            }
        }

        let existing_report_json = serde_json::to_string_pretty(existing_report)
            .map_err(|e| TerminalError::internal(format!("Failed to serialize report: {}", e)))?;

        let user_prompt = format!(
            "## User's Follow-up Question\n{}\n\n## Existing Report (JSON)\n```json\n{}\n```\n\n## New Research Findings\n{}",
            question, existing_report_json, new_context
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
            .map_err(|e| TerminalError::parse(format!("Failed to parse updated report: {}", e)))
    }

    /// Generate a brief summary of the changes made during follow-up research
    ///
    /// Returns a concise, formatted summary suitable for chat display.
    #[instrument(skip(self, existing_report, updated_report))]
    pub async fn summarize_followup_changes(
        &self,
        question: &str,
        existing_report: &SynthesizedReport,
        updated_report: &SynthesizedReport,
    ) -> Result<String, TerminalError> {
        let system_prompt = r#"You are a research assistant summarizing changes made to a report.

Generate a brief, readable summary of the NEW information that was added to answer the user's question.

Guidelines:
- Be concise - 2-4 bullet points maximum
- Use markdown formatting with bullet points (- )
- Each bullet should be 1-2 sentences max
- Focus on the key NEW findings, not everything in the report
- If a specific topic was requested (like a time period), highlight findings about that topic

Format your response as:
- First key finding
- Second key finding
- Third key finding (if applicable)"#;

        let user_prompt = format!(
            "User's question: {}\n\nOriginal executive summary:\n{}\n\nUpdated executive summary:\n{}\n\nWhat are the key NEW findings added to answer this question?",
            question,
            existing_report.executive_summary,
            updated_report.executive_summary
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
            .max_tokens(300u32)
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

        Ok(content.trim().to_string())
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
