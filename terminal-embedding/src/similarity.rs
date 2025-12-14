//! Cosine similarity calculations

use ndarray::ArrayView1;
use tracing::debug;

use crate::types::SimilarityMatch;

/// Calculate cosine similarity between two embeddings
///
/// Returns a value between 0.0 (completely different) and 1.0 (identical)
///
/// Formula: cos(θ) = (A · B) / (||A|| ||B||)
/// where:
/// - A · B is the dot product
/// - ||A|| and ||B|| are the magnitudes (L2 norms)
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    assert_eq!(
        a.len(),
        b.len(),
        "Embeddings must have same dimension (got {} and {})",
        a.len(),
        b.len()
    );

    let a_view = ArrayView1::from(a);
    let b_view = ArrayView1::from(b);

    let dot_product = a_view.dot(&b_view);
    let norm_a = a_view.dot(&a_view).sqrt();
    let norm_b = b_view.dot(&b_view).sqrt();

    // Avoid division by zero
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    (dot_product / (norm_a * norm_b)) as f64
}

/// Find top-K most similar markets for a news article
///
/// # Arguments
/// * `news_embedding` - The news article embedding vector
/// * `market_embeddings` - List of (market_id, platform, embedding) tuples
/// * `top_k` - Maximum number of results to return
/// * `threshold` - Minimum similarity score (0.0 - 1.0)
///
/// # Returns
/// Vector of SimilarityMatch sorted by score (highest first)
pub fn find_similar_markets(
    news_embedding: &[f32],
    market_embeddings: &[(String, String, Vec<f32>)],
    top_k: usize,
    threshold: f64,
) -> Vec<SimilarityMatch> {
    debug!(
        "Finding similar markets: {} candidates, top_k={}, threshold={}",
        market_embeddings.len(),
        top_k,
        threshold
    );

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

    debug!("Found {} matches above threshold", matches.len());

    // Sort by score descending
    matches.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Limit to top_k
    matches.truncate(top_k);

    if !matches.is_empty() {
        debug!(
            "Top match: market_id={}, score={:.3}",
            matches[0].market_id, matches[0].score
        );
    }

    matches
}

/// Batch similarity calculation - find similar markets for multiple news articles
///
/// More efficient than calling find_similar_markets multiple times
pub fn batch_find_similar(
    news_embeddings: &[(String, Vec<f32>)], // (article_id, embedding)
    market_embeddings: &[(String, String, Vec<f32>)],
    top_k: usize,
    threshold: f64,
) -> Vec<(String, Vec<SimilarityMatch>)> {
    news_embeddings
        .iter()
        .map(|(article_id, news_emb)| {
            let matches = find_similar_markets(news_emb, market_embeddings, top_k, threshold);
            (article_id.clone(), matches)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 1.0).abs() < 1e-6, "Identical vectors should have similarity ~1.0");
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-6, "Orthogonal vectors should have similarity ~0.0");
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![-1.0, -2.0, -3.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim + 1.0).abs() < 1e-6, "Opposite vectors should have similarity ~-1.0");
    }

    #[test]
    fn test_find_similar_markets() {
        let news_emb = vec![1.0, 0.0, 0.0];

        let market_embs = vec![
            ("market1".to_string(), "polymarket".to_string(), vec![1.0, 0.0, 0.0]), // Perfect match
            ("market2".to_string(), "polymarket".to_string(), vec![0.8, 0.6, 0.0]), // High similarity
            ("market3".to_string(), "kalshi".to_string(), vec![0.0, 1.0, 0.0]),     // Orthogonal
        ];

        let matches = find_similar_markets(&news_emb, &market_embs, 10, 0.5);

        assert_eq!(matches.len(), 2, "Should find 2 markets above threshold");
        assert_eq!(matches[0].market_id, "market1", "market1 should be top match");
        assert!((matches[0].score - 1.0).abs() < 1e-6, "market1 score should be ~1.0");
    }

    #[test]
    fn test_batch_find_similar() {
        let news_embs = vec![
            ("article1".to_string(), vec![1.0, 0.0, 0.0]),
            ("article2".to_string(), vec![0.0, 1.0, 0.0]),
        ];

        let market_embs = vec![
            ("market1".to_string(), "polymarket".to_string(), vec![1.0, 0.0, 0.0]),
            ("market2".to_string(), "kalshi".to_string(), vec![0.0, 1.0, 0.0]),
        ];

        let results = batch_find_similar(&news_embs, &market_embs, 5, 0.9);

        assert_eq!(results.len(), 2, "Should have results for 2 articles");
        assert_eq!(results[0].0, "article1");
        assert_eq!(results[0].1.len(), 1, "article1 should match 1 market");
        assert_eq!(results[0].1[0].market_id, "market1");
    }
}
