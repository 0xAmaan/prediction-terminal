//! Research service for AI-powered market analysis
//!
//! This service orchestrates the deep research agent, managing research jobs
//! and broadcasting progress updates via channels.

use std::{collections::HashMap, sync::Arc};

use terminal_core::{Platform, TerminalError};
use terminal_research::{
    ChatHistory, ChatMessage, ExaClient, ExaSearchResult, FollowUpAnalysis, MarketContext,
    OpenAIClient, OrderBookSummary, RecentTrade, ResearchJob, ResearchProgress, ResearchStatus,
    ResearchStorage, ResearchUpdate, ResearchVersion, SubQuestion, SynthesizedReport,
    fetch_resolution_sources,
};
use tokio::sync::{broadcast, RwLock};
use tracing::{info, instrument, warn};

use crate::MarketService;

/// Service for managing AI-powered market research
pub struct ResearchService {
    market_service: Arc<MarketService>,
    exa_client: ExaClient,
    openai_client: OpenAIClient,
    storage: Option<ResearchStorage>,
    jobs: RwLock<HashMap<String, ResearchJob>>,
    update_tx: broadcast::Sender<ResearchUpdate>,
}

impl ResearchService {
    /// Create a new research service
    ///
    /// Requires EXA_API_KEY and OPENAI_API_KEY environment variables to be set.
    /// AWS credentials are optional - if not provided, caching will be disabled.
    pub async fn new(market_service: Arc<MarketService>) -> Result<Self, TerminalError> {
        let exa_client = ExaClient::new()?;
        let openai_client = OpenAIClient::new()?;
        let (update_tx, _) = broadcast::channel(100);

        // Try to initialize storage, but don't fail if AWS credentials aren't configured
        let storage = match ResearchStorage::new().await {
            Ok(s) => {
                info!("S3 research cache enabled");
                Some(s)
            }
            Err(e) => {
                warn!("S3 research cache disabled: {}", e);
                None
            }
        };

        Ok(Self {
            market_service,
            exa_client,
            openai_client,
            storage,
            jobs: RwLock::new(HashMap::new()),
            update_tx,
        })
    }

    /// Subscribe to research updates
    pub fn subscribe(&self) -> broadcast::Receiver<ResearchUpdate> {
        self.update_tx.subscribe()
    }

    /// Start a new research job for a market
    ///
    /// Returns the job immediately - research executes in background via `execute_research`.
    /// If a cached result exists and is still valid (< 24 hours old), returns it immediately
    /// with `cached: true`.
    #[instrument(skip(self))]
    pub async fn start_research(
        &self,
        platform: Platform,
        market_id: &str,
    ) -> Result<ResearchJob, TerminalError> {
        // Check S3 cache first
        if let Some(ref storage) = self.storage {
            let cache_key = ResearchStorage::cache_key(platform, market_id);
            match storage.get_cached(&cache_key).await {
                Ok(Some(mut cached_job)) => {
                    info!("Returning cached research for {}/{}", platform, market_id);
                    cached_job.cached = true;

                    // Store in local jobs map for API access
                    {
                        let mut jobs = self.jobs.write().await;
                        jobs.insert(cached_job.id.clone(), cached_job.clone());
                    }

                    return Ok(cached_job);
                }
                Ok(None) => {
                    info!("No valid cache for {}/{}", platform, market_id);
                }
                Err(e) => {
                    warn!("Cache lookup failed: {}", e);
                    // Continue without cache
                }
            }
        }

        // Fetch market details
        let market = self.market_service.get_market(platform, market_id).await?;

        // Create new job
        let job = ResearchJob::new(platform, market_id, &market.title);
        let job_id = job.id.clone();

        // Store job
        {
            let mut jobs = self.jobs.write().await;
            jobs.insert(job_id.clone(), job.clone());
        }

        // Return job immediately, research runs in background
        Ok(job)
    }

    /// Execute the research pipeline for a job
    ///
    /// This should be called in a background task after `start_research`.
    #[instrument(skip(self))]
    pub async fn execute_research(&self, job_id: &str) -> Result<(), TerminalError> {
        // Get job
        let job = {
            let jobs = self.jobs.read().await;
            jobs.get(job_id).cloned()
        }
        .ok_or_else(|| TerminalError::not_found(format!("Job not found: {}", job_id)))?;

        // Fetch market
        let market = self
            .market_service
            .get_market(job.platform, &job.market_id)
            .await?;

        // Execute research pipeline
        let result = self.run_research_pipeline(&job, &market).await;

        match result {
            Ok(report) => {
                self.update_job_completed(job_id, report).await;
            }
            Err(e) => {
                self.update_job_failed(job_id, &e.to_string()).await;
            }
        }

        Ok(())
    }

    /// Run the full research pipeline
    async fn run_research_pipeline(
        &self,
        job: &ResearchJob,
        _market: &terminal_core::PredictionMarket,
    ) -> Result<SynthesizedReport, TerminalError> {
        let job_id = &job.id;

        // Build rich market context with real-time data
        let context = self
            .build_market_context(job.platform, &job.market_id)
            .await?;

        // Step 1: Decompose question (with market context)
        self.update_status(job_id, ResearchStatus::Decomposing)
            .await;
        self.update_progress(job_id, "Analyzing market question...", 1, 5, None)
            .await;

        let questions = self.openai_client.decompose_question(&context).await?;

        info!(
            "Decomposed into {} sub-questions",
            questions.sub_questions.len()
        );

        // Step 2: Execute searches
        // Note: Exa API has a 5 requests/second rate limit. We add delays between
        // requests to stay well under this limit and avoid 429 errors.
        self.update_status(job_id, ResearchStatus::Searching).await;
        let total_searches = questions.sub_questions.len() as u32;

        let mut search_results: Vec<(SubQuestion, Vec<ExaSearchResult>)> = Vec::new();

        for (i, question) in questions.sub_questions.iter().enumerate() {
            // Rate limit: Wait 250ms between Exa API calls (max 4 req/sec, well under 5 req/sec limit)
            if i > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            }

            self.update_progress(
                job_id,
                &format!("Searching: {}", question.category),
                2,
                5,
                Some(&question.search_query),
            )
            .await;

            self.update_search_progress(job_id, i as u32, total_searches)
                .await;

            let results = match question.category.as_str() {
                "news" => {
                    self.exa_client
                        .search_news(&question.search_query, 7, 5)
                        .await?
                }
                _ => {
                    self.exa_client
                        .search_research(&question.search_query, 5)
                        .await?
                }
            };

            search_results.push((question.clone(), results.results));
        }

        // Step 3: Analyze (included in synthesis)
        self.update_status(job_id, ResearchStatus::Analyzing).await;
        self.update_progress(job_id, "Analyzing search results...", 3, 5, None)
            .await;

        // Step 4: Synthesize report (with market context)
        self.update_status(job_id, ResearchStatus::Synthesizing)
            .await;
        self.update_progress(job_id, "Generating research report...", 4, 5, None)
            .await;

        let mut report = self
            .openai_client
            .synthesize_report(&context, &questions, &search_results)
            .await?;

        // Step 5: Generate trading analysis
        self.update_progress(job_id, "Generating trading analysis...", 5, 5, None)
            .await;

        let trading_analysis = self
            .openai_client
            .generate_trading_analysis(&context, &report, &search_results)
            .await
            .ok(); // Don't fail the whole job if trading analysis fails

        // Attach trading analysis to report
        report.trading_analysis = trading_analysis;

        Ok(report)
    }

    /// Update job status and broadcast
    async fn update_status(&self, job_id: &str, status: ResearchStatus) {
        {
            let mut jobs = self.jobs.write().await;
            if let Some(job) = jobs.get_mut(job_id) {
                job.status = status;
                job.updated_at = chrono::Utc::now();
            }
        }

        let _ = self.update_tx.send(ResearchUpdate::StatusChanged {
            job_id: job_id.to_string(),
            status,
        });
    }

    /// Update job progress and broadcast
    async fn update_progress(
        &self,
        job_id: &str,
        step: &str,
        completed: u32,
        total: u32,
        current_query: Option<&str>,
    ) {
        let progress = ResearchProgress {
            current_step: step.to_string(),
            total_steps: total,
            completed_steps: completed,
            current_query: current_query.map(String::from),
            searches_completed: 0,
            searches_total: 0,
        };

        {
            let mut jobs = self.jobs.write().await;
            if let Some(job) = jobs.get_mut(job_id) {
                job.progress = progress.clone();
                job.updated_at = chrono::Utc::now();
            }
        }

        let _ = self.update_tx.send(ResearchUpdate::ProgressUpdate {
            job_id: job_id.to_string(),
            progress,
        });
    }

    /// Update search progress counters
    async fn update_search_progress(&self, job_id: &str, completed: u32, total: u32) {
        {
            let mut jobs = self.jobs.write().await;
            if let Some(job) = jobs.get_mut(job_id) {
                job.progress.searches_completed = completed;
                job.progress.searches_total = total;
                job.updated_at = chrono::Utc::now();
            }
        }
    }

    /// Mark job as completed with report and save to S3 cache
    async fn update_job_completed(&self, job_id: &str, report: SynthesizedReport) {
        let completed_job = {
            let mut jobs = self.jobs.write().await;
            if let Some(job) = jobs.get_mut(job_id) {
                job.status = ResearchStatus::Completed;
                job.report = Some(report.clone());
                job.updated_at = chrono::Utc::now();
                Some(job.clone())
            } else {
                None
            }
        };

        // Save to S3 cache
        if let (Some(ref storage), Some(ref job)) = (&self.storage, &completed_job) {
            if let Err(e) = storage.save(job).await {
                warn!("Failed to cache research result: {}", e);
            }
        }

        let _ = self.update_tx.send(ResearchUpdate::Completed {
            job_id: job_id.to_string(),
            report,
        });
    }

    /// Mark job as failed with error
    async fn update_job_failed(&self, job_id: &str, error: &str) {
        {
            let mut jobs = self.jobs.write().await;
            if let Some(job) = jobs.get_mut(job_id) {
                job.status = ResearchStatus::Failed;
                job.error = Some(error.to_string());
                job.updated_at = chrono::Utc::now();
            }
        }

        let _ = self.update_tx.send(ResearchUpdate::Failed {
            job_id: job_id.to_string(),
            error: error.to_string(),
        });
    }

    /// Get a job by ID
    pub async fn get_job(&self, job_id: &str) -> Option<ResearchJob> {
        let jobs = self.jobs.read().await;
        jobs.get(job_id).cloned()
    }

    /// List all jobs
    pub async fn list_jobs(&self) -> Vec<ResearchJob> {
        let jobs = self.jobs.read().await;
        jobs.values().cloned().collect()
    }

    /// Build rich market context for AI research
    ///
    /// Fetches market details, recent trades, order book, and resolution source content
    /// to provide the AI with accurate real-time market data.
    #[instrument(skip(self))]
    async fn build_market_context(
        &self,
        platform: Platform,
        market_id: &str,
    ) -> Result<MarketContext, TerminalError> {
        // Fetch market details
        let market = self.market_service.get_market(platform, market_id).await?;

        // Fetch recent trades (best effort - don't fail if unavailable)
        let trades = self
            .market_service
            .get_trades(platform, market_id, Some(10), None)
            .await
            .ok();

        // Fetch order book (best effort)
        let order_book = self
            .market_service
            .get_orderbook(platform, market_id)
            .await
            .ok();

        // Fetch resolution source URLs if resolution rules contain URLs
        // This provides current data from the resolution source (e.g., leaderboard rankings)
        let resolution_source_content = fetch_resolution_sources(market.resolution_source.as_deref()).await;
        if !resolution_source_content.is_empty() {
            info!(
                "Fetched {} resolution sources for {}/{}",
                resolution_source_content.len(),
                platform,
                market_id
            );
        }

        // Convert Decimal to f64 for the context
        let current_price = market
            .yes_price
            .to_string()
            .parse::<f64>()
            .ok();

        let total_volume = market
            .volume
            .to_string()
            .parse::<f64>()
            .ok();

        // Build recent trades list
        let recent_trades: Vec<RecentTrade> = trades
            .map(|th| {
                th.trades
                    .into_iter()
                    .take(10)
                    .map(|t| RecentTrade {
                        price: t.price.to_string().parse().unwrap_or(0.0),
                        size: t.quantity.to_string().parse().unwrap_or(0.0),
                        side: t
                            .side
                            .map(|s| match s {
                                terminal_core::TradeSide::Buy => "buy".to_string(),
                                terminal_core::TradeSide::Sell => "sell".to_string(),
                            })
                            .unwrap_or_else(|| "unknown".to_string()),
                        timestamp: t.timestamp.to_rfc3339(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Build order book summary
        let order_book_summary = order_book.map(|ob| {
            let best_bid = ob.yes_bids.first().map(|l| {
                l.price.to_string().parse::<f64>().unwrap_or(0.0)
            });
            let best_ask = ob.yes_asks.first().map(|l| {
                l.price.to_string().parse::<f64>().unwrap_or(0.0)
            });
            let spread = match (best_bid, best_ask) {
                (Some(bid), Some(ask)) => Some(ask - bid),
                _ => None,
            };

            // Calculate depth within 10% of best price
            let bid_depth_10pct = calculate_depth(&ob.yes_bids, best_bid, 0.10);
            let ask_depth_10pct = calculate_depth(&ob.yes_asks, best_ask, 0.10);

            OrderBookSummary {
                best_bid,
                best_ask,
                spread,
                bid_depth_10pct,
                ask_depth_10pct,
            }
        });

        Ok(MarketContext {
            title: market.title,
            description: market.description,
            current_price,
            price_24h_ago: None, // TODO: Could fetch from price history if needed
            volume_24h: None,    // Not directly available from PredictionMarket
            total_volume,
            num_traders: None, // Not directly available from PredictionMarket
            recent_trades,
            order_book_summary,
            resolution_rules: market.resolution_source,
            resolution_source_content,
        })
    }

    /// Get cached research by platform and market ID (without starting new research)
    ///
    /// Returns Ok(Some(job)) if cached research exists and is valid (< 24 hours old)
    /// Returns Ok(None) if no cached research exists or it has expired
    #[instrument(skip(self))]
    pub async fn get_cached_research(
        &self,
        platform: Platform,
        market_id: &str,
    ) -> Result<Option<ResearchJob>, TerminalError> {
        // Check S3 cache
        if let Some(ref storage) = self.storage {
            let cache_key = ResearchStorage::cache_key(platform, market_id);
            match storage.get_cached(&cache_key).await {
                Ok(Some(mut cached_job)) => {
                    info!("Found cached research for {}/{}", platform, market_id);
                    cached_job.cached = true;
                    return Ok(Some(cached_job));
                }
                Ok(None) => {
                    info!("No valid cache for {}/{}", platform, market_id);
                    return Ok(None);
                }
                Err(e) => {
                    warn!("Cache lookup failed: {}", e);
                    return Err(e);
                }
            }
        }

        // No storage configured
        Ok(None)
    }

    /// List all versions for a market's research
    ///
    /// Returns a list of version metadata sorted by creation time (newest first).
    #[instrument(skip(self))]
    pub async fn list_versions(
        &self,
        platform: Platform,
        market_id: &str,
    ) -> Result<Vec<ResearchVersion>, TerminalError> {
        if let Some(ref storage) = self.storage {
            storage.list_versions(platform, market_id).await
        } else {
            // No storage configured, return empty list
            Ok(Vec::new())
        }
    }

    /// Get a specific version of research
    ///
    /// The version_key should be the filename like "v1702389600000.json"
    #[instrument(skip(self))]
    pub async fn get_version(
        &self,
        platform: Platform,
        market_id: &str,
        version_key: &str,
    ) -> Result<Option<ResearchJob>, TerminalError> {
        if let Some(ref storage) = self.storage {
            storage.get_version(platform, market_id, version_key).await
        } else {
            // No storage configured
            Ok(None)
        }
    }

    // ========================================================================
    // Chat Methods
    // ========================================================================

    /// Get chat history for a market's research
    ///
    /// Returns empty ChatHistory if no chat exists yet.
    #[instrument(skip(self))]
    pub async fn get_chat(
        &self,
        platform: Platform,
        market_id: &str,
    ) -> Result<ChatHistory, TerminalError> {
        if let Some(ref storage) = self.storage {
            storage.get_chat(platform, market_id).await
        } else {
            // No storage configured, return empty
            Ok(ChatHistory::new())
        }
    }

    /// Send a chat message and get a response
    ///
    /// This implements the follow-up research flow:
    /// 1. Analyze if the question can be answered from existing research
    /// 2. If yes, answer directly
    /// 3. If no, trigger targeted research and update the document
    #[instrument(skip(self))]
    pub async fn send_chat_message(
        &self,
        platform: Platform,
        market_id: &str,
        message: &str,
    ) -> Result<ChatMessage, TerminalError> {
        // Save user message
        let mut user_msg = ChatMessage::user(message);
        if let Some(ref storage) = self.storage {
            storage
                .append_message(platform, market_id, user_msg.clone())
                .await?;
        }

        // Get existing research
        let existing_job = self.get_cached_research(platform, market_id).await?;
        let existing_report = match existing_job.and_then(|j| j.report) {
            Some(report) => report,
            None => {
                // No existing research, return a helpful message
                let assistant_msg = ChatMessage::assistant(
                    "No research exists for this market yet. Please wait for the initial research to complete before asking follow-up questions.",
                );
                if let Some(ref storage) = self.storage {
                    storage
                        .append_message(platform, market_id, assistant_msg.clone())
                        .await?;
                }
                return Ok(assistant_msg);
            }
        };

        // Analyze the follow-up question
        let analysis = self
            .openai_client
            .analyze_followup(message, &existing_report)
            .await?;

        info!(
            "Follow-up analysis for {}/{}: can_answer_from_context={}, reasoning={}",
            platform, market_id, analysis.can_answer_from_context, analysis.reasoning
        );

        let (answer, research_triggered) = if analysis.can_answer_from_context {
            // Answer directly from existing research
            let answer = analysis.answer.unwrap_or_else(|| {
                // Fallback: generate answer if not provided in analysis
                analysis.reasoning.clone()
            });
            (answer, false)
        } else {
            // Need to do research - mark user message as triggering research
            user_msg.research_triggered = true;

            // Update user message with research_triggered flag
            if let Some(ref storage) = self.storage {
                // Re-fetch and update the chat history to mark the message
                let mut history = storage.get_chat(platform, market_id).await?;
                if let Some(last_user_msg) = history
                    .messages
                    .iter_mut()
                    .rev()
                    .find(|m| m.role == terminal_research::ChatRole::User && m.content == message)
                {
                    last_user_msg.research_triggered = true;
                }
                storage.save_chat(platform, market_id, &history).await?;
            }

            // Execute follow-up research
            let (answer, updated_report) = self
                .execute_followup_research(
                    platform,
                    market_id,
                    message,
                    &existing_report,
                    &analysis,
                )
                .await?;

            // Save updated report as new version
            if let Some(ref storage) = self.storage {
                let mut job = ResearchJob::new(platform, market_id, &existing_report.title);
                job.status = ResearchStatus::Completed;
                job.report = Some(updated_report.clone());
                job.updated_at = chrono::Utc::now();
                storage.save(&job).await?;
                info!("Saved updated research version for {}/{}", platform, market_id);

                // Broadcast the update
                let _ = self.update_tx.send(ResearchUpdate::FollowUpCompleted {
                    job_id: job.id,
                    report: updated_report,
                });
            }

            (answer, true)
        };

        // Create and save assistant response
        let mut assistant_msg = ChatMessage::assistant(&answer);
        assistant_msg.research_triggered = research_triggered;

        if let Some(ref storage) = self.storage {
            storage
                .append_message(platform, market_id, assistant_msg.clone())
                .await?;
        }

        info!(
            "Chat message processed for {}/{}: research_triggered={}",
            platform, market_id, research_triggered
        );

        Ok(assistant_msg)
    }

    /// Execute follow-up research to answer a question
    ///
    /// Returns the answer and the updated report.
    async fn execute_followup_research(
        &self,
        platform: Platform,
        market_id: &str,
        question: &str,
        existing_report: &SynthesizedReport,
        analysis: &FollowUpAnalysis,
    ) -> Result<(String, SynthesizedReport), TerminalError> {
        // Get search queries from analysis or generate new ones
        let search_queries = if !analysis.search_queries.is_empty() {
            analysis.search_queries.clone()
        } else {
            self.openai_client
                .generate_search_queries(question, existing_report)
                .await?
        };

        info!(
            "Executing {} searches for follow-up on {}/{}",
            search_queries.len(),
            platform,
            market_id
        );

        // Execute searches with rate limiting (Exa API: 5 req/sec limit)
        let mut search_results: Vec<(String, Vec<ExaSearchResult>)> = Vec::new();
        for (i, query) in search_queries.iter().enumerate() {
            // Rate limit: Wait 250ms between Exa API calls
            if i > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            }
            let results = self.exa_client.search_news(query, 7, 5).await?;
            search_results.push((query.clone(), results.results));
        }

        // Update the report with new findings
        let mut updated_report = self
            .openai_client
            .update_report(existing_report, &search_results, question)
            .await?;

        // Preserve trading_analysis from existing report (it's not regenerated during follow-up)
        if updated_report.trading_analysis.is_none() {
            updated_report.trading_analysis = existing_report.trading_analysis.clone();
        }

        // Generate a concise summary of the changes for the chat
        let summary = self
            .openai_client
            .summarize_followup_changes(question, existing_report, &updated_report)
            .await
            .unwrap_or_else(|_| {
                "The report has been updated with new details in the relevant sections.".to_string()
            });

        let answer = format!(
            "I've researched your question and updated the report with new findings.\n\n**Summary of new information:**\n\n{}",
            summary
        );

        Ok((answer, updated_report))
    }
}

impl Clone for ResearchService {
    fn clone(&self) -> Self {
        Self {
            market_service: self.market_service.clone(),
            exa_client: self.exa_client.clone(),
            openai_client: self.openai_client.clone(),
            storage: None, // Storage is not cloned - each instance would need its own
            jobs: RwLock::new(HashMap::new()), // Fresh jobs map
            update_tx: self.update_tx.clone(),
        }
    }
}

/// Calculate the total depth (in dollars) within a percentage of the best price
fn calculate_depth(
    levels: &[terminal_core::OrderBookLevel],
    best_price: Option<f64>,
    pct_range: f64,
) -> f64 {
    let Some(best) = best_price else {
        return 0.0;
    };

    let threshold = best * pct_range;

    levels
        .iter()
        .filter(|l| {
            let price = l.price.to_string().parse::<f64>().unwrap_or(0.0);
            (price - best).abs() <= threshold
        })
        .map(|l| l.quantity.to_string().parse::<f64>().unwrap_or(0.0))
        .sum()
}
