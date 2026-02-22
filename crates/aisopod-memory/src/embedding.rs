//! Embedding provider trait and implementations.

pub mod mock;
pub mod openai;

pub use mock::MockEmbeddingProvider;
pub use openai::{EmbeddingProvider, OpenAiEmbeddingProvider};
