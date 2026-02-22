//! Core memory data types for the aisopod memory system.
//!
//! This module defines the fundamental data structures used throughout
//! the memory system, including entries, metadata, filters, and query options.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The source of a memory entry.
///
/// Indicates whether a memory was created by the agent, user, or system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemorySource {
    /// Memory created by the agent through its own reasoning or actions.
    Agent,
    /// Memory created by user input or explicit instructions.
    User,
    /// Memory created by the system for internal operations or configuration.
    System,
}

/// Metadata associated with a memory entry.
///
/// Contains information about the memory's origin, context, and importance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetadata {
    /// The source that created this memory.
    pub source: MemorySource,
    /// Optional session key linking this memory to a specific conversation.
    pub session_key: Option<String>,
    /// Tags for categorizing and filtering memories.
    pub tags: Vec<String>,
    /// Importance score between 0.0 (least important) and 1.0 (most important).
    pub importance: f32,
    /// Custom key-value pairs for additional metadata.
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for MemoryMetadata {
    fn default() -> Self {
        Self {
            source: MemorySource::System,
            session_key: None,
            tags: Vec::new(),
            importance: 0.5,
            custom: HashMap::new(),
        }
    }
}

/// A memory entry representing a single piece of stored information.
///
/// This is the core data structure for memory in the aisopod system,
/// containing the actual content along with its embedding for semantic search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Unique identifier for this memory entry.
    pub id: String,
    /// Identifier of the agent this memory belongs to.
    pub agent_id: String,
    /// The actual content of the memory.
    pub content: String,
    /// Vector embedding representing the semantic meaning of the content.
    pub embedding: Vec<f32>,
    /// Additional metadata about this memory.
    pub metadata: MemoryMetadata,
    /// Timestamp when this memory was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Timestamp when this memory was last updated.
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl MemoryEntry {
    /// Creates a new memory entry with default metadata.
    pub fn new(id: String, agent_id: String, content: String, embedding: Vec<f32>) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            agent_id,
            content,
            embedding,
            metadata: MemoryMetadata::default(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// A match result from a memory query.
///
/// Contains a memory entry along with a relevance score indicating
/// how well it matches the query.
#[derive(Debug, Clone)]
pub struct MemoryMatch {
    /// The matching memory entry.
    pub entry: MemoryEntry,
    /// Relevance score from the query (typically higher means better match).
    pub score: f32,
}

/// Filter criteria for querying and listing memories.
///
/// All fields are optional; filters are combined with AND logic.
#[derive(Debug, Clone, Default)]
pub struct MemoryFilter {
    /// Filter by specific agent ID.
    pub agent_id: Option<String>,
    /// Filter by memories containing all of these tags.
    pub tags: Option<Vec<String>>,
    /// Filter by memory source.
    pub source: Option<MemorySource>,
    /// Filter by minimum importance score (inclusive).
    pub importance_min: Option<f32>,
    /// Filter by specific session key.
    pub session_key: Option<String>,
    /// Filter by memories created after this timestamp.
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    /// Filter by memories created before this timestamp.
    pub created_before: Option<chrono::DateTime<chrono::Utc>>,
}

/// Options for configuring memory queries.
#[derive(Debug, Clone)]
pub struct MemoryQueryOptions {
    /// Maximum number of results to return.
    pub top_k: usize,
    /// Filter criteria to apply to the query.
    pub filter: MemoryFilter,
    /// Minimum score threshold; results below this are excluded.
    pub min_score: Option<f32>,
}

impl Default for MemoryQueryOptions {
    fn default() -> Self {
        Self {
            top_k: 10,
            filter: MemoryFilter::default(),
            min_score: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_source_serialization() {
        let source = MemorySource::Agent;
        let json = serde_json::to_string(&source).unwrap();
        assert_eq!(json, "\"Agent\"");
        let deserialized: MemorySource = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, MemorySource::Agent);
    }

    #[test]
    fn test_memory_metadata_default() {
        let metadata = MemoryMetadata::default();
        assert_eq!(metadata.source, MemorySource::System);
        assert_eq!(metadata.importance, 0.5);
    }

    #[test]
    fn test_memory_entry_new() {
        let entry = MemoryEntry::new(
            "test-id".to_string(),
            "agent-1".to_string(),
            "test content".to_string(),
            vec![0.1, 0.2, 0.3],
        );
        assert_eq!(entry.id, "test-id");
        assert_eq!(entry.agent_id, "agent-1");
        assert_eq!(entry.content, "test content");
        assert_eq!(entry.embedding, vec![0.1, 0.2, 0.3]);
    }

    #[test]
    fn test_memory_match() {
        let entry = MemoryEntry::new(
            "test-id".to_string(),
            "agent-1".to_string(),
            "test content".to_string(),
            vec![0.1, 0.2, 0.3],
        );
        let match_result = MemoryMatch {
            entry: entry.clone(),
            score: 0.95,
        };
        assert_eq!(match_result.score, 0.95);
        assert_eq!(match_result.entry.id, "test-id");
    }

    #[test]
    fn test_memory_filter_default() {
        let filter = MemoryFilter::default();
        assert!(filter.agent_id.is_none());
        assert!(filter.tags.is_none());
    }

    #[test]
    fn test_memory_query_options_default() {
        let options = MemoryQueryOptions::default();
        assert_eq!(options.top_k, 10);
        assert!(options.min_score.is_none());
    }
}
