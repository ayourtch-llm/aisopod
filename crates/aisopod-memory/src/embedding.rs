//! Embedding provider trait and implementations.

pub mod openai;

pub use openai::{EmbeddingProvider, OpenAiEmbeddingProvider};
