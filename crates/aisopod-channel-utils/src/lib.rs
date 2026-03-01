//! Shared Tier 2 & 3 Channel Utilities
//!
//! This crate provides common utilities for implementing messaging channel
//! plugins across different platforms. These utilities help ensure consistent
//! behavior and reduce code duplication across channel implementations.
//!
//! ## Modules
//!
//! - [`rate_limit`] - Rate limiting utilities for API calls
//! - [`markdown`] - Markdown format conversion between platforms
//! - [`media`] - Media transcoding and validation utilities
//! - [`retry`] - Retry logic for failed operations
//! - [`error_mapping`] - Common error types and conversion utilities
//!
//! ## Usage Examples
//!
//! ### Rate Limiting
//!
//! ```rust,ignore
//! use aisopod_channel_utils::rate_limit::RateLimiter;
//!
//! // Create a rate limiter for a specific platform
//! let mut limiter = RateLimiter::for_platform("discord");
//!
//! // Check if a request is allowed
//! match limiter.check("chat_id") {
//!     RateLimitResult::Allowed => {
//!         // Send message
//!     }
//!     RateLimitResult::Limited { retry_after } => {
//!         // Wait before retrying
//!         tokio::time::sleep(retry_after).await;
//!     }
//! }
//! ```
//!
//! ### Markdown Conversion
//!
//! ```rust,ignore
//! use aisopod_channel_utils::markdown::{parse_markdown, MarkdownFormat};
//!
//! // Parse markdown for a specific platform
//! let nodes = parse_markdown("Hello **world**!", &MarkdownFormat::Discord);
//!
//! // Convert between formats
//! // (Full conversion logic in the markdown module)
//! ```
//!
//! ### Media Validation
//!
//! ```rust,ignore
//! use aisopod_channel_utils::media::{validate_media, MediaConstraints, MediaType};
//!
//! // Get platform-specific constraints
//! let constraints = MediaConstraints::for_platform("telegram");
//!
//! // Validate media before sending
//! validate_media("/path/to/image.jpg", 5_000_000, &constraints, MediaType::Image)?;
//! ```
//!
//! ### Retry with Exponential Backoff
//!
//! ```rust,ignore
//! use aisopod_channel_utils::retry::{RetryExecutor, RetryConfig};
//!
//! let mut executor = RetryExecutor::new(|| {
//!     // Your operation here
//!     some_api_call()
//! });
//!
//! match executor.execute() {
//!     RetryResult::Success(result) => {
//!         // Operation succeeded
//!     }
//!     RetryResult::Failed(error) => {
//!         // All retries failed
//!         eprintln!("Failed after {} attempts: {}", error.attempts, error);
//!     }
//!     RetryResult::CircuitOpen => {
//!         // Circuit breaker is open
//!     }
//! }
//! ```
//!
//! ### Error Mapping
//!
//! ```rust,ignore
//! use aisopod_channel_utils::error_mapping::{ChannelError, error_from_http_status, ChannelResult};
//!
//! // Create platform-specific errors
//! let auth_error = ChannelError::authentication("discord", "Invalid token");
//! let rate_limit_error = ChannelError::rate_limit("signal", std::time::Duration::from_secs(30), Some(429));
//!
//! // Convert HTTP status codes to errors
//! let error = error_from_http_status(429, "Too many requests", "telegram");
//!
//! // Handle errors consistently across platforms
//! fn handle_result(result: ChannelResult<String>) {
//!     match result {
//!         Ok(data) => println!("Success: {}", data),
//!         Err(e) => {
//!             if e.is_rate_limit() {
//!                 println!("Rate limited: {}", e);
//!             } else if e.is_authentication() {
//!                 println!("Auth error: {}", e);
//!             } else {
//!                 eprintln!("Error: {}", e);
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! ## Supported Platforms
//!
//! | Platform | Rate Limits | Media Formats | Markdown |
//! |----------|-------------|---------------|----------|
//! | Signal | 60 req/60s | Images, Video, Audio | Standard |
//! | Discord | 50 req/60s | Images, Video, Audio | Discord/Markdown |
//! | Slack | 20 req/60s | Images, Files | mrkdwn |
//! | Telegram | 30 req/60s | Images, Video, Audio | Telegram/Markdown |
//! | WhatsApp | 25 req/60s | Images, Video, Audio | Standard |
//! | IRC | 5 req/10s | None | mIRC codes |
//! | Matrix | 30 req/60s | Images, Files | Markdown |
//!
//! ## Error Handling
//!
//! All channel utilities use [`ChannelResult`] for consistent error handling.
//! The [`ChannelError`] enum provides detailed error information with platform
//! context for debugging.
//!
//! ## Platform-Specific Implementation
//!
//! Channel implementations can implement the [`PlatformErrorMapper`] trait to
//! provide custom error mapping logic while maintaining consistency across
//! the codebase.

pub mod error_mapping;
pub mod markdown;
pub mod media;
pub mod rate_limit;
pub mod retry;

// Re-export common types for convenience
pub use error_mapping::{error_from_http_status, ChannelError, ChannelResult, PlatformErrorMapper};
pub use markdown::{parse_markdown, MarkdownFormat, MarkdownNode};
pub use media::{
    convert_media, detect_media_type_from_extension, get_mime_type, validate_media,
    ConversionOptions, MediaConstraints, MediaError, MediaInfo, MediaType,
};
pub use rate_limit::{RateLimitResult, RateLimiter};
