# Issue 083: Define Embedding Provider Trait and OpenAI Implementation

## Summary
Define an abstract `EmbeddingProvider` trait for generating vector embeddings from text and implement the first concrete provider using the OpenAI Embeddings API (`text-embedding-3-small`). Include an embedding cache to avoid redundant API calls.

## Location
- Crate: `aisopod-memory`
- File: `crates/aisopod-memory/src/embedding.rs` and `crates/aisopod-memory/src/embedding/openai.rs`

## Current Behavior
There is no mechanism for generating vector embeddings from text. The `MemoryEntry` struct requires an `embedding: Vec<f32>` field, but nothing produces these vectors.

## Expected Behavior
An `EmbeddingProvider` trait abstracts embedding generation, and an `OpenAiEmbeddingProvider` implements it by calling the OpenAI API. An in-memory cache prevents duplicate API calls for identical text.

## Impact
Embedding generation is required by both the store pipeline (generating embeddings for new memories) and the query pipeline (generating embeddings for search queries). Without this, the memory system cannot perform semantic search.

## Suggested Implementation
1. Create `crates/aisopod-memory/src/embedding.rs` (or a module directory `embedding/mod.rs`).
2. Define the trait:
   ```rust
   #[async_trait]
   pub trait EmbeddingProvider: Send + Sync {
       /// Generate an embedding vector for the given text.
       async fn embed(&self, text: &str) -> Result<Vec<f32>>;

       /// Generate embeddings for multiple texts in a batch.
       async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;

       /// Return the dimensionality of embeddings produced by this provider.
       fn dimensions(&self) -> usize;
   }
   ```
3. Create `crates/aisopod-memory/src/embedding/openai.rs`.
4. Define `OpenAiEmbeddingProvider` struct with fields:
   - `api_key: String`
   - `model: String` (default: `"text-embedding-3-small"`)
   - `dimensions: usize` (default: `1536`)
   - `client: reqwest::Client`
   - `cache: Arc<Mutex<HashMap<String, Vec<f32>>>>` (simple in-memory cache keyed by text hash)
5. Implement the constructor `OpenAiEmbeddingProvider::new(api_key: String, model: Option<String>, dimensions: Option<usize>) -> Self` that initializes the client and sets defaults.
6. Implement `EmbeddingProvider::embed()`:
   - Compute a hash of the input text and check the cache. If found, return the cached embedding.
   - Build an HTTP POST request to `https://api.openai.com/v1/embeddings` with JSON body `{ "model": model, "input": text, "dimensions": dimensions }`.
   - Set the `Authorization: Bearer {api_key}` header.
   - Parse the response JSON and extract the embedding vector from `data[0].embedding`.
   - Store the result in the cache before returning.
7. Implement `EmbeddingProvider::embed_batch()`:
   - Check cache for each text; collect cache misses.
   - Send a single API request with all cache-miss texts as `"input": [text1, text2, ...]`.
   - Parse the response and populate the cache.
   - Return all embeddings in the original input order.
8. Implement `EmbeddingProvider::dimensions()` — return `self.dimensions`.
9. Add `reqwest = { version = "0.12", features = ["json"] }` to `Cargo.toml` if not already present.
10. Re-export `EmbeddingProvider` and `OpenAiEmbeddingProvider` from `lib.rs`.
11. Run `cargo check -p aisopod-memory` to verify compilation.

## Dependencies
- Issue 081 (define memory types and MemoryStore trait)

## Acceptance Criteria
- [ ] `EmbeddingProvider` trait is defined with `embed()`, `embed_batch()`, and `dimensions()` methods
- [ ] `OpenAiEmbeddingProvider` implements the trait using the OpenAI Embeddings API
- [ ] The default model is `text-embedding-3-small` and the default dimensions are 1536
- [ ] An in-memory cache prevents duplicate API calls for the same text
- [ ] Embedding model and dimensions are configurable via the constructor
- [ ] `cargo check -p aisopod-memory` compiles without errors

---
*Created: 2026-02-15*
