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
- [ ] `process_includes()` resolves `@include` directives by loading and merging referenced files
- [ ] Paths are resolved relative to the directory of the file containing the directive
- [ ] Both single-string and array-of-strings include values are supported
- [ ] Circular includes are detected and produce a clear error message
- [ ] `@include` keys are removed from the final config
- [ ] Included files have their env vars expanded and their own includes processed recursively
- [ ] Integration tests verify include resolution and circular detection

---
*Created: 2026-02-15*
