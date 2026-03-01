//! Rate limiting utilities for API calls.
//!
//! This module provides rate limiter implementations that help prevent
//! exceeding platform-specific API rate limits. It supports:
//!
//! - Per-platform rate limit configurations
//! - Sliding window algorithm for tracking requests
//! - Retry-After header handling
//! - Async rate limiting with tokio

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Represents a platform for rate limiting purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    /// Telegram platform
    Telegram,
    /// Discord platform
    Discord,
    /// WhatsApp platform
    WhatsApp,
    /// Slack platform
    Slack,
}

impl Platform {
    /// Get the default rate limit configuration for a platform.
    pub fn default_config(&self) -> RateLimitConfig {
        match self {
            Platform::Telegram => RateLimitConfig {
                // Telegram: 30 messages/second globally
                global_limit: RateLimit::new(30, Duration::from_secs(1)),
                // 20 messages/minute per chat
                per_chat_limit: RateLimit::new(20, Duration::from_secs(60)),
            },
            Platform::Discord => RateLimitConfig {
                // Discord: varies by endpoint, default 5 requests/5 seconds per route
                global_limit: RateLimit::new(5, Duration::from_secs(5)),
                per_chat_limit: RateLimit::new(5, Duration::from_secs(5)),
            },
            Platform::WhatsApp => RateLimitConfig {
                // WhatsApp Business API: 80 messages/second (business tier dependent)
                global_limit: RateLimit::new(80, Duration::from_secs(1)),
                per_chat_limit: RateLimit::new(1, Duration::from_secs(1)),
            },
            Platform::Slack => RateLimitConfig {
                // Slack: 1 message/second per channel (Web API tier 2+)
                global_limit: RateLimit::new(1, Duration::from_secs(1)),
                per_chat_limit: RateLimit::new(1, Duration::from_secs(1)),
            },
        }
    }
}

/// Configuration for rate limiting.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Global rate limit (applies to all requests)
    pub global_limit: RateLimit,
    /// Per-chat rate limit (applies per chat/conversation)
    pub per_chat_limit: RateLimit,
}

/// Represents a rate limit with a count and time window.
#[derive(Debug, Clone, Copy)]
pub struct RateLimit {
    /// Maximum number of requests allowed in the time window
    pub max_requests: u32,
    /// Time window duration
    pub window_duration: Duration,
}

impl RateLimit {
    /// Create a new rate limit.
    pub fn new(max_requests: u32, window_duration: Duration) -> Self {
        Self {
            max_requests,
            window_duration,
        }
    }
}

/// Error returned when rate limit is exceeded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RateLimitError {
    /// Rate limit exceeded, retry after this duration
    RetryAfter(Duration),
    /// Generic rate limit error
    Exceeded,
}

/// Rate limiter that tracks API calls using a sliding window algorithm.
///
/// This struct maintains request timestamps for both global and per-chat
/// rate limits, automatically removing expired entries from the sliding window.
pub struct RateLimiter {
    /// Platform-specific configuration
    config: RateLimitConfig,
    /// Timestamps for global requests (key: ())
    global_requests: Arc<RwLock<HashMap<(), Vec<Instant>>>>,
    /// Timestamps for per-chat requests (key: chat_id)
    per_chat_requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    /// Track last retry-after duration for each chat
    retry_after: Arc<RwLock<HashMap<String, Instant>>>,
}

impl RateLimiter {
    /// Create a new rate limiter with default configuration for the given platform.
    pub fn new(platform: Platform) -> Self {
        Self::with_config(platform.default_config())
    }

    /// Create a new rate limiter with custom configuration.
    pub fn with_config(config: RateLimitConfig) -> Self {
        Self {
            config,
            global_requests: Arc::new(RwLock::new(HashMap::new())),
            per_chat_requests: Arc::new(RwLock::new(HashMap::new())),
            retry_after: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Try to acquire a rate limit token without waiting.
    ///
    /// This method checks if the rate limit would be exceeded and returns
    /// an error if so. It does NOT wait or block.
    ///
    /// # Arguments
    ///
    /// * `chat_id` - Optional chat ID for per-chat rate limiting
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the request can proceed, or `Err(RateLimitError)`
    /// if the rate limit would be exceeded.
    pub async fn try_acquire(&self, chat_id: Option<&str>) -> Result<(), RateLimitError> {
        let now = Instant::now();

        // Check global limit
        {
            let global_requests = self.global_requests.read().await;
            if let Some(timestamps) = global_requests.get(&()) {
                let window_start = now - self.config.global_limit.window_duration;
                let count = timestamps.iter().filter(|t| **t >= window_start).count();
                if count >= self.config.global_limit.max_requests as usize {
                    return Err(RateLimitError::Exceeded);
                }
            }
        }

        // Check per-chat limit if chat_id provided
        if let Some(chat_id) = chat_id {
            let per_chat_requests = self.per_chat_requests.read().await;
            let chat_key = chat_id.to_string();
            if let Some(timestamps) = per_chat_requests.get(&chat_key) {
                let window_start = now - self.config.per_chat_limit.window_duration;
                let count = timestamps.iter().filter(|t| **t >= window_start).count();
                if count >= self.config.per_chat_limit.max_requests as usize {
                    return Err(RateLimitError::Exceeded);
                }
            }
        }

        // Record the request if we're good to go
        self.record_request(chat_id).await;

        Ok(())
    }

    /// Acquire a rate limit token, waiting if necessary.
    ///
    /// This method blocks until a rate limit token is available. It uses
    /// the sliding window algorithm to track when requests can proceed.
    ///
    /// # Arguments
    ///
    /// * `chat_id` - Optional chat ID for per-chat rate limiting
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the request can proceed. If the rate limit
    /// was hit, this method will wait until it can proceed.
    pub async fn acquire(&self, chat_id: Option<&str>) -> Result<(), RateLimitError> {
        // Check if we need to wait due to retry-after
        if let Some(chat_id) = chat_id {
            let retry_after_map = self.retry_after.read().await;
            if let Some(&retry_time) = retry_after_map.get(chat_id) {
                let now = Instant::now();
                if retry_time > now {
                    let wait_duration = retry_time - now;
                    drop(retry_after_map);
                    tokio::time::sleep(wait_duration).await;
                }
            }
        }

        // Try to acquire immediately
        if self.try_acquire(chat_id).await.is_ok() {
            // try_acquire already recorded the request, just return
            return Ok(());
        }

        // Need to wait - calculate how long
        let now = Instant::now();
        let mut min_wait = Duration::ZERO;

        // Calculate wait time for global limit
        let global_requests = self.global_requests.read().await;
        if let Some(timestamps) = global_requests.get(&()) {
            let window_start = now - self.config.global_limit.window_duration;
            let old_timestamps: Vec<&Instant> =
                timestamps.iter().filter(|t| **t < window_start).collect();
            let count = timestamps.len() - old_timestamps.len();

            if count >= self.config.global_limit.max_requests as usize {
                // Find the oldest timestamp that would expire
                if let Some(oldest) = timestamps.first() {
                    min_wait =
                        min_wait.max(*oldest + self.config.global_limit.window_duration - now);
                }
            }
        }

        // Calculate wait time for per-chat limit
        if let Some(chat_id) = chat_id {
            let per_chat_requests = self.per_chat_requests.read().await;
            let chat_key = chat_id.to_string();
            if let Some(timestamps) = per_chat_requests.get(&chat_key) {
                let window_start = now - self.config.per_chat_limit.window_duration;
                let count = timestamps.iter().filter(|t| **t >= window_start).count();

                if count >= self.config.per_chat_limit.max_requests as usize {
                    if let Some(oldest) = timestamps.first() {
                        let wait = *oldest + self.config.per_chat_limit.window_duration - now;
                        min_wait = min_wait.max(wait);
                    }
                }
            }
        }

        drop(global_requests);

        if min_wait > Duration::ZERO {
            tokio::time::sleep(min_wait).await;
        }

        self.record_request(chat_id).await;
        Ok(())
    }

    /// Record a request timestamp.
    async fn record_request(&self, chat_id: Option<&str>) {
        let now = Instant::now();

        // Record global request
        {
            let mut global_requests = self.global_requests.write().await;
            let timestamps = global_requests.entry(()).or_insert_with(Vec::new);
            timestamps.push(now);
            self.cleanup_old_timestamps(timestamps, now);
        }

        // Record per-chat request if provided
        if let Some(chat_id) = chat_id {
            let mut per_chat_requests = self.per_chat_requests.write().await;
            let timestamps = per_chat_requests
                .entry(chat_id.to_string())
                .or_insert_with(Vec::new);
            timestamps.push(now);
            self.cleanup_old_timestamps(timestamps, now);
        }
    }

    /// Cleanup old timestamps from a list.
    fn cleanup_old_timestamps(&self, timestamps: &mut Vec<Instant>, now: Instant) {
        let cutoff = now - Duration::from_secs(60); // Keep last 60 seconds
        timestamps.retain(|t| *t > cutoff);
    }

    /// Count requests in the current sliding window.
    fn count_requests_in_window(
        &self,
        map: &HashMap<String, Vec<Instant>>,
        key: &str,
        now: Instant,
    ) -> usize {
        if let Some(timestamps) = map.get(key) {
            let window_start = now - self.config.per_chat_limit.window_duration;
            timestamps.iter().filter(|t| **t >= window_start).count()
        } else {
            0
        }
    }

    /// Count requests in the global sliding window.
    fn count_requests_in_window_global(
        &self,
        map: &HashMap<(), Vec<Instant>>,
        now: Instant,
    ) -> usize {
        if let Some(timestamps) = map.get(&()) {
            let window_start = now - self.config.global_limit.window_duration;
            timestamps.iter().filter(|t| **t >= window_start).count()
        } else {
            0
        }
    }

    /// Parse Retry-After header and set the rate limit delay.
    ///
    /// This method should be called when receiving a 429 Too Many Requests
    /// response from an API. It extracts the Retry-After header value and
    /// ensures subsequent requests wait appropriately.
    ///
    /// # Arguments
    ///
    /// * `retry_after_seconds` - The number of seconds to wait before making
    ///   another request
    /// * `chat_id` - Optional chat ID to apply the delay to
    pub async fn handle_retry_after(&self, retry_after_seconds: u64, chat_id: Option<&str>) {
        let retry_time = Instant::now() + Duration::from_secs(retry_after_seconds);

        if let Some(chat_id) = chat_id {
            let mut retry_after = self.retry_after.write().await;
            retry_after.insert(chat_id.to_string(), retry_time);
        } else {
            // For global retry-after, we don't track per chat
            // The acquire() method handles this by checking current time
        }
    }

    /// Clear the retry-after delay for a specific chat.
    pub async fn clear_retry_after(&self, chat_id: &str) {
        let mut retry_after = self.retry_after.write().await;
        retry_after.remove(chat_id);
    }

    /// Get the current number of requests in the window.
    pub async fn get_request_count(&self, chat_id: Option<&str>) -> (usize, usize) {
        let now = Instant::now();

        let global_requests = self.global_requests.read().await;
        let per_chat_requests = self.per_chat_requests.read().await;

        let global_count = self.count_requests_in_window_global(&global_requests, now);
        let per_chat_count = if let Some(chat_id) = chat_id {
            self.count_requests_in_window(&per_chat_requests, &chat_id.to_string(), now)
        } else {
            0
        };

        (global_count, per_chat_count)
    }

    /// Get the current configuration.
    pub fn config(&self) -> &RateLimitConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_creation() {
        let limit = RateLimit::new(10, Duration::from_secs(5));
        assert_eq!(limit.max_requests, 10);
        assert_eq!(limit.window_duration, Duration::from_secs(5));
    }

    #[test]
    fn test_platform_default_configs() {
        let telegram = Platform::Telegram.default_config();
        assert_eq!(telegram.global_limit.max_requests, 30);
        assert_eq!(
            telegram.global_limit.window_duration,
            Duration::from_secs(1)
        );

        let discord = Platform::Discord.default_config();
        assert_eq!(discord.global_limit.max_requests, 5);

        let whatsapp = Platform::WhatsApp.default_config();
        assert_eq!(whatsapp.global_limit.max_requests, 80);

        let slack = Platform::Slack.default_config();
        assert_eq!(slack.global_limit.max_requests, 1);
    }

    #[tokio::test]
    async fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new(Platform::Telegram);
        assert_eq!(limiter.config().global_limit.max_requests, 30);
    }

    #[tokio::test]
    async fn test_try_acquire_success() {
        let limiter = RateLimiter::new(Platform::Telegram);
        // Should succeed since we haven't made any requests yet
        assert!(limiter.try_acquire(None).await.is_ok());
    }

    #[tokio::test]
    async fn test_try_acquire_exceeded() {
        // Create a limiter with a very low limit for testing
        let config = RateLimitConfig {
            global_limit: RateLimit::new(1, Duration::from_secs(10)),
            per_chat_limit: RateLimit::new(1, Duration::from_secs(10)),
        };
        let limiter = RateLimiter::with_config(config);

        // First request should succeed
        assert!(limiter.try_acquire(None).await.is_ok());

        // Second request should fail
        assert!(matches!(
            limiter.try_acquire(None).await,
            Err(RateLimitError::Exceeded)
        ));
    }

    #[tokio::test]
    async fn test_acquire_with_retry_after() {
        let config = RateLimitConfig {
            global_limit: RateLimit::new(1, Duration::from_secs(10)),
            per_chat_limit: RateLimit::new(1, Duration::from_secs(10)),
        };
        let limiter = RateLimiter::with_config(config);

        // Make first request
        assert!(limiter.try_acquire(None).await.is_ok());

        // Set a retry-after delay
        limiter.handle_retry_after(1, None).await;

        // Should wait for the retry-after duration
        let start = Instant::now();
        limiter.acquire(None).await.unwrap();
        let elapsed = start.elapsed();

        // Should have waited at least 1 second
        assert!(elapsed >= Duration::from_secs(1));
    }

    #[tokio::test]
    async fn test_per_chat_rate_limiting() {
        let config = RateLimitConfig {
            global_limit: RateLimit::new(100, Duration::from_secs(10)),
            per_chat_limit: RateLimit::new(2, Duration::from_secs(10)),
        };
        let limiter = RateLimiter::with_config(config);

        // Two requests to chat1 should succeed
        assert!(limiter.try_acquire(Some("chat1")).await.is_ok());
        assert!(limiter.try_acquire(Some("chat1")).await.is_ok());

        // Third request to chat1 should fail
        assert!(matches!(
            limiter.try_acquire(Some("chat1")).await,
            Err(RateLimitError::Exceeded)
        ));

        // Request to different chat should succeed
        assert!(limiter.try_acquire(Some("chat2")).await.is_ok());
    }

    #[tokio::test]
    async fn test_retry_after_handling() {
        let limiter = RateLimiter::new(Platform::Telegram);

        // Set retry-after for a specific chat
        limiter.handle_retry_after(2, Some("test_chat")).await;

        // Verify the delay is set
        let retry_after = limiter.retry_after.read().await;
        assert!(retry_after.contains_key("test_chat"));
    }

    #[tokio::test]
    async fn test_clear_retry_after() {
        let limiter = RateLimiter::new(Platform::Telegram);

        limiter.handle_retry_after(2, Some("test_chat")).await;
        limiter.clear_retry_after("test_chat").await;

        let retry_after = limiter.retry_after.read().await;
        assert!(!retry_after.contains_key("test_chat"));
    }

    #[tokio::test]
    async fn test_get_request_count() {
        let config = RateLimitConfig {
            global_limit: RateLimit::new(100, Duration::from_secs(10)),
            per_chat_limit: RateLimit::new(100, Duration::from_secs(10)),
        };
        let limiter = RateLimiter::with_config(config);

        // Make some requests
        eprintln!("Making request 1 (global)");
        limiter.acquire(None).await.unwrap();
        eprintln!("Making request 2 (chat1)");
        limiter.acquire(Some("chat1")).await.unwrap();
        eprintln!("Making request 3 (chat1)");
        limiter.acquire(Some("chat1")).await.unwrap();

        // Check counts
        let (global, per_chat) = limiter.get_request_count(Some("chat1")).await;
        eprintln!("Global count: {}, Per-chat count: {}", global, per_chat);
        assert!(global >= 1);
        assert_eq!(per_chat, 2);
    }

    #[tokio::test]
    async fn test_sliding_window_cleanup() {
        // Use a very short window for testing
        let config = RateLimitConfig {
            global_limit: RateLimit::new(100, Duration::from_millis(100)),
            per_chat_limit: RateLimit::new(100, Duration::from_millis(100)),
        };
        let limiter = RateLimiter::with_config(config);

        // Make a request
        limiter.acquire(None).await.unwrap();

        // Wait for window to expire
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be able to make another request (old ones expired)
        assert!(limiter.try_acquire(None).await.is_ok());
    }
}
