# Issue 047: Implement Model Discovery and Capability Metadata

## Summary
Build a model discovery system that queries each registered provider for its available models, caches the results, and exposes rich capability metadata (context window size, vision support, tool support) through a unified `ModelInfo` catalog.

## Location
- Crate: `aisopod-provider`
- File: `crates/aisopod-provider/src/discovery.rs`

## Current Behavior
There is no centralized model catalog. Each provider's `list_models()` must be called individually, and there is no caching or aggregated view of all available models and their capabilities.

## Expected Behavior
After this issue is completed:
- A `ModelCatalog` struct aggregates models from all registered providers.
- Model listings are cached with a configurable TTL to avoid redundant API calls.
- Each model entry includes capability metadata: context window size, vision support, tool/function-calling support, and supported input modalities.
- A `refresh()` method re-queries all providers and updates the cache.
- Callers can look up a model by ID or search by capability.

## Impact
Model discovery lets the agent system and UI present available models to users, validate model selections, and make intelligent routing decisions based on model capabilities.

## Suggested Implementation
1. Create `crates/aisopod-provider/src/discovery.rs`.
2. Define the `ModelCatalog` struct:
   ```rust
   pub struct ModelCatalog {
       registry: Arc<ProviderRegistry>,
       cache: RwLock<ModelCache>,
       cache_ttl: Duration,
   }

   struct ModelCache {
       models: Vec<ModelInfo>,
       last_refresh: Option<Instant>,
   }
   ```
3. Implement methods on `ModelCatalog`:
   - `new(registry: Arc<ProviderRegistry>, cache_ttl: Duration) -> Self` — create with an empty cache.
   - `async refresh(&self) -> Result<()>` — iterate over all providers in the registry, call `list_models()` on each, and store the combined results in the cache with the current timestamp.
   - `async list_all(&self) -> Result<Vec<ModelInfo>>` — return cached models, refreshing first if the cache is expired or empty.
   - `async get_model(&self, model_id: &str) -> Result<Option<ModelInfo>>` — look up a specific model by ID.
   - `async find_by_capability(&self, vision: Option<bool>, tools: Option<bool>, min_context: Option<u32>) -> Result<Vec<ModelInfo>>` — filter models by capability requirements.
4. Ensure `ModelInfo` (defined in Issue 038) includes at minimum:
   - `id: String`
   - `name: String`
   - `provider: String`
   - `context_window: u32`
   - `supports_vision: bool`
   - `supports_tools: bool`
5. Handle provider errors gracefully during refresh — if one provider fails, log the error and continue with the others. Do not discard previously cached models for that provider.
6. Re-export `ModelCatalog` from `lib.rs`.
7. Add unit tests using a mock `ModelProvider` that returns a fixed model list:
   - Test that `list_all()` returns models from multiple providers.
   - Test that the cache is respected (no re-query within TTL).
   - Test that `refresh()` forces a re-query.
   - Test `find_by_capability()` filtering.
8. Run `cargo check -p aisopod-provider` and `cargo test -p aisopod-provider`.

## Dependencies
- Issue 038 (ModelProvider trait and core types — `ModelInfo` definition)
- Issue 039 (Provider registry — needed to iterate providers)

## Acceptance Criteria
- [ ] `ModelCatalog` aggregates models from all registered providers.
- [ ] Model listings are cached with a configurable TTL.
- [ ] `refresh()` re-queries all providers and updates the cache.
- [ ] `get_model()` looks up a model by ID.
- [ ] `find_by_capability()` filters models by vision, tool support, and context window.
- [ ] Provider errors during refresh are handled gracefully without losing cached data.
- [ ] Unit tests pass for caching, refresh, and filtering logic.
- [ ] `cargo check -p aisopod-provider` passes.

---
*Created: 2026-02-15*
