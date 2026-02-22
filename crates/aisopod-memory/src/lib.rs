//! # aisopod-memory
//!
//! Memory management, context windows, and conversation history storage.
//!
//! This crate provides the foundational types and trait for the aisopod memory system:
//!
//! - Core types: [`MemoryEntry`], [`MemoryMetadata`], [`MemorySource`], [`MemoryMatch`],
//!   [`MemoryFilter`], [`MemoryQueryOptions`]
//! - Trait: [`MemoryStore`] - async trait for memory persistence and retrieval
//! - Pipeline: [`MemoryQueryPipeline`] - end-to-end memory query orchestration
//! - Management: [`MemoryManager`] - automatic memory lifecycle management
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

pub mod embedding;
pub mod integration;
pub mod management;
pub mod pipeline;
pub mod sqlite;
pub mod store;
pub mod types;

#[cfg(feature = "lancedb")]
pub mod lancedb;

pub use embedding::MockEmbeddingProvider;
pub use embedding::{EmbeddingProvider, OpenAiEmbeddingProvider};
pub use integration::build_memory_context;
pub use management::{MemoryManager, MemoryManagerConfig};
pub use pipeline::MemoryQueryPipeline;
pub use store::MemoryStore;
pub use types::*;

#[cfg(feature = "lancedb")]
pub use lancedb::LanceDbMemoryStore;
