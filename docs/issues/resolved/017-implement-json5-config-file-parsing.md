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
- [x] `json5` crate is added as a dependency in `crates/aisopod-config/Cargo.toml`
- [x] `load_config_json5()` function reads and parses a JSON5 file into `AisopodConfig`
- [x] `load_config()` auto-detects `.json` and `.json5` extensions
- [x] Parse errors include file path context for debugging
- [x] A sample JSON5 fixture file exists for testing
- [x] Integration test verifies parsing a sample JSON5 config file

## Resolution
Issue 017 was successfully implemented with the following changes:

### Changes Made:
1. **Added json5 dependency** (`crates/aisopod-config/Cargo.toml`):
   - Added `json5 = "0.4"` as a dependency
   - Added `tempfile = "3"` as a dev-dependency for testing

2. **Created loader module** (`crates/aisopod-config/src/loader.rs`):
   - Implemented `load_config(path: &Path)` - auto-detects format from file extension
   - Implemented `load_config_json5(path: &Path)` - parses JSON5 files
   - Parse errors include file path context using anyhow's `with_context`

3. **Updated lib.rs** (`crates/aisopod-config/src/lib.rs`):
   - Added `pub mod loader;`
   - Exported `load_config` and `load_config_json5` functions

4. **Created test fixture** (`crates/aisopod-config/tests/fixtures/sample.json5`):
   - Sample JSON5 configuration with comments, unquoted keys, and trailing commas

5. **Added integration tests** (`crates/aisopod-config/tests/load_json5.rs`):
   - `test_load_sample_json5` - verifies parsing the sample fixture
   - `test_load_sample_with_auto_detect` - verifies auto-detection works

6. **Unit tests in loader.rs**:
   - Test basic JSON5 loading
   - Test auto-detection for .json5 and .json extensions
   - Test unsupported extension handling
   - Test JSON5 comments and trailing commas
   - Test file not found error handling
   - Test invalid JSON5 error handling

### Test Results:
- All 8 unit tests in loader.rs passed
- All 2 integration tests in load_json5.rs passed
- Full workspace build and test suite completed successfully

---
*Created: 2026-02-15*
*Resolved: 2026-02-16*
