# Issue 019: Implement Environment Variable Substitution

## Summary
Implement an `expand_env_vars()` function that processes `${VAR}` and `${VAR:-default}` patterns in configuration string values. The function recursively traverses a `serde_json::Value` tree and replaces all environment variable references with their resolved values, erroring on unset required variables.

## Location
- Crate: `aisopod-config`
- File: `crates/aisopod-config/src/env.rs`

## Current Behavior
Configuration files are parsed as-is. String values like `"${API_KEY}"` or `"${PORT:-8080}"` are treated as literal strings with no substitution.

## Expected Behavior
- `expand_env_vars(value: &mut serde_json::Value) -> Result<()>` recursively processes all string values
- `${VAR}` is replaced with the value of environment variable `VAR`, or returns an error if `VAR` is not set
- `${VAR:-default}` is replaced with the value of `VAR` if set, or `"default"` if not
- Multiple substitutions in a single string are supported (e.g., `"${HOST}:${PORT}"`)
- Non-string values (numbers, booleans, arrays, objects) are traversed but not modified directly
- The function is called in the config loading pipeline after parsing but before deserialization

## Impact
Environment variable substitution is essential for production deployments where secrets and environment-specific values should not be hard-coded in config files. This enables secure configuration management and 12-factor app compliance.

## Suggested Implementation
1. Create `crates/aisopod-config/src/env.rs`:
   ```rust
   use anyhow::{bail, Result};
   use regex::Regex;
   use serde_json::Value;
   use std::env;

   /// Recursively expand environment variable references in all string values.
   pub fn expand_env_vars(value: &mut Value) -> Result<()> {
       match value {
           Value::String(s) => {
               *s = expand_string(s)?;
           }
           Value::Array(arr) => {
               for item in arr.iter_mut() {
                   expand_env_vars(item)?;
               }
           }
           Value::Object(map) => {
               for (_key, val) in map.iter_mut() {
                   expand_env_vars(val)?;
               }
           }
           _ => {} // numbers, bools, null — no substitution needed
       }
       Ok(())
   }

   fn expand_string(input: &str) -> Result<String> {
       let re = Regex::new(r"\$\{([^}]+)\}")?;
       let mut result = input.to_string();

       // Iterate over all matches
       for cap in re.captures_iter(input) {
           let full_match = &cap[0]; // e.g., "${VAR:-default}"
           let inner = &cap[1];      // e.g., "VAR:-default"

           let (var_name, default_val) = if let Some(idx) = inner.find(":-") {
               (&inner[..idx], Some(&inner[idx + 2..]))
           } else {
               (inner, None)
           };

           let replacement = match env::var(var_name) {
               Ok(val) => val,
               Err(_) => match default_val {
                   Some(def) => def.to_string(),
                   None => bail!(
                       "Environment variable '{}' is not set and no default provided",
                       var_name
                   ),
               },
           };

           result = result.replacen(full_match, &replacement, 1);
       }

       Ok(result)
   }
   ```
2. Add the `regex` crate to `crates/aisopod-config/Cargo.toml` if not already present:
   ```toml
   [dependencies]
   regex = "1"
   ```
3. Update the config loading pipeline in `loader.rs` to call `expand_env_vars()` after parsing the raw JSON value but before deserializing into `AisopodConfig`:
   ```rust
   pub fn load_config_json5(path: &Path) -> Result<AisopodConfig> {
       let contents = std::fs::read_to_string(path)?;
       let mut value: serde_json::Value = json5::from_str(&contents)?;
       crate::env::expand_env_vars(&mut value)?;
       let config: AisopodConfig = serde_json::from_value(value)?;
       Ok(config)
   }
   ```
4. Declare the module in `lib.rs`:
   ```rust
   pub mod env;
   ```
5. Add unit tests in `crates/aisopod-config/src/env.rs`:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       use serde_json::json;

       #[test]
       fn test_expand_set_variable() {
           env::set_var("TEST_HOST", "localhost");
           let mut val = json!({"host": "${TEST_HOST}"});
           expand_env_vars(&mut val).unwrap();
           assert_eq!(val["host"], "localhost");
       }

       #[test]
       fn test_expand_with_default() {
           env::remove_var("UNSET_VAR");
           let mut val = json!({"port": "${UNSET_VAR:-8080}"});
           expand_env_vars(&mut val).unwrap();
           assert_eq!(val["port"], "8080");
       }

       #[test]
       fn test_error_on_unset_required() {
           env::remove_var("REQUIRED_VAR");
           let mut val = json!({"key": "${REQUIRED_VAR}"});
           assert!(expand_env_vars(&mut val).is_err());
       }
   }
   ```
6. Run `cargo test -p aisopod-config` to verify all tests pass.

## Dependencies
017

## Acceptance Criteria
- [ ] `expand_env_vars()` resolves `${VAR}` patterns using environment variables
- [ ] `${VAR:-default}` syntax provides fallback values for unset variables
- [ ] Unset required variables (no default) produce a clear error message
- [ ] Multiple substitutions in a single string are handled correctly
- [ ] Nested structures (objects, arrays) are recursively processed
- [ ] Unit tests verify substitution with set, unset, and default variables
- [ ] The function is integrated into the config loading pipeline

---
*Created: 2026-02-15*
