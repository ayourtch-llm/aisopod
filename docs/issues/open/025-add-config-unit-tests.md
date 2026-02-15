# Issue 025: Add Config Unit Tests

## Summary
Create a comprehensive test suite for the `aisopod-config` crate covering config parsing (JSON5 and TOML), validation, environment variable substitution, include directive processing, sensitive field handling, and default config generation. Include test fixtures with both valid and invalid sample configurations.

## Location
- Crate: `aisopod-config`
- File: `crates/aisopod-config/tests/`, `crates/aisopod-config/tests/fixtures/`

## Current Behavior
Individual modules may have some inline unit tests added during their implementation (Issues 016–024), but there is no unified, comprehensive test suite and no organized collection of test fixtures covering edge cases.

## Expected Behavior
- A `tests/` directory contains integration tests organized by feature area
- A `tests/fixtures/` directory contains sample config files (valid and invalid) in both JSON5 and TOML formats
- Tests cover all major features: parsing, validation, env substitution, includes, sensitive fields, generation, and hot reload
- Edge cases are tested: empty configs, missing fields, invalid types, circular includes, unset env vars, boundary values
- All tests pass and provide good coverage of the configuration system

## Impact
A thorough test suite ensures the configuration system is reliable and prevents regressions as the codebase evolves. It serves as living documentation of expected behavior and edge cases, giving contributors confidence to make changes.

## Suggested Implementation
1. Create the test fixtures directory `crates/aisopod-config/tests/fixtures/` if it does not already exist.
2. Create valid fixture files:
   - `tests/fixtures/valid_minimal.json5` — A minimal valid config with only required fields:
     ```json5
     {
       meta: { version: "1.0" },
     }
     ```
   - `tests/fixtures/valid_full.json5` — A complete config with all sections populated
   - `tests/fixtures/valid_minimal.toml` — TOML equivalent of the minimal config:
     ```toml
     [meta]
     version = "1.0"
     ```
   - `tests/fixtures/valid_full.toml` — TOML equivalent of the full config
3. Create invalid fixture files:
   - `tests/fixtures/invalid_port.json5` — Config with port `99999`:
     ```json5
     {
       gateway: { port: 99999 },
     }
     ```
   - `tests/fixtures/invalid_syntax.json5` — Malformed JSON5 (unclosed brace)
   - `tests/fixtures/invalid_unknown_field.json5` — Config with an unknown top-level key (for `deny_unknown_fields` testing)
4. Create include-related fixtures:
   - `tests/fixtures/with_include.json5` — Config referencing a fragment
   - `tests/fixtures/fragment_agents.json5` — An agent config fragment
   - `tests/fixtures/circular_a.json5` and `tests/fixtures/circular_b.json5` — Circular include pair
5. Create env-var fixture:
   - `tests/fixtures/with_env_vars.json5`:
     ```json5
     {
       gateway: {
         host: "${TEST_CONFIG_HOST:-localhost}",
         port: 8080,
       },
     }
     ```
6. Create integration test files:
   - `tests/parsing.rs` — Tests for JSON5 and TOML parsing:
     ```rust
     use std::path::PathBuf;
     use aisopod_config::load_config;

     #[test]
     fn test_parse_minimal_json5() {
         let config = load_config(&PathBuf::from("tests/fixtures/valid_minimal.json5")).unwrap();
         assert_eq!(config.meta.version, "1.0");
     }

     #[test]
     fn test_parse_minimal_toml() {
         let config = load_config(&PathBuf::from("tests/fixtures/valid_minimal.toml")).unwrap();
         assert_eq!(config.meta.version, "1.0");
     }

     #[test]
     fn test_parse_invalid_syntax_fails() {
         let result = load_config(&PathBuf::from("tests/fixtures/invalid_syntax.json5"));
         assert!(result.is_err());
     }

     #[test]
     fn test_unsupported_extension_fails() {
         let result = load_config(&PathBuf::from("tests/fixtures/config.yaml"));
         assert!(result.is_err());
     }
     ```
   - `tests/validation.rs` — Tests for semantic validation:
     ```rust
     use aisopod_config::AisopodConfig;

     #[test]
     fn test_default_config_validates() {
         let config = AisopodConfig::default();
         assert!(config.validate().is_ok());
     }

     #[test]
     fn test_invalid_port_fails_validation() {
         let mut config = AisopodConfig::default();
         config.gateway.port = 99999;
         let errors = config.validate().unwrap_err();
         assert!(errors.iter().any(|e| e.path.contains("port")));
     }

     #[test]
     fn test_port_zero_fails_validation() {
         let mut config = AisopodConfig::default();
         config.gateway.port = 0;
         let errors = config.validate().unwrap_err();
         assert!(errors.iter().any(|e| e.path.contains("port")));
     }
     ```
   - `tests/env_substitution.rs` — Tests for env var expansion:
     ```rust
     use serde_json::json;
     use aisopod_config::env::expand_env_vars;

     #[test]
     fn test_nested_object_expansion() {
         std::env::set_var("NESTED_TEST", "value");
         let mut val = json!({"a": {"b": {"c": "${NESTED_TEST}"}}});
         expand_env_vars(&mut val).unwrap();
         assert_eq!(val["a"]["b"]["c"], "value");
         std::env::remove_var("NESTED_TEST");
     }

     #[test]
     fn test_array_expansion() {
         std::env::set_var("ARR_TEST", "item");
         let mut val = json!(["${ARR_TEST}", "literal"]);
         expand_env_vars(&mut val).unwrap();
         assert_eq!(val[0], "item");
         assert_eq!(val[1], "literal");
         std::env::remove_var("ARR_TEST");
     }

     #[test]
     fn test_multiple_vars_in_one_string() {
         std::env::set_var("MULTI_A", "hello");
         std::env::set_var("MULTI_B", "world");
         let mut val = json!({"msg": "${MULTI_A} ${MULTI_B}"});
         expand_env_vars(&mut val).unwrap();
         assert_eq!(val["msg"], "hello world");
         std::env::remove_var("MULTI_A");
         std::env::remove_var("MULTI_B");
     }
     ```
   - `tests/includes.rs` — Tests for @include directive processing
   - `tests/sensitive.rs` — Tests for Sensitive<T> redaction
   - `tests/generation.rs` — Tests for default config generation round-trip
7. Run `cargo test -p aisopod-config` and verify all tests pass.
8. Review test output for any uncovered edge cases and add additional tests as needed.

## Dependencies
016, 017, 018, 019, 020, 021, 022, 023, 024

## Acceptance Criteria
- [ ] Test fixtures directory contains valid and invalid sample configs in both JSON5 and TOML formats
- [ ] Integration tests exist for config parsing (both formats), validation, env substitution, includes, sensitive fields, and generation
- [ ] Edge cases are covered: empty configs, boundary values, circular includes, unset env vars, invalid syntax
- [ ] All tests pass when running `cargo test -p aisopod-config`
- [ ] Tests serve as documentation of expected behavior for each config feature
- [ ] Test coverage includes both happy paths and error cases

---
*Created: 2026-02-15*
