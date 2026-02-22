//! OpenAI embedding provider implementation.

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use tokio::sync::Mutex;
use tracing::instrument;

/// A hash map with cache entries for embeddings.
type EmbeddingCache = HashMap<u64, Vec<f32>>;

/// Trait for embedding providers that generate vector embeddings from text.
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate an embedding vector for the given text.
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for multiple texts in a batch.
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;

    /// Return the dimensionality of embeddings produced by this provider.
    fn dimensions(&self) -> usize;
}

/// OpenAI embedding provider that generates vector embeddings using the OpenAI API.
pub struct OpenAiEmbeddingProvider {
    api_key: String,
    model: String,
    dimensions: usize,
    client: Client,
    cache: Mutex<EmbeddingCache>,
}

impl OpenAiEmbeddingProvider {
    /// Create a new OpenAI embedding provider with default model and dimensions.
    ///
    /// # Arguments
    /// * `api_key` - OpenAI API key
    /// * `model` - Optional model name, defaults to "text-embedding-3-small"
    /// * `dimensions` - Optional dimensions, defaults to 1536
    pub fn new(api_key: String, model: Option<String>, dimensions: Option<usize>) -> Self {
        let model = model.unwrap_or_else(|| "text-embedding-3-small".to_string());
        let dimensions = dimensions.unwrap_or(1536);
        let client = Client::new();

        Self {
            api_key,
            model,
            dimensions,
            client,
            cache: Mutex::new(EmbeddingCache::new()),
        }
    }

    /// Compute a hash of the input text for cache lookup.
    fn compute_hash(text: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        hasher.finish()
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAiEmbeddingProvider {
    /// Generate an embedding vector for the given text.
    #[instrument(skip(self, text), fields(model = self.model, dimensions = self.dimensions))]
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let hash = Self::compute_hash(text);

        // Check cache first
        {
            let cache = self.cache.lock().await;
            if let Some(embedding) = cache.get(&hash) {
                tracing::debug!("Cache hit for text");
                return Ok(embedding.clone());
            }
        }

        // Build the request body
        let body = serde_json::json!({
            "model": self.model,
            "input": text,
            "dimensions": self.dimensions
        });

        // Make the API call
        let response = self
            .client
            .post("https://api.openai.com/v1/embeddings")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "OpenAI API error (status {}): {}",
                status,
                error_text
            ));
        }

        // Parse the response
        let json: serde_json::Value = response.json().await?;

        // Extract embedding from data[0].embedding
        let embedding: Vec<f32> = {
            let embedding_array = json["data"][0]["embedding"]
                .as_array()
                .ok_or_else(|| anyhow!("Invalid response: missing or invalid embedding array"))?;

            let mut result = Vec::with_capacity(embedding_array.len());
            for v in embedding_array {
                let f = v
                    .as_f64()
                    .ok_or_else(|| anyhow!("Invalid embedding value"))?;
                result.push(f as f32);
            }
            result
        };

        // Store in cache
        {
            let mut cache = self.cache.lock().await;
            cache.insert(hash, embedding.clone());
        }

        Ok(embedding)
    }

    /// Generate embeddings for multiple texts in a batch.
    #[instrument(skip(self, texts), fields(model = self.model, dimensions = self.dimensions))]
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Check cache for each text
        let mut results: Vec<Option<Vec<f32>>> = Vec::with_capacity(texts.len());
        let mut cache_misses: Vec<usize> = Vec::new();
        let mut cache_miss_texts: Vec<String> = Vec::new();

        for (i, text) in texts.iter().enumerate() {
            let hash = Self::compute_hash(text);
            let cache = self.cache.lock().await;
            if let Some(embedding) = cache.get(&hash) {
                results.push(Some(embedding.clone()));
            } else {
                results.push(None);
                cache_misses.push(i);
                cache_miss_texts.push(text.to_string());
            }
        }

        // If all cache hits, return early
        if cache_misses.is_empty() {
            return Ok(results.into_iter().map(|e| e.unwrap()).collect());
        }

        // Build the request body for cache misses
        let body = serde_json::json!({
            "model": self.model,
            "input": cache_miss_texts,
            "dimensions": self.dimensions
        });

        // Make the API call
        let response = self
            .client
            .post("https://api.openai.com/v1/embeddings")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "OpenAI API error (status {}): {}",
                status,
                error_text
            ));
        }

        // Parse the response
        let json: serde_json::Value = response.json().await?;

        // Extract embeddings from data
        let embeddings_array = json["data"].as_array().ok_or_else(|| {
            anyhow!(
                "Invalid response: missing or invalid data array (expected array, got {:?})",
                json["data"]
            )
        })?;

        if embeddings_array.len() != cache_misses.len() {
            return Err(anyhow!(
                "Response mismatch: expected {} embeddings, got {}",
                cache_misses.len(),
                embeddings_array.len()
            ));
        }

        // Store results in cache and build results vector
        let mut cache = self.cache.lock().await;
        for (i, &miss_index) in cache_misses.iter().enumerate() {
            let embedding: Vec<f32> = {
                let embedding_array = embeddings_array[i]["embedding"]
                    .as_array()
                    .ok_or_else(|| anyhow!("Invalid embedding at index {}", i))?;

                let mut result = Vec::with_capacity(embedding_array.len());
                for v in embedding_array {
                    let f = v
                        .as_f64()
                        .ok_or_else(|| anyhow!("Invalid embedding value"))?;
                    result.push(f as f32);
                }
                result
            };

            // Cache this embedding
            let hash = Self::compute_hash(texts[miss_index]);
            cache.insert(hash, embedding.clone());

            // Update results
            results[miss_index] = Some(embedding);
        }

        // Convert Option<Vec<f32>> to Vec<f32>
        Ok(results.into_iter().map(|e| e.unwrap()).collect())
    }

    /// Return the dimensionality of embeddings produced by this provider.
    fn dimensions(&self) -> usize {
        self.dimensions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash() {
        let hash1 = OpenAiEmbeddingProvider::compute_hash("test text");
        let hash2 = OpenAiEmbeddingProvider::compute_hash("test text");
        let hash3 = OpenAiEmbeddingProvider::compute_hash("different text");

        assert_eq!(hash1, hash2, "Same text should produce same hash");
        assert_ne!(hash1, hash3, "Different text should produce different hash");
    }
}
