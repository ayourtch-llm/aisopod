# Issue 109: Define Plugin Manifest Format and Parser

## Summary
Define the `aisopod.plugin.toml` manifest schema for describing plugin metadata and capabilities, implement a parser using the `toml` crate, and validate manifest fields including id, name, version, entry point, and capabilities.

## Location
- Crate: `aisopod-plugin`
- File: `crates/aisopod-plugin/src/manifest.rs`

## Current Behavior
No manifest format exists for describing plugins. Plugin metadata is only available programmatically through the `PluginMeta` struct (Issue 107).

## Expected Behavior
A standardized `aisopod.plugin.toml` file format describes each plugin's identity, version, entry point, capabilities, and dependencies. A parser reads and validates these manifests, producing clear error messages for invalid or incomplete files. The parsed manifest is used during plugin discovery and loading.

## Impact
The manifest format is essential for dynamic plugin loading (Issue 112) and provides a human-readable description of each plugin. It enables tooling to inspect plugins without loading them.

## Suggested Implementation
1. **Define the manifest schema:**
   ```toml
   [plugin]
   id = "my-plugin"
   name = "My Plugin"
   version = "0.1.0"
   description = "A sample plugin"
   author = "Author Name"
   entry_point = "libmy_plugin"

   [capabilities]
   channels = ["custom-channel"]
   tools = ["custom-tool"]
   providers = []
   commands = ["my-command"]
   hooks = ["BeforeAgentRun", "AfterAgentRun"]

   [compatibility]
   min_host_version = "0.1.0"
   max_host_version = "1.0.0"
   ```
2. **Define the manifest types in `manifest.rs`:**
   ```rust
   use serde::Deserialize;

   #[derive(Debug, Deserialize)]
   pub struct PluginManifest {
       pub plugin: PluginManifestInfo,
       pub capabilities: Option<PluginCapabilities>,
       pub compatibility: Option<PluginCompatibility>,
   }

   #[derive(Debug, Deserialize)]
   pub struct PluginManifestInfo {
       pub id: String,
       pub name: String,
       pub version: String,
       pub description: String,
       pub author: String,
       pub entry_point: String,
   }

   #[derive(Debug, Deserialize)]
   pub struct PluginCapabilities {
       pub channels: Option<Vec<String>>,
       pub tools: Option<Vec<String>>,
       pub providers: Option<Vec<String>>,
       pub commands: Option<Vec<String>>,
       pub hooks: Option<Vec<String>>,
   }

   #[derive(Debug, Deserialize)]
   pub struct PluginCompatibility {
       pub min_host_version: Option<String>,
       pub max_host_version: Option<String>,
   }
   ```
3. **Implement the parser:**
   ```rust
   impl PluginManifest {
       pub fn from_file(path: &std::path::Path) -> Result<Self, ManifestError> {
           let content = std::fs::read_to_string(path)
               .map_err(|e| ManifestError::Io(e))?;
           Self::from_str(&content)
       }

       pub fn from_str(content: &str) -> Result<Self, ManifestError> {
           let manifest: PluginManifest = toml::from_str(content)
               .map_err(|e| ManifestError::Parse(e.to_string()))?;
           manifest.validate()?;
           Ok(manifest)
       }
   }
   ```
4. **Implement validation:**
   ```rust
   impl PluginManifest {
       fn validate(&self) -> Result<(), ManifestError> {
           if self.plugin.id.is_empty() {
               return Err(ManifestError::Validation("plugin.id must not be empty".into()));
           }
           if self.plugin.name.is_empty() {
               return Err(ManifestError::Validation("plugin.name must not be empty".into()));
           }
           // Validate semver format for version
           semver::Version::parse(&self.plugin.version)
               .map_err(|_| ManifestError::Validation(
                   format!("plugin.version '{}' is not valid semver", self.plugin.version)
               ))?;
           Ok(())
       }
   }
   ```
5. **Define `ManifestError`:**
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum ManifestError {
       #[error("IO error: {0}")]
       Io(#[from] std::io::Error),
       #[error("Parse error: {0}")]
       Parse(String),
       #[error("Validation error: {0}")]
       Validation(String),
   }
   ```
6. **Add unit tests** for valid manifests, missing required fields, invalid version strings, and empty files.

## Dependencies
- Issue 107 (Plugin trait and PluginMeta types)

## Acceptance Criteria
- [ ] `aisopod.plugin.toml` manifest schema is defined and documented
- [ ] `PluginManifest` struct and related types are implemented
- [ ] Parser reads manifest files from disk using the `toml` crate
- [ ] Validation catches missing required fields (id, name, version, entry_point)
- [ ] Invalid version strings produce clear error messages
- [ ] Compatibility section supports min/max host version
- [ ] `ManifestError` provides descriptive error messages
- [ ] Unit tests cover valid and invalid manifest scenarios
- [ ] `cargo build -p aisopod-plugin` compiles without errors

---
*Created: 2026-02-15*
