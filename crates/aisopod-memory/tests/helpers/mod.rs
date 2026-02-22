//! Test helpers for the aisopod-memory crate.
//!
//! This module provides shared utilities for testing:
//! - `test_store()`: Creates an in-memory SQLite store
//! - `MockEmbeddingProvider`: Creates deterministic embeddings for testing

use std::sync::Arc;

use aisopod_memory::sqlite::SqliteMemoryStore;
use aisopod_memory::{EmbeddingProvider, MockEmbeddingProvider};
use anyhow::Result;

/// Creates a new in-memory SQLite memory store for testing.
///
/// This helper function opens a new SQLite database in memory (`:memory:`)
/// with the specified embedding dimension, ready for testing.
///
/// # Arguments
/// * `embedding_dim` - The dimensionality of embeddings to store (default: 4)
///
/// # Returns
/// Returns a new `SqliteMemoryStore` configured for testing.
///
/// # Example
/// ```ignore
/// let store = test_store(4);
/// // Use the store in tests...
/// ```
pub fn test_store(embedding_dim: usize) -> SqliteMemoryStore {
    SqliteMemoryStore::new(":memory:", embedding_dim).expect("Failed to create test store")
}

/// Creates a new in-memory SQLite memory store with a custom embedding provider.
///
/// # Arguments
/// * `embedding_dim` - The dimensionality of embeddings to store
/// * `embedder` - The embedding provider to use
///
/// # Returns
/// Returns a new `SqliteMemoryStore` with the specified embedder.
pub fn test_store_with_embedder(
    embedding_dim: usize,
    embedder: Arc<dyn EmbeddingProvider>,
) -> SqliteMemoryStore {
    SqliteMemoryStore::new_with_embedder(":memory:", embedding_dim, embedder)
        .expect("Failed to create test store with embedder")
}

/// Creates a new in-memory SQLite memory store with a MockEmbeddingProvider.
///
/// # Arguments
/// * `embedding_dim` - The dimensionality of embeddings to store
///
/// # Returns
/// Returns a new `SqliteMemoryStore` with a default MockEmbeddingProvider.
pub fn test_store_with_mock_provider(embedding_dim: usize) -> SqliteMemoryStore {
    let embedder: Arc<dyn EmbeddingProvider> = Arc::new(MockEmbeddingProvider::new(embedding_dim));
    SqliteMemoryStore::new_with_embedder(":memory:", embedding_dim, embedder)
        .expect("Failed to create test store with mock provider")
}
