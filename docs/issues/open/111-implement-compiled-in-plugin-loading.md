# Issue 111: Implement Compiled-In Plugin Loading (Phase 1)

## Summary
Implement feature-gated compiled-in plugin loading so that built-in plugins can be selectively included at compile time via Cargo features. This is Phase 1 of the plugin loading strategy, providing zero runtime overhead for unused plugins.

## Location
- Crate: `aisopod-plugin`
- File: `crates/aisopod-plugin/src/builtin.rs`, `crates/aisopod-plugin/Cargo.toml`

## Current Behavior
The `PluginRegistry` (Issue 110) can register and manage plugins, but there is no mechanism for automatically loading built-in plugins based on compile-time feature flags.

## Expected Behavior
Each built-in plugin is gated behind a Cargo feature flag (e.g., `plugin-telegram`, `plugin-openai`). A `register_builtin_plugins()` function checks which features are enabled and registers the corresponding plugins with the registry. Plugins not included via features produce zero runtime overhead.

## Impact
This is the primary plugin loading strategy for production builds. It allows users to create minimal custom builds with only the plugins they need, reducing binary size and attack surface.

## Suggested Implementation
1. **Add feature flags to `Cargo.toml`:**
   ```toml
   [features]
   default = []
   plugin-telegram = ["dep:aisopod-channel-telegram"]
   plugin-discord = ["dep:aisopod-channel-discord"]
   plugin-openai = ["dep:aisopod-provider-openai"]
   plugin-anthropic = ["dep:aisopod-provider-anthropic"]
   all-plugins = ["plugin-telegram", "plugin-discord", "plugin-openai", "plugin-anthropic"]
   ```
2. **Create `builtin.rs` with conditional compilation:**
   ```rust
   use crate::PluginRegistry;

   pub fn register_builtin_plugins(registry: &mut PluginRegistry) -> Result<(), crate::RegistryError> {
       #[cfg(feature = "plugin-telegram")]
       {
           let plugin = aisopod_channel_telegram::TelegramPlugin::new();
           registry.register(std::sync::Arc::new(plugin))?;
           tracing::info!("Registered built-in plugin: telegram");
       }

       #[cfg(feature = "plugin-openai")]
       {
           let plugin = aisopod_provider_openai::OpenAiPlugin::new();
           registry.register(std::sync::Arc::new(plugin))?;
           tracing::info!("Registered built-in plugin: openai");
       }

       // Additional plugins follow the same pattern...

       Ok(())
   }
   ```
3. **Add a helper to list available built-in plugins:**
   ```rust
   pub fn list_available_builtins() -> Vec<&'static str> {
       let mut plugins = Vec::new();

       #[cfg(feature = "plugin-telegram")]
       plugins.push("telegram");

       #[cfg(feature = "plugin-discord")]
       plugins.push("discord");

       #[cfg(feature = "plugin-openai")]
       plugins.push("openai");

       #[cfg(feature = "plugin-anthropic")]
       plugins.push("anthropic");

       plugins
   }
   ```
4. **Integrate with the application startup** in the binary crate:
   ```rust
   let mut registry = PluginRegistry::new();
   register_builtin_plugins(&mut registry)?;
   registry.init_all(&ctx).await?;
   ```
5. **Document** how to enable/disable plugins at build time:
   - `cargo build --features plugin-telegram,plugin-openai`
   - `cargo build --features all-plugins`
   - `cargo build` (no plugins, minimal build)

## Dependencies
- Issue 110 (PluginRegistry lifecycle management)

## Acceptance Criteria
- [ ] Cargo feature flags are defined for each built-in plugin
- [ ] `register_builtin_plugins()` conditionally registers plugins based on enabled features
- [ ] Disabled plugins produce zero runtime overhead (no code included)
- [ ] `list_available_builtins()` reports which plugins are compiled in
- [ ] Building with `--features all-plugins` includes all built-in plugins
- [ ] Building with no features produces a minimal binary
- [ ] `cargo build -p aisopod-plugin` compiles without errors (with and without features)

---
*Created: 2026-02-15*
