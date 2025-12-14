# Embedding + Semantic Mapping Architecture

## Overview

This document describes the **semantic layer** for news-to-market matching using vector embeddings. This augments (not replaces) the existing keyword-based filtering.

## Goals

1. **Semantic Understanding**: Match news by meaning, not just exact keywords
   - Example: "Federal Reserve rate hike" matches markets about "Fed policy", "Jerome Powell", "interest rates"

2. **Cross-Market Intelligence**: One news article relevant to multiple markets
   - Example: "Bitcoin ETF approval" relevant to BTC price, crypto regulation, ETF markets

3. **Entity Resolution**: Understand equivalent terms
   - "Fed" = "Federal Reserve" = "FOMC" = "Jerome Powell announcement"

4. **Improved Recall**: Find relevant news that keyword matching misses
   - Keyword: might miss "central bank policy" for "Fed" markets
   - Semantic: understands the relationship

## Architecture

### Components

```
┌─────────────────────────────────────────────────────────────┐
│                      News Service                            │
│  ┌──────────────┐  ┌───────────────┐  ┌─────────────────┐  │
│  │   Keyword    │  │   Semantic    │  │   Hybrid        │  │
│  │   Matching   │─▶│   Matching    │─▶│   Scoring       │  │
│  │  (existing)  │  │     (new)     │  │   (combined)    │  │
│  └──────────────┘  └───────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
          ┌───────────────────────────────────────┐
          │      Embedding Service                │
          │  ┌─────────────┐  ┌──────────────┐   │
          │  │  Generate   │  │   Similarity │   │
          │  │  Embeddings │  │   Scoring    │   │
          │  │  (OpenAI)   │  │  (cosine)    │   │
          │  └─────────────┘  └──────────────┘   │
          └───────────────────────────────────────┘
                              │
                              ▼
          ┌───────────────────────────────────────┐
          │      Embedding Store (SQLite)         │
          │  ┌──────────────────────────────────┐ │
          │  │  market_embeddings               │ │
          │  │  - market_id (TEXT PRIMARY KEY)  │ │
          │  │  - platform (TEXT)               │ │
          │  │  - embedding_text (TEXT)         │ │
          │  │  - embedding (BLOB)              │ │
          │  │  - dimension (INTEGER)           │ │
          │  │  - created_at (INTEGER)          │ │
          │  │  - updated_at (INTEGER)          │ │
          │  └──────────────────────────────────┘ │
          └───────────────────────────────────────┘
```

### Data Flow

#### 1. Market Embedding Generation (Background Task)
```
New Market Added
    │
    ▼
Build Embedding Text
  - Market title
  - Market description
  - Outcome titles (for context)
  - Category/tags
    │
    ▼
Generate Embedding via OpenAI
  - Model: text-embedding-3-small (1536 dims)
  - Cost: $0.00002 per 1K tokens
    │
    ▼
Store in SQLite
  - market_id: unique identifier
  - embedding: binary blob (1536 floats)
  - embedding_text: original text (for debugging)
```

#### 2. News Article Semantic Matching (Real-time)
```
News Article Arrives
    │
    ▼
Build Article Text
  - Title (3x weight)
  - Summary (1x weight)
    │
    ▼
Generate Embedding
  - Same model as markets
  - Cache for 24h (articles don't change)
    │
    ▼
Load Market Embeddings
  - Filter to active markets only
  - ~100-500 markets typically
    │
    ▼
Calculate Cosine Similarity
  - Formula: cos(θ) = A·B / (||A|| ||B||)
  - Fast: single matrix multiplication
  - Returns scores 0.0 (unrelated) to 1.0 (identical)
    │
    ▼
Filter by Threshold
  - Threshold: 0.70 (high similarity)
  - Typical relevant match: 0.72-0.85
  - Identical texts: 0.95-1.0
    │
    ▼
Return Matched Markets + Scores
```

#### 3. Hybrid Scoring (Combine Keyword + Semantic)
```
For each news article:

  keyword_score = keyword_matching_score (existing)
  semantic_score = max(cosine_similarities)

  # Combine scores (weighted average)
  final_score = (0.4 * keyword_score) + (0.6 * semantic_score)

  # Accept if either method finds relevance
  is_relevant = (keyword_score > 0.35) OR (semantic_score > 0.70)
```

## Implementation Plan

### Step 1: Add Dependencies

**Cargo.toml additions**:
```toml
[dependencies]
# OpenAI for embeddings
async-openai = "0.24"

# Vector operations
ndarray = "0.16"  # For efficient array operations

# Serialization for embeddings
bincode = "1.3"  # Serialize f32 arrays to BLOB
```

### Step 2: Database Schema

**New table**: `market_embeddings`
```sql
CREATE TABLE IF NOT EXISTS market_embeddings (
    market_id TEXT PRIMARY KEY,
    platform TEXT NOT NULL,
    embedding_text TEXT NOT NULL,     -- Original text used for embedding
    embedding BLOB NOT NULL,           -- 1536 f32 values (6144 bytes)
    dimension INTEGER NOT NULL,        -- Embedding dimension (1536)
    model TEXT NOT NULL,               -- Model version (text-embedding-3-small)
    created_at INTEGER NOT NULL,       -- Unix timestamp
    updated_at INTEGER NOT NULL        -- Unix timestamp
);

CREATE INDEX idx_market_embeddings_platform ON market_embeddings(platform);
CREATE INDEX idx_market_embeddings_updated ON market_embeddings(updated_at);
```

**New table**: `news_embeddings` (cache)
```sql
CREATE TABLE IF NOT EXISTS news_embeddings (
    article_id TEXT PRIMARY KEY,      -- SHA256 hash of URL
    embedding BLOB NOT NULL,           -- 1536 f32 values
    embedding_text TEXT NOT NULL,      -- Original text
    created_at INTEGER NOT NULL,       -- Unix timestamp
    expires_at INTEGER NOT NULL        -- TTL: 24 hours
);

CREATE INDEX idx_news_embeddings_expires ON news_embeddings(expires_at);
```

### Step 3: New Crate Structure

**Create**: `terminal-embedding/` crate

```
terminal-embedding/
├── Cargo.toml
└── src/
    ├── lib.rs              # Public API
    ├── client.rs           # OpenAI embedding client
    ├── store.rs            # SQLite storage
    ├── similarity.rs       # Cosine similarity calculations
    └── types.rs            # Embedding types
```

### Step 4: Core Types

```rust
// terminal-embedding/src/types.rs

/// Embedding vector (1536 dimensions for text-embedding-3-small)
pub type EmbeddingVector = Vec<f32>;

/// Market embedding with metadata
pub struct MarketEmbedding {
    pub market_id: String,
    pub platform: String,
    pub embedding_text: String,
    pub embedding: EmbeddingVector,
    pub dimension: usize,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// News article embedding (cached)
pub struct NewsEmbedding {
    pub article_id: String,
    pub embedding: EmbeddingVector,
    pub embedding_text: String,
    pub created_at: DateTime<Utc>,
}

/// Similarity match result
pub struct SimilarityMatch {
    pub market_id: String,
    pub platform: String,
    pub score: f64,  // Cosine similarity 0.0 - 1.0
}
```

### Step 5: Embedding Client

```rust
// terminal-embedding/src/client.rs

use async_openai::{Client, types::{CreateEmbeddingRequest, EmbeddingInput}};

pub struct EmbeddingClient {
    client: Client,
    model: String, // "text-embedding-3-small"
    dimension: usize, // 1536
}

impl EmbeddingClient {
    pub fn new(api_key: String) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        Self {
            client: Client::with_config(config),
            model: "text-embedding-3-small".to_string(),
            dimension: 1536,
        }
    }

    /// Generate embedding for market
    pub async fn embed_market(
        &self,
        title: &str,
        description: Option<&str>,
        outcomes: Option<&Vec<String>>,
    ) -> Result<EmbeddingVector> {
        // Build rich context
        let mut text = format!("Market: {}", title);

        if let Some(desc) = description {
            text.push_str(&format!("\nDescription: {}", desc));
        }

        if let Some(opts) = outcomes {
            text.push_str(&format!("\nOutcomes: {}", opts.join(", ")));
        }

        self.generate_embedding(&text).await
    }

    /// Generate embedding for news article
    pub async fn embed_news(
        &self,
        title: &str,
        summary: &str,
    ) -> Result<EmbeddingVector> {
        // Weight title more heavily (3x repetition)
        let text = format!("{}\n{}\n{}\n{}", title, title, title, summary);

        self.generate_embedding(&text).await
    }

    async fn generate_embedding(&self, text: &str) -> Result<EmbeddingVector> {
        let request = CreateEmbeddingRequest {
            model: self.model.clone(),
            input: EmbeddingInput::String(text.to_string()),
            encoding_format: None,
            user: None,
        };

        let response = self.client
            .embeddings()
            .create(request)
            .await?;

        Ok(response.data[0].embedding.clone())
    }
}
```

### Step 6: Similarity Calculation

```rust
// terminal-embedding/src/similarity.rs

use ndarray::{Array1, ArrayView1};

/// Calculate cosine similarity between two embeddings
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    assert_eq!(a.len(), b.len(), "Embeddings must have same dimension");

    let a_view = ArrayView1::from(a);
    let b_view = ArrayView1::from(b);

    let dot_product = a_view.dot(&b_view);
    let norm_a = a_view.dot(&a_view).sqrt();
    let norm_b = b_view.dot(&b_view).sqrt();

    (dot_product / (norm_a * norm_b)) as f64
}

/// Find top-K most similar markets for a news article
pub fn find_similar_markets(
    news_embedding: &[f32],
    market_embeddings: &[(String, String, Vec<f32>)], // (id, platform, embedding)
    top_k: usize,
    threshold: f64,
) -> Vec<SimilarityMatch> {
    let mut matches: Vec<SimilarityMatch> = market_embeddings
        .iter()
        .map(|(market_id, platform, embedding)| {
            let score = cosine_similarity(news_embedding, embedding);
            SimilarityMatch {
                market_id: market_id.clone(),
                platform: platform.clone(),
                score,
            }
        })
        .filter(|m| m.score >= threshold)
        .collect();

    // Sort by score descending
    matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    matches.truncate(top_k);

    matches
}
```

### Step 7: Embedding Store (SQLite)

```rust
// terminal-embedding/src/store.rs

pub struct EmbeddingStore {
    pool: SqlitePool,
}

impl EmbeddingStore {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;

        // Create tables
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS market_embeddings (
                market_id TEXT PRIMARY KEY,
                platform TEXT NOT NULL,
                embedding_text TEXT NOT NULL,
                embedding BLOB NOT NULL,
                dimension INTEGER NOT NULL,
                model TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )"
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    /// Save market embedding
    pub async fn save_market_embedding(&self, emb: &MarketEmbedding) -> Result<()> {
        let embedding_bytes = bincode::serialize(&emb.embedding)?;

        sqlx::query(
            "INSERT INTO market_embeddings
             (market_id, platform, embedding_text, embedding, dimension, model, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(market_id) DO UPDATE SET
                embedding = excluded.embedding,
                embedding_text = excluded.embedding_text,
                updated_at = excluded.updated_at"
        )
        .bind(&emb.market_id)
        .bind(&emb.platform)
        .bind(&emb.embedding_text)
        .bind(&embedding_bytes)
        .bind(emb.dimension as i64)
        .bind("text-embedding-3-small")
        .bind(emb.created_at.timestamp())
        .bind(emb.updated_at.timestamp())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Load all active market embeddings
    pub async fn load_all_market_embeddings(&self) -> Result<Vec<(String, String, Vec<f32>)>> {
        let rows = sqlx::query(
            "SELECT market_id, platform, embedding
             FROM market_embeddings"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut results = Vec::new();
        for row in rows {
            let market_id: String = row.get("market_id");
            let platform: String = row.get("platform");
            let embedding_bytes: Vec<u8> = row.get("embedding");
            let embedding: Vec<f32> = bincode::deserialize(&embedding_bytes)?;
            results.push((market_id, platform, embedding));
        }

        Ok(results)
    }
}
```

### Step 8: Integration with NewsService

```rust
// terminal-services/src/news_service.rs

pub struct NewsService {
    // ... existing fields ...
    embedding_client: Option<EmbeddingClient>,
    embedding_store: Option<EmbeddingStore>,
}

impl NewsService {
    /// Get market news with semantic matching
    pub async fn get_market_news_with_semantics(
        &self,
        market_title: &str,
        market_id: &str,
        limit: usize,
    ) -> Result<NewsFeed> {
        // 1. Get keyword-based results (existing)
        let keyword_results = self.get_market_news(market_title, market_id, limit).await?;

        // 2. If semantic search enabled, also get semantic matches
        if let (Some(client), Some(store)) = (&self.embedding_client, &self.embedding_store) {
            // Get all market embeddings
            let market_embeddings = store.load_all_market_embeddings().await?;

            // Get embeddings for RSS articles
            let rss_items = self.get_rss_items().await?;
            let mut semantic_results = Vec::new();

            for item in rss_items {
                // Generate embedding for article
                let news_embedding = client.embed_news(&item.title, &item.summary).await?;

                // Find similar markets
                let matches = find_similar_markets(
                    &news_embedding,
                    &market_embeddings,
                    10,      // top 10 markets
                    0.70,    // 70% similarity threshold
                );

                // Check if our market is in the matches
                for match in matches {
                    if match.market_id == market_id {
                        let mut item_copy = item.clone();
                        item_copy.relevance_score = match.score;
                        semantic_results.push(item_copy);
                        break;
                    }
                }
            }

            // 3. Combine keyword + semantic results
            let combined = combine_results(keyword_results, semantic_results, limit);
            return Ok(combined);
        }

        // Fallback to keyword-only
        Ok(keyword_results)
    }
}

fn combine_results(
    keyword: NewsFeed,
    semantic: Vec<NewsItem>,
    limit: usize,
) -> NewsFeed {
    let mut combined = std::collections::HashMap::new();

    // Add keyword results (prefer these)
    for item in keyword.items {
        combined.insert(item.id.clone(), item);
    }

    // Add semantic results (only if not already present)
    for item in semantic {
        combined.entry(item.id.clone()).or_insert(item);
    }

    // Sort by relevance score
    let mut items: Vec<NewsItem> = combined.into_values().collect();
    items.sort_by(|a, b| {
        b.relevance_score
            .partial_cmp(&a.relevance_score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.published_at.cmp(&a.published_at))
    });
    items.truncate(limit);

    NewsFeed {
        items,
        total_count: combined.len(),
        next_cursor: None,
    }
}
```

## Background Jobs

### Market Embedding Regeneration
Run daily or when markets are updated:

```rust
// Pseudo-code for background job
async fn regenerate_all_market_embeddings() {
    let markets = market_service.get_all_markets(None).await?;

    for market in markets {
        let embedding = embedding_client.embed_market(
            &market.title,
            market.description.as_deref(),
            Some(&market.outcomes.iter().map(|o| o.title.clone()).collect()),
        ).await?;

        embedding_store.save_market_embedding(&MarketEmbedding {
            market_id: market.id,
            platform: market.platform.to_string(),
            embedding_text: market.title.clone(),
            embedding,
            dimension: 1536,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }).await?;
    }
}
```

## Performance Considerations

### Embedding Generation Costs

**OpenAI Pricing** (text-embedding-3-small):
- Cost: $0.00002 per 1K tokens
- Average market: ~50-100 tokens → $0.000002 per market
- 1000 markets: $0.002 (negligible)
- Average news article: ~100 tokens → $0.000002 per article
- 1000 articles/day: $0.002/day = $0.60/month

**Total estimated cost**: $1-3/month for typical usage

### Latency

**Embedding generation**:
- OpenAI API: ~50-200ms per request
- Batch requests: can embed 100+ texts in single request

**Similarity calculation**:
- 100 markets: ~1-2ms (very fast, just dot products)
- 500 markets: ~5-10ms

**Caching strategy**:
- Market embeddings: regenerate daily (markets don't change often)
- News embeddings: cache for 24h (articles don't change)
- Preload market embeddings on service startup

### Storage

**SQLite storage**:
- 1 embedding: 6144 bytes (1536 floats × 4 bytes)
- 1000 markets: ~6 MB
- 10,000 news cache: ~60 MB
- Total: < 100 MB (tiny)

## Evaluation Metrics

### Success Criteria

1. **Improved Recall**: Find 20-30% more relevant articles
2. **Better Cross-Market**: Same article appears in 2-3 related markets
3. **User Feedback**: Track click-through rates on semantic vs keyword results

### A/B Testing

```rust
// Log which method found the article
pub struct NewsItem {
    // ... existing fields ...
    pub discovery_method: DiscoveryMethod,
}

pub enum DiscoveryMethod {
    Keyword,
    Semantic,
    Both,
}
```

Track metrics:
- % articles found by each method
- User engagement by discovery method
- False positive rates

## Future Enhancements

### 1. Fine-tuned Embeddings
Train custom embedding model on prediction market data:
- Better understanding of market-specific terminology
- Improved cross-market relationships

### 2. Multi-Vector Embeddings
Generate separate embeddings for:
- Market title
- Market description
- Outcomes
- Recent price action

Combine with weighted similarity.

### 3. Temporal Embeddings
Add time decay to similarity scores:
- Recent news → higher weight
- Old news → lower weight even if semantically similar

### 4. Real-time Embedding
Generate embeddings in streaming pipeline:
- Embed news as it arrives
- Instantly match to markets
- Push notifications for high-relevance matches

---

## Implementation Checklist

- [ ] Create `terminal-embedding` crate
- [ ] Add OpenAI client for embeddings
- [ ] Implement cosine similarity function
- [ ] Create SQLite schema for embeddings
- [ ] Implement embedding store (save/load)
- [ ] Generate embeddings for existing markets
- [ ] Integrate semantic matching into NewsService
- [ ] Add API endpoint for semantic search
- [ ] Test on sample markets
- [ ] Deploy and monitor metrics

**Estimated time**: 4-6 hours for MVP implementation
