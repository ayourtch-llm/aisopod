# Issue 021: Implement Config Validation

## Summary
Add a `validate()` method on `AisopodConfig` that checks semantic rules beyond what serde deserialization provides. Validate constraints such as port ranges (1–65535), valid model format strings, non-empty required fields, and produce detailed error messages that include the path to the invalid field.

## Location
- Crate: `aisopod-config`
- File: `crates/aisopod-config/src/validation.rs`

## Current Behavior
Configuration is only validated structurally by serde deserialization (correct types, known fields). There is no semantic validation — for example, a port value of `99999` or an empty agent name would be accepted without error.

## Expected Behavior
- `AisopodConfig::validate() -> Result<(), Vec<ValidationError>>` checks all semantic rules
- `ValidationError` includes the field path (e.g., `"gateway.port"`) and a human-readable message
- Validation rules include:
  - Port numbers in range 1–65535
  - Non-empty required string fields (e.g., agent names, model IDs)
  - Valid format for model identifier strings
  - No duplicate agent names or binding IDs
- All violations are collected (not fail-fast) so users can fix multiple issues at once
- Validation is called in the config loading pipeline after deserialization

## Impact
Semantic validation catches configuration errors early with clear, actionable messages. Without it, invalid configs would cause cryptic runtime errors deep in application logic, making debugging difficult for users.

## Suggested Implementation
1. Create `crates/aisopod-config/src/validation.rs`:
   ```rust
   use crate::types::AisopodConfig;
   use std::fmt;

   #[derive(Debug, Clone)]
   pub struct ValidationError {
       pub path: String,
       pub message: String,
   }

   impl fmt::Display for ValidationError {
       fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
           write!(f, "{}: {}", self.path, self.message)
       }
   }

   impl AisopodConfig {
       /// Validate semantic rules across the entire configuration.
       pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
           let mut errors = Vec::new();

           self.validate_gateway(&mut errors);
           self.validate_agents(&mut errors);
           self.validate_models(&mut errors);
           self.validate_meta(&mut errors);

           if errors.is_empty() {
               Ok(())
           } else {
               Err(errors)
           }
       }

       fn validate_gateway(&self, errors: &mut Vec<ValidationError>) {
           let port = self.gateway.port;
           if port < 1 || port > 65535 {
               errors.push(ValidationError {
                   path: "gateway.port".to_string(),
                   message: format!(
                       "Port must be between 1 and 65535, got {}",
                       port
                   ),
               });
           }
       }

       fn validate_agents(&self, _errors: &mut Vec<ValidationError>) {
           // Check for duplicate agent names, non-empty names, etc.
       }

       fn validate_models(&self, _errors: &mut Vec<ValidationError>) {
           // Check model format strings, provider references, etc.
       }

       fn validate_meta(&self, errors: &mut Vec<ValidationError>) {
           if self.meta.version.is_empty() {
               errors.push(ValidationError {
                   path: "meta.version".to_string(),
                   message: "Version must not be empty".to_string(),
               });
           }
       }
   }
   ```
2. Declare the module in `lib.rs`:
   ```rust
   pub mod validation;
   pub use validation::ValidationError;
   ```
3. Integrate validation into the config loading pipeline in `loader.rs`:
   ```rust
   pub fn load_config(path: &Path) -> Result<AisopodConfig> {
       // ... parse, expand env vars, process includes ...
       let config: AisopodConfig = serde_json::from_value(value)?;
       config.validate().map_err(|errs| {
           let messages: Vec<String> = errs.iter().map(|e| e.to_string()).collect();
           anyhow::anyhow!("Config validation failed:\n  {}", messages.join("\n  "))
       })?;
       Ok(config)
   }
   ```
4. Add unit tests:
   ```rust
   #[cfg(test)]
   mod tests {
       use crate::types::AisopodConfig;

       #[test]
       fn test_default_config_is_valid() {
           let config = AisopodConfig::default();
           assert!(config.validate().is_ok());
       }

       #[test]
       fn test_invalid_port_detected() {
           let mut config = AisopodConfig::default();
           config.gateway.port = 99999;
           let errors = config.validate().unwrap_err();
           assert!(errors.iter().any(|e| e.path == "gateway.port"));
       }
   }
   ```
5. Run `cargo test -p aisopod-config` to verify all tests pass.

## Dependencies
016

## Acceptance Criteria
- [ ] `AisopodConfig::validate()` exists and checks semantic rules
- [ ] `ValidationError` includes the field path and a human-readable message
- [ ] Port range validation catches invalid ports
- [ ] Empty required fields are flagged
- [ ] All violations are collected, not fail-fast
- [ ] Default config passes validation
- [ ] Validation is integrated into the config loading pipeline
- [ ] Unit tests verify both valid and invalid configurations

---
*Created: 2026-02-15*
