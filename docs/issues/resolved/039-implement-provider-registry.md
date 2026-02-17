# Issue 039: Implement Provider Registry

## Summary
Create a `ProviderRegistry` that stores and manages concrete `ModelProvider` implementations, allowing registration, lookup by provider ID, listing, and model alias resolution.

## Location
- Crate: `aisopod-provider`
- File: `crates/aisopod-provider/src/registry.rs`

## Current Behavior
There is no central place to register or look up AI model providers. Code that needs a provider has no way to discover which providers are available or resolve friendly model names to a specific provider and model pair.

## Expected Behavior
After this issue is completed:
- A `ProviderRegistry` struct holds `Arc<dyn ModelProvider>` instances keyed by provider ID.
- Callers can register a provider, look up a provider by ID, and list all registered providers.
- Model aliases (e.g., `"claude-sonnet"` → provider `"anthropic"`, model `"claude-3-5-sonnet"`) can be registered and resolved.

## Impact
The registry is the central dispatch point for all provider interactions. The agent system, session management, and gateway all use the registry to route chat completion requests to the correct provider.

## Suggested Implementation
1. Create `crates/aisopod-provider/src/registry.rs`.
2. Define the `ProviderRegistry` struct:
   ```rust
   pub struct ProviderRegistry {
       providers: HashMap<String, Arc<dyn ModelProvider>>,
       aliases: HashMap<String, ModelAlias>,
   }

   pub struct ModelAlias {
       pub provider_id: String,
       pub model_id: String,
   }
   ```
3. Implement methods on `ProviderRegistry`:
   - `new() -> Self` — create an empty registry.
   - `register(&mut self, provider: Arc<dyn ModelProvider>)` — insert a provider keyed by its `id()`.
   - `get(&self, provider_id: &str) -> Option<Arc<dyn ModelProvider>>` — look up by ID.
   - `list(&self) -> Vec<Arc<dyn ModelProvider>>` — return all registered providers.
   - `register_alias(&mut self, alias: &str, provider_id: &str, model_id: &str)` — add a model alias.
   - `resolve_alias(&self, alias: &str) -> Option<&ModelAlias>` — resolve an alias to a provider/model pair.
   - `resolve_model(&self, name: &str) -> Option<(Arc<dyn ModelProvider>, String)>` — given a model name or alias, return the provider and the canonical model ID.
4. Re-export `ProviderRegistry` and `ModelAlias` from `lib.rs`.
5. Add doc-comments to every public item.
6. Run `cargo check -p aisopod-provider`.

## Dependencies
- Issue 038 (ModelProvider trait and core types)

## Resolution
The ProviderRegistry was implemented with the following components:

### Files Created
- **`crates/aisopod-provider/src/registry.rs`** - New file containing the ProviderRegistry implementation

### Files Modified
- **`crates/aisopod-provider/src/lib.rs`** - Added `registry` module and re-exported `ProviderRegistry` and `ModelAlias`

### Implementation Details
- **`ProviderRegistry`** - Core struct with:
  - `providers: HashMap<String, Arc<dyn ModelProvider>>` - Stores providers keyed by ID
  - `aliases: HashMap<String, ModelAlias>` - Stores model aliases
  
- **Methods Implemented**:
  - `new()` - Creates an empty registry
  - `register(&mut self, provider: Arc<dyn ModelProvider>)` - Registers a provider by its ID
  - `unregister(&mut self, provider_id: &str)` - Removes a provider
  - `get(&self, provider_id: &str) -> Option<Arc<dyn ModelProvider>>` - Looks up provider by ID
  - `list_providers(&self) -> Vec<Arc<dyn ModelProvider>>` - Returns all registered providers
  - `register_alias(&mut self, alias: &str, provider_id: &str, model_id: &str)` - Registers a model alias
  - `resolve_alias(&self, alias: &str) -> Option<&ModelAlias>` - Resolves alias to provider/model pair
  - `resolve_model(&self, name: &str) -> Option<(Arc<dyn ModelProvider>, String)>` - Resolves model name or alias to provider and model ID

- **`ModelAlias`** - Simple struct containing `provider_id` and `model_id` fields

- **`Default`** trait implemented for `ProviderRegistry` (returns empty registry)

### Testing
- Added 15 comprehensive unit tests for registry operations
- All tests pass (`cargo test -p aisopod-provider`)
- Build passes with `RUSTFLAGS=-Awarnings`

### Acceptance Criteria Met
- [x] `ProviderRegistry` struct is defined and exported.
- [x] Providers can be registered and retrieved by ID.
- [x] `list()` returns all registered providers.
- [x] Model aliases can be registered and resolved to a provider/model pair.
- [x] `resolve_model()` works for both direct provider IDs and aliases.
- [x] `cargo check -p aisopod-provider` passes.

---
*Created: 2026-02-15*
*Resolved: 2026-02-17*
