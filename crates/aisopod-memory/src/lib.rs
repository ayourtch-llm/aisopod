//! # aisopod-memory
//!
//! Memory management, context windows, and conversation history storage.
//!
//! This crate provides the foundational types and trait for the aisopod memory system:
//!
//! - Core types: [`MemoryEntry`], [`MemoryMetadata`], [`MemorySource`], [`MemoryMatch`],
//!   [`MemoryFilter`], [`MemoryQueryOptions`]
//! - Trait: [`MemoryStore`] - async trait for memory persistence and retrieval
//!
//! ## Example
//!
//! ```rust
//! use aisopod_memory::{MemoryEntry, MemoryStore, MemoryQueryOptions};
//! use anyhow::Result;
//!
//! async fn example(store: &impl MemoryStore) -> Result<()> {
//!     // Store a memory
//!     let entry = MemoryEntry::new(
//!         "id-1".to_string(),
//!         "agent-1".to_string(),
//!         "test content".to_string(),
//!         vec![0.1, 0.2, 0.3],
//!     );
//!     store.store(entry).await?;
//!
//!     // Query memories
//!     let results = store.query("search query", MemoryQueryOptions::default()).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod types;
pub mod store;

pub use types::*;
pub use store::MemoryStore;
