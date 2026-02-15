# Issue 023: Implement Default Config Generation

## Summary
Add a `generate_default_config()` function that produces a commented default configuration file in either TOML or JSON5 format. This allows users to bootstrap a new configuration with sensible defaults and inline documentation.

## Location
- Crate: `aisopod-config`
- File: `crates/aisopod-config/src/generate.rs`

## Current Behavior
Users must manually create configuration files from scratch or copy examples. There is no programmatic way to generate a default config file with comments explaining each section.

## Expected Behavior
- `generate_default_config(format: ConfigFormat) -> Result<String>` returns a formatted config string
- `ConfigFormat` enum supports `Json5` and `Toml` variants
- The generated output includes inline comments describing each configuration section
- The generated config is valid and can be parsed back by the loader without modification
- Default values match those from `AisopodConfig::default()`

## Impact
Default config generation improves onboarding by giving users a working starting point with documentation built in. It also serves as living documentation of the configuration schema, reducing the need for external docs.

## Suggested Implementation
1. Create `crates/aisopod-config/src/generate.rs`:
   ```rust
   use anyhow::Result;
   use crate::types::AisopodConfig;

   #[derive(Debug, Clone, Copy)]
   pub enum ConfigFormat {
       Json5,
       Toml,
   }

   /// Generate a default configuration file as a formatted string.
   pub fn generate_default_config(format: ConfigFormat) -> Result<String> {
       let config = AisopodConfig::default();

       match format {
           ConfigFormat::Json5 => generate_json5(&config),
           ConfigFormat::Toml => generate_toml(&config),
       }
   }

   fn generate_json5(config: &AisopodConfig) -> Result<String> {
       let json_value = serde_json::to_value(config)?;
       let raw = serde_json::to_string_pretty(&json_value)?;

       // Prepend a header comment
       let output = format!(
           "// Aisopod default configuration\n\
            // Edit this file to customize your setup.\n\
            // See documentation for all available options.\n\n\
            {}\n",
           raw
       );
       Ok(output)
   }

   fn generate_toml(config: &AisopodConfig) -> Result<String> {
       let raw = toml::to_string_pretty(config)?;

       let output = format!(
           "# Aisopod default configuration\n\
            # Edit this file to customize your setup.\n\
            # See documentation for all available options.\n\n\
            {}\n",
           raw
       );
       Ok(output)
   }
   ```
2. Declare the module in `lib.rs`:
   ```rust
   pub mod generate;
   pub use generate::{generate_default_config, ConfigFormat};
   ```
3. Add unit tests:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       use crate::loader::load_config_json5;
       use std::io::Write;
       use tempfile::NamedTempFile;

       #[test]
       fn test_generated_json5_is_parseable() {
           let output = generate_default_config(ConfigFormat::Json5).unwrap();
           assert!(output.contains("Aisopod default configuration"));

           // Write to temp file and parse it back
           let mut file = NamedTempFile::with_suffix(".json").unwrap();
           file.write_all(output.as_bytes()).unwrap();
           let config = load_config_json5(file.path()).unwrap();
           // Verify it matches defaults
           let defaults = AisopodConfig::default();
           assert_eq!(config.meta.version, defaults.meta.version);
       }

       #[test]
       fn test_generated_toml_is_parseable() {
           let output = generate_default_config(ConfigFormat::Toml).unwrap();
           assert!(output.contains("Aisopod default configuration"));

           let mut file = NamedTempFile::with_suffix(".toml").unwrap();
           file.write_all(output.as_bytes()).unwrap();
           let config = crate::loader::load_config_toml(file.path()).unwrap();
           let defaults = AisopodConfig::default();
           assert_eq!(config.meta.version, defaults.meta.version);
       }
   }
   ```
4. Add `tempfile` as a dev dependency in `Cargo.toml`:
   ```toml
   [dev-dependencies]
   tempfile = "3"
   ```
5. Run `cargo test -p aisopod-config` to verify all tests pass.

## Dependencies
016, 017, 018

## Acceptance Criteria
- [ ] `generate_default_config()` function produces config strings in JSON5 and TOML formats
- [ ] `ConfigFormat` enum supports `Json5` and `Toml` variants
- [ ] Generated output includes header comments describing the configuration
- [ ] Generated JSON5 config can be parsed by `load_config_json5()` without errors
- [ ] Generated TOML config can be parsed by `load_config_toml()` without errors
- [ ] Generated values match `AisopodConfig::default()`
- [ ] Unit tests verify generation and round-trip parsing for both formats

---
*Created: 2026-02-15*
