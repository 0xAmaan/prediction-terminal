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
use crate::types::{MarketContext, OrderBookSummary, RecentTrade};

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
    /// Purpose of this question for trading analysis
    #[serde(default)]
    pub purpose: Option<String>, // "base_rate", "market_pricing", "catalyst", "contrarian", "resolution", "information_asymmetry"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesizedReport {
    pub title: String,
    pub executive_summary: String,
    pub sections: Vec<ReportSection>,
    pub key_factors: Vec<KeyFactor>,
    pub confidence_assessment: String,
    pub sources: Vec<String>,
    /// Trading-specific analysis (fair value, catalysts, resolution, contrarian view)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trading_analysis: Option<crate::types::TradingAnalysis>,
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

    #[instrument(skip(self, context))]
    pub async fn decompose_question(
        &self,
        context: &MarketContext,
    ) -> Result<DecomposedQuestions, TerminalError> {
        let system_prompt = r#"You are a quantitative prediction market analyst. Your job is to find EDGE—where the market price might be wrong.

You will receive a market question with current price and trading data. Decompose it into 5-6 sub-questions designed to find trading edge, NOT just general information.

REQUIRED sub-question types (include at least one of each):

1. BASE RATE: "What is the historical frequency of [this type of event]?"
   - Category: "historical"
   - Purpose: "base_rate"
   - Example: "What percentage of incumbent presidents win re-election historically?"

2. MARKET PRICING: "What assumptions is the market making at [current price]?"
   - Category: "analysis"
   - Purpose: "market_pricing"
   - Helps understand what's priced in

3. CATALYSTS: "What upcoming events could move this market significantly?"
   - Category: "news"
   - Purpose: "catalyst"
   - Focus on dated, scheduled events

4. CONTRARIAN: "What's the case against the current market consensus?"
   - Category: "analysis"
   - Purpose: "contrarian"
   - If market is high, search for bear case; if low, search for bull case

5. RESOLUTION: "How exactly does this market resolve and what are edge cases?"
   - Category: "analysis"
   - Purpose: "resolution"
   - Focus on the specific resolution criteria

6. INFORMATION ASYMMETRY: "What might informed traders know that isn't public?"
   - Category: "news"
   - Purpose: "information_asymmetry"
   - Recent insider activity, smart money movements

Respond with valid JSON in this exact format:
{
  "main_question": "The original market question",
  "sub_questions": [
    {
      "question": "Specific sub-question",
      "category": "news|analysis|historical|expert_opinion",
      "search_query": "Optimized search query for this question",
      "purpose": "base_rate|market_pricing|catalyst|contrarian|resolution|information_asymmetry"
    }
  ]
}

Generate exactly 6 sub-questions, one for each purpose."#;

        let user_prompt = format!(
            r#"## Market
Title: {}
Description: {}

## Current Market Data
- Current Price: {}
- 24h Change: {}
- 24h Volume: {}
- Total Volume: {}
{}
{}

Decompose this into research sub-questions that account for the current market state."#,
            context.title,
            context.description.as_deref().unwrap_or("No description"),
            format_price(context.current_price),
            format_price_change(context.current_price, context.price_24h_ago),
            format_volume(context.volume_24h),
            format_volume(context.total_volume),
            format_recent_trades(&context.recent_trades),
            format_order_book(&context.order_book_summary),
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

    #[instrument(skip(self, context, search_results))]
    pub async fn synthesize_report(
        &self,
        context: &MarketContext,
        questions: &DecomposedQuestions,
        search_results: &[(SubQuestion, Vec<ExaSearchResult>)],
    ) -> Result<SynthesizedReport, TerminalError> {
        let system_prompt = r#"You are a research analyst creating comprehensive reports for prediction market traders.

Synthesize the provided search results into a detailed research report. Be objective and balanced.

You have access to:
1. Market title and description
2. Current market data (price, volume, recent trades, order book)
3. Research findings from web searches

Use the market data to contextualize your analysis. Reference the current probability and recent price movements in your executive summary and confidence assessment.

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

        // Build research context from search results
        let mut research_data = String::new();
        for (question, results) in search_results {
            research_data.push_str(&format!("\n## {}\n", question.question));
            for result in results.iter().take(5) {
                if let Some(title) = &result.title {
                    research_data.push_str(&format!("\n### {}\n", title));
                }
                research_data.push_str(&format!("URL: {}\n", result.url));
                if let Some(date) = &result.published_date {
                    research_data.push_str(&format!("Date: {}\n", date));
                }
                if let Some(highlights) = &result.highlights {
                    for highlight in highlights {
                        research_data.push_str(&format!("- {}\n", highlight));
                    }
                }
                if let Some(text) = &result.text {
                    // Truncate text to avoid token limits
                    let truncated: String = text.chars().take(1500).collect();
                    research_data.push_str(&format!("\n{}\n", truncated));
                }
            }
        }

        let user_prompt = format!(
            r#"## Market
Title: {}
Description: {}

## Current Market Data
- Current Price: {}
- 24h Change: {}
- 24h Volume: {}
- Total Volume: {}
{}
{}

## Research Data
{}"#,
            context.title,
            context.description.as_deref().unwrap_or("No description"),
            format_price(context.current_price),
            format_price_change(context.current_price, context.price_24h_ago),
            format_volume(context.volume_24h),
            format_volume(context.total_volume),
            format_recent_trades(&context.recent_trades),
            format_order_book(&context.order_book_summary),
            research_data
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

    /// Generate trading-focused analysis from research results
    ///
    /// Takes the market context, synthesized report, and search results to produce
    /// actionable trading intelligence including fair value estimates, catalysts,
    /// resolution analysis, and contrarian viewpoints.
    #[instrument(skip(self, context, report, search_results))]
    pub async fn generate_trading_analysis(
        &self,
        context: &MarketContext,
        report: &SynthesizedReport,
        search_results: &[(SubQuestion, Vec<crate::exa::ExaSearchResult>)],
    ) -> Result<crate::types::TradingAnalysis, TerminalError> {
        let system_prompt = r#"You are a quantitative trading analyst for prediction markets.
Your job is to translate research into actionable trading intelligence.

You will receive:
1. Market context (current price, volume, order flow)
2. A research report with findings
3. Raw search results

Your task is to produce a TradingAnalysis with:

1. FAIR VALUE ESTIMATE
   - Estimate a probability RANGE (not a point estimate)
   - Be specific: "0.52 to 0.58" not "around 0.55"
   - Consider base rates, current evidence, and uncertainty
   - Compare to current market price to identify edge
   - If genuinely uncertain, use a wide range (e.g., 0.40 to 0.60)

2. CATALYSTS
   - List specific upcoming events with dates when known
   - Estimate impact level (high/medium/low)
   - Indicate likely direction if event is favorable

3. RESOLUTION ANALYSIS
   - Summarize EXACTLY how this market resolves
   - Flag ANY ambiguities in resolution criteria
   - Note historical edge cases from similar markets

4. CONTRARIAN ANALYSIS
   - State what the market consensus appears to be
   - Make the strongest case AGAINST that consensus
   - List specific reasons the crowd might be wrong
   - What would need to happen for the contrarian view to win

Be intellectually honest. If there's no edge, say so. If uncertainty is high, reflect that in a wide fair value range.

Respond with valid JSON matching this exact schema:
{
  "fair_value_low": 0.52,
  "fair_value_high": 0.58,
  "current_price": 0.55,
  "implied_edge": 0.0,
  "estimate_confidence": "high|medium|low",
  "fair_value_reasoning": "Explanation of fair value estimate",
  "catalysts": [
    {
      "date": "2025-01-15 or null if unknown",
      "event": "Description of the event",
      "expected_impact": "high|medium|low",
      "direction_if_positive": "bullish|bearish or null"
    }
  ],
  "resolution_analysis": {
    "resolution_summary": "Plain English summary of how this market resolves",
    "resolution_source": "The exact source used for resolution or null",
    "ambiguity_flags": ["List of potential ambiguities"],
    "historical_edge_cases": ["Historical edge cases from similar markets"]
  },
  "contrarian_case": {
    "consensus_view": "What the market consensus appears to be",
    "contrarian_case": "The case for why consensus might be wrong",
    "mispricing_reasons": ["Specific reasons the crowd could be wrong"],
    "contrarian_triggers": ["What would need to happen for contrarian view to win"]
  }
}"#;

        // Build search findings summary
        let search_findings = search_results
            .iter()
            .take(10)
            .map(|(q, results)| {
                let findings: String = results
                    .iter()
                    .take(2)
                    .filter_map(|r| r.text.as_ref())
                    .map(|t| {
                        if t.len() > 200 {
                            format!("{}...", &t[..200])
                        } else {
                            t.clone()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" | ");
                format!("Q: {}\nFindings: {}", q.question, findings)
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        // Format key factors
        let key_factors_str = report
            .key_factors
            .iter()
            .map(|f| format!("- {} ({}, {} confidence)", f.factor, f.impact, f.confidence))
            .collect::<Vec<_>>()
            .join("\n");

        // Format trade flow summary
        let trade_flow = if context.recent_trades.is_empty() {
            "No recent trades".to_string()
        } else {
            let buys: Vec<_> = context
                .recent_trades
                .iter()
                .filter(|t| t.side == "buy")
                .collect();
            let sells: Vec<_> = context
                .recent_trades
                .iter()
                .filter(|t| t.side == "sell")
                .collect();
            let buy_volume: f64 = buys.iter().map(|t| t.size).sum();
            let sell_volume: f64 = sells.iter().map(|t| t.size).sum();
            let flow_direction = if buy_volume > sell_volume * 1.5 {
                "bullish flow"
            } else if sell_volume > buy_volume * 1.5 {
                "bearish flow"
            } else {
                "balanced"
            };
            format!(
                "{} buys (${:.0}), {} sells (${:.0}) - {}",
                buys.len(),
                buy_volume,
                sells.len(),
                sell_volume,
                flow_direction
            )
        };

        let user_prompt = format!(
            r#"## Market
Title: {title}
Current Price: {price}

## Market Data
- 24h Volume: {volume}
- Recent Trade Flow: {trade_flow}
- Order Book: {orderbook}

## Research Report Summary
{executive_summary}

## Key Factors Found
{key_factors}

## Raw Search Findings
{search_findings}

Generate a TradingAnalysis for this market."#,
            title = context.title,
            price = format_price(context.current_price),
            volume = format_volume(context.volume_24h),
            trade_flow = trade_flow,
            orderbook = format_order_book(&context.order_book_summary),
            executive_summary = report.executive_summary,
            key_factors = key_factors_str,
            search_findings = search_findings,
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

        let json_str = extract_json(content)?;

        let mut analysis: crate::types::TradingAnalysis = serde_json::from_str(&json_str)
            .map_err(|e| TerminalError::parse(format!("Failed to parse trading analysis: {}", e)))?;

        // Calculate implied edge and set current price
        let fair_value_midpoint = (analysis.fair_value_low + analysis.fair_value_high) / 2.0;
        analysis.current_price = context.current_price.unwrap_or(0.0);
        analysis.implied_edge = fair_value_midpoint - analysis.current_price;

        Ok(analysis)
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

// ============================================================================
// Market Context Formatting Helpers
// ============================================================================

/// Format a price as percentage
fn format_price(price: Option<f64>) -> String {
    price
        .map(|p| format!("{:.1}%", p * 100.0))
        .unwrap_or_else(|| "Unknown".to_string())
}

/// Format the price change between current and previous
fn format_price_change(current: Option<f64>, previous: Option<f64>) -> String {
    match (current, previous) {
        (Some(c), Some(p)) => {
            let change = (c - p) * 100.0;
            if change >= 0.0 {
                format!("+{:.1}%", change)
            } else {
                format!("{:.1}%", change)
            }
        }
        _ => "Unknown".to_string(),
    }
}

/// Format volume with appropriate units (K, M)
fn format_volume(volume: Option<f64>) -> String {
    volume
        .map(|v| {
            if v >= 1_000_000.0 {
                format!("${:.1}M", v / 1_000_000.0)
            } else if v >= 1_000.0 {
                format!("${:.1}K", v / 1_000.0)
            } else {
                format!("${:.0}", v)
            }
        })
        .unwrap_or_else(|| "Unknown".to_string())
}

/// Format recent trades for the prompt
fn format_recent_trades(trades: &[RecentTrade]) -> String {
    if trades.is_empty() {
        return String::new();
    }

    let mut output = String::from("\n## Recent Trades\n");
    for trade in trades.iter().take(10) {
        output.push_str(&format!(
            "- {} {:.1}% (${:.0}) at {}\n",
            trade.side.to_uppercase(),
            trade.price * 100.0,
            trade.size,
            trade.timestamp
        ));
    }
    output
}

/// Format order book summary for the prompt
fn format_order_book(summary: &Option<OrderBookSummary>) -> String {
    match summary {
        Some(ob) => format!(
            "\n## Order Book\n- Best Bid: {}\n- Best Ask: {}\n- Spread: {}\n- Bid Depth (10%): ${:.0}\n- Ask Depth (10%): ${:.0}",
            format_price(ob.best_bid),
            format_price(ob.best_ask),
            ob.spread
                .map(|s| format!("{:.1}%", s * 100.0))
                .unwrap_or_else(|| "Unknown".to_string()),
            ob.bid_depth_10pct,
            ob.ask_depth_10pct,
        ),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_price() {
        assert_eq!(format_price(Some(0.73)), "73.0%");
        assert_eq!(format_price(None), "Unknown");
    }

    #[test]
    fn test_format_price_change() {
        assert_eq!(format_price_change(Some(0.75), Some(0.70)), "+5.0%");
        assert_eq!(format_price_change(Some(0.65), Some(0.70)), "-5.0%");
        assert_eq!(format_price_change(None, Some(0.70)), "Unknown");
    }

    #[test]
    fn test_format_volume() {
        assert_eq!(format_volume(Some(1_500_000.0)), "$1.5M");
        assert_eq!(format_volume(Some(50_000.0)), "$50.0K");
        assert_eq!(format_volume(Some(500.0)), "$500");
        assert_eq!(format_volume(None), "Unknown");
    }
}
