//! Mock implementations for Tier 2 channel integration tests.
//!
//! This module provides mock implementations of external services
//! for Signal, iMessage, Google Chat, and Microsoft Teams,
//! allowing integration tests to run without external dependencies.

pub mod signal_mock;
pub mod googlechat_mock;
pub mod msteams_mock;

// iMessage mocks are platform-specific (macOS only)
#[cfg(target_os = "macos")]
pub mod imessage_mock;
