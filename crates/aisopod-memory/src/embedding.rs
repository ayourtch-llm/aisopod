//! Embedding provider trait and implementations.

pub mod openai;
#[cfg(test)]
pub mod mock;

pub use openai::{EmbeddingProvider, OpenAiEmbeddingProvider};

#[cfg(test)]
pub use mock::MockEmbeddingProvider;
