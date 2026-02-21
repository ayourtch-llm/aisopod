//! Session compaction functionality for managing conversation history.
//!
//! This module provides compaction strategies and methods to reduce
//! the size of session history by summarizing or removing old messages.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Record of compaction operations for a session.
///
/// Tracks when the last compaction occurred and how many times
/// the session has been compacted.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompactionRecord {
    /// The number of times this session has been compacted.
    pub compaction_count: u32,
    /// When the session was last compacted.
    pub last_compacted_at: Option<DateTime<Utc>>,
    /// Optional summary text from the most recent compaction.
    pub summary: Option<String>,
}

/// Strategy for compacting a session's message history.
///
/// Different strategies offer different approaches to reducing
/// conversation history while preserving important context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompactionStrategy {
    /// No compaction: return the current compaction record unchanged.
    None,
    /// Sliding window: keep only the most recent messages, delete older ones.
    SlidingWindow {
        /// Maximum number of messages to keep after compaction.
        max_messages: u32,
    },
    /// Summarize: delete all messages and replace with a single system summary.
    Summarize,
}

impl Default for CompactionStrategy {
    fn default() -> Self {
        CompactionStrategy::None
    }
}
