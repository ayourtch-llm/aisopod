//! Shared utilities for channel implementations.
//!
//! This module provides common functionality that can be used across all channel
//! implementations to ensure consistent behavior and reduce code duplication.
//!
//! ## Modules
//!
//! - [`formatting`] - Message formatting normalization across platforms
//! - [`media`] - Media transcoding and validation
//! - [`rate_limit`] - Rate limiting for API calls
//! - [`connection`] - Connection state management and reconnection logic
//! - [`errors`] - Common error types for channel operations

pub mod connection;
pub mod errors;
pub mod formatting;
pub mod media;
pub mod rate_limit;

// Re-export common types for convenience
pub use connection::{ConnectionManager, ConnectionState};
pub use errors::ChannelError;
pub use formatting::NormalizedMarkdown;
pub use media::MediaAttachment;
pub use rate_limit::RateLimiter;
