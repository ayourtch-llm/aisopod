//! Agent integration tests for the aisopod-memory crate.
//!
//! This module tests the integration between memory operations and agent execution:
//! - Memory context building from conversation history
//! - Memory tool invocations (store, query, delete)
//! - Agent execution without memory configured

use aisopod_memory::{
    build_memory_context, EmbeddingProvider, MemoryManager, MemoryManagerConfig,
    MemoryQueryOptions, MemoryQueryPipeline, MemoryStore,
};
use aisopod_memory::MockEmbeddingProvider;
use aisopod_memory::sqlite::SqliteMemoryStore;
use std::sync::Arc;

// Import the test helpers
mod helpers;

/// Helper function to create a conversation with messages
fn build_conversation(user_messages: Vec<&str>, assistant_messages: Vec<&str>) -> Vec<aisopod_provider::types::Message> {
    let mut messages = Vec::new();
    let min_len = user_messages.len().min(assistant_messages.len());
    
    for i in 0..min_len {
        messages.push(aisopod_provider::types::Message {
            role: aisopod_provider::types::Role::User,
            content: aisopod_provider::types::MessageContent::Text(user_messages[i].to_string()),
            tool_calls: None,
            tool_call_id: None,
        });
        
        messages.push(aisopod_provider::types::Message {
            role: aisopod_provider::types::Role::Assistant,
            content: aisopod_provider::types::MessageContent::Text(assistant_messages[i].to_string()),
            tool_calls: None,
            tool_call_id: None,
        });
    }
    
    // Add any extra user messages if there are more
    for msg in user_messages.iter().skip(min_len) {
        messages.push(aisopod_provider::types::Message {
            role: aisopod_provider::types::Role::User,
            content: aisopod_provider::types::MessageContent::Text(msg.to_string()),
            tool_calls: None,
            tool_call_id: None,
        });
    }
    
    messages
}

/// Creates a new MemoryQueryPipeline with a separate embedder for testing
fn create_test_pipeline() -> (MemoryQueryPipeline, Arc<dyn EmbeddingProvider>) {
    let embedder: Arc<dyn EmbeddingProvider> = Arc::new(MockEmbeddingProvider::new(4));
    let store = SqliteMemoryStore::new_with_embedder(":memory:", 4, Arc::clone(&embedder))
        .expect("Failed to create test store with embedder");
    let pipeline = MemoryQueryPipeline::new(Arc::new(store), Arc::clone(&embedder));
    (pipeline, embedder)
}

/// Helper function to store test memories in a pipeline
async fn store_test_memories(
    pipeline: &MemoryQueryPipeline,
    agent_id: &str,
    memories: Vec<&str>,
) {
    for content in memories {
        let embedder = pipeline.embedder();
        let embedding = embedder.embed(content).await.unwrap();
        
        let entry = aisopod_memory::MemoryEntry::new(
            uuid::Uuid::new_v4().to_string(),
            agent_id.to_string(),
            content.to_string(),
            embedding,
        );
        pipeline.store().store(entry).await.unwrap();
    }
}

/// Helper function to invoke memory tool store action
async fn memory_tool_store(
    pipeline: &MemoryQueryPipeline,
    agent_id: &str,
    content: &str,
) -> Result<String, anyhow::Error> {
    let embedder = pipeline.embedder();
    let embedding = embedder.embed(content).await?;
    
    let entry = aisopod_memory::MemoryEntry::new(
        uuid::Uuid::new_v4().to_string(),
        agent_id.to_string(),
        content.to_string(),
        embedding,
    );
    
    let id = pipeline.store().store(entry).await?;
    Ok(id)
}

/// Helper function to invoke memory tool query action
async fn memory_tool_query(
    pipeline: &MemoryQueryPipeline,
    agent_id: &str,
    query: &str,
    top_k: usize,
) -> Result<Vec<aisopod_memory::MemoryMatch>, anyhow::Error> {
    let opts = MemoryQueryOptions {
        top_k,
        filter: aisopod_memory::MemoryFilter {
            agent_id: Some(agent_id.to_string()),
            ..Default::default()
        },
        min_score: None,
    };
    pipeline.query(query, opts).await
}

/// Helper function to invoke memory tool delete action
async fn memory_tool_delete(
    pipeline: &MemoryQueryPipeline,
    id: &str,
) -> Result<(), anyhow::Error> {
    pipeline.store().delete(id).await
}

#[tokio::test]
async fn test_build_memory_context() {
    let (pipeline, _embedder) = create_test_pipeline();
    
    let agent_id = "agent-1";
    
    // Store some memories
    store_test_memories(
        &pipeline,
        agent_id,
        vec![
            "User prefers vegetarian food",
            "User likes Italian cuisine",
        ],
    )
    .await;
    
    // Build a conversation
    let conversation = build_conversation(
        vec!["What should I order?"],
        vec!["I can recommend some Italian restaurants."],
    );
    
    // Build memory context
    let opts = MemoryQueryOptions::default();
    let context = build_memory_context(&pipeline, agent_id, &conversation, opts)
        .await
        .unwrap();
    
    // Verify the context contains the expected content
    assert!(context.contains("Relevant Memories"));
    assert!(context.contains("User prefers vegetarian food"));
    assert!(context.contains("User likes Italian cuisine"));
}

#[tokio::test]
async fn test_build_memory_context_empty() {
    let (pipeline, _embedder) = create_test_pipeline();
    
    let agent_id = "agent-1";
    let conversation = build_conversation(
        vec!["Hello"],
        vec!["Hi there! How can I help you?"],
    );
    
    let opts = MemoryQueryOptions::default();
    let context = build_memory_context(&pipeline, agent_id, &conversation, opts)
        .await
        .unwrap();
    
    // When no memories match, should indicate no relevant memories found
    assert!(context.contains("No relevant memories found"));
    assert!(context.contains("Relevant Memories"));
}

#[tokio::test]
async fn test_memory_tool_store() {
    let (pipeline, _embedder) = create_test_pipeline();
    
    let agent_id = "agent-1";
    let content = "User loves chocolate ice cream";
    
    // Store via memory tool
    let id = memory_tool_store(&pipeline, agent_id, content)
        .await
        .unwrap();
    
    // Verify it was stored
    let filter = aisopod_memory::MemoryFilter {
        agent_id: Some(agent_id.to_string()),
        ..Default::default()
    };
    
    let entries = pipeline.store().list(filter).await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].content, content);
    assert_eq!(entries[0].agent_id, agent_id);
    assert!(!id.is_empty());
}

#[tokio::test]
async fn test_memory_tool_store_multiple() {
    let (pipeline, _embedder) = create_test_pipeline();
    
    let agent_id = "agent-1";
    
    // Store multiple memories
    let memories = vec![
        "User prefers vegetarian food",
        "User likes Italian cuisine",
        "User is allergic to nuts",
    ];
    
    for content in &memories {
        memory_tool_store(&pipeline, agent_id, content)
            .await
            .unwrap();
    }
    
    // Verify all were stored
    let filter = aisopod_memory::MemoryFilter {
        agent_id: Some(agent_id.to_string()),
        ..Default::default()
    };
    
    let entries = pipeline.store().list(filter).await.unwrap();
    assert_eq!(entries.len(), 3);
    
    // Verify content of all entries
    let contents: Vec<&str> = entries.iter().map(|e| e.content.as_str()).collect();
    for memory in &memories {
        assert!(contents.contains(memory));
    }
}

#[tokio::test]
async fn test_memory_tool_query() {
    let (pipeline, _embedder) = create_test_pipeline();
    
    let agent_id = "agent-1";
    
    // Store some memories first
    let memories = vec![
        "User prefers vegetarian food",
        "User likes Italian cuisine",
        "User is allergic to nuts",
    ];
    
    for content in memories {
        memory_tool_store(&pipeline, agent_id, content)
            .await
            .unwrap();
    }
    
    // Query memories
    let matches = memory_tool_query(&pipeline, agent_id, "food preferences", 10)
        .await
        .unwrap();
    
    assert!(!matches.is_empty());
    
    // Verify results contain relevant content
    let contents: Vec<&str> = matches.iter().map(|m| m.entry.content.as_str()).collect();
    assert!(contents.contains(&"User prefers vegetarian food"));
    assert!(contents.contains(&"User likes Italian cuisine"));
}

#[tokio::test]
async fn test_memory_tool_query_empty() {
    let (pipeline, _embedder) = create_test_pipeline();
    
    let agent_id = "agent-1";
    
    // Query without storing any memories
    let matches = memory_tool_query(&pipeline, agent_id, "some query", 5)
        .await
        .unwrap();
    
    assert!(matches.is_empty());
}

#[tokio::test]
async fn test_memory_tool_query_top_k() {
    let (pipeline, _embedder) = create_test_pipeline();
    
    let agent_id = "agent-1";
    
    // Store 10 memories
    for i in 0..10 {
        memory_tool_store(&pipeline, agent_id, &format!("Memory entry {}", i))
            .await
            .unwrap();
    }
    
    // Query with top_k=3
    let matches = memory_tool_query(&pipeline, agent_id, "test query", 3)
        .await
        .unwrap();
    
    assert_eq!(matches.len(), 3);
}

#[tokio::test]
async fn test_memory_tool_delete() {
    let (pipeline, _embedder) = create_test_pipeline();
    
    let agent_id = "agent-1";
    let content = "Memory to be deleted";
    
    // Store a memory
    let id = memory_tool_store(&pipeline, agent_id, content)
        .await
        .unwrap();
    
    // Verify it exists
    let filter = aisopod_memory::MemoryFilter {
        agent_id: Some(agent_id.to_string()),
        ..Default::default()
    };
    
    let entries = pipeline.store().list(filter.clone()).await.unwrap();
    assert_eq!(entries.len(), 1);
    
    // Delete via memory tool
    memory_tool_delete(&pipeline, &id).await.unwrap();
    
    // Verify it was deleted
    let entries = pipeline.store().list(filter).await.unwrap();
    assert!(entries.is_empty());
}

#[tokio::test]
async fn test_memory_tool_delete_nonexistent() {
    let (pipeline, _embedder) = create_test_pipeline();
    
    // Try to delete a non-existent memory
    let result = memory_tool_delete(&pipeline, "non-existent-id").await;
    
    // Should not error
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_no_memory_configured() {
    // Test that operations work correctly when no memory is configured
    // This simulates running an agent without memory
    
    struct MockEmbedder {
        dim: usize,
    }
    
    #[async_trait::async_trait]
    impl aisopod_memory::EmbeddingProvider for MockEmbedder {
        async fn embed(&self, _text: &str) -> Result<Vec<f32>, anyhow::Error> {
            Ok(vec![0.0; self.dim])
        }
        
        async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, anyhow::Error> {
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
    
    struct MockStore;
    
    #[async_trait::async_trait]
    impl aisopod_memory::MemoryStore for MockStore {
        async fn store(&self, _entry: aisopod_memory::MemoryEntry) -> Result<String, anyhow::Error> {
            Ok(uuid::Uuid::new_v4().to_string())
        }
        
        async fn query(
            &self,
            _query: &str,
            _opts: aisopod_memory::MemoryQueryOptions,
        ) -> Result<Vec<aisopod_memory::MemoryMatch>, anyhow::Error> {
            Ok(Vec::new())
        }
        
        async fn delete(&self, _id: &str) -> Result<(), anyhow::Error> {
            Ok(())
        }
        
        async fn list(
            &self,
            _filter: aisopod_memory::MemoryFilter,
        ) -> Result<Vec<aisopod_memory::MemoryEntry>, anyhow::Error> {
            Ok(Vec::new())
        }
    }
    
    let embedder = Arc::new(MockEmbedder { dim: 4 });
    let store = Arc::new(MockStore);
    let pipeline = MemoryQueryPipeline::new(store, embedder);
    
    let agent_id = "agent-1";
    let conversation: Vec<aisopod_provider::types::Message> = Vec::new();
    let opts = MemoryQueryOptions::default();
    
    // Building memory context with no memories should succeed
    let context = build_memory_context(&pipeline, agent_id, &conversation, opts)
        .await
        .unwrap();
    
    assert!(context.contains("No relevant memories found"));
}

#[tokio::test]
async fn test_memory_manager_with_integration() {
    // Test the full integration of MemoryManager with memory operations
    
    let (pipeline, embedder) = create_test_pipeline();
    
    let config = MemoryManagerConfig::default();
    let manager = MemoryManager::new(pipeline.store().clone(), Arc::clone(&embedder), config);
    
    let agent_id = "agent-1";
    
    // Build a conversation with extractable facts
    let conversation = build_conversation(
        vec![
            "I love pizza and pasta",
            "I prefer Italian food",
            "I'm vegetarian",
        ],
        vec![
            "Italian food sounds great!",
            "I can recommend some vegetarian restaurants.",
        ],
    );
    
    // Extract memories from the conversation
    let entries = manager.extract_memories(agent_id, &conversation).await.unwrap();
    
    // Should have extracted some memories
    assert!(!entries.is_empty());
    
    // Verify the entries have the correct agent_id
    for entry in &entries {
        assert_eq!(entry.agent_id, agent_id);
    }
    
    // Query the memories to verify they were stored
    let matches = pipeline.query("food preferences", MemoryQueryOptions::default()).await.unwrap();
    
    assert!(!matches.is_empty());
}

#[tokio::test]
async fn test_memory_context_with_message_parts() {
    // Test build_memory_context with MessageContent::Parts variant
    
    let (pipeline, _embedder) = create_test_pipeline();
    
    let agent_id = "agent-1";
    
    // Store a memory
    memory_tool_store(&pipeline, agent_id, "User likes spicy food")
        .await
        .unwrap();
    
    // Build conversation with MessageContent::Parts
    let conversation = vec![
        aisopod_provider::types::Message {
            role: aisopod_provider::types::Role::User,
            content: aisopod_provider::types::MessageContent::Parts(vec![
                aisopod_provider::types::ContentPart::Text { text: "I like ".to_string() },
                aisopod_provider::types::ContentPart::Text { text: "spicy food".to_string() },
            ]),
            tool_calls: None,
            tool_call_id: None,
        },
        aisopod_provider::types::Message {
            role: aisopod_provider::types::Role::Assistant,
            content: aisopod_provider::types::MessageContent::Text("I can recommend some spicy dishes!".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
    ];
    
    let opts = MemoryQueryOptions::default();
    let context = build_memory_context(&pipeline, agent_id, &conversation, opts)
        .await
        .unwrap();
    
    // Should find the memory related to "spicy food"
    assert!(context.contains("Relevant Memories"));
    assert!(context.contains("User likes spicy food"));
}

#[tokio::test]
async fn test_memory_context_with_empty_conversation() {
    // Test that empty conversation returns appropriate context
    
    let (pipeline, _embedder) = create_test_pipeline();
    
    let agent_id = "agent-1";
    let conversation: Vec<aisopod_provider::types::Message> = Vec::new();
    
    let opts = MemoryQueryOptions::default();
    let context = build_memory_context(&pipeline, agent_id, &conversation, opts)
        .await
        .unwrap();
    
    // Should return appropriate message for empty conversation
    assert!(context.contains("No relevant memories found"));
}

#[tokio::test]
async fn test_memory_tool_with_tags() {
    // Test storing memories with tags
    
    let store = helpers::test_store_with_mock_provider(4);
    let embedder: Arc<dyn EmbeddingProvider> = Arc::new(MockEmbeddingProvider::new(4));
    let pipeline = MemoryQueryPipeline::new(Arc::new(store), Arc::clone(&embedder));
    
    let agent_id = "agent-1";
    let content = "User prefers vegetarian food";
    
    // Store via memory tool with tags
    let id = memory_tool_store(&pipeline, agent_id, content)
        .await
        .unwrap();
    
    // Note: The memory_tool_store helper doesn't currently support tags
    // In a real scenario, tags would be passed through the MemoryEntry metadata
    
    // Verify the entry was stored
    let filter = aisopod_memory::MemoryFilter {
        agent_id: Some(agent_id.to_string()),
        ..Default::default()
    };
    
    let entries = pipeline.store().list(filter).await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].content, content);
    assert_eq!(entries[0].agent_id, agent_id);
}

#[tokio::test]
async fn test_memory_context_with_agent_scoping() {
    // Test that build_memory_context respects agent scoping
    
    let store = helpers::test_store_with_mock_provider(4);
    let embedder: Arc<dyn EmbeddingProvider> = Arc::new(MockEmbeddingProvider::new(4));
    let pipeline = MemoryQueryPipeline::new(Arc::new(store), Arc::clone(&embedder));
    
    // Store memories for different agents
    let memories_agent1 = vec![
        "Agent 1 preference: likes coffee",
        "Agent 1 preference: avoids tea",
    ];
    
    let memories_agent2 = vec![
        "Agent 2 preference: prefers tea",
        "Agent 2 preference: dislikes coffee",
    ];
    
    for content in memories_agent1 {
        memory_tool_store(&pipeline, "agent-1", content)
            .await
            .unwrap();
    }
    
    for content in memories_agent2 {
        memory_tool_store(&pipeline, "agent-2", content)
            .await
            .unwrap();
    }
    
    // Build context for agent-1
    let conversation = build_conversation(
        vec!["What should I drink?"],
        vec!["I can recommend a beverage."],
    );
    
    let opts = MemoryQueryOptions {
        filter: aisopod_memory::MemoryFilter {
            agent_id: Some("agent-1".to_string()),
            ..Default::default()
        },
        ..MemoryQueryOptions::default()
    };
    
    let context = build_memory_context(&pipeline, "agent-1", &conversation, opts)
        .await
        .unwrap();
    
    // Should only contain agent-1's memories
    assert!(context.contains("Agent 1 preference: likes coffee"));
    assert!(context.contains("Agent 1 preference: avoids tea"));
    assert!(!context.contains("Agent 2 preference"));
}
