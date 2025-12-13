//! Deep Research Agent for Prediction Market Analysis
//!
//! This crate provides AI-powered research capabilities for prediction markets,
//! using Exa AI for semantic search and OpenAI for question decomposition and synthesis.

pub mod exa;
pub mod openai;
pub mod storage;
pub mod types;

pub use exa::{ExaClient, ExaSearchRequest, ExaSearchResponse, ExaSearchResult};
pub use openai::{
    DecomposedQuestions, FollowUpAnalysis, KeyFactor, OpenAIClient, ReportSection, SubQuestion,
    SynthesizedReport,
};
pub use storage::ResearchStorage;
pub use types::{
    Catalyst, CatalystImpact, ChatHistory, ChatMessage, ChatRole, ContrarianAnalysis, Direction,
    DocumentEdit, DocumentEditOperation, EstimateConfidence, FollowUpRequest, FollowUpResponse,
    MarketContext, OrderBookSummary, RecentTrade, ResearchJob, ResearchProgress, ResearchStatus,
    ResearchUpdate, ResearchVersion, ResearchVersionList, ResolutionAnalysis, TradingAnalysis,
};
