//! Automatic memory management for the aisopod memory system.
//!
//! This module provides the `MemoryManager` struct that handles automatic
//! memory lifecycle management including extraction, scoring, consolidation,
//! expiration, and quota enforcement.

use aisopod_provider::types::{Message, MessageContent, Role};
use crate::embedding::EmbeddingProvider;
use crate::store::MemoryStore;
use crate::types::{MemoryEntry, MemoryFilter, MemorySource, MemoryMetadata};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Configuration for memory management behavior.
///
/// Controls thresholds and limits for automatic memory management operations.
#[derive(Debug, Clone)]
pub struct MemoryManagerConfig {
    /// Maximum number of memories to keep per agent.
    pub max_memories_per_agent: usize,
    /// Number of days after which memories expire (if low importance).
    pub expiration_days: u64,
    /// Minimum importance threshold for keeping memories during expiration.
    pub min_importance_threshold: f32,
    /// Cosine similarity threshold for memory consolidation.
    pub consolidation_similarity_threshold: f32,
}

impl Default for MemoryManagerConfig {
    fn default() -> Self {
        Self {
            max_memories_per_agent: 1000,
            expiration_days: 90,
            min_importance_threshold: 0.1,
            consolidation_similarity_threshold: 0.92,
        }
    }
}

/// Manages automatic memory lifecycle operations.
///
/// This struct provides methods for:
/// - Extracting facts from conversations and storing them as memories
/// - Scoring memory importance based on frequency, recency, and base importance
/// - Consolidating similar memories to reduce redundancy
/// - Expiring old or low-importance memories
/// - Enforcing per-agent storage quotas
pub struct MemoryManager {
    store: Arc<dyn MemoryStore>,
    embedder: Arc<dyn EmbeddingProvider>,
    config: MemoryManagerConfig,
}

impl MemoryManager {
    /// Creates a new `MemoryManager`.
    ///
    /// # Arguments
    /// * `store` - The underlying memory store for persistence
    /// * `embedder` - The embedding provider for generating vector embeddings
    /// * `config` - Configuration for memory management behavior
    pub fn new(
        store: Arc<dyn MemoryStore>,
        embedder: Arc<dyn EmbeddingProvider>,
        config: MemoryManagerConfig,
    ) -> Self {
        Self {
            store,
            embedder,
            config,
        }
    }

    /// Gets a reference to the store.
    pub fn store(&self) -> &Arc<dyn MemoryStore> {
        &self.store
    }

    /// Extracts key facts from conversation transcripts and stores them as memories.
    ///
    /// Iterates through conversation messages, identifies key facts, decisions,
    /// preferences, and instructions using simple heuristics, generates embeddings,
    /// and stores each as a memory entry.
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID these memories belong to
    /// * `conversation` - List of messages to extract facts from
    ///
    /// # Returns
    /// Returns a list of newly created memory entries.
    ///
    /// # Errors
    /// Returns an error if storage or embedding generation fails.
    pub async fn extract_memories(
        &self,
        agent_id: &str,
        conversation: &[Message],
    ) -> Result<Vec<MemoryEntry>> {
        let mut entries = Vec::new();

        for message in conversation {
            // Skip empty messages
            let content = match &message.content {
                MessageContent::Text(text) => text.clone(),
                MessageContent::Parts(parts) => {
                    // Concatenate all text parts
                    parts
                        .iter()
                        .filter_map(|p| match p {
                            aisopod_provider::types::ContentPart::Text { text } => Some(text.clone()),
                            _ => None, // Ignore non-text parts
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                }
                // Handle future variants of non-exhaustive enum
                _ => continue,
            };

            // If message content is empty (e.g., only non-text parts), skip
            if content.is_empty() {
                continue;
            }

            // Identify key facts using heuristics
            let facts = Self::extract_facts_from_content(&content, message.role.clone());

            for (fact, is_explicit_memory) in facts {
                // Generate embedding for the fact
                let embedding = self.embedder.embed(&fact).await?;

                // Calculate importance based on heuristics
                let base_importance = if is_explicit_memory {
                    0.9
                } else {
                    0.5
                };

                let entry = MemoryEntry {
                    id: uuid::Uuid::new_v4().to_string(),
                    agent_id: agent_id.to_string(),
                    content: fact,
                    embedding,
                    metadata: MemoryMetadata {
                        source: match message.role {
                            Role::User => MemorySource::User,
                            Role::Assistant => MemorySource::Agent,
                            Role::System => MemorySource::System,
                            Role::Tool => MemorySource::System,
                            // Handle any future Role variants
                            _ => MemorySource::System,
                        },
                        session_key: None,
                        tags: Vec::new(),
                        importance: base_importance,
                        custom: std::collections::HashMap::new(),
                    },
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                // Store the entry
                self.store.store(entry.clone()).await?;
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    /// Extracts facts from conversation content using simple heuristics.
    ///
    /// Looks for sentences indicating:
    /// - User preferences ("I like", "I prefer", "My favorite")
    /// - Explicit memory requests ("remember that", "don't forget")
    /// - General assertions and facts
    ///
    /// # Arguments
    /// * `content` - The text content to extract facts from
    /// * `role` - The role of the message sender
    ///
    /// # Returns
    /// Returns a list of (fact, is_explicit_memory) tuples.
    fn extract_facts_from_content(
        content: &str,
        role: Role,
    ) -> Vec<(String, bool)> {
        let mut facts = Vec::new();
        let is_user = matches!(role, Role::User);

        // Split into sentences (simple heuristic)
        let sentences: Vec<&str> = content
            .split('.')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for sentence in sentences {
            let lower = sentence.to_lowercase();
            let is_explicit_memory = lower.contains("remember that")
                || lower.contains("don't forget")
                || lower.contains("keep in mind");

            // For user messages, look for preferences
            if is_user {
                let has_preference = lower.contains("i like")
                    || lower.contains("i prefer")
                    || lower.contains("my favorite")
                    || lower.contains("i want")
                    || lower.contains("i need");

                if has_preference || is_explicit_memory {
                    facts.push((sentence.to_string(), true));
                    continue;
                }
            }

            // Extract assertions and facts
            if Self::is_fact_like(sentence) {
                facts.push((sentence.to_string(), is_explicit_memory));
            }
        }

        facts
    }

    /// Determines if a sentence looks like a fact or assertion.
    fn is_fact_like(sentence: &str) -> bool {
        let lower = sentence.to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().collect();

        // Check for common fact-indicating patterns
        let has_named_entity = words.iter().any(|w| {
            w.len() > 3 && w.chars().next().map_or(false, |c| c.is_uppercase())
        });

        let has_verb = words.iter().any(|w| {
            ["is", "are", "was", "were", "has", "have", "had", "does", "do", "did"]
                .contains(w)
                || w.ends_with("ing")
                || w.ends_with("ed")
        });

        let has_object = words.iter().any(|w| {
            ["the", "a", "an", "this", "that", "these", "those"].contains(w)
        });

        // A fact-like sentence typically has a subject, verb, and object
        has_verb && has_object && (has_named_entity || words.len() > 3)
    }

    /// Scores the importance of a memory entry based on multiple factors.
    ///
    /// Computes a weighted score combining:
    /// - Base importance (from metadata): 40%
    /// - Frequency factor (access_count): 30%
    /// - Recency factor (days since last access): 30%
    ///
    /// # Arguments
    /// * `entry` - The memory entry to score
    /// * `access_count` - Number of times this memory has been accessed
    /// * `last_accessed` - When this memory was last accessed
    ///
    /// # Returns
    /// A weighted importance score between 0.0 and 1.0.
    pub fn score_importance(
        &self,
        entry: &MemoryEntry,
        access_count: u32,
        last_accessed: DateTime<Utc>,
    ) -> f32 {
        // Base importance from metadata (40%)
        let base = entry.metadata.importance.max(0.0).min(1.0);

        // Frequency factor: min(1.0, access_count / 10) (30%)
        let frequency_factor = (access_count as f32 / 10.0).min(1.0);

        // Recency factor: exponential decay based on days since last access (30%)
        let now = Utc::now();
        let elapsed = now.signed_duration_since(last_accessed);
        let days_old = elapsed.num_days() as f32;

        // Exponential decay: factor halves every 7 days
        let recency_factor = 2.0_f32.powf(-days_old / 7.0).max(0.0).min(1.0);

        // Weighted combination
        let score = base * 0.4 + frequency_factor * 0.3 + recency_factor * 0.3;

        score.max(0.0).min(1.0)
    }

    /// Consolidates similar memories for an agent by merging duplicates.
    ///
    /// Lists all memories for the agent, computes pairwise cosine similarity
    /// of embeddings, and merges entries above the threshold into a single entry.
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID to consolidate memories for
    ///
    /// # Returns
    /// Returns the number of entries consolidated (merged).
    ///
    /// # Errors
    /// Returns an error if listing or storage fails.
    pub async fn consolidate(&self, agent_id: &str) -> Result<u32> {
        let filter = MemoryFilter {
            agent_id: Some(agent_id.to_string()),
            tags: None,
            source: None,
            importance_min: None,
            session_key: None,
            created_after: None,
            created_before: None,
        };

        let memories = self.store.list(filter).await?;

        if memories.len() < 2 {
            return Ok(0);
        }

        let mut consolidated_count = 0u32;
        let mut to_remove = std::collections::HashSet::new();

        // Compare each pair of memories
        for i in 0..memories.len() {
            if to_remove.contains(&i) {
                continue;
            }

            let mem_i = &memories[i];

            for j in (i + 1)..memories.len() {
                if to_remove.contains(&j) {
                    continue;
                }

                let mem_j = &memories[j];

                // Compute cosine similarity of embeddings
                let similarity = Self::cosine_similarity(&mem_i.embedding, &mem_j.embedding);

                if similarity >= self.config.consolidation_similarity_threshold {
                    // Merge: keep higher importance, average embedding
                    let new_importance = mem_i.metadata.importance.max(mem_j.metadata.importance);
                    let new_embedding: Vec<f32> = mem_i
                        .embedding
                        .iter()
                        .zip(mem_j.embedding.iter())
                        .map(|(a, b)| (a + b) / 2.0)
                        .collect();

                    // Update mem_i with merged values
                    let mut updated_mem = mem_i.clone();
                    updated_mem.metadata.importance = new_importance;
                    updated_mem.embedding = new_embedding;
                    updated_mem.updated_at = Utc::now();

                    // Store updated entry
                    self.store.store(updated_mem).await?;

                    // Mark mem_j for deletion
                    to_remove.insert(j);
                    consolidated_count += 1;
                }
            }
        }

        // Delete consolidated memories
        for idx in to_remove {
            let mem = &memories[idx];
            self.store.delete(&mem.id).await?;
        }

        Ok(consolidated_count)
    }

    /// Computes cosine similarity between two vectors.
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.is_empty() || b.is_empty() || a.len() != b.len() {
            return 0.0;
        }

        let mut dot_product = 0.0;
        let mut a_magnitude = 0.0;
        let mut b_magnitude = 0.0;

        for i in 0..a.len() {
            dot_product += a[i] * b[i];
            a_magnitude += a[i] * a[i];
            b_magnitude += b[i] * b[i];
        }

        let a_magnitude = a_magnitude.sqrt();
        let b_magnitude = b_magnitude.sqrt();

        if a_magnitude == 0.0 || b_magnitude == 0.0 {
            return 0.0;
        }

        dot_product / (a_magnitude * b_magnitude)
    }

    /// Expires old or low-importance memories for an agent.
    ///
    /// Deletes memories where `created_at` is older than `expiration_days`
    /// AND `importance` is below `min_importance_threshold`.
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID to expire memories for
    ///
    /// # Returns
    /// Returns the number of entries expired.
    ///
    /// # Errors
    /// Returns an error if listing or deletion fails.
    pub async fn expire(&self, agent_id: &str) -> Result<u32> {
        let filter = MemoryFilter {
            agent_id: Some(agent_id.to_string()),
            tags: None,
            source: None,
            importance_min: None,
            session_key: None,
            created_after: None,
            created_before: None,
        };

        let memories = self.store.list(filter).await?;

        let cutoff = Utc::now() - Duration::days(self.config.expiration_days as i64);

        let mut expired_count = 0u32;

        for mem in memories {
            if mem.created_at < cutoff && mem.metadata.importance < self.config.min_importance_threshold
            {
                self.store.delete(&mem.id).await?;
                expired_count += 1;
            }
        }

        Ok(expired_count)
    }

    /// Enforces per-agent storage quotas by evicting lowest-importance memories.
    ///
    /// If an agent's memory count exceeds `max_memories_per_agent`, deletes
    /// the lowest-importance entries until the count is within limits.
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID to enforce quota for
    ///
    /// # Returns
    /// Returns the number of entries evicted.
    ///
    /// # Errors
    /// Returns an error if listing or deletion fails.
    pub async fn enforce_quota(&self, agent_id: &str) -> Result<u32> {
        let filter = MemoryFilter {
            agent_id: Some(agent_id.to_string()),
            tags: None,
            source: None,
            importance_min: None,
            session_key: None,
            created_after: None,
            created_before: None,
        };

        let mut memories = self.store.list(filter).await?;

        if memories.len() <= self.config.max_memories_per_agent {
            return Ok(0);
        }

        // Sort by importance (ascending) to find lowest importance entries
        memories.sort_by(|a, b| {
            a.metadata
                .importance
                .partial_cmp(&b.metadata.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let num_to_evict = memories.len() - self.config.max_memories_per_agent;

        // Delete lowest importance entries
        let mut evicted_count = 0u32;
        for mem in memories.iter().take(num_to_evict) {
            self.store.delete(&mem.id).await?;
            evicted_count += 1;
        }

        Ok(evicted_count)
    }

    /// Runs all memory management operations in sequence.
    ///
    /// Executes expiration, consolidation, and quota enforcement
    /// for the given agent, in that order.
    ///
    /// # Arguments
    /// * `agent_id` - The agent ID to maintain memories for
    ///
    /// # Returns
    /// Returns Ok(()) on success, Err if any operation fails.
    pub async fn maintain(&self, agent_id: &str) -> Result<()> {
        self.expire(agent_id).await?;
        self.consolidate(agent_id).await?;
        self.enforce_quota(agent_id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::MockEmbeddingProvider;
    use crate::sqlite::SqliteMemoryStore;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_memory_manager_config_default() {
        let config = MemoryManagerConfig::default();
        assert_eq!(config.max_memories_per_agent, 1000);
        assert_eq!(config.expiration_days, 90);
        assert_eq!(config.min_importance_threshold, 0.1);
        assert_eq!(config.consolidation_similarity_threshold, 0.92);
    }

    #[tokio::test]
    async fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];
        let d = vec![-1.0, 0.0, 0.0];

        assert!((MemoryManager::cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
        assert!((MemoryManager::cosine_similarity(&a, &c) - 0.0).abs() < 0.001);
        assert!((MemoryManager::cosine_similarity(&a, &d) - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_extract_facts_from_content_user() {
        let content = "I like pizza. My favorite color is blue. Remember that I'm allergic to peanuts.";
        let facts = MemoryManager::extract_facts_from_content(content, Role::User);
        
        assert!(facts.len() >= 2, "Should extract at least 2 facts for user preferences");
    }

    #[test]
    fn test_extract_facts_from_content_assistant() {
        let content = "The weather is sunny today. I can help you with that.";
        let facts = MemoryManager::extract_facts_from_content(content, Role::Assistant);
        
        // Assistant facts are extracted but not preferences
        assert!(facts.len() >= 1, "Should extract at least 1 fact from assistant");
    }

    #[test]
    fn test_is_fact_like() {
        assert!(MemoryManager::is_fact_like("The cat is black."));
        assert!(MemoryManager::is_fact_like("John likes apples."));
        assert!(!MemoryManager::is_fact_like("Hello."));
        assert!(!MemoryManager::is_fact_like("Run."));
    }

    // ==================== Memory Management Tests ====================

    /// Helper to create a test manager with in-memory SQLite
    fn test_manager(embedding_dim: usize, max_memories: usize) -> (MemoryManager, Arc<dyn MemoryStore>) {
        let store = Arc::new(SqliteMemoryStore::new(":memory:", embedding_dim).unwrap());
        let embedder = Arc::new(MockEmbeddingProvider::new(embedding_dim));
        let config = MemoryManagerConfig {
            max_memories_per_agent: max_memories,
            expiration_days: 90,
            min_importance_threshold: 0.1,
            consolidation_similarity_threshold: 0.92,
        };
        let manager = MemoryManager::new(store.clone(), embedder, config);
        (manager, store)
    }

    #[tokio::test]
    async fn test_expire_old_entries() {
        let (manager, _) = test_manager(4, 1000);

        // Store entries with old timestamps
        let old_time = Utc::now() - Duration::days(100); // Older than expiration threshold

        // Low importance old entry - should be expired
        let entry1 = MemoryEntry {
            created_at: old_time,
            updated_at: old_time,
            metadata: crate::types::MemoryMetadata {
                importance: 0.05, // Below threshold
                ..Default::default()
            },
            ..MemoryEntry::new(
                "old-low-importance".to_string(),
                "agent-1".to_string(),
                "old low importance".to_string(),
                vec![0.1, 0.2, 0.3, 0.4],
            )
        };
        manager.store.store(entry1).await.unwrap();

        // High importance old entry - should NOT be expired
        let entry2 = MemoryEntry {
            created_at: old_time,
            updated_at: old_time,
            metadata: crate::types::MemoryMetadata {
                importance: 0.5, // Above threshold
                ..Default::default()
            },
            ..MemoryEntry::new(
                "old-high-importance".to_string(),
                "agent-1".to_string(),
                "old high importance".to_string(),
                vec![0.1, 0.2, 0.3, 0.4],
            )
        };
        manager.store.store(entry2).await.unwrap();

        // Recent entry - should NOT be expired
        let entry3 = MemoryEntry {
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: crate::types::MemoryMetadata {
                importance: 0.1, // At threshold
                ..Default::default()
            },
            ..MemoryEntry::new(
                "recent".to_string(),
                "agent-1".to_string(),
                "recent content".to_string(),
                vec![0.1, 0.2, 0.3, 0.4],
            )
        };
        manager.store.store(entry3).await.unwrap();

        // Run expiration
        let expired = manager.expire("agent-1").await.unwrap();
        assert_eq!(expired, 1); // Only the old low-importance entry should be expired

        // Verify remaining entries
        let filter = MemoryFilter {
            agent_id: Some("agent-1".to_string()),
            ..Default::default()
        };
        let entries = manager.store.list(filter).await.unwrap();
        assert_eq!(entries.len(), 2);
        let ids: Vec<&str> = entries.iter().map(|e| e.id.as_str()).collect();
        assert!(ids.contains(&"old-high-importance"));
        assert!(ids.contains(&"recent"));
    }

    #[tokio::test]
    async fn test_expire_preserves_important() {
        let (manager, _) = test_manager(4, 1000);

        // Store an old entry with high importance
        let old_time = Utc::now() - Duration::days(100);
        let entry = MemoryEntry {
            created_at: old_time,
            updated_at: old_time,
            metadata: crate::types::MemoryMetadata {
                importance: 0.5, // High importance
                ..Default::default()
            },
            ..MemoryEntry::new(
                "important-old".to_string(),
                "agent-1".to_string(),
                "important old content".to_string(),
                vec![0.1, 0.2, 0.3, 0.4],
            )
        };
        manager.store.store(entry).await.unwrap();

        // Run expiration
        let expired = manager.expire("agent-1").await.unwrap();
        assert_eq!(expired, 0); // Should not expire high importance

        // Verify the entry still exists
        let filter = MemoryFilter {
            agent_id: Some("agent-1".to_string()),
            ..Default::default()
        };
        let entries = manager.store.list(filter).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "important-old");
    }

    #[tokio::test]
    async fn test_consolidate_similar() {
        let (manager, store) = test_manager(4, 1000);

        // Store two entries with very similar embeddings (cosine similarity > 0.92)
        let embedding = vec![0.5, 0.5, 0.5, 0.5];
        
        let entry1 = MemoryEntry {
            embedding: embedding.clone(),
            metadata: crate::types::MemoryMetadata {
                importance: 0.8,
                ..Default::default()
            },
            ..MemoryEntry::new(
                "similar-1".to_string(),
                "agent-1".to_string(),
                "similar content 1".to_string(),
                embedding.clone(),
            )
        };
        manager.store.store(entry1).await.unwrap();

        let entry2 = MemoryEntry {
            embedding: vec![0.49, 0.51, 0.49, 0.51], // Very similar
            metadata: crate::types::MemoryMetadata {
                importance: 0.7,
                ..Default::default()
            },
            ..MemoryEntry::new(
                "similar-2".to_string(),
                "agent-1".to_string(),
                "similar content 2".to_string(),
                vec![0.49, 0.51, 0.49, 0.51],
            )
        };
        manager.store.store(entry2).await.unwrap();

        // Run consolidation
        let consolidated = manager.consolidate("agent-1").await.unwrap();
        assert_eq!(consolidated, 1); // Two similar entries should be merged

        // Verify only one entry remains
        let filter = MemoryFilter {
            agent_id: Some("agent-1".to_string()),
            ..Default::default()
        };
        let entries = store.list(filter).await.unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[tokio::test]
    async fn test_consolidate_preserves_different() {
        let (manager, store) = test_manager(4, 1000);

        // Store two entries with dissimilar embeddings
        let entry1 = MemoryEntry {
            embedding: vec![1.0, 0.0, 0.0, 0.0],
            ..MemoryEntry::new(
                "different-1".to_string(),
                "agent-1".to_string(),
                "content 1".to_string(),
                vec![1.0, 0.0, 0.0, 0.0],
            )
        };
        manager.store.store(entry1).await.unwrap();

        let entry2 = MemoryEntry {
            embedding: vec![0.0, 1.0, 0.0, 0.0],
            ..MemoryEntry::new(
                "different-2".to_string(),
                "agent-1".to_string(),
                "content 2".to_string(),
                vec![0.0, 1.0, 0.0, 0.0],
            )
        };
        manager.store.store(entry2).await.unwrap();

        // Run consolidation
        let consolidated = manager.consolidate("agent-1").await.unwrap();
        assert_eq!(consolidated, 0); // Different entries should not be merged

        // Verify both entries remain
        let filter = MemoryFilter {
            agent_id: Some("agent-1".to_string()),
            ..Default::default()
        };
        let entries = store.list(filter).await.unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[tokio::test]
    async fn test_enforce_quota() {
        let (manager, store) = test_manager(4, 3); // Max 3 memories

        // Store 5 entries
        for i in 0..5 {
            let entry = MemoryEntry {
                metadata: crate::types::MemoryMetadata {
                    importance: (i as f32) / 5.0, // Importance: 0.0, 0.2, 0.4, 0.6, 0.8
                    ..Default::default()
                },
                ..MemoryEntry::new(
                    format!("quota-{}", i),
                    "agent-1".to_string(),
                    format!("content {}", i),
                    vec![0.1 * (i as f32), 0.2, 0.3, 0.4],
                )
            };
            manager.store.store(entry).await.unwrap();
        }

        // Verify 5 entries exist
        let filter = MemoryFilter {
            agent_id: Some("agent-1".to_string()),
            ..Default::default()
        };
        let entries = store.list(filter.clone()).await.unwrap();
        assert_eq!(entries.len(), 5);

        // Run quota enforcement
        let evicted = manager.enforce_quota("agent-1").await.unwrap();
        assert_eq!(evicted, 2); // Should evict 2 entries (5 - 3 = 2)

        // Verify only 3 entries remain (the highest importance ones)
        let entries = store.list(filter).await.unwrap();
        assert_eq!(entries.len(), 3);

        // Verify the lowest importance entries were evicted
        let importances: Vec<f32> = entries.iter().map(|e| e.metadata.importance).collect();
        assert!(importances.iter().all(|&x| x >= 0.4)); // Should keep importances 0.4, 0.6, 0.8
    }

    #[tokio::test]
    async fn test_maintain_runs_all() {
        let (manager, store) = test_manager(4, 2); // Max 2 memories

        // Store 4 entries to test quota enforcement
        for i in 0..4 {
            let entry = MemoryEntry {
                metadata: crate::types::MemoryMetadata {
                    importance: (i as f32) / 4.0,
                    ..Default::default()
                },
                ..MemoryEntry::new(
                    format!("maintain-{}", i),
                    "agent-1".to_string(),
                    format!("content {}", i),
                    vec![0.1 * (i as f32), 0.2, 0.3, 0.4],
                )
            };
            manager.store.store(entry).await.unwrap();
        }

        // Run maintain (should run expiration, consolidation, and quota enforcement)
        manager.maintain("agent-1").await.unwrap();

        // Verify quota was enforced (should have 2 entries max)
        let filter = MemoryFilter {
            agent_id: Some("agent-1".to_string()),
            ..Default::default()
        };
        let entries = store.list(filter).await.unwrap();
        assert!(entries.len() <= 2);
    }
}
