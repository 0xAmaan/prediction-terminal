//! API-specific types for Exa.ai and Firecrawl

use serde::{Deserialize, Serialize};

// ============================================================================
// Exa.ai Types
// ============================================================================

/// Exa.ai search request
#[derive(Debug, Serialize)]
pub struct ExaSearchRequest {
    /// Search query
    pub query: String,
    /// Number of results to return
    #[serde(rename = "numResults")]
    pub num_results: usize,
    /// Use autoprompt for better results
    #[serde(rename = "useAutoprompt")]
    pub use_autoprompt: bool,
    /// Search type: "neural" for semantic search
    #[serde(rename = "type")]
    pub search_type: String,
    /// Category filter (e.g., "news")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Start date filter (ISO 8601)
    #[serde(rename = "startPublishedDate", skip_serializing_if = "Option::is_none")]
    pub start_published_date: Option<String>,
    /// End date filter (ISO 8601)
    #[serde(rename = "endPublishedDate", skip_serializing_if = "Option::is_none")]
    pub end_published_date: Option<String>,
    /// Domains to exclude from results
    #[serde(rename = "excludeDomains", skip_serializing_if = "Option::is_none")]
    pub exclude_domains: Option<Vec<String>>,
    /// Domains to include (whitelist) - only return results from these domains
    #[serde(rename = "includeDomains", skip_serializing_if = "Option::is_none")]
    pub include_domains: Option<Vec<String>>,
    /// Content options
    pub contents: ExaContentsOptions,
}

/// Options for content extraction
#[derive(Debug, Serialize)]
pub struct ExaContentsOptions {
    /// Text extraction options
    pub text: ExaTextOptions,
    /// Highlight extraction options
    pub highlights: ExaHighlightsOptions,
}

/// Text extraction options
#[derive(Debug, Serialize)]
pub struct ExaTextOptions {
    /// Maximum characters to extract
    #[serde(rename = "maxCharacters")]
    pub max_characters: usize,
    /// Include HTML tags
    #[serde(rename = "includeHtmlTags")]
    pub include_html_tags: bool,
}

/// Highlight extraction options
#[derive(Debug, Serialize)]
pub struct ExaHighlightsOptions {
    /// Number of highlights to extract
    #[serde(rename = "numSentences")]
    pub num_sentences: usize,
    /// Highlights per URL
    #[serde(rename = "highlightsPerUrl")]
    pub highlights_per_url: usize,
}

/// Exa.ai search response
#[derive(Debug, Deserialize)]
pub struct ExaSearchResponse {
    /// Search results
    pub results: Vec<ExaResult>,
    /// Request ID for debugging
    #[serde(rename = "requestId")]
    pub request_id: Option<String>,
}

/// A single Exa.ai search result
#[derive(Debug, Deserialize)]
pub struct ExaResult {
    /// Result ID
    pub id: String,
    /// Page URL
    pub url: String,
    /// Page title
    pub title: String,
    /// Publication date (ISO 8601)
    #[serde(rename = "publishedDate")]
    pub published_date: Option<String>,
    /// Author name
    pub author: Option<String>,
    /// Extracted text content
    pub text: Option<String>,
    /// Highlighted sentences
    pub highlights: Option<Vec<String>>,
    /// Relevance score (0.0 - 1.0)
    pub score: f64,
    /// Image URL
    pub image: Option<String>,
}

// ============================================================================
// Firecrawl Types
// ============================================================================

/// Firecrawl scrape request
#[derive(Debug, Serialize)]
pub struct FirecrawlScrapeRequest {
    /// URL to scrape
    pub url: String,
    /// Output formats to return
    pub formats: Vec<String>,
    /// Only extract main content
    #[serde(rename = "onlyMainContent")]
    pub only_main_content: bool,
}

/// Firecrawl scrape response
#[derive(Debug, Deserialize)]
pub struct FirecrawlScrapeResponse {
    /// Whether the scrape was successful
    pub success: bool,
    /// Scraped data
    pub data: Option<FirecrawlScrapeData>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Scraped data from Firecrawl
#[derive(Debug, Deserialize)]
pub struct FirecrawlScrapeData {
    /// Markdown content
    pub markdown: Option<String>,
    /// HTML content
    pub html: Option<String>,
    /// Page metadata
    pub metadata: Option<FirecrawlMetadata>,
}

/// Page metadata from Firecrawl
#[derive(Debug, Deserialize)]
pub struct FirecrawlMetadata {
    /// Page title
    pub title: Option<String>,
    /// Page description
    pub description: Option<String>,
    /// Language
    pub language: Option<String>,
    /// Source URL
    #[serde(rename = "sourceURL")]
    pub source_url: Option<String>,
    /// OG image URL
    #[serde(rename = "ogImage")]
    pub og_image: Option<String>,
    /// Site name
    #[serde(rename = "siteName")]
    pub site_name: Option<String>,
}

/// Scraped article content
#[derive(Debug, Clone)]
pub struct ArticleContent {
    /// Markdown content
    pub markdown: String,
    /// Article title
    pub title: Option<String>,
    /// Article description
    pub description: Option<String>,
    /// Image URL
    pub image_url: Option<String>,
    /// Site name
    pub site_name: Option<String>,
}
