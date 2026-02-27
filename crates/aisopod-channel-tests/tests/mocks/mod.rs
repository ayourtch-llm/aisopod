//! Mock implementations for Tier 2 and Tier 3 channel integration tests.
//!
//! This module provides mock implementations of external services
//! for Signal, iMessage, Google Chat, Microsoft Teams (Tier 2),
//! and Nextcloud, Twitch, Nostr, LINE, Lark/Feishu, Zalo (Tier 3),
//! allowing integration tests to run without external dependencies.

pub mod signal_mock;
pub mod googlechat_mock;
pub mod msteams_mock;
pub mod irc_mock;
pub mod matrix_mock;
pub mod mattermost_mock;

// iMessage mocks are platform-specific (macOS only)
#[cfg(target_os = "macos")]
pub mod imessage_mock;

// Tier 3 channel mocks
pub mod nextcloud_mock;
pub mod twitch_mock;
pub mod nostr_mock;
pub mod line_mock;
pub mod lark_mock;
pub mod zalo_mock;
