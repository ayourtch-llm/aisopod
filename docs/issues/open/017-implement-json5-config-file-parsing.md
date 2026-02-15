# Issue 017: Implement JSON5 Config File Parsing

## Summary
Add the `json5` crate as a dependency and implement a `load_config_json5(path)` function that reads a JSON5 configuration file from disk and parses it into an `AisopodConfig` struct. Support auto-detection of format from file extension (`.json`, `.json5`).

## Location
- Crate: `aisopod-config`
- File: `crates/aisopod-config/src/loader.rs`, `crates/aisopod-config/Cargo.toml`

## Current Behavior
The `aisopod-config` crate has configuration types defined (from Issue 016) but no way to load a configuration from a file. There is no file parsing logic.

## Expected Behavior
- A `loader` module provides a `load_config_json5(path: &Path) -> Result<AisopodConfig>` function
- A top-level `load_config(path: &Path) -> Result<AisopodConfig>` function auto-detects format from file extension
- JSON5 files (`.json5`, `.json`) are parsed correctly, including comments, trailing commas, and unquoted keys
- Parse errors include the file path and line number for easy debugging

## Impact
JSON5 is the primary configuration format for aisopod (ported from OpenClaw). This is the first concrete step toward a working config loading pipeline and unblocks environment variable substitution and include directive processing.

## Suggested Implementation
1. Add the `json5` crate to `crates/aisopod-config/Cargo.toml`:
   ```toml
   [dependencies]
   json5 = "0.4"
   ```
2. Create `crates/aisopod-config/src/loader.rs`:
   ```rust
   use std::path::Path;
   use anyhow::{Context, Result};
   use crate::types::AisopodConfig;

   /// Load a configuration file, auto-detecting format from extension.
   pub fn load_config(path: &Path) -> Result<AisopodConfig> {
       let ext = path.extension()
           .and_then(|e| e.to_str())
           .unwrap_or("");
       match ext {
           "json" | "json5" => load_config_json5(path),
           _ => anyhow::bail!(
               "Unsupported config file extension: '{}'. Use .json5, .json, or .toml",
               ext
           ),
       }
   }

   /// Load and parse a JSON5 configuration file.
   pub fn load_config_json5(path: &Path) -> Result<AisopodConfig> {
       let contents = std::fs::read_to_string(path)
           .with_context(|| format!("Failed to read config file: {}", path.display()))?;
       let config: AisopodConfig = json5::from_str(&contents)
           .with_context(|| format!("Failed to parse JSON5 config: {}", path.display()))?;
       Ok(config)
   }
   ```
3. Update `crates/aisopod-config/src/lib.rs` to declare the loader module:
   ```rust
   pub mod loader;
   pub use loader::load_config;
   ```
4. Create a sample JSON5 test fixture at `crates/aisopod-config/tests/fixtures/sample.json5`:
   ```json5
   {
     // Sample aisopod configuration
     meta: {
       version: "1.0",
     },
     gateway: {
       host: "127.0.0.1",
       port: 8080,
     },
   }
   ```
5. Add a basic integration test in `crates/aisopod-config/tests/load_json5.rs`:
   ```rust
   use std::path::PathBuf;
   use aisopod_config::load_config;

   #[test]
   fn test_load_sample_json5() {
       let path = PathBuf::from("tests/fixtures/sample.json5");
       let config = load_config(&path).expect("Failed to load config");
       assert_eq!(config.meta.version, "1.0");
       assert_eq!(config.gateway.port, 8080);
   }
   ```
6. Run `cargo test -p aisopod-config` to verify the integration test passes.

## Dependencies
016

## Acceptance Criteria
- [ ] `json5` crate is added as a dependency in `crates/aisopod-config/Cargo.toml`
- [ ] `load_config_json5()` function reads and parses a JSON5 file into `AisopodConfig`
- [ ] `load_config()` auto-detects `.json` and `.json5` extensions
- [ ] Parse errors include file path context for debugging
- [ ] A sample JSON5 fixture file exists for testing
- [ ] Integration test verifies parsing a sample JSON5 config file

---
*Created: 2026-02-15*
