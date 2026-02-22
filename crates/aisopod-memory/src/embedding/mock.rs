//! Mock embedding provider for testing.
//!
//! This module provides a `MockEmbeddingProvider` that returns deterministic
//! embeddings based on a hash of the input text, normalized to a unit vector.
//! This allows tests to run without requiring real API calls or API keys.

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::{Arc, Mutex};

use crate::EmbeddingProvider;

/// A mock embedding provider that generates deterministic embeddings.
///
/// The embedding is computed as:
/// 1. Hash the input text
/// 2. Create a vector where each dimension is derived from the hash
/// 3. Normalize to a unit vector
///
/// This ensures the same text always produces the same embedding, while
/// different texts produce different embeddings.
pub struct MockEmbeddingProvider {
    /// Dimensionality of the generated embeddings
    dimensions: usize,
    /// Cache of already computed embeddings for performance
    cache: Arc<Mutex<HashMap<String, Vec<f32>>>>,
}

impl MockEmbeddingProvider {
    /// Creates a new `MockEmbeddingProvider` with the specified embedding dimension.
    ///
    /// # Arguments
    /// * `dimensions` - The dimensionality of embeddings to generate
    pub fn new(dimensions: usize) -> Self {
        Self {
            dimensions,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Computes a hash of the input text.
    fn compute_hash(text: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        hasher.finish()
    }

    /// Generates a deterministic embedding vector from text.
    ///
    /// The embedding is created by:
    /// 1. Using the hash to generate pseudo-random values for each dimension
    /// 2. Normalizing the vector to unit length
    fn generate_embedding(dimensions: usize, text: &str) -> Vec<f32> {
        let hash = Self::compute_hash(text);
        let mut embedding = Vec::with_capacity(dimensions);

        // Use the hash to generate values for each dimension
        // We use a simple hash-based approach that ensures determinism
        let mut current_hash = hash;
        for _ in 0..dimensions {
            // Generate a value in range [-1, 1] from the hash
            let value = ((current_hash % 1000) as f32 - 500.0) / 500.0;
            embedding.push(value);
            // Mix the hash for the next dimension
            current_hash = current_hash
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1);
        }

        // Normalize to unit vector
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for x in embedding.iter_mut() {
                *x /= magnitude;
            }
        }

        embedding
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Check cache first
        {
            let cache = self.cache.lock().unwrap();
            if let Some(embedding) = cache.get(text) {
                return Ok(embedding.clone());
            }
        }

        // Generate the embedding
        let embedding = Self::generate_embedding(self.dimensions, text);

        // Cache it
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(text.to_string(), embedding.clone());
        }

        Ok(embedding)
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            let embedding = self.embed(text).await?;
            results.push(embedding);
        }
        Ok(results)
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }
}

impl Default for MockEmbeddingProvider {
    fn default() -> Self {
        Self::new(1536)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_provider_new() {
        let provider = MockEmbeddingProvider::new(4);
        assert_eq!(provider.dimensions(), 4);
    }

    #[tokio::test]
    async fn test_mock_provider_deterministic() {
        let mut provider = MockEmbeddingProvider::new(4);
        let text = "test text";

        let embedding1 = provider.embed(text).await.unwrap();
        let embedding2 = provider.embed(text).await.unwrap();

        assert_eq!(embedding1, embedding2);
    }

    #[tokio::test]
    async fn test_mock_provider_different_texts() {
        let mut provider = MockEmbeddingProvider::new(4);

        let emb1 = provider.embed("text one").await.unwrap();
        let emb2 = provider.embed("text two").await.unwrap();

        // Different texts should produce different embeddings
        assert_ne!(emb1, emb2);
    }

    #[tokio::test]
    async fn test_mock_provider_unit_vector() {
        let mut provider = MockEmbeddingProvider::new(4);
        let embedding = provider.embed("test").await.unwrap();

        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!(
            (magnitude - 1.0).abs() < 0.0001,
            "Embedding should be normalized to unit vector"
        );
    }

    #[tokio::test]
    async fn test_mock_provider_different_dimensions() {
        let mut provider_4 = MockEmbeddingProvider::new(4);
        let mut provider_8 = MockEmbeddingProvider::new(8);

        let emb4 = provider_4.embed("test").await.unwrap();
        let emb8 = provider_8.embed("test").await.unwrap();

        assert_eq!(emb4.len(), 4);
        assert_eq!(emb8.len(), 8);
    }

    #[tokio::test]
    async fn test_mock_provider_cache() {
        let mut provider = MockEmbeddingProvider::new(4);
        let text = "cached text";

        // First call
        let embedding1 = provider.embed(text).await.unwrap();

        // Modify the cache
        let mut new_embedding = embedding1.clone();
        new_embedding[0] = new_embedding[0] + 0.1;

        // Second call should return the cached original
        let embedding2 = provider.embed(text).await.unwrap();

        assert_eq!(embedding1, embedding2);
    }
}
