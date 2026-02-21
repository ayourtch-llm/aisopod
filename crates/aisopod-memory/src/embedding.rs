//! Embedding provider trait and implementations.

pub mod openai;
pub mod mock;

pub use openai::{EmbeddingProvider, OpenAiEmbeddingProvider};
pub use mock::MockEmbeddingProvider;
