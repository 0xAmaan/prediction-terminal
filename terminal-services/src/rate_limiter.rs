//! Shared rate limiter for API calls
//!
//! Provides a token bucket rate limiter with minimum inter-request delay
//! that can be shared across multiple services for coordinated rate limiting.

use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use tokio::sync::Mutex;
use tokio::time::Instant;
use tracing::{debug, info};

/// Minimum delay between Exa API requests (250ms = max 4 req/sec)
/// This provides safety margin under Exa's 5/sec limit even with timing variations
pub const EXA_MIN_REQUEST_INTERVAL_MS: u64 = 250;

/// Rate limiter that enforces minimum delay between requests
///
/// Unlike a simple sliding window counter, this ensures requests are
/// SPACED OUT over time, preventing burst patterns that can trigger
/// server-side rate limits.
///
/// ## Key Design: Reservation-Based Scheduling
///
/// When multiple tasks call `acquire()` concurrently, each task "reserves"
/// a future time slot BEFORE releasing the lock. This prevents the race
/// condition where multiple tasks see the same timestamp and all decide
/// to wait the same amount of time.
#[derive(Debug)]
pub struct RateLimiter {
    /// The next available time slot (when the next request can be made)
    /// Stored as milliseconds since an arbitrary epoch (Instant::now() at creation)
    next_available_ms: Mutex<u64>,
    /// The epoch instant (when this limiter was created)
    epoch: Instant,
    /// Minimum interval between requests
    min_interval: Duration,
    /// Name for logging purposes
    name: String,
    /// Counter for debugging - total requests processed
    total_requests: AtomicU64,
    /// Counter for debugging - requests that had to wait
    waited_requests: AtomicU64,
}

impl RateLimiter {
    /// Create a new rate limiter with the specified minimum interval between requests
    pub fn new(min_interval_ms: u64, name: &str) -> Self {
        let now = Instant::now();
        Self {
            next_available_ms: Mutex::new(0), // First request can go immediately
            epoch: now,
            min_interval: Duration::from_millis(min_interval_ms),
            name: name.to_string(),
            total_requests: AtomicU64::new(0),
            waited_requests: AtomicU64::new(0),
        }
    }

    /// Create a rate limiter configured for Exa API (250ms between requests = max 4/sec)
    pub fn for_exa() -> Arc<Self> {
        Arc::new(Self::new(EXA_MIN_REQUEST_INTERVAL_MS, "Exa"))
    }

    /// Convert an Instant to milliseconds since our epoch
    fn instant_to_ms(&self, instant: Instant) -> u64 {
        instant.duration_since(self.epoch).as_millis() as u64
    }

    /// Convert milliseconds since epoch to an Instant
    fn ms_to_instant(&self, ms: u64) -> Instant {
        self.epoch + Duration::from_millis(ms)
    }

    /// Acquire permission to make a request, waiting if necessary
    ///
    /// This method ensures that at least `min_interval` has passed since
    /// the last request before allowing a new one. This prevents burst
    /// patterns and ensures smooth, predictable request rates.
    ///
    /// ## Race Condition Prevention
    ///
    /// The key insight is that we RESERVE our time slot while holding the lock,
    /// then release the lock and wait. This ensures:
    /// 1. Each concurrent caller gets a DIFFERENT time slot
    /// 2. No two requests will fire at the same time
    /// 3. Requests are properly spaced even under high concurrency
    pub async fn acquire(&self) {
        let request_num = self.total_requests.fetch_add(1, Ordering::Relaxed) + 1;
        let now = Instant::now();
        let now_ms = self.instant_to_ms(now);

        // Acquire lock and reserve our time slot
        let (wait_until, wait_duration) = {
            let mut next_available = self.next_available_ms.lock().await;

            if now_ms >= *next_available {
                // We can go immediately - reserve now + interval for the next caller
                let our_slot = now_ms;
                *next_available = now_ms + self.min_interval.as_millis() as u64;

                info!(
                    "[RATE_LIMITER:{}] #{} IMMEDIATE - slot reserved at {}ms, next available at {}ms",
                    self.name, request_num, our_slot, *next_available
                );

                (None, Duration::ZERO)
            } else {
                // We need to wait - reserve the next available slot
                let our_slot = *next_available;
                let wait_ms = our_slot - now_ms;
                *next_available = our_slot + self.min_interval.as_millis() as u64;

                self.waited_requests.fetch_add(1, Ordering::Relaxed);

                info!(
                    "[RATE_LIMITER:{}] #{} QUEUED - must wait {}ms, slot at {}ms, next available at {}ms",
                    self.name, request_num, wait_ms, our_slot, *next_available
                );

                (Some(self.ms_to_instant(our_slot)), Duration::from_millis(wait_ms))
            }
            // Lock is released here
        };

        // Wait outside the lock if needed
        if let Some(target_time) = wait_until {
            let actual_wait = target_time.saturating_duration_since(Instant::now());
            if !actual_wait.is_zero() {
                debug!(
                    "[RATE_LIMITER:{}] #{} sleeping for {:?}",
                    self.name, request_num, actual_wait
                );
                tokio::time::sleep(actual_wait).await;
            }

            info!(
                "[RATE_LIMITER:{}] #{} READY after waiting {:?}",
                self.name, request_num, wait_duration
            );
        }
    }

    /// Check if a request can be made immediately without waiting
    ///
    /// Returns `true` if no waiting would be required,
    /// `false` if a request would need to wait.
    pub async fn can_acquire_immediately(&self) -> bool {
        let now_ms = self.instant_to_ms(Instant::now());
        let next_available = self.next_available_ms.lock().await;
        now_ms >= *next_available
    }

    /// Get the minimum interval between requests
    pub fn min_interval(&self) -> Duration {
        self.min_interval
    }

    /// Get statistics about this rate limiter (for debugging)
    pub fn stats(&self) -> RateLimiterStats {
        RateLimiterStats {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            waited_requests: self.waited_requests.load(Ordering::Relaxed),
            min_interval_ms: self.min_interval.as_millis() as u64,
            name: self.name.clone(),
        }
    }
}

/// Statistics about rate limiter usage
#[derive(Debug, Clone)]
pub struct RateLimiterStats {
    pub total_requests: u64,
    pub waited_requests: u64,
    pub min_interval_ms: u64,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_first_request_immediate() {
        let limiter = RateLimiter::new(100, "test");

        let start = Instant::now();
        limiter.acquire().await;
        let elapsed = start.elapsed();

        // First request should be immediate (< 20ms allowing for test overhead)
        assert!(elapsed.as_millis() < 20, "First request took {:?}", elapsed);
    }

    #[tokio::test]
    async fn test_second_request_waits() {
        let limiter = RateLimiter::new(100, "test");

        // First request
        limiter.acquire().await;

        // Second request should wait ~100ms
        let start = Instant::now();
        limiter.acquire().await;
        let elapsed = start.elapsed();

        // Should have waited at least 90ms (allowing some timing variance)
        assert!(elapsed.as_millis() >= 90, "Should have waited at least 90ms, but only waited {:?}", elapsed);
        // But not too long
        assert!(elapsed.as_millis() < 150, "Waited too long: {:?}", elapsed);
    }

    #[tokio::test]
    async fn test_request_after_interval_immediate() {
        let limiter = RateLimiter::new(50, "test");

        // First request
        limiter.acquire().await;

        // Wait longer than the interval
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Next request should be immediate
        let start = Instant::now();
        limiter.acquire().await;
        let elapsed = start.elapsed();

        assert!(elapsed.as_millis() < 20, "Request after interval took {:?}", elapsed);
    }

    #[tokio::test]
    async fn test_can_acquire_immediately() {
        let limiter = RateLimiter::new(100, "test");

        // Before any request
        assert!(limiter.can_acquire_immediately().await);

        // After a request
        limiter.acquire().await;
        assert!(!limiter.can_acquire_immediately().await);

        // After waiting
        tokio::time::sleep(Duration::from_millis(110)).await;
        assert!(limiter.can_acquire_immediately().await);
    }

    /// Test that concurrent requests are properly serialized
    /// This is the key test - it verifies the race condition fix
    #[tokio::test]
    async fn test_concurrent_requests_serialized() {
        let limiter = Arc::new(RateLimiter::new(50, "concurrent_test"));
        let num_concurrent = 5;

        // Spawn multiple concurrent tasks
        let mut handles = Vec::new();
        let start = Instant::now();

        for i in 0..num_concurrent {
            let limiter = Arc::clone(&limiter);
            handles.push(tokio::spawn(async move {
                limiter.acquire().await;
                let acquired_at = start.elapsed();
                info!("Task {} acquired at {:?}", i, acquired_at);
                acquired_at
            }));
        }

        // Collect all acquisition times
        let mut times: Vec<Duration> = Vec::new();
        for handle in handles {
            times.push(handle.await.unwrap());
        }

        // Sort times to analyze spacing
        times.sort();

        // Verify that requests are spaced at least min_interval apart
        // (with some tolerance for timing variations)
        for i in 1..times.len() {
            let gap = times[i] - times[i - 1];
            // Allow 40ms minimum gap (50ms interval minus 10ms tolerance)
            assert!(
                gap.as_millis() >= 40,
                "Gap between request {} and {} was only {:?}, expected >= 40ms",
                i - 1,
                i,
                gap
            );
        }

        // Total time should be roughly (n-1) * interval
        // 5 requests with 50ms spacing = ~200ms minimum
        let total = times.last().unwrap();
        assert!(
            total.as_millis() >= 180, // Allow some tolerance
            "Total time was only {:?}, expected >= 180ms for 5 requests",
            total
        );

        // Check stats
        let stats = limiter.stats();
        assert_eq!(stats.total_requests, 5);
        assert!(stats.waited_requests >= 4, "Expected at least 4 waits, got {}", stats.waited_requests);
    }
}
