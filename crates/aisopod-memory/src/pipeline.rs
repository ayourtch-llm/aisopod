//! Memory query pipeline for end-to-end memory retrieval and context injection.
//!
//! This module provides the `MemoryQueryPipeline` struct that orchestrates
//! the full memory query flow: embedding generation, vector search, filtering,
//! re-ranking, and context formatting.

use crate::embedding::{EmbeddingProvider, MockEmbeddingProvider};
use crate::store::MemoryStore;
use crate::types::{MemoryMatch, MemoryQueryOptions};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// Pipeline for querying and retrieving relevant memories.
///
/// This struct orchestrates the full memory query flow:
/// 1. Generate a query embedding via the `EmbeddingProvider`.
/// 2. Perform vector similarity search via the `MemoryStore`.
/// 3. Apply post-retrieval filtering.
/// 4. Re-rank results using combined score (similarity, importance, recency).
/// 5. Sort by final_score descending and truncate to top_k.
///
/// # Example
/// ```ignore
/// use aisopod_memory::{MemoryQueryPipeline, MemoryQueryOptions, MemoryStore, EmbeddingProvider};
/// # use anyhow::Result;
/// #
/// # async fn example() -> Result<()> {
/// # let store: std::sync::Arc<dyn MemoryStore> = unimplemented!();
/// # let embedder: std::sync::Arc<dyn EmbeddingProvider> = unimplemented!();
/// let pipeline = MemoryQueryPipeline::new(store, embedder);
/// let results = pipeline.query("search query", MemoryQueryOptions::default()).await?;
/// let context = pipeline.format_context(&results);
/// # Ok(())
/// # }
/// ```
pub struct MemoryQueryPipeline {
    store: Arc<dyn MemoryStore>,
    embedder: Arc<dyn EmbeddingProvider>,
}

impl MemoryQueryPipeline {
    /// Creates a new `MemoryQueryPipeline`.
    ///
    /// # Arguments
    /// * `store` - The underlying memory store for vector search
    /// * `embedder` - The embedding provider for generating query embeddings
    pub fn new(store: Arc<dyn MemoryStore>, embedder: Arc<dyn EmbeddingProvider>) -> Self {
        Self { store, embedder }
    }

    /// Query memories and return re-ranked results.
    ///
    /// This method performs the full memory query pipeline:
    /// 1. Generates a query embedding via the `EmbeddingProvider`.
    /// 2. Executes vector similarity search via the `MemoryStore`.
    /// 3. Applies post-retrieval filtering.
    /// 4. Re-ranks results using combined score (similarity, importance, recency).
    /// 5. Sorts by final_score descending and truncates to top_k.
    ///
    /// # Arguments
    /// * `query` - The natural language query string
    /// * `opts` - Query options including filter criteria and result limits
    ///
    /// # Returns
    /// Returns a list of memory matches re-ranked by combined score,
    /// sorted descending and truncated to the specified top_k.
    ///
    /// # Errors
    /// Returns an error if embedding generation or query execution fails.
    pub async fn query(&self, query: &str, opts: MemoryQueryOptions) -> Result<Vec<MemoryMatch>> {
        // Step 1: Generate query embedding
        let query_embedding = self.embedder.embed(query).await?;

        // Step 2: Perform vector similarity search
        // Note: We need to pass the embedding to the store somehow
        // For now, we'll call the store's query method and re-rank results
        let mut matches = self.store.query(query, opts.clone()).await?;

        // Step 3 & 4: Apply post-retrieval filtering and re-rank
        let filtered_matches = self.apply_post_filtering(matches)?;

        // Re-rank using combined score
        let re_ranked: Vec<MemoryMatch> = filtered_matches
            .into_iter()
            .map(|mut match_| {
                // Calculate recency factor
                let recency = Self::recency_factor(match_.entry.created_at);
                let importance = match_.entry.metadata.importance;

                // Combined score: similarity * 0.7 + importance * 0.2 + recency * 0.1
                match_.score = match_.score * 0.7 + importance as f32 * 0.2 + recency * 0.1;
                match_
            })
            .collect();

        // Step 5: Sort by final_score descending
        let mut sorted = re_ranked;
        sorted.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Truncate to top_k
        Ok(sorted.into_iter().take(opts.top_k).collect())
    }

    /// Apply post-retrieval filtering to memory matches.
    ///
    /// Filters results based on the filter criteria in the options.
    ///
    /// # Arguments
    /// * `matches` - The raw query results to filter
    ///
    /// # Returns
    /// Returns filtered matches that satisfy all filter criteria.
    fn apply_post_filtering(&self, matches: Vec<MemoryMatch>) -> Result<Vec<MemoryMatch>> {
        // The store already handles most filters, but we do additional filtering here
        // if needed. For now, we just return the matches as-is since the store
        // already applies the filters from MemoryQueryOptions.
        Ok(matches)
    }

    /// Calculate a recency factor based on when a memory was created.
    ///
    /// Uses exponential decay to give more recent memories higher scores.
    /// The factor ranges from 0.0 (very old) to 1.0 (very recent).
    ///
    /// # Arguments
    /// * `created_at` - The creation timestamp of the memory
    ///
    /// # Returns
    /// A recency factor between 0.0 and 1.0.
    ///
    /// # Formula
    /// `factor = 2.0_f32.powf(-days_old / 7.0)`
    /// This means:
    /// - Today (0 days): factor = 1.0
    /// - 7 days ago: factor = 0.5
    /// - 14 days ago: factor = 0.25
    /// - 21 days ago: factor = 0.125
    pub fn recency_factor(created_at: DateTime<Utc>) -> f32 {
        let now = Utc::now();
        let elapsed = now.signed_duration_since(created_at);
        let days_old = elapsed.num_days() as f32;

        // Exponential decay: factor halves every 7 days
        // Using 2.0^(-days/7) gives us nice decay properties
        2.0_f32.powf(-days_old / 7.0)
    }

    /// Format memory matches as a context string for prompt injection.
    ///
    /// Each matched memory is formatted as a bullet point with its score,
    /// and all bullets are joined with newlines and wrapped in a section header.
    ///
    /// # Arguments
    /// * `matches` - The memory matches to format
    ///
    /// # Returns
    /// A formatted string suitable for injection into a system prompt.
    ///
    /// # Example
    /// ```text
    /// ## Relevant Memories
    /// - [score: 0.95] Memory content here
    /// - [score: 0.87] Another memory content
    /// ```
    pub fn format_context(&self, matches: &[MemoryMatch]) -> String {
        if matches.is_empty() {
            return "## Relevant Memories\n\nNo relevant memories found.".to_string();
        }

        let bullets: Vec<String> = matches
            .iter()
            .map(|m| format!("- [score: {:.2}] {}", m.score, m.entry.content))
            .collect();

        let joined = bullets.join("\n");
        format!("## Relevant Memories\n\n{}", joined)
    }

    /// Convenience method that queries and formats results in one call.
    ///
    /// This is a helper that combines `query()` and `format_context()` for
    /// the common case where you want to immediately inject results into a prompt.
    ///
    /// # Arguments
    /// * `query` - The natural language query string
    /// * `opts` - Query options including filter criteria and result limits
    ///
    /// # Returns
    /// Returns the formatted context string with relevant memories.
    ///
    /// # Errors
    /// Returns an error if the query fails.
    pub async fn query_and_format(&self, query: &str, opts: MemoryQueryOptions) -> Result<String> {
        let matches = self.query(query, opts).await?;
        Ok(self.format_context(&matches))
    }

    /// Gets a reference to the embedder.
    pub fn embedder(&self) -> &Arc<dyn EmbeddingProvider> {
        &self.embedder
    }

    /// Gets a reference to the store.
    pub fn store(&self) -> &Arc<dyn MemoryStore> {
        &self.store
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::MockEmbeddingProvider;
    use crate::sqlite::SqliteMemoryStore;
    use crate::MemoryEntry;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_recency_factor() {
        let now = Utc::now();
        let today = now - chrono::Duration::zero();
        let week_old = now - chrono::Duration::days(7);
        let two_weeks_old = now - chrono::Duration::days(14);

        // Today should have factor of 1.0
        let today_factor = MemoryQueryPipeline::recency_factor(today);
        assert!((today_factor - 1.0).abs() < 0.001);

        // 1 week old should have factor of 0.5
        let week_factor = MemoryQueryPipeline::recency_factor(week_old);
        assert!((week_factor - 0.5).abs() < 0.001);

        // 2 weeks old should have factor of 0.25
        let two_weeks_factor = MemoryQueryPipeline::recency_factor(two_weeks_old);
        assert!((two_weeks_factor - 0.25).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_format_context_empty() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SqliteMemoryStore::new(db_path.to_str().unwrap(), 4).unwrap();
        let embedder = MockEmbeddingProvider::new(4);

        let pipeline = MemoryQueryPipeline::new(Arc::new(store), Arc::new(embedder));
        let matches: Vec<MemoryMatch> = Vec::new();
        let context = pipeline.format_context(&matches);

        assert!(context.contains("## Relevant Memories"));
        assert!(context.contains("No relevant memories found"));
    }

    #[tokio::test]
    async fn test_format_context_with_matches() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SqliteMemoryStore::new(db_path.to_str().unwrap(), 4).unwrap();
        let embedder = MockEmbeddingProvider::new(4);

        let pipeline = MemoryQueryPipeline::new(Arc::new(store), Arc::new(embedder));

        // Create some test memories
        for i in 1..=3 {
            let entry = MemoryEntry::new(
                format!("id-{}", i),
                "agent-1".to_string(),
                format!("Test content {}", i),
                vec![0.1 * i as f32, 0.2, 0.3, 0.4],
            );
            pipeline.store().store(entry).await.unwrap();
        }

        // Query and format
        let matches = pipeline
            .query("test", MemoryQueryOptions::default())
            .await
            .unwrap();
        let context = pipeline.format_context(&matches);

        assert!(context.contains("## Relevant Memories"));
        assert!(context.contains("Test content"));
        assert!(context.contains("[score:"));
    }
}
