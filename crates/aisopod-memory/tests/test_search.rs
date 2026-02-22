//! Similarity search tests for the aisopod-memory crate.
//!
//! This module tests vector similarity search functionality:
//! - Returns closest matches
//! - Top-K limiting
//! - Min-score threshold filtering
//! - Agent scoping

use aisopod_memory::sqlite::SqliteMemoryStore;
use aisopod_memory::MockEmbeddingProvider;
use aisopod_memory::{
    EmbeddingProvider, MemoryEntry, MemoryFilter, MemoryMetadata, MemoryQueryOptions, MemorySource,
    MemoryStore,
};
use std::sync::Arc;

// Import the test helpers
mod helpers;

/// Helper function to create a memory entry with a specific embedding
fn make_entry_with_embedding(
    id: &str,
    agent_id: &str,
    content: &str,
    embedding: Vec<f32>,
) -> MemoryEntry {
    MemoryEntry {
        id: id.to_string(),
        agent_id: agent_id.to_string(),
        content: content.to_string(),
        embedding,
        metadata: MemoryMetadata {
            source: MemorySource::Agent,
            importance: 0.5,
            ..Default::default()
        },
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

#[tokio::test]
async fn test_similarity_search_returns_closest() {
    let store = helpers::test_store_with_mock_provider(4);

    // Store entries with known embeddings
    // Entry 1: [0.7, 0.7, 0.0, 0.0] (normalized)
    let entry1 = make_entry_with_embedding(
        "entry-1",
        "agent-1",
        "Vector close to query",
        vec![0.707, 0.707, 0.0, 0.0], // ~45 degrees
    );

    // Entry 2: [0.0, 0.0, 0.7, 0.7] (normalized)
    let entry2 = make_entry_with_embedding(
        "entry-2",
        "agent-1",
        "Vector far from query",
        vec![0.0, 0.0, 0.707, 0.707], // ~45 degrees on other axis
    );

    store.store(entry1).await.unwrap();
    store.store(entry2).await.unwrap();

    // Query with vector close to entry1
    let opts = MemoryQueryOptions {
        top_k: 10,
        filter: MemoryFilter {
            agent_id: Some("agent-1".to_string()),
            ..Default::default()
        },
        min_score: None,
    };

    let results = store.query("Vector close to query", opts).await.unwrap();

    // Entry 1 should be ranked higher (closer to query)
    assert!(!results.is_empty());
    assert_eq!(results[0].entry.id, "entry-1");
    assert!(results[0].score > results[1].score);
}

#[tokio::test]
async fn test_similarity_search_top_k() {
    let store = helpers::test_store_with_mock_provider(4);

    // Store 20 entries with different embeddings
    for i in 0..20 {
        let content = format!("Content entry {}", i);
        let embedder = MockEmbeddingProvider::new(4);
        let embedding = embedder.embed(&content).await.unwrap();

        let entry = MemoryEntry {
            id: format!("entry-{}", i),
            agent_id: "agent-1".to_string(),
            content,
            embedding,
            metadata: MemoryMetadata {
                source: MemorySource::Agent,
                importance: 0.5,
                ..Default::default()
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        store.store(entry).await.unwrap();
    }

    // Query with top_k=5
    let opts = MemoryQueryOptions {
        top_k: 5,
        filter: MemoryFilter {
            agent_id: Some("agent-1".to_string()),
            ..Default::default()
        },
        min_score: None,
    };

    let results = store.query("test query", opts).await.unwrap();

    assert_eq!(results.len(), 5);
}

#[tokio::test]
async fn test_similarity_search_min_score() {
    let store = helpers::test_store_with_mock_provider(4);

    // Store several entries
    for i in 0..10 {
        let content = format!("Content entry {}", i);
        let embedder = MockEmbeddingProvider::new(4);
        let embedding = embedder.embed(&content).await.unwrap();

        let entry = MemoryEntry {
            id: format!("entry-{}", i),
            agent_id: "agent-1".to_string(),
            content,
            embedding,
            metadata: MemoryMetadata {
                source: MemorySource::Agent,
                importance: 0.5,
                ..Default::default()
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        store.store(entry).await.unwrap();
    }

    // Query with a high min_score threshold
    // Since all entries are generated with the same embedder,
    // some should have low similarity scores
    let opts = MemoryQueryOptions {
        top_k: 10,
        filter: MemoryFilter {
            agent_id: Some("agent-1".to_string()),
            ..Default::default()
        },
        min_score: Some(0.95), // High threshold
    };

    let results = store.query("test query", opts).await.unwrap();

    // All results should meet the minimum score
    for result in &results {
        assert!(result.score >= 0.95);
    }

    // Query with a low min_score threshold
    let opts = MemoryQueryOptions {
        top_k: 10,
        filter: MemoryFilter {
            agent_id: Some("agent-1".to_string()),
            ..Default::default()
        },
        min_score: Some(0.0), // Low threshold
    };

    let results = store.query("test query", opts).await.unwrap();

    // Should return more results
    assert!(!results.is_empty());
}

#[tokio::test]
async fn test_similarity_search_agent_scoped() {
    let store = helpers::test_store_with_mock_provider(4);

    // Store entries for different agents
    for i in 0..10 {
        let agent_id = if i < 5 { "agent-A" } else { "agent-B" };
        let content = format!("Agent {} content {}", agent_id, i);
        let embedder = MockEmbeddingProvider::new(4);
        let embedding = embedder.embed(&content).await.unwrap();

        let entry = MemoryEntry {
            id: format!("entry-{}", i),
            agent_id: agent_id.to_string(),
            content,
            embedding,
            metadata: MemoryMetadata {
                source: MemorySource::Agent,
                importance: 0.5,
                ..Default::default()
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        store.store(entry).await.unwrap();
    }

    // Query scoped to agent-A
    let opts = MemoryQueryOptions {
        top_k: 10,
        filter: MemoryFilter {
            agent_id: Some("agent-A".to_string()),
            ..Default::default()
        },
        min_score: None,
    };

    let results = store.query("test query", opts).await.unwrap();

    // All results should be from agent-A
    for result in &results {
        assert_eq!(result.entry.agent_id, "agent-A");
    }
    assert_eq!(results.len(), 5); // Should get all 5 agent-A entries

    // Query scoped to agent-B
    let opts = MemoryQueryOptions {
        top_k: 10,
        filter: MemoryFilter {
            agent_id: Some("agent-B".to_string()),
            ..Default::default()
        },
        min_score: None,
    };

    let results = store.query("test query", opts).await.unwrap();

    // All results should be from agent-B
    for result in &results {
        assert_eq!(result.entry.agent_id, "agent-B");
    }
    assert_eq!(results.len(), 5); // Should get all 5 agent-B entries

    // Query scoped to non-existent agent
    let opts = MemoryQueryOptions {
        top_k: 10,
        filter: MemoryFilter {
            agent_id: Some("agent-C".to_string()),
            ..Default::default()
        },
        min_score: None,
    };

    let results = store.query("test query", opts).await.unwrap();

    // Should return no results
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_similarity_search_ranking() {
    let store = helpers::test_store_with_mock_provider(4);

    // Store entries where we can control the embeddings
    // Create entries with different similarities to a query

    // Entry with very similar embedding
    let entry_similar = make_entry_with_embedding(
        "similar",
        "agent-1",
        "Very similar to query",
        vec![0.5, 0.5, 0.5, 0.5],
    );

    // Entry with moderately similar embedding
    let entry_moderate = make_entry_with_embedding(
        "moderate",
        "agent-1",
        "Moderately similar to query",
        vec![0.5, 0.5, -0.5, -0.5], // Some opposition
    );

    // Entry with dissimilar embedding
    let entry_dissimilar = make_entry_with_embedding(
        "dissimilar",
        "agent-1",
        "Not similar to query",
        vec![-0.5, -0.5, 0.5, 0.5], // Opposite direction
    );

    store.store(entry_similar).await.unwrap();
    store.store(entry_moderate).await.unwrap();
    store.store(entry_dissimilar).await.unwrap();

    // Query
    let opts = MemoryQueryOptions {
        top_k: 10,
        filter: MemoryFilter {
            agent_id: Some("agent-1".to_string()),
            ..Default::default()
        },
        min_score: None,
    };

    let results = store.query("test query", opts).await.unwrap();

    // Results should be ranked by similarity
    assert_eq!(results.len(), 3);

    // First result should be the most similar
    let first_score = results[0].score;
    let second_score = results[1].score;
    let third_score = results[2].score;

    assert!(first_score >= second_score);
    assert!(second_score >= third_score);
}

#[tokio::test]
async fn test_similarity_search_empty_store() {
    let store = helpers::test_store_with_mock_provider(4);

    // Query an empty store
    let opts = MemoryQueryOptions {
        top_k: 10,
        filter: MemoryFilter::default(),
        min_score: None,
    };

    let results = store.query("test query", opts).await.unwrap();

    assert!(results.is_empty());
}

#[tokio::test]
async fn test_similarity_search_with_importance_filter() {
    let store = helpers::test_store_with_mock_provider(4);

    // Store entries with different importance levels
    let entry_low =
        make_entry_with_embedding("low", "agent-1", "Low importance", vec![0.5, 0.5, 0.5, 0.5]);
    let mut entry_low = entry_low;
    entry_low.metadata.importance = 0.1;

    let entry_high = make_entry_with_embedding(
        "high",
        "agent-1",
        "High importance",
        vec![0.5, 0.5, 0.5, 0.5], // Same embedding
    );
    let mut entry_high = entry_high;
    entry_high.metadata.importance = 0.9;

    store.store(entry_low).await.unwrap();
    store.store(entry_high).await.unwrap();

    // Query with min_score=0 (should return both)
    let opts = MemoryQueryOptions {
        top_k: 10,
        filter: MemoryFilter {
            agent_id: Some("agent-1".to_string()),
            ..Default::default()
        },
        min_score: Some(0.0),
    };

    let results = store.query("test query", opts).await.unwrap();

    // Both should be returned (re-ranked by combined score)
    assert_eq!(results.len(), 2);
}
