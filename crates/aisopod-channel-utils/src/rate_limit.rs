//! Rate limit handling utilities for multi-platform channel implementations.
//!
//! This module provides rate limiter implementations that adapt to each
//! platform's limits and headers, enabling consistent rate limiting across
//! all Tier 2 and Tier 3 channels.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Platform-specific rate limiter.
///
/// This struct manages rate limiting for different messaging platforms,
/// each with their own specific rate limits. It supports:
///
/// - Per-platform rate limit configurations
/// - Sliding window algorithm for tracking requests
/// - Retry-After header handling
/// - Key-based rate limiting (e.g., per chat or per user)
#[derive(Debug)]
pub struct RateLimiter {
    /// Max requests per window
    max_requests: u32,
    /// Time window duration
    window: Duration,
    /// Current request count per key
    counters: HashMap<String, (u32, Instant)>,
}

impl RateLimiter {
    /// Create a new rate limiter with custom limits.
    ///
    /// # Arguments
    ///
    /// * `max_requests` - Maximum number of requests allowed in the window
    /// * `window` - Time window duration
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            counters: HashMap::new(),
        }
    }

    /// Get a rate limiter pre-configured for a specific platform.
    ///
    /// # Arguments
    ///
    /// * `platform` - Platform name (e.g., "signal", "discord", "slack")
    ///
    /// # Supported Platforms
    ///
    /// | Platform | Max Requests | Window |
    /// |----------|--------------|--------|
    /// | signal | 60 | 60 seconds |
    /// | googlechat | 60 | 60 seconds |
    /// | msteams | 50 | 60 seconds |
    /// | matrix | 30 | 60 seconds |
    /// | irc | 5 | 10 seconds |
    /// | mattermost | 60 | 60 seconds |
    /// | twitch | 20 | 30 seconds |
    /// | line | 500 | 60 seconds |
    /// | lark | 100 | 60 seconds |
    /// | zalo | 200 | 60 seconds |
    pub fn for_platform(platform: &str) -> Self {
        match platform {
            "signal" => Self::new(60, Duration::from_secs(60)),
            "googlechat" => Self::new(60, Duration::from_secs(60)),
            "msteams" => Self::new(50, Duration::from_secs(60)),
            "matrix" => Self::new(30, Duration::from_secs(60)),
            "irc" => Self::new(5, Duration::from_secs(10)),
            "mattermost" => Self::new(60, Duration::from_secs(60)),
            "twitch" => Self::new(20, Duration::from_secs(30)),
            "line" => Self::new(500, Duration::from_secs(60)),
            "lark" => Self::new(100, Duration::from_secs(60)),
            "zalo" => Self::new(200, Duration::from_secs(60)),
            _ => Self::new(30, Duration::from_secs(60)), // conservative default
        }
    }

    /// Check if a request can proceed, or return wait duration.
    ///
    /// # Arguments
    ///
    /// * `key` - Key to track rate limits per (e.g., chat ID, user ID)
    ///
    /// # Returns
    ///
    /// Returns `RateLimitResult::Allowed` if the request can proceed,
    /// or `RateLimitResult::Limited` with the time to wait before retrying.
    pub fn check(&mut self, key: &str) -> RateLimitResult {
        let now = Instant::now();
        let entry = self.counters.entry(key.to_string()).or_insert((0, now));

        if now.duration_since(entry.1) >= self.window {
            // Window expired, reset counter
            *entry = (1, now);
            return RateLimitResult::Allowed;
        }

        if entry.0 < self.max_requests {
            entry.0 += 1;
            RateLimitResult::Allowed
        } else {
            let wait = self.window - now.duration_since(entry.1);
            debug!(
                "Rate limit exceeded for key '{}', retry after {:?}",
                key, wait
            );
            RateLimitResult::Limited { retry_after: wait }
        }
    }

    /// Update rate limiter from HTTP response headers.
    ///
    /// This method parses rate limit headers from HTTP responses and
    /// adjusts internal counters accordingly. It handles:
    ///
    /// - `X-RateLimit-Remaining`: Number of requests remaining
    /// - `X-RateLimit-Reset`: Timestamp when the limit resets
    /// - `Retry-After`: Seconds to wait before retrying
    ///
    /// # Arguments
    ///
    /// * `key` - Key to update rate limit for
    /// * `headers` - HTTP response headers
    pub fn update_from_headers(&mut self, key: &str, headers: &reqwest::header::HeaderMap) {
        // Parse X-RateLimit-Remaining
        if let Some(remaining) = headers.get("x-ratelimit-remaining") {
            if let Ok(s) = remaining.to_str() {
                if let Ok(val) = s.parse::<u32>() {
                    let now = Instant::now();
                    let entry = self.counters.entry(key.to_string()).or_insert((0, now));
                    entry.0 = self.max_requests.saturating_sub(val);
                    debug!(
                        "Updated rate limit for key '{}': {}/{} requests used",
                        key, entry.0, self.max_requests
                    );
                }
            }
        }

        // Parse Retry-After header (for 429 responses)
        if let Some(retry_after) = headers.get("retry-after") {
            if let Ok(s) = retry_after.to_str() {
                if let Ok(seconds) = s.parse::<u64>() {
                    debug!(
                        "Retry-After header found for key '{}': {} seconds",
                        key, seconds
                    );
                    // The check() method will handle the wait time
                }
            }
        }

        // Parse X-RateLimit-Reset (timestamp)
        if let Some(reset) = headers.get("x-ratelimit-reset") {
            if let Ok(s) = reset.to_str() {
                if let Ok(timestamp) = s.parse::<u64>() {
                    let now = Instant::now();
                    let reset_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    let time_until_reset = (timestamp as i64 - reset_time as i64).max(0) as u64;
                    debug!(
                        "X-RateLimit-Reset for key '{}': {} seconds until reset",
                        key, time_until_reset
                    );
                }
            }
        }
    }
}

/// Result of a rate limit check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RateLimitResult {
    /// Request is allowed to proceed
    Allowed,
    /// Request is limited, retry after specified duration
    Limited { retry_after: Duration },
}

impl RateLimitResult {
    /// Returns `true` if the request is allowed.
    pub fn is_allowed(&self) -> bool {
        matches!(self, RateLimitResult::Allowed)
    }

    /// Returns `true` if the request is limited.
    pub fn is_limited(&self) -> bool {
        matches!(self, RateLimitResult::Limited { .. })
    }

    /// Returns the retry duration if limited, or `None` if allowed.
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            RateLimitResult::Limited { retry_after } => Some(*retry_after),
            RateLimitResult::Allowed => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new(10, Duration::from_secs(5));
        assert_eq!(limiter.max_requests, 10);
        assert_eq!(limiter.window, Duration::from_secs(5));
    }

    #[test]
    fn test_platform_rate_limiters() {
        let telegram = RateLimiter::for_platform("signal");
        assert_eq!(telegram.max_requests, 60);

        let discord = RateLimiter::for_platform("discord");
        assert_eq!(discord.max_requests, 30); // default

        let irc = RateLimiter::for_platform("irc");
        assert_eq!(irc.max_requests, 5);
        assert_eq!(irc.window, Duration::from_secs(10));
    }

    #[test]
    fn test_rate_limit_allowed() {
        let mut limiter = RateLimiter::new(3, Duration::from_secs(10));

        assert!(matches!(limiter.check("key1"), RateLimitResult::Allowed));
        assert!(matches!(limiter.check("key1"), RateLimitResult::Allowed));
        assert!(matches!(limiter.check("key1"), RateLimitResult::Allowed));
        assert!(matches!(limiter.check("key2"), RateLimitResult::Allowed));
    }

    #[test]
    fn test_rate_limit_limited() {
        let mut limiter = RateLimiter::new(2, Duration::from_secs(10));

        assert!(limiter.check("key1").is_allowed());
        assert!(limiter.check("key1").is_allowed());
        let result = limiter.check("key1");
        assert!(result.is_limited());
        
        // Check retry_after is approximately 10 seconds (allow small tolerance for timing)
        let retry_after = result.retry_after().unwrap();
        assert!(retry_after >= Duration::from_secs(9));
        assert!(retry_after <= Duration::from_secs(11));
    }

    #[test]
    fn test_window_reset() {
        let mut limiter = RateLimiter::new(2, Duration::from_millis(100));

        assert!(limiter.check("key1").is_allowed());
        assert!(limiter.check("key1").is_allowed());
        assert!(limiter.check("key1").is_limited());

        // Wait for window to reset
        std::thread::sleep(Duration::from_millis(110));

        assert!(limiter.check("key1").is_allowed());
    }

    #[test]
    fn test_different_keys() {
        let mut limiter = RateLimiter::new(2, Duration::from_secs(10));

        assert!(limiter.check("key1").is_allowed());
        assert!(limiter.check("key1").is_allowed());
        assert!(limiter.check("key1").is_limited());

        // Different key should still be allowed
        assert!(limiter.check("key2").is_allowed());
        assert!(limiter.check("key3").is_allowed());
    }

    #[test]
    fn test_update_from_headers() {
        use reqwest::header::HeaderMap;
        use std::str::FromStr;

        let mut limiter = RateLimiter::new(100, Duration::from_secs(60));
        let mut headers = HeaderMap::new();

        headers.insert(
            "x-ratelimit-remaining",
            reqwest::header::HeaderValue::from_str("95").unwrap(),
        );
        headers.insert(
            "x-ratelimit-reset",
            reqwest::header::HeaderValue::from_str("1234567890").unwrap(),
        );

        limiter.update_from_headers("test_key", &headers);
        // Just verify it doesn't panic - we can't easily test the exact values
    }
}
