//! Integration tests for environment variable substitution
//!
//! These tests verify that environment variables in configuration
//! are properly expanded with various patterns and edge cases.

use aisopod_config::env::expand_env_vars;
use serde_json::json;
use std::env;

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

    assert!(
        result.is_err(),
        "Expected error for unset required variable"
    );
    let err = result.unwrap_err();
    // Check that the error or its source mentions the variable
    let err_str = err.to_string();
    let has_variable_name = err_str.contains("REQUIRED_VAR")
        || err
            .source()
            .map(|e| e.to_string().contains("REQUIRED_VAR"))
            .unwrap_or(false);
    assert!(
        has_variable_name,
        "Error should mention the unset variable, got: {}",
        err_str
    );
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
fn test_nested_object_expansion() {
    env::set_var("NESTED_TEST", "value");
    let mut val = json!({"a": {"b": {"c": "${NESTED_TEST}"}}});
    expand_env_vars(&mut val).unwrap();
    assert_eq!(val["a"]["b"]["c"], "value");
    env::remove_var("NESTED_TEST");
}

#[test]
fn test_array_expansion() {
    env::set_var("ARR_TEST", "item");
    let mut val = json!(["${ARR_TEST}", "literal"]);
    expand_env_vars(&mut val).unwrap();
    assert_eq!(val[0], "item");
    assert_eq!(val[1], "literal");
    env::remove_var("ARR_TEST");
}

#[test]
fn test_multiple_vars_in_one_string() {
    env::set_var("MULTI_A", "hello");
    env::set_var("MULTI_B", "world");
    let mut val = json!({"msg": "${MULTI_A} ${MULTI_B}"});
    expand_env_vars(&mut val).unwrap();
    assert_eq!(val["msg"], "hello world");
    env::remove_var("MULTI_A");
    env::remove_var("MULTI_B");
}

#[test]
fn test_default_with_colon_in_value() {
    env::remove_var("UNSET_VAR");
    let mut val = json!({"path": "${UNSET_VAR:-/default/path:with:colons}"});
    expand_env_vars(&mut val).unwrap();
    assert_eq!(val["path"], "/default/path:with:colons");
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
    assert_eq!(val["null"], serde_json::Value::Null);
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
fn test_deeply_nested_expansion() {
    env::set_var("DEEP_VALUE", "found");
    let mut val = json!({
        "level1": {
            "level2": {
                "level3": {
                    "level4": {
                        "value": "${DEEP_VALUE}"
                    }
                }
            }
        }
    });
    expand_env_vars(&mut val).unwrap();
    assert_eq!(
        val["level1"]["level2"]["level3"]["level4"]["value"],
        "found"
    );
    env::remove_var("DEEP_VALUE");
}

#[test]
fn test_array_of_objects_with_env_vars() {
    env::set_var("API_HOST", "api.example.com");
    let mut val = json!({
        "endpoints": [
            {"url": "${API_HOST}/v1", "name": "v1"},
            {"url": "${API_HOST}/v2", "name": "v2"}
        ]
    });
    expand_env_vars(&mut val).unwrap();
    assert_eq!(val["endpoints"][0]["url"], "api.example.com/v1");
    assert_eq!(val["endpoints"][1]["url"], "api.example.com/v2");
    env::remove_var("API_HOST");
}

#[test]
fn test_mixed_defaults_and_required_vars() {
    env::set_var("REQUIRED", "present");
    let mut val = json!({
        "required": "${REQUIRED}",
        "optional": "${OPTIONAL:-default}"
    });
    expand_env_vars(&mut val).unwrap();
    assert_eq!(val["required"], "present");
    assert_eq!(val["optional"], "default");
    env::remove_var("REQUIRED");
}

#[test]
fn test_env_var_with_special_chars() {
    env::set_var("PATH_VAR", "/some/path/with-dashes_and_underscores");
    let mut val = json!({"path": "${PATH_VAR}"});
    expand_env_vars(&mut val).unwrap();
    assert_eq!(val["path"], "/some/path/with-dashes_and_underscores");
    env::remove_var("PATH_VAR");
}

#[test]
fn test_default_with_empty_string() {
    env::remove_var("UNSET_VAR");
    let mut val = json!({"value": "${UNSET_VAR:-}"});
    expand_env_vars(&mut val).unwrap();
    assert_eq!(val["value"], "");
}

#[test]
fn test_multiple_defaults_in_one_value() {
    env::remove_var("FIRST");
    env::remove_var("SECOND");
    let mut val = json!({
        "msg": "First: ${FIRST:-default1}, Second: ${SECOND:-default2}"
    });
    expand_env_vars(&mut val).unwrap();
    assert_eq!(val["msg"], "First: default1, Second: default2");
}

#[test]
fn test_consecutive_substitutions() {
    env::set_var("A", "x");
    env::set_var("B", "y");
    let mut val = json!({"msg": "${A}${A}${B}${B}"});
    expand_env_vars(&mut val).unwrap();
    assert_eq!(val["msg"], "xxyy");
    env::remove_var("A");
    env::remove_var("B");
}

#[test]
fn test_complex_nested_structure() {
    env::set_var("ENV_VAR", "expanded");
    let mut val = json!({
        "outer": {
            "inner": [
                {"key": "value1"},
                {"key": "${ENV_VAR}"},
                {"nested": {"deep": "${ENV_VAR}"}}
            ]
        }
    });
    expand_env_vars(&mut val).unwrap();
    assert_eq!(val["outer"]["inner"][1]["key"], "expanded");
    assert_eq!(val["outer"]["inner"][2]["nested"]["deep"], "expanded");
    env::remove_var("ENV_VAR");
}

#[test]
fn test_env_var_in_array_indices() {
    env::set_var("INDEX_VAR", "test");
    let mut val = json!({
        "array": ["first", "second", "third"]
    });
    expand_env_vars(&mut val).unwrap();
    // Should not modify array values
    assert_eq!(val["array"][0], "first");
    assert_eq!(val["array"][1], "second");
    assert_eq!(val["array"][2], "third");
    env::remove_var("INDEX_VAR");
}

#[test]
fn test_substitution_with_numeric_string() {
    env::set_var("PORT_STR", "8080");
    let mut val = json!({"port": "${PORT_STR}"});
    expand_env_vars(&mut val).unwrap();
    // In JSON, this will be a string value
    assert_eq!(val["port"], "8080");
    env::remove_var("PORT_STR");
}

#[test]
fn test_very_long_default_value() {
    env::remove_var("UNSET_VAR");
    let long_default = "a".repeat(1000);
    let mut val = json!({"value": "${UNSET_VAR:-".to_owned() + &long_default + "}"});
    expand_env_vars(&mut val).unwrap();
    assert_eq!(val["value"], long_default);
}

#[test]
fn test_concurrent_var_substitution() {
    env::set_var("VAR1", "a");
    env::set_var("VAR2", "b");
    env::set_var("VAR3", "c");
    let mut val = json!({
        "result": "${VAR1}${VAR2}${VAR3}"
    });
    expand_env_vars(&mut val).unwrap();
    assert_eq!(val["result"], "abc");
    env::remove_var("VAR1");
    env::remove_var("VAR2");
    env::remove_var("VAR3");
}
