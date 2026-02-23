# Issue 109: Define Plugin Manifest Format and Parser

## Summary

This issue implemented a standardized TOML-based manifest format for aisopod plugins. The manifest system allows plugins to declare their metadata, capabilities, and compatibility requirements in a human-readable format (`aisopod.plugin.toml`), which is used during plugin discovery and loading.

## Key Implementation Details

### TOML Manifest Structure

The manifest file follows a clean, organized structure with three main sections:

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

### Struct Definitions

The implementation uses `serde::Deserialize` to automatically parse TOML into Rust structs:

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct PluginManifest {
    pub plugin: PluginManifestInfo,
    #[serde(default)]
    pub capabilities: Option<PluginCapabilities>,
    #[serde(default)]
    pub compatibility: Option<PluginCompatibility>,
}
```

Key design decisions:
- `PluginManifestInfo` contains all required fields
- `PluginCapabilities` and `PluginCompatibility` are optional
- `#[serde(default)]` allows missing optional sections

### Validation Logic

The `validate()` method checks required fields and format:

```rust
fn validate(&self) -> Result<(), ManifestError> {
    if self.plugin.id.trim().is_empty() {
        return Err(ManifestError::Validation("plugin.id must not be empty".to_string()));
    }
    Version::parse(&self.plugin.version).map_err(|_| {
        ManifestError::Validation(format!(
            "plugin.version '{}' is not valid semver",
            self.plugin.version
        ))
    })?;
    // ... more validation
    Ok(())
}
```

### Error Handling

The `ManifestError` enum provides clear error categories:

```rust
#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Validation error: {0}")]
    Validation(String),
}
```

The `#[from]` attribute on `Io` enables automatic conversion from `std::io::Error`.

## Files Created/Modified

### New Files
1. **`crates/aisopod-plugin/src/manifest.rs`** - Complete manifest implementation (601 lines)
   - Struct definitions with documentation
   - Parser implementation (`from_file`, `from_str`)
   - Validation logic
   - Comprehensive unit tests (18 test cases)

### Modified Files
1. **`Cargo.toml`** - Added `semver = "1"` and `toml = "0.8"` to workspace dependencies
2. **`crates/aisopod-plugin/Cargo.toml`** - Added semver, toml dependencies
3. **`crates/aisopod-plugin/src/lib.rs`** - Added `manifest` module and exported types
4. **`docs/issues/open/107-define-plugin-trait-and-pluginmeta-types.md`** - Moved to resolved
5. **`docs/issues/open/108-implement-pluginapi-for-capability-registration.md`** - Moved to resolved

## Lessons Learned

### 1. TOML Option Handling with serde

When using `serde` with TOML, missing optional sections are parsed as `None` rather than an empty struct. The `#[serde(default)]` attribute is crucial for handling this:

```rust
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PluginCapabilities {
    #[serde(default)]  // Converts missing to Some(vec![])
    pub channels: Option<Vec<String>>,
}
```

Without `#[serde(default)]`, accessing `manifest.capabilities.unwrap()` would panic if the section is missing.

### 2. PartialEq Derive for Tests

The `#[derive(PartialEq)]` trait must be explicitly added to structs used in assertions. Even though `PluginCapabilities` and `PluginCompatibility` contain only types that implement `PartialEq` (Vec<String>, Option), the derive macro is not automatically inherited when composing structs.

### 3. Semver Validation

The `semver::Version::parse()` function returns a `Result`, making it ideal for validation:
- It correctly validates standard versions: "0.1.0", "1.0.0"
- It correctly validates prerelease: "1.0.0-alpha.1"
- It correctly validates build metadata: "1.0.0+build.123"
- It correctly rejects invalid: "not-a-version"

### 4. Error Message Quality

The toml crate's parser error messages are already descriptive, so the `ManifestError::Parse` variant doesn't add much value beyond wrapping the original error. This is acceptable as it maintains a consistent error type for callers.

### 5. Test Design Pattern

Testing both success and failure cases is important:
- Valid manifest with minimal required fields
- Valid manifest with all optional sections
- Invalid id (empty, whitespace-only)
- Invalid name (empty)
- Invalid version (not semver)
- Invalid entry_point (empty)
- Parse errors (malformed TOML)

The test `test_validation_whitespace_id` is particularly important as it ensures the validation handles edge cases like `"   "` for id fields.

## Acceptance Criteria Met

All acceptance criteria from the issue have been met:

- [x] `aisopod.plugin.toml` manifest schema is defined and documented
- [x] `PluginManifest` struct and related types are implemented
- [x] Parser reads manifest files from disk using the `toml` crate
- [x] Validation catches missing required fields (id, name, version, entry_point)
- [x] Invalid version strings produce clear error messages
- [x] Compatibility section supports min/max host version
- [x] `ManifestError` provides descriptive error messages
- [x] Unit tests cover valid and invalid manifest scenarios
- [x] `cargo build -p aisopod-plugin` compiles without errors
- [x] `cargo test -p aisopod-plugin` passes (18 unit tests)
- [x] All public types and methods have documentation comments

## Verification Summary

**Verification Date:** 2026-02-23  
**Status:** ✅ All acceptance criteria verified

### Build Verification
```bash
cargo build -p aisopod-plugin      # ✅ PASS
cargo test -p aisopod-plugin       # ✅ PASS (18/18 tests)
cargo doc -p aisopod-plugin        # ✅ PASS (0 warnings)
cargo build                        # ✅ PASS (workspace)
cargo test                         # ✅ PASS (workspace)
```

### Test Coverage
- 18 unit tests covering:
  - Valid manifests (minimal, with capabilities, with compatibility)
  - Semver edge cases (prerelease, build metadata)
  - Validation errors (empty id, name, version, entry_point)
  - Whitespace-only validation
  - Parse errors (malformed TOML)
  - Multiple errors in one manifest
  - Defaults for optional sections
  - Debug and Clone implementations

### Documentation
- All public types have documentation comments
- Module-level documentation with example TOML format
- Example code in doc comments for public methods
- Cargo doc generates without warnings

## Changes Summary

### Files Changed
1. `Cargo.toml` - Added semver and toml workspace dependencies
2. `crates/aisopod-plugin/Cargo.toml` - Added semver and toml dependencies
3. `crates/aisopod-plugin/src/manifest.rs` - New file with complete implementation
4. `crates/aisopod-plugin/src/lib.rs` - Added manifest module exports
5. `docs/learnings/109-define-plugin-manifest-format-and-parser.md` - New learning documentation

### Commit Messages
```
Issue 109: Define plugin manifest format and parser
```

## Next Steps

After this issue is resolved, the following can be addressed:
- **Issue 112**: Implement compiled-in plugin loading (uses the manifest)
- **Issue 113**: Implement dynamic shared library plugin loading (uses the manifest)
- **Issue 115**: Implement plugin configuration and add plugin system tests
- Plugin discovery mechanisms that read manifest files
