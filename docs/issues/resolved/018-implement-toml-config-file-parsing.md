# Issue 018: Implement TOML Config File Parsing

## Summary
Add the `toml` crate as a dependency and implement a `load_config_toml(path)` function that reads and parses a TOML configuration file into an `AisopodConfig` struct. Integrate TOML support into the main `load_config()` function to auto-detect `.toml` files alongside JSON5.

## Location
- Crate: `aisopod-config`
- File: `crates/aisopod-config/src/loader.rs`, `crates/aisopod-config/Cargo.toml`

## Current Behavior
The `load_config()` function (from Issue 017) supports JSON5 format but returns an error for `.toml` files. There is no TOML parsing support.

## Expected Behavior
- A `load_config_toml(path: &Path) -> Result<AisopodConfig>` function parses TOML config files
- The existing `load_config()` function's match statement includes `.toml` extension routing to `load_config_toml()`
- TOML and JSON5 configs are functionally equivalent — the same logical configuration can be expressed in either format

## Impact
TOML is a popular configuration format in the Rust ecosystem. Supporting it alongside JSON5 gives users flexibility and aligns with Rust community conventions. This completes the config file parsing layer of the loading pipeline.

## Suggested Implementation
1. Add the `toml` crate to `crates/aisopod-config/Cargo.toml`:
   ```toml
   [dependencies]
   toml = "0.8"
   ```
2. Add the `load_config_toml()` function in `crates/aisopod-config/src/loader.rs`:
   ```rust
   /// Load and parse a TOML configuration file.
   pub fn load_config_toml(path: &Path) -> Result<AisopodConfig> {
       let contents = std::fs::read_to_string(path)
           .with_context(|| format!("Failed to read config file: {}", path.display()))?;
       let config: AisopodConfig = toml::from_str(&contents)
           .with_context(|| format!("Failed to parse TOML config: {}", path.display()))?;
       Ok(config)
   }
   ```
3. Update the `load_config()` match statement to include TOML:
   ```rust
   match ext {
       "json" | "json5" => load_config_json5(path),
       "toml" => load_config_toml(path),
       _ => anyhow::bail!(
           "Unsupported config file extension: '{}'. Use .json5, .json, or .toml",
           ext
       ),
   }
   ```
4. Create a sample TOML test fixture at `crates/aisopod-config/tests/fixtures/sample.toml`:
   ```toml
   [meta]
   version = "1.0"

   [gateway]
   host = "127.0.0.1"
   port = 8080
   ```
5. Add an integration test in `crates/aisopod-config/tests/load_toml.rs`:
   ```rust
   use std::path::PathBuf;
   use aisopod_config::load_config;

   #[test]
   fn test_load_sample_toml() {
       let path = PathBuf::from("tests/fixtures/sample.toml");
       let config = load_config(&path).expect("Failed to load config");
       assert_eq!(config.meta.version, "1.0");
       assert_eq!(config.gateway.port, 8080);
   }
   ```
6. Run `cargo test -p aisopod-config` to verify both JSON5 and TOML tests pass.

## Dependencies
016, 017

## Acceptance Criteria
- [x] `toml` crate is added as a dependency in `crates/aisopod-config/Cargo.toml`
- [x] `load_config_toml()` function reads and parses a TOML file into `AisopodConfig`
- [x] `load_config()` auto-detects `.toml` extension and routes to `load_config_toml()`
- [x] A sample TOML fixture file exists for testing
- [x] Integration test verifies parsing a sample TOML config file
- [x] Both JSON5 and TOML tests pass together

## Resolution

Issue 018 was implemented by the previous agent and committed in commit 8e05ee042f9a00628082cd96cd2343cece3be0d3.

### Changes Made:
1. Added `toml = "0.8"` dependency to `crates/aisopod-config/Cargo.toml`
2. Added `load_config_toml()` function in `crates/aisopod-config/src/loader.rs`
3. Updated `load_config()` match statement to include `.toml` extension routing
4. Exported `load_config_toml` from `crates/aisopod-config/src/lib.rs`
5. Created sample TOML test fixture at `crates/aisopod-config/tests/fixtures/sample.toml`
6. Added integration tests in `crates/aisopod-config/tests/load_toml.rs`

### Implementation Details:
- The `load_config_toml()` function reads the file, parses it as TOML using `toml::from_str()`, expands environment variables, processes @include directives, deserializes to `AisopodConfig`, and validates the result.
- The `load_config()` function's match statement now routes `.toml` files to `load_config_toml()`.
- A comprehensive `load_config_toml_str()` helper function was also added for testing.
- All tests pass, verifying that TOML configs are parsed correctly alongside JSON5.

### Verification:
- Git log shows commit 8e05ee0 with the message "Issue 018: Implement TOML Config File Parsing"
- Files created/modified:
  - `crates/aisopod-config/Cargo.toml` - Added toml dependency
  - `crates/aisopod-config/src/lib.rs` - Exported load_config_toml
  - `crates/aisopod-config/src/loader.rs` - Added load_config_toml() and updated match
  - `crates/aisopod-config/tests/fixtures/sample.toml` - Test fixture
  - `crates/aisopod-config/tests/load_toml.rs` - Integration tests

---
*Created: 2026-02-15*
*Resolved: 2026-02-16*
