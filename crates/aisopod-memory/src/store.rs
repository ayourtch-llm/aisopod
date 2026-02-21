//! MemoryStore trait for memory persistence and retrieval.
//!
//! This module defines the core trait that all memory backends must implement,
//! providing a standardized interface for storing, querying, and managing memories.

use crate::types::{MemoryEntry, MemoryFilter, MemoryMatch, MemoryQueryOptions};
use anyhow::Result;

/// Trait for memory storage and retrieval operations.
///
/// All memory backends (SQLite-Vec, LanceDB, etc.) must implement this trait
/// to provide a consistent interface for memory operations.
#[async_trait::async_trait]
pub trait MemoryStore: Send + Sync {
    /// Stores a memory entry and returns its assigned ID.
    ///
    /// If the entry has an empty ID, a new unique ID will be generated.
    /// If the entry already exists, it will be updated.
    ///
    /// # Arguments
    /// * `entry` - The memory entry to store
    ///
    /// # Returns
    /// Returns the ID of the stored entry on success.
    ///
    /// # Errors
    /// Returns an error if storage fails (e.g., database error, serialization error).
    async fn store(&self, entry: MemoryEntry) -> Result<String>;

    /// Performs a semantic search for matching memories.
    ///
    /// Executes a query against the memory store using the provided query string
    /// and options. The query is typically converted to an embedding for semantic
    /// similarity matching.
    ///
    /// # Arguments
    /// * `query` - The query string to search for
    /// * `opts` - Query options including filter criteria and result limits
    ///
    /// # Returns
    /// Returns a list of memory matches sorted by relevance score (descending),
    /// limited to `top_k` results.
    ///
    /// # Errors
    /// Returns an error if the query fails (e.g., invalid query, database error).
    async fn query(&self, query: &str, opts: MemoryQueryOptions) -> Result<Vec<MemoryMatch>>;

    /// Deletes a memory entry by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the memory to delete
    ///
    /// # Returns
    /// Returns Ok(()) if deletion was successful.
    ///
    /// # Errors
    /// Returns an error if deletion fails (e.g., database error).
    async fn delete(&self, id: &str) -> Result<()>;

    /// Lists memory entries matching the given filter.
    ///
    /// Returns all memories that satisfy the filter criteria, without
    /// any semantic ranking (unlike `query`).
    ///
    /// # Arguments
    /// * `filter` - Filter criteria to apply
    ///
    /// # Returns
    /// Returns a list of matching memory entries.
    ///
    /// # Errors
    /// Returns an error if listing fails (e.g., database error).
    async fn list(&self, filter: MemoryFilter) -> Result<Vec<MemoryEntry>>;
}
