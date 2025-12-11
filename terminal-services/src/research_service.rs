//! Research service for AI-powered market analysis
//!
//! This service orchestrates the deep research agent, managing research jobs
//! and broadcasting progress updates via channels.

use std::{collections::HashMap, sync::Arc};

use terminal_core::{Platform, TerminalError};
use terminal_research::{
    ExaClient, ExaSearchResult, OpenAIClient, ResearchJob, ResearchProgress, ResearchStatus,
    ResearchUpdate, SubQuestion, SynthesizedReport,
};
use tokio::sync::{broadcast, RwLock};
use tracing::{info, instrument};

use crate::MarketService;

/// Service for managing AI-powered market research
pub struct ResearchService {
    market_service: Arc<MarketService>,
    exa_client: ExaClient,
    openai_client: OpenAIClient,
    jobs: RwLock<HashMap<String, ResearchJob>>,
    update_tx: broadcast::Sender<ResearchUpdate>,
}

impl ResearchService {
    /// Create a new research service
    ///
    /// Requires EXA_API_KEY and OPENAI_API_KEY environment variables to be set.
    pub fn new(market_service: Arc<MarketService>) -> Result<Self, TerminalError> {
        let exa_client = ExaClient::new()?;
        let openai_client = OpenAIClient::new()?;
        let (update_tx, _) = broadcast::channel(100);

        Ok(Self {
            market_service,
            exa_client,
            openai_client,
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
    #[instrument(skip(self))]
    pub async fn start_research(
        &self,
        platform: Platform,
        market_id: &str,
    ) -> Result<ResearchJob, TerminalError> {
        // Fetch market details
        let market = self.market_service.get_market(platform, market_id).await?;

        // TODO: Implement S3 cache lookup

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
        market: &terminal_core::PredictionMarket,
    ) -> Result<SynthesizedReport, TerminalError> {
        let job_id = &job.id;

        // Step 1: Decompose question
        self.update_status(job_id, ResearchStatus::Decomposing)
            .await;
        self.update_progress(job_id, "Analyzing market question...", 1, 4, None)
            .await;

        let questions = self
            .openai_client
            .decompose_question(&market.title, market.description.as_deref().unwrap_or(""))
            .await?;

        info!(
            "Decomposed into {} sub-questions",
            questions.sub_questions.len()
        );

        // Step 2: Execute searches
        self.update_status(job_id, ResearchStatus::Searching).await;
        let total_searches = questions.sub_questions.len() as u32;

        let mut search_results: Vec<(SubQuestion, Vec<ExaSearchResult>)> = Vec::new();

        for (i, question) in questions.sub_questions.iter().enumerate() {
            self.update_progress(
                job_id,
                &format!("Searching: {}", question.category),
                2,
                4,
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
        self.update_progress(job_id, "Analyzing search results...", 3, 4, None)
            .await;

        // Step 4: Synthesize report
        self.update_status(job_id, ResearchStatus::Synthesizing)
            .await;
        self.update_progress(job_id, "Generating research report...", 4, 4, None)
            .await;

        let report = self
            .openai_client
            .synthesize_report(
                &market.title,
                market.description.as_deref().unwrap_or(""),
                &questions,
                &search_results,
            )
            .await?;

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

    /// Mark job as completed with report
    async fn update_job_completed(&self, job_id: &str, report: SynthesizedReport) {
        {
            let mut jobs = self.jobs.write().await;
            if let Some(job) = jobs.get_mut(job_id) {
                job.status = ResearchStatus::Completed;
                job.report = Some(report.clone());
                job.updated_at = chrono::Utc::now();
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
}

impl Clone for ResearchService {
    fn clone(&self) -> Self {
        Self {
            market_service: self.market_service.clone(),
            exa_client: self.exa_client.clone(),
            openai_client: self.openai_client.clone(),
            jobs: RwLock::new(HashMap::new()), // Fresh jobs map
            update_tx: self.update_tx.clone(),
        }
    }
}
