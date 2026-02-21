//! Memory integration utilities for agent execution.
//!
//! This module provides utilities for integrating memory queries
//! into the agent execution pipeline, including pre-run memory
//! injection and conversation analysis.

use crate::pipeline::MemoryQueryPipeline;
use crate::types::MemoryQueryOptions;
use aisopod_provider::types::{Message, Role};
use anyhow::Result;
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
    let mut last_n = conversation
        .iter()
        .rev()
        .take(5)
        .collect::<Vec<_>>();
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
                            aisopod_provider::types::ContentPart::Text { text } => Some(text.clone()),
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

    // If the query is empty, return early with no memories message
    if query.trim().is_empty() {
        return Ok(format_memory_context(&[], agent_id));
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
        .map(|m| {
            format!("- [score: {:.2}] {}", m.score, m.entry.content)
        })
        .collect();

    let joined = bullets.join("\n");
    format!("## Relevant Memories\n\n{}", joined)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::OpenAiEmbeddingProvider;
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
        let embedder = OpenAiEmbeddingProvider::new("test-key".to_string(), None, Some(4));

        let pipeline = MemoryQueryPipeline::new(Arc::new(store), Arc::new(embedder));
        let conversation: Vec<Message> = Vec::new();

        let context = build_memory_context(&pipeline, "agent-1", &conversation, MemoryQueryOptions::default())
            .await
            .unwrap();

        assert!(context.contains("No relevant memories found"));
    }

    #[tokio::test]
    async fn test_build_memory_context_with_messages() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SqliteMemoryStore::new(db_path.to_str().unwrap(), 4).unwrap();
        let embedder = OpenAiEmbeddingProvider::new("test-key".to_string(), None, Some(4));

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

        let context = build_memory_context(&pipeline, "agent-1", &conversation, MemoryQueryOptions::default())
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
        let embedder = OpenAiEmbeddingProvider::new("test-key".to_string(), None, Some(4));

        let pipeline = MemoryQueryPipeline::new(Arc::new(store), Arc::new(embedder));

        // Store some memories
        let entry = MemoryEntry::new(
            "test-id".to_string(),
            "agent-1".to_string(),
            "User prefers vegetarian food".to_string(),
            vec![0.1, 0.2, 0.3, 0.4],
        );
        pipeline.store.store(entry).await.unwrap();

        let conversation = vec![
            Message {
                role: Role::User,
                content: MessageContent::Text("What should I order?".to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        let context = build_memory_context(&pipeline, "agent-1", &conversation, MemoryQueryOptions::default())
            .await
            .unwrap();

        assert!(context.contains("Relevant Memories"));
        assert!(context.contains("User prefers vegetarian food"));
    }
}
