//! Memory integration for agent execution.
//!
//! This module provides:
//! - Pre-run memory injection into system prompts
//! - Post-run memory extraction from conversations
//! - A `memory` tool that agents can use to store, query, and delete memories

use aisopod_provider::types::{Message, MessageContent, Role};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::types::{AgentRunParams, ToolSchema};
use aisopod_memory::{build_memory_context, MemoryManager, MemoryQueryOptions, MemoryQueryPipeline};
use aisopod_tools::{Tool, ToolContext, ToolResult};

/// Configuration for memory integration in the agent engine.
#[derive(Clone)]
pub struct MemoryConfig {
    /// Query options for pre-run memory injection
    pub query_options: MemoryQueryOptions,
    /// Whether to extract memories after agent runs
    pub extract_after_run: bool,
    /// Minimum message count before extraction is triggered
    pub min_messages_for_extraction: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            query_options: MemoryQueryOptions::default(),
            extract_after_run: true,
            min_messages_for_extraction: 2,
        }
    }
}

/// Builds the system prompt with memory context injected.
///
/// This function queries relevant memories from the conversation and
/// injects them into the system prompt before agent execution.
///
/// # Arguments
/// * `pipeline` - The memory query pipeline
/// * `agent_id` - The agent ID for filtering memories
/// * `messages` - The conversation messages
/// * `config` - Memory configuration
/// * `base_prompt` - The base system prompt
///
/// # Returns
/// The system prompt with memory context injected, or the base prompt
/// if memory is not available or an error occurs.
pub async fn inject_memory_context(
    pipeline: Option<&Arc<MemoryQueryPipeline>>,
    agent_id: &str,
    messages: &[Message],
    config: &MemoryConfig,
    base_prompt: &str,
) -> String {
    // If no pipeline is provided, return base prompt
    let Some(pipeline) = pipeline else {
        return base_prompt.to_string();
    };

    // Build memory context
    match build_memory_context(pipeline, agent_id, messages, config.query_options.clone()).await {
        Ok(memory_context) => {
            // Inject memory context into the system prompt
            if base_prompt.is_empty() {
                memory_context
            } else {
                format!("{}\n\n{}", base_prompt, memory_context)
            }
        }
        Err(e) => {
            // Log error but continue with base prompt
            eprintln!("Failed to build memory context: {}", e);
            base_prompt.to_string()
        }
    }
}

/// Extracts and stores memories from a conversation after agent execution.
///
/// This function analyzes the conversation and stores key facts as memories.
///
/// # Arguments
/// * `manager` - The memory manager for extraction
/// * `agent_id` - The agent ID for the memories
/// * `messages` - The conversation messages
/// * `config` - Memory configuration
///
/// # Returns
/// Ok(()) if extraction was successful or skipped, Err if extraction failed.
pub async fn extract_memories_after_run(
    manager: Option<&Arc<MemoryManager>>,
    agent_id: &str,
    messages: &[Message],
    config: &MemoryConfig,
) -> Result<()> {
    // If no manager is provided, skip extraction
    let Some(manager) = manager else {
        return Ok(());
    };

    // Check if we have enough messages for meaningful extraction
    if messages.len() < config.min_messages_for_extraction {
        return Ok(());
    }

    // Extract and store memories
    manager.extract_memories(agent_id, messages).await?;

    Ok(())
}

/// A tool that allows agents to interact with the memory system.
///
/// This tool supports three actions:
/// - `store`: Store a new memory with content and optional tags
/// - `query`: Query memories using a query string
/// - `delete`: Delete a memory by ID
#[derive(Clone)]
pub struct MemoryTool {
    /// The query pipeline for querying memories
    pipeline: Arc<MemoryQueryPipeline>,
    /// The memory manager for storing and deleting memories
    manager: Arc<MemoryManager>,
}

impl MemoryTool {
    /// Creates a new MemoryTool with the given pipeline and manager.
    pub fn new(pipeline: Arc<MemoryQueryPipeline>, manager: Arc<MemoryManager>) -> Self {
        Self { pipeline, manager }
    }

    /// Parses the action from tool parameters.
    fn parse_action(params: &Value) -> Result<String> {
        let action = params
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'action' field in parameters"))?;

        match action {
            "store" | "query" | "delete" => Ok(action.to_string()),
            _ => Err(anyhow::anyhow!(
                "Invalid action '{}'. Must be 'store', 'query', or 'delete'",
                action
            )),
        }
    }

    /// Handles the store action - stores a new memory.
    async fn handle_store(
        &self,
        params: &Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult> {
        // Extract content
        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'content' field for store action"))?;

        // Extract optional tags
        let tags: Vec<String> = params
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // Generate embedding using the pipeline's embedder
        let embedder = self.pipeline.embedder();
        let embedding = embedder.embed(content).await?;

        // Create memory entry with generated ID
        let entry = aisopod_memory::MemoryEntry::new(
            uuid::Uuid::new_v4().to_string(),
            ctx.agent_id.clone(),
            content.to_string(),
            embedding,
        );

        // Update metadata with tags
        let mut updated_entry = entry;
        updated_entry.metadata.tags = tags;

        // Store the entry using the manager's store
        let manager_store = self.manager.store();
        manager_store.store(updated_entry).await?;

        Ok(ToolResult::success("Memory stored successfully".to_string()))
    }

    /// Handles the query action - queries and returns relevant memories.
    async fn handle_query(
        &self,
        params: &Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult> {
        // Extract query string
        let query = params
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'query' field for query action"))?;

        // Extract optional top_k
        let top_k = params
            .get("top_k")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .unwrap_or(5);

        // Build options
        let opts = MemoryQueryOptions {
            top_k,
            ..Default::default()
        };

        // Query and format results
        let context = self.pipeline.query_and_format(query, opts).await?;

        Ok(ToolResult::success(context))
    }

    /// Handles the delete action - deletes a memory by ID.
    async fn handle_delete(
        &self,
        params: &Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult> {
        // Extract memory ID
        let id = params
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'id' field for delete action"))?;

        // Get the store from manager and delete
        let manager_store = self.manager.store();
        manager_store.delete(id).await?;

        Ok(ToolResult::success(format!("Memory '{}' deleted successfully", id)))
    }
}

#[async_trait::async_trait]
impl Tool for MemoryTool {
    fn name(&self) -> &str {
        "memory"
    }

    fn description(&self) -> &str {
        "A tool for storing, querying, and deleting memories. Use this tool to remember important information from the conversation or to retrieve relevant past memories."
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["store", "query", "delete"],
                    "description": "The action to perform: 'store', 'query', or 'delete'"
                },
                "content": {
                    "type": "string",
                    "description": "The content to store (required for 'store' action)"
                },
                "tags": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Optional tags for categorizing memories (for 'store' action)"
                },
                "query": {
                    "type": "string",
                    "description": "The query string for searching memories (required for 'query' action)"
                },
                "top_k": {
                    "type": "integer",
                    "minimum": 1,
                    "default": 5,
                    "description": "Maximum number of results to return (for 'query' action)"
                },
                "id": {
                    "type": "string",
                    "description": "The ID of the memory to delete (required for 'delete' action)"
                }
            },
            "required": ["action"],
            "anyOf": [
                {
                    "required": ["action", "content"],
                    "description": "For 'store' action"
                },
                {
                    "required": ["action", "query"],
                    "description": "For 'query' action"
                },
                {
                    "required": ["action", "id"],
                    "description": "For 'delete' action"
                }
            ]
        })
    }

    async fn execute(&self, params: serde_json::Value, ctx: &ToolContext) -> Result<ToolResult> {
        let action = Self::parse_action(&params)?;

        match action.as_str() {
            "store" => self.handle_store(&params, ctx).await,
            "query" => self.handle_query(&params, ctx).await,
            "delete" => self.handle_delete(&params, ctx).await,
            _ => Err(anyhow::anyhow!("Unexpected action: {}", action)),
        }
    }
}

/// Creates a ToolSchema for the memory tool.
pub fn create_memory_tool_schema() -> ToolSchema {
    let parameters = serde_json::json!({
        "type": "object",
        "properties": {
            "action": {
                "type": "string",
                "enum": ["store", "query", "delete"],
                "description": "The action to perform: 'store', 'query', or 'delete'"
            },
            "content": {
                "type": "string",
                "description": "The content to store (required for 'store' action)"
            },
            "tags": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Optional tags for categorizing memories (for 'store' action)"
            },
            "query": {
                "type": "string",
                "description": "The query string for searching memories (required for 'query' action)"
            },
            "top_k": {
                "type": "integer",
                "minimum": 1,
                "default": 5,
                "description": "Maximum number of results to return (for 'query' action)"
            },
            "id": {
                "type": "string",
                "description": "The ID of the memory to delete (required for 'delete' action)"
            }
        },
        "required": ["action"],
        "anyOf": [
            {
                "required": ["action", "content"],
                "description": "For 'store' action"
            },
            {
                "required": ["action", "query"],
                "description": "For 'query' action"
            },
            {
                "required": ["action", "id"],
                "description": "For 'delete' action"
            }
        ]
    });

    ToolSchema::new(
        "memory",
        "A tool for store, query, and delete operations on memories. Use this tool to remember important information from the conversation or to retrieve relevant past memories.",
        parameters,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use aisopod_memory::{MemoryManagerConfig, MemoryStore};
    use tempfile::tempdir;

    #[test]
    fn test_memory_tool_schema() {
        let schema = create_memory_tool_schema();
        assert_eq!(schema.name, "memory");
        assert!(schema.description.contains("store"));
        assert!(schema.description.contains("query"));
        assert!(schema.description.contains("delete"));
    }

    #[test]
    fn test_memory_tool_name() {
        // This would require a full setup with pipeline and manager
        // For now, just verify the schema compiles
        let _schema = create_memory_tool_schema();
    }

    #[test]
    fn test_memory_config_default() {
        let config = MemoryConfig::default();
        assert!(config.extract_after_run);
        assert_eq!(config.min_messages_for_extraction, 2);
    }
}
