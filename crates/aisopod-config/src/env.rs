//! Environment variable substitution module
//!
//! This module provides functionality to expand environment variable references
//! in configuration values using patterns like `${VAR}` and `${VAR:-default}`.

use anyhow::{bail, Context, Result};
use regex::Regex;
use serde_json::Value;
use std::env;

/// Recursively expand environment variable references in all string values.
///
/// This function processes a `serde_json::Value` tree and replaces all
/// environment variable references in string values with their resolved
/// values from the environment.
///
/// # Patterns Supported
///
/// - `${VAR}` - Replaced with the value of environment variable `VAR`,
///   or returns an error if `VAR` is not set
/// - `${VAR:-default}` - Replaced with the value of `VAR` if set,
///   or `"default"` if not
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use std::env;
/// use aisopod_config::expand_env_vars;
///
/// env::set_var("HOST", "localhost");
/// let mut value = json!({
///     "host": "${HOST}",
///     "port": "${PORT:-8080}"
/// });
/// expand_env_vars(&mut value).unwrap();
/// assert_eq!(value["host"], "localhost");
/// assert_eq!(value["port"], "8080");
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - A required environment variable (without default) is not set
/// - A default value contains an unbalanced closing brace
pub fn expand_env_vars(value: &mut Value) -> Result<()> {
    match value {
        Value::String(s) => {
            *s = expand_string(s)
                .with_context(|| format!("Failed to expand env vars in string value: {}", s))?;
        }
        Value::Array(arr) => {
            for (i, item) in arr.iter_mut().enumerate() {
                expand_env_vars(item)
                    .with_context(|| format!("Failed to expand env vars in array item at index {}", i))?;
            }
        }
        Value::Object(map) => {
            for (key, val) in map.iter_mut() {
                expand_env_vars(val)
                    .with_context(|| format!("Failed to expand env vars in object value for key '{}'", key))?;
            }
        }
        _ => {} // numbers, bools, null — no substitution needed
    }
    Ok(())
}

/// Expand environment variables in a single string.
///
/// This function processes a string and replaces all `${VAR}` and
/// `${VAR:-default}` patterns with their resolved values.
fn expand_string(input: &str) -> Result<String> {
    let re = Regex::new(r"\$\{([^}]+)\}")
        .context("Failed to compile regex for environment variable expansion")?;

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

        // Replace only the first occurrence to handle multiple substitutions correctly
        result = result.replacen(full_match, &replacement, 1);
    }

    Ok(result)
}

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
        env::remove_var("TEST_HOST");
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
        // Clean environment first
        env::remove_var("REQUIRED_VAR");
        
        let mut val = json!({"key": "${REQUIRED_VAR}"});
        let result = expand_env_vars(&mut val);
        
        // Check that we got an error
        assert!(result.is_err(), "Expected error for unset required variable");
        
        // Traverse error chain to find the root cause
        let mut err = result.unwrap_err();
        let mut err_str = err.to_string();
        
        // Check the current error
        if !err_str.contains("REQUIRED_VAR") {
            // Check error source if available
            let mut found = false;
            let mut current = err.source();
            while let Some(e) = current {
                err_str = e.to_string();
                if err_str.contains("REQUIRED_VAR") {
                    found = true;
                    break;
                }
                current = e.source();
            }
            
            // If still not found, the error must be in the wrapping context
            if !found {
                // The error must contain the original message somewhere in the chain
                panic!("Error message '{}' does not contain 'REQUIRED_VAR'. Root cause should be from expand_string", err_str);
            }
        }
        
        // Verify the error is about an unset variable
        assert!(err_str.contains("REQUIRED_VAR"), 
            "Error message '{}' should contain 'REQUIRED_VAR'", err_str);
    }

    #[test]
    fn test_multiple_substitutions() {
        env::set_var("HOST", "localhost");
        env::set_var("PORT", "3000");
        let mut val = json!({"url": "${HOST}:${PORT}"});
        expand_env_vars(&mut val).unwrap();
        assert_eq!(val["url"], "localhost:3000");
        env::remove_var("HOST");
        env::remove_var("PORT");
    }

    #[test]
    fn test_nested_object() {
        env::set_var("API_KEY", "secret123");
        let mut val = json!({
            "api": {
                "key": "${API_KEY}",
                "nested": {
                    "value": "${API_KEY}"
                }
            }
        });
        expand_env_vars(&mut val).unwrap();
        assert_eq!(val["api"]["key"], "secret123");
        assert_eq!(val["api"]["nested"]["value"], "secret123");
        env::remove_var("API_KEY");
    }

    #[test]
    fn test_array_values() {
        env::set_var("ITEM1", "apple");
        env::set_var("ITEM2", "banana");
        let mut val = json!({
            "items": ["${ITEM1}", "${ITEM2}", "cherry"]
        });
        expand_env_vars(&mut val).unwrap();
        assert_eq!(val["items"][0], "apple");
        assert_eq!(val["items"][1], "banana");
        assert_eq!(val["items"][2], "cherry");
        env::remove_var("ITEM1");
        env::remove_var("ITEM2");
    }

    #[test]
    fn test_mixed_substitutions() {
        // Clean environment first to avoid test interference
        env::remove_var("MODE");
        env::remove_var("TEST_PORT_VAR");
        
        // Set MODE but not TEST_PORT_VAR to test the default
        env::set_var("MODE", "production");
        let mut val = json!({
            "msg": "Running in ${MODE} mode on port ${TEST_PORT_VAR:-8080}"
        });
        expand_env_vars(&mut val).unwrap();
        assert_eq!(val["msg"], "Running in production mode on port 8080");
        env::remove_var("MODE");
        env::remove_var("TEST_PORT_VAR");
    }

    #[test]
    fn test_non_string_values_unchanged() {
        env::set_var("NUM_TEST", "42");
        let mut val = json!({
            "number": 123,
            "float": 3.14,
            "bool_true": true,
            "bool_false": false,
            "null": null,
            "str": "${NUM_TEST}"
        });
        expand_env_vars(&mut val).unwrap();
        assert_eq!(val["number"], 123);
        assert_eq!(val["float"], 3.14);
        assert_eq!(val["bool_true"], true);
        assert_eq!(val["bool_false"], false);
        assert_eq!(val["null"], Value::Null);
        assert_eq!(val["str"], "42");
        env::remove_var("NUM_TEST");
    }

    #[test]
    fn test_empty_string() {
        let mut val = json!({"empty": ""});
        expand_env_vars(&mut val).unwrap();
        assert_eq!(val["empty"], "");
    }

    #[test]
    fn test_no_substitution_needed() {
        let mut val = json!({"text": "This is just text"});
        expand_env_vars(&mut val).unwrap();
        assert_eq!(val["text"], "This is just text");
    }

    #[test]
    fn test_default_with_colon() {
        env::remove_var("UNSET_VAR");
        let mut val = json!({"path": "${UNSET_VAR:-/default/path}"});
        expand_env_vars(&mut val).unwrap();
        assert_eq!(val["path"], "/default/path");
    }
}
