//! Engagement tracking and relevance scoring for Discord messages

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use std::collections::HashSet;

use super::config::EngagementThreshold;

/// Engagement metrics for a Discord message
#[derive(Debug, Clone)]
pub struct EngagementMetrics {
    /// Discord message ID
    pub message_id: u64,
    /// Total reaction count across all emoji
    pub reaction_count: u32,
    /// Number of replies in thread
    pub reply_count: u32,
    /// Number of unique users who reacted
    pub unique_reactors: u32,
    /// Last time metrics were updated
    pub last_updated: DateTime<Utc>,
}

impl EngagementMetrics {
    /// Create new engagement metrics for a message
    pub fn new(message_id: u64) -> Self {
        Self {
            message_id,
            reaction_count: 0,
            reply_count: 0,
            unique_reactors: 0,
            last_updated: Utc::now(),
        }
    }

    /// Check if metrics meet engagement threshold
    pub fn meets_threshold(&self, threshold: &EngagementThreshold) -> bool {
        self.reaction_count >= threshold.reactions || self.reply_count >= threshold.replies
    }
}

/// Tracks engagement metrics for Discord messages
pub struct EngagementTracker {
    /// Map of message_id -> engagement metrics
    metrics: DashMap<u64, EngagementMetrics>,
    /// Map of message_id -> set of user IDs who reacted
    reactors: DashMap<u64, HashSet<u64>>,
    /// Map of parent message ID -> set of reply message IDs
    thread_messages: DashMap<u64, HashSet<u64>>,
}

impl EngagementTracker {
    /// Create a new engagement tracker
    pub fn new() -> Self {
        Self {
            metrics: DashMap::new(),
            reactors: DashMap::new(),
            thread_messages: DashMap::new(),
        }
    }

    /// Record a reaction being added to a message
    pub fn on_reaction_add(&self, message_id: u64, user_id: u64) {
        // Track unique reactor
        let mut reactors = self.reactors.entry(message_id).or_insert_with(HashSet::new);
        let _is_new = reactors.insert(user_id);
        let unique_count = reactors.len() as u32;
        drop(reactors); // Release lock

        // Update metrics
        let mut metrics = self
            .metrics
            .entry(message_id)
            .or_insert_with(|| EngagementMetrics::new(message_id));

        metrics.reaction_count += 1;
        metrics.unique_reactors = unique_count;
        metrics.last_updated = Utc::now();

        tracing::debug!(
            "Reaction added to message {}: {} reactions from {} unique users",
            message_id,
            metrics.reaction_count,
            metrics.unique_reactors
        );
    }

    /// Record a reaction being removed from a message
    pub fn on_reaction_remove(&self, message_id: u64, user_id: u64) {
        // Remove from reactors set
        if let Some(mut reactors) = self.reactors.get_mut(&message_id) {
            reactors.remove(&user_id);
            let unique_count = reactors.len() as u32;
            drop(reactors); // Release lock

            // Update metrics
            if let Some(mut metrics) = self.metrics.get_mut(&message_id) {
                metrics.reaction_count = metrics.reaction_count.saturating_sub(1);
                metrics.unique_reactors = unique_count;
                metrics.last_updated = Utc::now();
            }
        }
    }

    /// Record a message reply (thread message)
    pub fn on_message_reply(&self, parent_id: u64, reply_id: u64) {
        // Track thread structure
        let mut thread = self
            .thread_messages
            .entry(parent_id)
            .or_insert_with(HashSet::new);
        thread.insert(reply_id);
        let reply_count = thread.len() as u32;
        drop(thread); // Release lock

        // Update parent message metrics
        let mut metrics = self
            .metrics
            .entry(parent_id)
            .or_insert_with(|| EngagementMetrics::new(parent_id));

        metrics.reply_count = reply_count;
        metrics.last_updated = Utc::now();

        tracing::debug!(
            "Reply added to message {}: {} total replies",
            parent_id,
            reply_count
        );
    }

    /// Get engagement metrics for a message
    pub fn get_metrics(&self, message_id: u64) -> Option<EngagementMetrics> {
        self.metrics.get(&message_id).map(|m| m.clone())
    }

    /// Get or create metrics for a message
    pub fn get_or_create_metrics(&self, message_id: u64) -> EngagementMetrics {
        self.metrics
            .entry(message_id)
            .or_insert_with(|| EngagementMetrics::new(message_id))
            .clone()
    }

    /// Clean up old metrics (remove messages older than specified hours)
    pub fn cleanup_old_metrics(&self, hours: i64) {
        let cutoff = Utc::now() - chrono::Duration::hours(hours);
        let old_ids: Vec<u64> = self
            .metrics
            .iter()
            .filter(|entry| entry.last_updated < cutoff)
            .map(|entry| entry.message_id)
            .collect();

        for message_id in old_ids {
            self.metrics.remove(&message_id);
            self.reactors.remove(&message_id);
            self.thread_messages.remove(&message_id);
        }
    }
}

impl Default for EngagementTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate relevance score for a Discord message
///
/// Combines engagement metrics and keyword matching to produce a score between 0.0 and 1.0
pub fn calculate_relevance_score(
    engagement: &EngagementMetrics,
    threshold: &EngagementThreshold,
    keyword_match_score: f64,
) -> f64 {
    // Base score from keyword matching (0.0 - 0.5)
    let mut score = keyword_match_score * 0.5;

    // Engagement bonus (0.0 - 0.5)
    let reaction_ratio = if threshold.reactions > 0 {
        engagement.reaction_count as f64 / threshold.reactions as f64
    } else {
        0.0
    };

    let reply_ratio = if threshold.replies > 0 {
        engagement.reply_count as f64 / threshold.replies as f64
    } else {
        0.0
    };

    // Take the maximum ratio (reactions OR replies can drive score)
    let engagement_ratio = reaction_ratio.max(reply_ratio);

    // Cap engagement bonus at 2x threshold
    let engagement_bonus = (engagement_ratio.min(2.0) / 2.0) * 0.5;
    score += engagement_bonus;

    // Recency bonus (0.0 - 0.1): Boost very recent messages
    let age = Utc::now() - engagement.last_updated;
    let age_hours = age.num_hours();
    let recency_bonus = if age_hours < 1 {
        0.1 // Less than 1 hour old
    } else if age_hours < 6 {
        0.05 // Less than 6 hours old
    } else {
        0.0
    };

    score += recency_bonus;

    // Clamp to [0.0, 1.0]
    score.min(1.0).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engagement_metrics_new() {
        let metrics = EngagementMetrics::new(12345);
        assert_eq!(metrics.message_id, 12345);
        assert_eq!(metrics.reaction_count, 0);
        assert_eq!(metrics.reply_count, 0);
        assert_eq!(metrics.unique_reactors, 0);
    }

    #[test]
    fn test_meets_threshold() {
        let mut metrics = EngagementMetrics::new(12345);
        let threshold = EngagementThreshold {
            reactions: 5,
            replies: 3,
        };

        // Below threshold
        assert!(!metrics.meets_threshold(&threshold));

        // Meets reaction threshold
        metrics.reaction_count = 5;
        assert!(metrics.meets_threshold(&threshold));

        // Meets reply threshold
        metrics.reaction_count = 0;
        metrics.reply_count = 3;
        assert!(metrics.meets_threshold(&threshold));
    }

    #[test]
    fn test_tracker_reaction_add() {
        let tracker = EngagementTracker::new();

        tracker.on_reaction_add(100, 1);
        tracker.on_reaction_add(100, 2);
        tracker.on_reaction_add(100, 1); // Same user, different emoji

        let metrics = tracker.get_metrics(100).unwrap();
        assert_eq!(metrics.reaction_count, 3);
        assert_eq!(metrics.unique_reactors, 2);
    }

    #[test]
    fn test_tracker_reaction_remove() {
        let tracker = EngagementTracker::new();

        tracker.on_reaction_add(100, 1);
        tracker.on_reaction_add(100, 2);
        tracker.on_reaction_remove(100, 1);

        let metrics = tracker.get_metrics(100).unwrap();
        assert_eq!(metrics.reaction_count, 1);
        assert_eq!(metrics.unique_reactors, 1);
    }

    #[test]
    fn test_tracker_replies() {
        let tracker = EngagementTracker::new();

        tracker.on_message_reply(100, 200);
        tracker.on_message_reply(100, 201);
        tracker.on_message_reply(100, 202);

        let metrics = tracker.get_metrics(100).unwrap();
        assert_eq!(metrics.reply_count, 3);
    }

    #[test]
    fn test_relevance_score_high_engagement() {
        let metrics = EngagementMetrics {
            message_id: 1,
            reaction_count: 15,
            reply_count: 5,
            unique_reactors: 10,
            last_updated: Utc::now(),
        };

        let threshold = EngagementThreshold {
            reactions: 10,
            replies: 3,
        };

        // High engagement, no keyword match
        let score = calculate_relevance_score(&metrics, &threshold, 0.0);
        assert!(score > 0.5); // Should have high score from engagement alone
        assert!(score < 0.7); // But capped since no recency bonus from current time
    }

    #[test]
    fn test_relevance_score_keyword_match() {
        let metrics = EngagementMetrics {
            message_id: 1,
            reaction_count: 2,
            reply_count: 1,
            unique_reactors: 2,
            last_updated: Utc::now(),
        };

        let threshold = EngagementThreshold {
            reactions: 10,
            replies: 5,
        };

        // Low engagement, but strong keyword match
        let score = calculate_relevance_score(&metrics, &threshold, 0.9);
        assert!(score > 0.4); // Should have decent score from keywords
        assert!(score < 0.7); // But not too high due to low engagement
    }

    #[test]
    fn test_relevance_score_combined() {
        let metrics = EngagementMetrics {
            message_id: 1,
            reaction_count: 10,
            reply_count: 5,
            unique_reactors: 8,
            last_updated: Utc::now(),
        };

        let threshold = EngagementThreshold {
            reactions: 10,
            replies: 5,
        };

        // Both high engagement and keyword match
        let score = calculate_relevance_score(&metrics, &threshold, 0.8);
        assert!(score > 0.8); // Should have very high score
    }

    #[test]
    fn test_relevance_score_clamped() {
        let metrics = EngagementMetrics {
            message_id: 1,
            reaction_count: 50,
            reply_count: 20,
            unique_reactors: 30,
            last_updated: Utc::now(),
        };

        let threshold = EngagementThreshold {
            reactions: 5,
            replies: 3,
        };

        // Extremely high engagement
        let score = calculate_relevance_score(&metrics, &threshold, 1.0);
        assert_eq!(score, 1.0); // Should be clamped at 1.0
    }
}
