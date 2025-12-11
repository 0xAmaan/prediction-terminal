use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use terminal_core::Platform;
use uuid::Uuid;

use crate::openai::SynthesizedReport;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchJob {
    pub id: String,
    pub platform: Platform,
    pub market_id: String,
    pub market_title: String,
    pub status: ResearchStatus,
    pub progress: ResearchProgress,
    pub report: Option<SynthesizedReport>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub cached: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResearchStatus {
    Pending,
    Decomposing,
    Searching,
    Analyzing,
    Synthesizing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResearchProgress {
    pub current_step: String,
    pub total_steps: u32,
    pub completed_steps: u32,
    pub current_query: Option<String>,
    pub searches_completed: u32,
    pub searches_total: u32,
}

impl ResearchJob {
    pub fn new(platform: Platform, market_id: &str, market_title: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            platform,
            market_id: market_id.to_string(),
            market_title: market_title.to_string(),
            status: ResearchStatus::Pending,
            progress: ResearchProgress::default(),
            report: None,
            error: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            cached: false,
        }
    }

    pub fn cache_key(&self) -> String {
        format!("research/{:?}/{}", self.platform, self.market_id).to_lowercase()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResearchUpdate {
    StatusChanged {
        job_id: String,
        status: ResearchStatus,
    },
    ProgressUpdate {
        job_id: String,
        progress: ResearchProgress,
    },
    Completed {
        job_id: String,
        report: SynthesizedReport,
    },
    Failed {
        job_id: String,
        error: String,
    },
}
