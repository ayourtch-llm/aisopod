//! Vector storage and retrieval tests for the aisopod-memory crate.
//!
//! This module tests the core storage operations:
//! - Storing memory entries
//! - Retrieving memories
//! - Deleting memories
//! - Listing memories with various filters

use aisopod_memory::sqlite::SqliteMemoryStore;
use aisopod_memory::{EmbeddingProvider, MockEmbeddingProvider};
use aisopod_memory::{MemoryEntry, MemoryFilter, MemoryMetadata, MemorySource, MemoryStore};
use std::sync::Arc;

// Import the test helpers
mod helpers;

/// Helper function to create a test memory entry
fn make_entry(id: &str, agent_id: &str, content: &str, embedding: &[f32]) -> MemoryEntry {
    MemoryEntry {
        id: id.to_string(),
        agent_id: agent_id.to_string(),
        content: content.to_string(),
        embedding: embedding.to_vec(),
        metadata: MemoryMetadata::default(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

/// Helper function to create a test memory entry with an agent
async fn make_agent_entry(agent_id: &str, content: &str) -> MemoryEntry {
    let embedder = MockEmbeddingProvider::new(4);
    let embedding = embedder.embed(content).await.unwrap();

    MemoryEntry {
        id: uuid::Uuid::new_v4().to_string(),
        agent_id: agent_id.to_string(),
        content: content.to_string(),
        embedding,
        metadata: MemoryMetadata {
            source: MemorySource::Agent,
            tags: vec![],
            importance: 0.5,
            ..Default::default()
        },
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[tokio::test]
async fn test_store_and_retrieve() {
    let store = helpers::test_store_with_mock_provider(4);

    // Create and store a memory entry
    let entry = make_entry(
        "test-id-1",
        "agent-1",
        "Test content for storage",
        &[0.1, 0.2, 0.3, 0.4],
    );

    let id = store.store(entry.clone()).await.unwrap();

    // Retrieve using list with filter
    let filter = MemoryFilter {
        agent_id: Some("agent-1".to_string()),
        ..Default::default()
    };

    let entries = store.list(filter).await.unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].id, id);
    assert_eq!(entries[0].content, "Test content for storage");
    assert_eq!(entries[0].agent_id, "agent-1");
}

#[tokio::test]
async fn test_store_generates_id() {
    let store = helpers::test_store_with_mock_provider(4);

    // Create an entry with empty ID
    let mut entry = make_entry(
        "",
        "agent-1",
        "Content with auto-generated ID",
        &[0.1, 0.2, 0.3, 0.4],
    );
    entry.id = String::new();

    let id = store.store(entry.clone()).await.unwrap();

    // Verify ID was generated (should be a UUID)
    assert!(!id.is_empty());
    assert_eq!(id.len(), 36); // UUID format: 8-4-4-4-12 characters
    assert!(id.parse::<uuid::Uuid>().is_ok());
}

#[tokio::test]
async fn test_delete_entry() {
    let store = helpers::test_store_with_mock_provider(4);

    // Create and store an entry
    let entry = make_entry(
        "delete-id",
        "agent-1",
        "To be deleted",
        &[0.1, 0.2, 0.3, 0.4],
    );
    store.store(entry).await.unwrap();

    // Verify it exists
    let filter = MemoryFilter {
        agent_id: Some("agent-1".to_string()),
        ..Default::default()
    };
    let entries = store.list(filter.clone()).await.unwrap();
    assert_eq!(entries.len(), 1);

    // Delete the entry
    store.delete("delete-id").await.unwrap();

    // Verify it no longer exists
    let entries = store.list(filter).await.unwrap();
    assert!(entries.is_empty());
}

#[tokio::test]
async fn test_delete_nonexistent() {
    let store = helpers::test_store_with_mock_provider(4);

    // Try to delete a non-existent entry - should not error
    store.delete("non-existent-id").await.unwrap();

    // Store an entry to verify store still works
    let entry = make_entry("real-id", "agent-1", "Real content", &[0.1, 0.2, 0.3, 0.4]);
    store.store(entry).await.unwrap();

    let filter = MemoryFilter {
        agent_id: Some("agent-1".to_string()),
        ..Default::default()
    };
    let entries = store.list(filter).await.unwrap();
    assert_eq!(entries.len(), 1);
}

#[tokio::test]
async fn test_list_empty() {
    let store = helpers::test_store_with_mock_provider(4);

    // List from empty store
    let filter = MemoryFilter::default();
    let entries = store.list(filter).await.unwrap();

    assert!(entries.is_empty());
    assert_eq!(entries.len(), 0);
}

#[tokio::test]
async fn test_list_with_agent_filter() {
    let store = helpers::test_store_with_mock_provider(4);

    // Store entries for different agents
    let entry_a1 = make_entry("a1", "agent-A", "Agent A entry 1", &[0.1, 0.2, 0.3, 0.4]);
    let entry_a2 = make_entry("a2", "agent-A", "Agent A entry 2", &[0.2, 0.3, 0.4, 0.5]);
    let entry_b1 = make_entry("b1", "agent-B", "Agent B entry 1", &[0.3, 0.4, 0.5, 0.6]);

    store.store(entry_a1).await.unwrap();
    store.store(entry_a2).await.unwrap();
    store.store(entry_b1).await.unwrap();

    // List with agent-A filter
    let filter = MemoryFilter {
        agent_id: Some("agent-A".to_string()),
        ..Default::default()
    };
    let entries = store.list(filter).await.unwrap();

    assert_eq!(entries.len(), 2);
    assert!(entries.iter().all(|e| e.agent_id == "agent-A"));

    // List with agent-B filter
    let filter = MemoryFilter {
        agent_id: Some("agent-B".to_string()),
        ..Default::default()
    };
    let entries = store.list(filter).await.unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].agent_id, "agent-B");

    // List with non-existent agent
    let filter = MemoryFilter {
        agent_id: Some("agent-C".to_string()),
        ..Default::default()
    };
    let entries = store.list(filter).await.unwrap();

    assert!(entries.is_empty());
}

#[tokio::test]
async fn test_list_with_tag_filter() {
    let store = helpers::test_store_with_mock_provider(4);

    // Store entries with different tags
    let entry_tags1 = MemoryEntry {
        id: "tags-1".to_string(),
        agent_id: "agent-1".to_string(),
        content: "Tagged entry".to_string(),
        embedding: vec![0.1, 0.2, 0.3, 0.4],
        metadata: MemoryMetadata {
            source: MemorySource::Agent,
            tags: vec!["important".to_string(), "reviewed".to_string()],
            importance: 0.5,
            ..Default::default()
        },
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let entry_notags = make_entry("no-tags", "agent-1", "No tags entry", &[0.2, 0.3, 0.4, 0.5]);

    let entry_tag2 = MemoryEntry {
        id: "tags-2".to_string(),
        agent_id: "agent-1".to_string(),
        content: "Tagged entry 2".to_string(),
        embedding: vec![0.3, 0.4, 0.5, 0.6],
        metadata: MemoryMetadata {
            source: MemorySource::Agent,
            tags: vec!["draft".to_string()],
            importance: 0.5,
            ..Default::default()
        },
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    store.store(entry_tags1).await.unwrap();
    store.store(entry_notags).await.unwrap();
    store.store(entry_tag2).await.unwrap();

    // List with "important" tag filter
    let filter = MemoryFilter {
        tags: Some(vec!["important".to_string()]),
        ..Default::default()
    };
    let entries = store.list(filter).await.unwrap();

    // First check what entries are in the database
    let all_filter = MemoryFilter {
        ..Default::default()
    };
    let all_entries = store.list(all_filter).await.unwrap();
    eprintln!("All entries in database: {}", all_entries.len());
    for entry in &all_entries {
        eprintln!(
            "  Entry: id={}, content={}, tags={:?}",
            entry.id, entry.content, entry.metadata.tags
        );
    }

    eprintln!("Found {} entries with 'important' tag", entries.len());
    for entry in &entries {
        eprintln!(
            "  Entry: id={}, content={}, tags={:?}",
            entry.id, entry.content, entry.metadata.tags
        );
    }

    assert_eq!(entries.len(), 1);
    assert!(entries[0].content.contains("Tagged entry"));

    // List with "reviewed" tag filter (should also find the tagged entry)
    let filter = MemoryFilter {
        tags: Some(vec!["reviewed".to_string()]),
        ..Default::default()
    };
    let entries = store.list(filter).await.unwrap();

    assert_eq!(entries.len(), 1);
}

#[tokio::test]
async fn test_list_with_importance_filter() {
    let store = helpers::test_store_with_mock_provider(4);

    // Create entries with different importance levels
    let entry_low = make_entry("low", "agent-1", "Low importance", &[0.1, 0.2, 0.3, 0.4]);
    let mut entry_low = entry_low;
    entry_low.metadata.importance = 0.1;

    let entry_mid = make_entry("mid", "agent-1", "Medium importance", &[0.2, 0.3, 0.4, 0.5]);
    let mut entry_mid = entry_mid;
    entry_mid.metadata.importance = 0.5;

    let entry_high = make_entry("high", "agent-1", "High importance", &[0.3, 0.4, 0.5, 0.6]);
    let mut entry_high = entry_high;
    entry_high.metadata.importance = 0.9;

    store.store(entry_low).await.unwrap();
    store.store(entry_mid).await.unwrap();
    store.store(entry_high).await.unwrap();

    // List with minimum importance 0.5
    let filter = MemoryFilter {
        importance_min: Some(0.5),
        ..Default::default()
    };
    let entries = store.list(filter).await.unwrap();

    assert_eq!(entries.len(), 2);
    let contents: Vec<&str> = entries.iter().map(|e| e.content.as_str()).collect();
    assert!(contents.contains(&"Medium importance"));
    assert!(contents.contains(&"High importance"));

    // List with minimum importance 0.8
    let filter = MemoryFilter {
        importance_min: Some(0.8),
        ..Default::default()
    };
    let entries = store.list(filter).await.unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].content, "High importance");

    // List with minimum importance 0.0 (all entries)
    let filter = MemoryFilter {
        importance_min: Some(0.0),
        ..Default::default()
    };
    let entries = store.list(filter).await.unwrap();

    assert_eq!(entries.len(), 3);
}

#[tokio::test]
async fn test_store_overwrites_existing() {
    let store = helpers::test_store_with_mock_provider(4);

    // Create and store an entry
    let entry1 = make_entry(
        "same-id",
        "agent-1",
        "Original content",
        &[0.1, 0.2, 0.3, 0.4],
    );
    store.store(entry1).await.unwrap();

    // Verify the entry
    let filter = MemoryFilter {
        agent_id: Some("agent-1".to_string()),
        ..Default::default()
    };
    let entries = store.list(filter.clone()).await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].content, "Original content");

    // Store a new entry with the same ID (should update)
    let entry2 = make_entry(
        "same-id",
        "agent-1",
        "Updated content",
        &[0.5, 0.6, 0.7, 0.8],
    );
    store.store(entry2).await.unwrap();

    // Verify the entry was updated
    let entries = store.list(filter).await.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].content, "Updated content");
    assert_eq!(entries[0].embedding, vec![0.5, 0.6, 0.7, 0.8]);
}
