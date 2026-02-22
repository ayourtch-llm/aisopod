//! Memory integration utilities for agent execution.
//!
//! This module provides utilities for integrating memory queries
//! into the agent execution pipeline, including pre-run memory
//! injection and conversation analysis.

use crate::embedding::{EmbeddingProvider, MockEmbeddingProvider};
use crate::pipeline::MemoryQueryPipeline;
use crate::store::MemoryStore;
use crate::types::{MemoryFilter, MemoryMatch, MemoryQueryOptions};
use aisopod_provider::types::{Message, Role};
use anyhow::Result;
use std::any::Any;
use std::sync::Arc;

/// Builds a memory context by querying relevant memories from the conversation.
///
/// This function extracts the last N messages from the conversation (default 5),
/// concatenates their content into a query string, and uses the memory query
/// pipeline to find and format relevant memories.
///
/// # Arguments
/// * `pipeline` - The memory query pipeline to use for querying
/// * `agent_id` - The agent ID for filtering memories
/// * `conversation` - The conversation history to analyze
/// * `opts` - Query options for memory retrieval
///
/// # Returns
/// Returns a formatted string containing relevant memories, suitable for
/// injection into a system prompt. If no memories are found, returns a
/// message indicating no relevant memories were found.
///
/// # Example
/// ```ignore
/// use aisopod_memory::{MemoryQueryPipeline, MemoryQueryOptions, build_memory_context};
/// # use anyhow::Result;
/// #
/// # async fn example() -> Result<()> {
/// # let pipeline: std::sync::Arc<MemoryQueryPipeline> = unimplemented!();
/// # let conversation: Vec<aisopod_provider::types::Message> = Vec::new();
/// let context = build_memory_context(&pipeline, "agent-1", &conversation, MemoryQueryOptions::default()).await?;
/// # Ok(())
/// # }
/// ```
pub async fn build_memory_context(
    pipeline: &MemoryQueryPipeline,
    agent_id: &str,
    conversation: &[Message],
    opts: MemoryQueryOptions,
) -> Result<String> {
    // Extract the last N messages as query context (default: last 5)
    let mut last_n = conversation.iter().rev().take(5).collect::<Vec<_>>();
    last_n.reverse();

    // Concatenate message content into a single query string
    let query_parts: Vec<String> = last_n
        .iter()
        .filter_map(|msg| {
            match &msg.content {
                aisopod_provider::types::MessageContent::Text(text) => Some(text.clone()),
                aisopod_provider::types::MessageContent::Parts(parts) => {
                    // Extract text from parts
                    let text_parts: Vec<String> = parts
                        .iter()
                        .filter_map(|part| match part {
                            aisopod_provider::types::ContentPart::Text { text } => {
                                Some(text.clone())
                            }
                            _ => None,
                        })
                        .collect();
                    if text_parts.is_empty() {
                        None
                    } else {
                        Some(text_parts.join(" "))
                    }
                }
                // Handle future variants of non-exhaustive enum
                _ => None,
            }
        })
        .collect();

    // Join all parts with a space separator
    let query = query_parts.join(" ");

    // If the query is empty, query for all memories for the agent instead
    if query.trim().is_empty() {
        // Get all memories for the agent using list() with the filter
        let filter = MemoryFilter {
            agent_id: Some(agent_id.to_string()),
            ..opts.filter.clone()
        };
        let memories = pipeline.store().list(filter).await?;
        
        // Convert to MemoryMatch with default scores
        let mut matches: Vec<MemoryMatch> = memories
            .into_iter()
            .map(|entry| MemoryMatch {
                entry,
                score: 1.0, // Default score for list-based results
            })
            .collect();
        
        // Apply the same re-ranking logic as the pipeline
        for match_ in matches.iter_mut() {
            let recency = MemoryQueryPipeline::recency_factor(match_.entry.created_at);
            let importance = match_.entry.metadata.importance;
            // Combined score: similarity * 0.7 + importance * 0.2 + recency * 0.1
            match_.score = match_.score * 0.7 + importance as f32 * 0.2 + recency * 0.1;
        }
        
        // Sort by score descending
        matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Truncate to top_k
        let matches: Vec<MemoryMatch> = matches.into_iter()
            .take(opts.top_k)
            .collect();
        
        return Ok(format_memory_context(&matches, agent_id));
    }

    // Query memories using the conversation context as the query
    let matches = pipeline.query(&query, opts).await?;

    // Format the results
    Ok(format_memory_context(&matches, agent_id))
}

/// Formats memory matches as a context string for prompt injection.
///
/// # Arguments
/// * `matches` - The memory matches to format
/// * `agent_id` - The agent ID (for future filtering if needed)
///
/// # Returns
/// A formatted string suitable for injection into a system prompt.
fn format_memory_context(matches: &[crate::types::MemoryMatch], agent_id: &str) -> String {
    if matches.is_empty() {
        return "## Relevant Memories\n\nNo relevant memories found.".to_string();
    }

    let bullets: Vec<String> = matches
        .iter()
        .map(|m| format!("- [score: {:.2}] {}", m.score, m.entry.content))
        .collect();

    let joined = bullets.join("\n");
    format!("## Relevant Memories\n\n{}", joined)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::MockEmbeddingProvider;
    use crate::pipeline::MemoryQueryPipeline;
    use crate::sqlite::SqliteMemoryStore;
    use crate::types::{MemoryEntry, MemoryQueryOptions};
    use aisopod_provider::types::{Message, MessageContent, Role};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_build_memory_context_empty_conversation() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SqliteMemoryStore::new(db_path.to_str().unwrap(), 4).unwrap();
        let embedder = MockEmbeddingProvider::new(4);

        let pipeline = MemoryQueryPipeline::new(Arc::new(store), Arc::new(embedder));
        let conversation: Vec<Message> = Vec::new();

        let context = build_memory_context(
            &pipeline,
            "agent-1",
            &conversation,
            MemoryQueryOptions::default(),
        )
        .await
        .unwrap();

        assert!(context.contains("No relevant memories found"));
    }

    #[tokio::test]
    async fn test_build_memory_context_with_messages() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SqliteMemoryStore::new(db_path.to_str().unwrap(), 4).unwrap();
        let embedder = MockEmbeddingProvider::new(4);

        let pipeline = MemoryQueryPipeline::new(Arc::new(store), Arc::new(embedder));

        // Create a conversation
        let conversation = vec![
            Message {
                role: Role::User,
                content: MessageContent::Text("I like pizza".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: Role::Assistant,
                content: MessageContent::Text("I can help you order pizza".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        let context = build_memory_context(
            &pipeline,
            "agent-1",
            &conversation,
            MemoryQueryOptions::default(),
        )
        .await
        .unwrap();

        // Should have a header even with no matching memories
        assert!(context.contains("## Relevant Memories"));
    }

    #[tokio::test]
    async fn test_build_memory_context_with_memories() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SqliteMemoryStore::new(db_path.to_str().unwrap(), 4).unwrap();
        let embedder = MockEmbeddingProvider::new(4);

        let pipeline = MemoryQueryPipeline::new(Arc::new(store), Arc::new(embedder));

        // Store some memories
        let entry = MemoryEntry::new(
            "test-id".to_string(),
            "agent-1".to_string(),
            "User prefers vegetarian food".to_string(),
            vec![0.1, 0.2, 0.3, 0.4],
        );
        pipeline.store().store(entry).await.unwrap();

        let conversation = vec![Message {
            role: Role::User,
            content: MessageContent::Text("What should I order?".to_string()),
            tool_calls: None,
            tool_call_id: None,
        }];

        let context = build_memory_context(
            &pipeline,
            "agent-1",
            &conversation,
            MemoryQueryOptions::default(),
        )
        .await
        .unwrap();

        assert!(context.contains("Relevant Memories"));
        assert!(context.contains("User prefers vegetarian food"));
    }

    // ==================== Memory Tool Tests ====================

    /// Simulates a memory tool call with store action
    pub async fn memory_tool_store(
        store: &Arc<dyn crate::store::MemoryStore>,
        agent_id: &str,
        content: &str,
    ) -> Result<String> {
        // Generate embedding using mock (for testing)
        let mut mock_embedder = crate::MockEmbeddingProvider::new(4);
        let embedding = mock_embedder.embed(content).await?;

        let entry = MemoryEntry::new(
            uuid::Uuid::new_v4().to_string(),
            agent_id.to_string(),
            content.to_string(),
            embedding,
        );
        let id = store.store(entry).await?;
        Ok(id)
    }

    /// Simulates a memory tool call with query action
    pub async fn memory_tool_query(
        store: &Arc<dyn crate::store::MemoryStore>,
        agent_id: &str,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<MemoryMatch>> {
        let opts = MemoryQueryOptions {
            top_k,
            filter: MemoryFilter {
                agent_id: Some(agent_id.to_string()),
                ..Default::default()
            },
            min_score: Some(0.0),
        };
        store.query(query, opts).await
    }

    /// Simulates a memory tool call with delete action
    pub async fn memory_tool_delete(
        store: &Arc<dyn crate::store::MemoryStore>,
        id: &str,
    ) -> Result<()> {
        store.delete(id).await
    }

    #[tokio::test]
    async fn test_memory_tool_store() {
        let store = Arc::new(SqliteMemoryStore::new(":memory:", 4).unwrap());
        let agent_id = "agent-1";
        let content = "User loves chocolate ice cream";

        // Store via memory tool
        let id = memory_tool_store(
            &(Arc::clone(&store) as Arc<dyn MemoryStore>),
            agent_id,
            content,
        )
        .await
        .unwrap();

        // Verify it was stored
        let filter = MemoryFilter {
            agent_id: Some(agent_id.to_string()),
            ..Default::default()
        };
        let entries = store.list(filter.clone()).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content, content);
        assert_eq!(entries[0].agent_id, agent_id);
    }

    #[tokio::test]
    async fn test_memory_tool_query() {
        let store = Arc::new(SqliteMemoryStore::new(":memory:", 4).unwrap());
        let agent_id = "agent-1";

        // Store some memories first
        let content1 = "User prefers vegetarian food";
        let content2 = "User likes Italian cuisine";
        memory_tool_store(
            &(Arc::clone(&store) as Arc<dyn MemoryStore>),
            agent_id,
            content1,
        )
        .await
        .unwrap();
        memory_tool_store(
            &(Arc::clone(&store) as Arc<dyn MemoryStore>),
            agent_id,
            content2,
        )
        .await
        .unwrap();

        // Query memories
        let matches = memory_tool_query(
            &(Arc::clone(&store) as Arc<dyn MemoryStore>),
            agent_id,
            "food preferences",
            10,
        )
        .await
        .unwrap();
        assert!(!matches.is_empty());

        // Verify results contain relevant content
        let contents: Vec<&str> = matches.iter().map(|m| m.entry.content.as_str()).collect();
        assert!(contents.contains(&"User prefers vegetarian food"));
        assert!(contents.contains(&"User likes Italian cuisine"));
    }

    #[tokio::test]
    async fn test_memory_tool_delete() {
        let store = Arc::new(SqliteMemoryStore::new(":memory:", 4).unwrap());
        let agent_id = "agent-1";
        let content = "Memory to be deleted";

        // Store a memory
        let id = memory_tool_store(
            &(Arc::clone(&store) as Arc<dyn MemoryStore>),
            agent_id,
            content,
        )
        .await
        .unwrap();

        // Verify it exists
        let filter = MemoryFilter {
            agent_id: Some(agent_id.to_string()),
            ..Default::default()
        };
        let entries = store.list(filter.clone()).await.unwrap();
        assert_eq!(entries.len(), 1);

        // Delete via memory tool
        memory_tool_delete(&(Arc::clone(&store) as Arc<dyn MemoryStore>), &id)
            .await
            .unwrap();

        // Verify it was deleted
        let entries = store.list(filter).await.unwrap();
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn test_no_memory_configured() {
        // Test that operations work correctly when no memory is configured
        // This simulates running an agent without memory

        // Create a minimal mock embedder using a simple struct that implements the trait
        struct MockEmbedder {
            dim: usize,
        }

        #[async_trait::async_trait]
        impl EmbeddingProvider for MockEmbedder {
            async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
                Ok(vec![0.0; self.dim])
            }

            async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
                let mut results = Vec::with_capacity(texts.len());
                for _ in texts {
                    results.push(vec![0.0; self.dim]);
                }
                Ok(results)
            }

            fn dimensions(&self) -> usize {
                self.dim
            }
        }

        let embedder = Arc::new(MockEmbedder { dim: 4 });

        // In this case, we're testing that operations don't fail when
        // memory is not properly configured (simulating the scenario)

        // The key is that operations should handle missing configuration gracefully
        // For example, building memory context with no memories should return empty result

        let context = build_memory_context_helper(
            Arc::clone(&embedder) as Arc<dyn EmbeddingProvider>,
            "agent-1",
        )
        .await;
        assert!(context.contains("No relevant memories found"));
    }

    /// Helper for no_memory_configured test
    async fn build_memory_context_helper(
        embedder: Arc<dyn EmbeddingProvider>,
        agent_id: &str,
    ) -> String {
        // Create a minimal mock store for testing
        struct MockStore;

        #[async_trait::async_trait]
        impl crate::store::MemoryStore for MockStore {
            async fn store(&self, _entry: crate::types::MemoryEntry) -> Result<String> {
                Ok(uuid::Uuid::new_v4().to_string())
            }

            async fn query(
                &self,
                _query: &str,
                _opts: crate::types::MemoryQueryOptions,
            ) -> Result<Vec<crate::types::MemoryMatch>> {
                Ok(Vec::new())
            }

            async fn delete(&self, _id: &str) -> Result<()> {
                Ok(())
            }

            async fn list(
                &self,
                _filter: crate::types::MemoryFilter,
            ) -> Result<Vec<crate::types::MemoryEntry>> {
                Ok(Vec::new())
            }

            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        let store = Arc::new(MockStore);
        let pipeline = MemoryQueryPipeline::new(store, embedder.clone());

        // Empty conversation
        let conversation: Vec<Message> = Vec::new();
        let opts = MemoryQueryOptions::default();

        build_memory_context(&pipeline, agent_id, &conversation, opts)
            .await
            .unwrap_or_else(|_| "Error building context".to_string())
    }

    // ==================== Additional Integration Tests ====================

    #[tokio::test]
    async fn test_build_memory_context_with_recent_messages() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SqliteMemoryStore::new(db_path.to_str().unwrap(), 4).unwrap();
        let embedder = MockEmbeddingProvider::new(4);

        let pipeline = MemoryQueryPipeline::new(Arc::new(store), Arc::new(embedder));

        // Store a relevant memory
        let entry = MemoryEntry::new(
            "test-id".to_string(),
            "agent-1".to_string(),
            "User prefers spicy food".to_string(),
            vec![0.1, 0.2, 0.3, 0.4],
        );
        pipeline.store().store(entry).await.unwrap();

        // Build conversation with many messages (last 5 should be used)
        let conversation = vec![
            Message {
                role: Role::User,
                content: MessageContent::Text("First message".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: Role::Assistant,
                content: MessageContent::Text("Second message".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: Role::User,
                content: MessageContent::Text("Third message".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: Role::Assistant,
                content: MessageContent::Text("Fourth message".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: Role::User,
                content: MessageContent::Text("Fifth message - spicy food".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: Role::Assistant,
                content: MessageContent::Text("Sixth message".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        let opts = MemoryQueryOptions::default();
        let context = build_memory_context(&pipeline, "agent-1", &conversation, opts)
            .await
            .unwrap();

        // Should contain the memory about spicy food
        assert!(context.contains("User prefers spicy food"));
    }

    #[tokio::test]
    async fn test_build_memory_context_with_empty_query() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SqliteMemoryStore::new(db_path.to_str().unwrap(), 4).unwrap();
        let embedder = MockEmbeddingProvider::new(4);

        let pipeline = MemoryQueryPipeline::new(Arc::new(store), Arc::new(embedder));

        // Empty conversation
        let conversation: Vec<Message> = Vec::new();

        let opts = MemoryQueryOptions::default();
        let context = build_memory_context(&pipeline, "agent-1", &conversation, opts)
            .await
            .unwrap();

        // Should return empty context
        assert!(context.contains("No relevant memories found"));
    }

    #[tokio::test]
    async fn test_build_memory_context_with_whitespace_only() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SqliteMemoryStore::new(db_path.to_str().unwrap(), 4).unwrap();
        let embedder = MockEmbeddingProvider::new(4);

        let pipeline = MemoryQueryPipeline::new(Arc::new(store), Arc::new(embedder));

        // Conversation with only whitespace content
        let conversation = vec![
            Message {
                role: Role::User,
                content: MessageContent::Text("   ".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: Role::Assistant,
                content: MessageContent::Text("\t\n".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        let opts = MemoryQueryOptions::default();
        let context = build_memory_context(&pipeline, "agent-1", &conversation, opts)
            .await
            .unwrap();

        // Should return empty context (whitespace-only content is treated as empty)
        assert!(context.contains("No relevant memories found"));
    }

    #[tokio::test]
    async fn test_memory_context_with_multiple_memories() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SqliteMemoryStore::new(db_path.to_str().unwrap(), 4).unwrap();
        let embedder = MockEmbeddingProvider::new(4);

        let pipeline = MemoryQueryPipeline::new(Arc::new(store), Arc::new(embedder));

        // Store multiple memories
        for i in 1..=5 {
            let entry = MemoryEntry::new(
                format!("mem-{}", i),
                "agent-1".to_string(),
                format!("Memory content {}", i),
                vec![0.1 * (i as f32), 0.2, 0.3, 0.4],
            );
            pipeline.store().store(entry).await.unwrap();
        }

        // Query
        let opts = MemoryQueryOptions::default();
        let context = build_memory_context(&pipeline, "agent-1", &[], opts)
            .await
            .unwrap();

        // Should contain all memories
        assert!(context.contains("Relevant Memories"));
        for i in 1..=5 {
            assert!(context.contains(&format!("Memory content {}", i)));
        }
    }

    #[tokio::test]
    async fn test_memory_context_with_no_memories_match() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SqliteMemoryStore::new(db_path.to_str().unwrap(), 4).unwrap();
        let embedder = MockEmbeddingProvider::new(4);

        let pipeline = MemoryQueryPipeline::new(Arc::new(store), Arc::new(embedder));

        // Don't store any memory - this tests the case where there are no memories for the agent
        // Query with empty conversation (which retrieves all memories for the agent)
        let opts = MemoryQueryOptions::default();
        let context = build_memory_context(&pipeline, "agent-1", &[], opts)
            .await
            .unwrap();

        // Should return empty context when no memories exist
        eprintln!("Context for no match test:\n{}", context);
        assert!(context.contains("## Relevant Memories"));
        assert!(context.contains("No relevant memories found"));
    }

    #[tokio::test]
    async fn test_memory_tool_query_multiple_results() {
        let store = Arc::new(SqliteMemoryStore::new(":memory:", 4).unwrap());
        let agent_id = "agent-1";

        // Store multiple memories
        for i in 0..5 {
            let content = format!("Memory content {}", i);
            let embedding = crate::MockEmbeddingProvider::new(4).embed(&content).await.unwrap();

            let entry = MemoryEntry::new(
                uuid::Uuid::new_v4().to_string(),
                agent_id.to_string(),
                content,
                embedding,
            );
            store.store(entry).await.unwrap();
        }

        // Query memories
        let opts = MemoryQueryOptions {
            top_k: 10,
            filter: MemoryFilter {
                agent_id: Some(agent_id.to_string()),
                ..Default::default()
            },
            min_score: Some(0.0),
        };
        let matches = store.query("content", opts).await.unwrap();

        // Should return multiple results
        assert!(!matches.is_empty());
        assert_eq!(matches.len(), 5);
    }

    #[tokio::test]
    async fn test_memory_tool_delete_idempotent() {
        // Test that deleting a non-existent memory doesn't error
        let store = Arc::new(SqliteMemoryStore::new(":memory:", 4).unwrap());
        let agent_id = "agent-1";

        // Store a memory
        let content = "Memory to delete";
        let embedding = crate::MockEmbeddingProvider::new(4).embed(content).await.unwrap();

        let entry = MemoryEntry::new(
            uuid::Uuid::new_v4().to_string(),
            agent_id.to_string(),
            content.to_string(),
            embedding,
        );
        let id = store.store(entry).await.unwrap();

        // Delete the memory
        store.delete(&id).await.unwrap();

        // Delete again (idempotent - should not error)
        store.delete(&id).await.unwrap();
    }

    #[tokio::test]
    async fn test_memory_context_with_importance_ranking() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SqliteMemoryStore::new(db_path.to_str().unwrap(), 4).unwrap();
        let embedder = MockEmbeddingProvider::new(4);

        let pipeline = MemoryQueryPipeline::new(Arc::new(store), Arc::new(embedder));

        // Store memories with different importance levels but same content
        for importance in [0.2, 0.5, 0.8] {
            let entry = MemoryEntry {
                metadata: crate::types::MemoryMetadata {
                    importance,
                    ..Default::default()
                },
                ..MemoryEntry::new(
                    uuid::Uuid::new_v4().to_string(),
                    "agent-1".to_string(),
                    "Same content different importance".to_string(),
                    vec![0.1, 0.2, 0.3, 0.4],
                )
            };
            pipeline.store().store(entry).await.unwrap();
        }

        // Query
        let opts = MemoryQueryOptions::default();
        let context = build_memory_context(&pipeline, "agent-1", &[], opts)
            .await
            .unwrap();

        // Should contain the memory
        assert!(context.contains("Relevant Memories"));
        assert!(context.contains("Same content different importance"));
    }

    #[tokio::test]
    async fn test_memory_context_with_empty_agent_id() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SqliteMemoryStore::new(db_path.to_str().unwrap(), 4).unwrap();
        let embedder = MockEmbeddingProvider::new(4);

        let pipeline = MemoryQueryPipeline::new(Arc::new(store), Arc::new(embedder));

        // Store a memory for empty agent_id
        let entry = MemoryEntry::new(
            "test-id".to_string(),
            "".to_string(), // Empty agent_id
            "Test content".to_string(),
            vec![0.1, 0.2, 0.3, 0.4],
        );
        pipeline.store().store(entry).await.unwrap();

        // Query with empty agent_id
        let opts = MemoryQueryOptions {
            filter: MemoryFilter {
                agent_id: Some("".to_string()),
                ..Default::default()
            },
            ..MemoryQueryOptions::default()
        };
        let context = build_memory_context(&pipeline, "", &[], opts).await.unwrap();

        // Should find the memory stored with empty agent_id
        eprintln!("Context for empty agent_id test:\n{}", context);
        assert!(context.contains("Relevant Memories"));
        assert!(context.contains("Test content"));
    }
}
