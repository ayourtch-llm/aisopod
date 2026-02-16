# Issue 020: Implement @include Directive Support

## Summary
Implement `@include` directive processing that scans a parsed configuration for `"@include"` keys, loads the referenced files, and merges their contents into the parent configuration. Support relative path resolution from the config file's directory and detect circular includes.

## Location
- Crate: `aisopod-config`
- File: `crates/aisopod-config/src/includes.rs`

## Current Behavior
If a config file contains `"@include": "path/to/fragment.json5"`, the key-value pair is passed through as a regular field, causing a deserialization error or being silently ignored.

## Expected Behavior
- `process_includes(value: &mut serde_json::Value, base_path: &Path, seen: &mut HashSet<PathBuf>) -> Result<()>` scans for `@include` keys
- When `"@include": "path/to/file.json5"` is found, the referenced file is loaded, its env vars expanded, and its contents merged into the parent object
- Paths are resolved relative to the directory of the file containing the `@include`
- A `HashSet` of visited file paths detects circular includes and returns a clear error
- `@include` keys are removed from the final config after processing
- Both single string and array-of-strings include values are supported

## Impact
Include directives allow users to split large configurations into manageable fragments (e.g., separate files for agents, models, and tools). This is a key usability feature ported from OpenClaw's configuration system.

## Suggested Implementation
1. Create `crates/aisopod-config/src/includes.rs`:
   ```rust
   use std::collections::HashSet;
   use std::path::{Path, PathBuf};
   use anyhow::{bail, Context, Result};
   use serde_json::Value;

   /// Process @include directives in a parsed config value.
   pub fn process_includes(
       value: &mut Value,
       base_path: &Path,
       seen: &mut HashSet<PathBuf>,
   ) -> Result<()> {
       if let Value::Object(map) = value {
           // Check for @include directive
           if let Some(include_val) = map.remove("@include") {
               let paths = match &include_val {
                   Value::String(s) => vec![s.clone()],
                   Value::Array(arr) => arr
                       .iter()
                       .map(|v| v.as_str().map(String::from).ok_or_else(|| {
                           anyhow::anyhow!("@include array elements must be strings")
                       }))
                       .collect::<Result<Vec<_>>>()?,
                   _ => bail!("@include value must be a string or array of strings"),
               };

               for include_path_str in paths {
                   let include_path = base_path.join(&include_path_str);
                   let canonical = include_path.canonicalize()
                       .with_context(|| format!(
                           "Failed to resolve @include path: {}",
                           include_path.display()
                       ))?;

                   if !seen.insert(canonical.clone()) {
                       bail!(
                           "Circular @include detected: {}",
                           canonical.display()
                       );
                   }

                   // Load and parse the included file
                   let contents = std::fs::read_to_string(&canonical)
                       .with_context(|| format!(
                           "Failed to read included file: {}",
                           canonical.display()
                       ))?;
                   let mut fragment: Value = json5::from_str(&contents)
                       .with_context(|| format!(
                           "Failed to parse included file: {}",
                           canonical.display()
                       ))?;

                   // Expand env vars in the fragment
                   crate::env::expand_env_vars(&mut fragment)?;

                   // Recursively process includes in the fragment
                   let fragment_dir = canonical.parent().unwrap_or(base_path);
                   process_includes(&mut fragment, fragment_dir, seen)?;

                   // Merge fragment into parent
                   if let Value::Object(fragment_map) = fragment {
                       for (k, v) in fragment_map {
                           map.entry(&k).or_insert(v);
                       }
                   }
               }
           }

           // Recursively process remaining object values
           for (_key, val) in map.iter_mut() {
               process_includes(val, base_path, seen)?;
           }
       }

       Ok(())
   }
   ```
2. Declare the module in `lib.rs`:
   ```rust
   pub mod includes;
   ```
3. Integrate into the config loading pipeline in `loader.rs`, after env var expansion:
   ```rust
   pub fn load_config_json5(path: &Path) -> Result<AisopodConfig> {
       let contents = std::fs::read_to_string(path)?;
       let mut value: serde_json::Value = json5::from_str(&contents)?;
       crate::env::expand_env_vars(&mut value)?;
       let canonical = path.canonicalize()?;
       let base_dir = canonical.parent().unwrap();
       let mut seen = std::collections::HashSet::new();
       seen.insert(canonical.clone());
       crate::includes::process_includes(&mut value, base_dir, &mut seen)?;
       let config: AisopodConfig = serde_json::from_value(value)?;
       Ok(config)
   }
   ```
4. Create test fixtures:
   - `crates/aisopod-config/tests/fixtures/main_with_include.json5`:
     ```json5
     {
       "@include": "fragment.json5",
       meta: { version: "1.0" },
     }
     ```
   - `crates/aisopod-config/tests/fixtures/fragment.json5`:
     ```json5
     {
       gateway: { host: "127.0.0.1", port: 9090 },
     }
     ```
   - `crates/aisopod-config/tests/fixtures/circular_a.json5`:
     ```json5
     { "@include": "circular_b.json5" }
     ```
   - `crates/aisopod-config/tests/fixtures/circular_b.json5`:
     ```json5
     { "@include": "circular_a.json5" }
     ```
5. Add integration tests verifying include resolution and circular detection.
6. Run `cargo test -p aisopod-config` to verify all tests pass.

## Dependencies
017, 019

## Acceptance Criteria
- [x] `process_includes()` resolves `@include` directives by loading and merging referenced files
- [x] Paths are resolved relative to the directory of the file containing the directive
- [x] Both single-string and array-of-strings include values are supported
- [x] Circular includes are detected and produce a clear error message
- [x] `@include` keys are removed from the final config
- [x] Included files have their env vars expanded and their own includes processed recursively
- [x] Integration tests verify include resolution and circular detection

## Resolution

Implementation completed on 2026-02-16.

### Changes Made

1. **Created `crates/aisopod-config/src/includes.rs`**
   - Implemented `process_includes()` function that scans config for `@include` directives
   - Supports both single string and array of strings for `@include` values
   - Resolves paths relative to the file containing the directive
   - Uses `HashSet<PathBuf>` to detect circular includes
   - Merges included file contents into parent config using `map.entry(&k).or_insert(v)`
   - Removes `@include` keys from the final config
   - Recursively processes includes in included files

2. **Updated `crates/aisopod-config/src/lib.rs`**
   - Added `pub mod includes;` declaration
   - Added `includes` module to the documentation

3. **Updated `crates/aisopod-config/src/loader.rs`**
   - Added `use std::collections::HashSet;` import
   - Integrated includes processing in `load_config_json5()` after env var expansion
   - Uses `HashSet` with canonicalized paths for circular detection
   - Properly handles error context for debugging

4. **Created test fixtures**
   - `fragment.json5` - Simple fragment with gateway config
   - `main_with_include.json5` - Main config with @include directive
   - `circular_a.json5` and `circular_b.json5` - Circular include test case
   - `models_fragment.json5` and `tools_fragment.json5` - Array include test case
   - `nested_include.json5` - Nested include test case

5. **Added integration tests in `tests/load_json5.rs`**
   - `test_load_config_with_include` - Single string @include test
   - `test_load_config_with_array_include` - Array of @include paths test
   - `test_load_config_circular_include` - Circular include detection test
   - `test_nested_include_processing` - Nested includes test
   - `test_include_with_env_vars` - Environment variable expansion in included files
   - `test_include_removes_include_key` - Verifies @include key is removed

6. **Added unit tests in `includes.rs`**
   - Tests for single string, array, and invalid @include values
   - Tests for circular include detection
   - Tests for nonexistent file handling
   - Tests for merge behavior

### Test Results

All tests pass successfully:
- 27 unit tests in aisopod-config
- 8 integration tests for @include functionality
- 7 existing TOML tests continue to pass
- 2 doc tests pass

---
*Created: 2026-02-15*
*Resolved: 2026-02-16*
