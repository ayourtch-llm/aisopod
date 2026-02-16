# Issue 022: Implement Sensitive Field Handling

## Summary
Create a `Sensitive<T>` wrapper type that redacts its contents when displayed via `Display` or `Debug` traits. Use this wrapper to mark API keys, tokens, and passwords in configuration types so that logging and UI serialization never expose secrets.

## Location
- Crate: `aisopod-config`
- File: `crates/aisopod-config/src/sensitive.rs`

## Current Behavior
Sensitive fields like API keys and tokens are stored as plain `String` values. If the configuration is logged or displayed (e.g., via `Debug`), secret values appear in plaintext in logs and error messages.

## Expected Behavior
- `Sensitive<T>` wraps a value and redacts it on `Display` and `Debug` (shows `"***REDACTED***"`)
- `Sensitive<T>` implements `Serialize` and `Deserialize` transparently (serializes/deserializes as the inner type)
- `Sensitive<T>` implements `Clone` and provides `.expose()` to access the inner value
- A serialization option or method masks sensitive fields for UI display (e.g., `serialize_redacted()`)
- `AuthConfig` and other types with secret fields use `Sensitive<String>` for API keys, tokens, and passwords

## Impact
Preventing accidental exposure of secrets in logs, error messages, and debug output is a critical security requirement. Without this, API keys could leak through standard logging, crash reports, or user-facing error messages.

## Suggested Implementation
1. Create `crates/aisopod-config/src/sensitive.rs`:
   ```rust
   use serde::{Deserialize, Deserializer, Serialize, Serializer};
   use std::fmt;

   /// A wrapper that redacts its contents in Display and Debug output.
   #[derive(Clone)]
   pub struct Sensitive<T>(T);

   impl<T> Sensitive<T> {
       pub fn new(value: T) -> Self {
           Self(value)
       }

       /// Access the inner value. Use sparingly and only when the actual
       /// value is needed (e.g., for making API calls).
       pub fn expose(&self) -> &T {
           &self.0
       }
   }

   impl<T> fmt::Display for Sensitive<T> {
       fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
           write!(f, "***REDACTED***")
       }
   }

   impl<T> fmt::Debug for Sensitive<T> {
       fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
           write!(f, "Sensitive(***REDACTED***)")
       }
   }

   impl<T: Serialize> Serialize for Sensitive<T> {
       fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
           self.0.serialize(serializer)
       }
   }

   impl<'de, T: Deserialize<'de>> Deserialize<'de> for Sensitive<T> {
       fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
           T::deserialize(deserializer).map(Sensitive)
       }
   }

   impl<T: Default> Default for Sensitive<T> {
       fn default() -> Self {
           Self(T::default())
       }
   }
   ```
2. Declare the module in `lib.rs`:
   ```rust
   pub mod sensitive;
   pub use sensitive::Sensitive;
   ```
3. Update `AuthConfig` in `crates/aisopod-config/src/types/auth.rs` to use `Sensitive<String>` for secret fields:
   ```rust
   use crate::sensitive::Sensitive;

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct AuthConfig {
       #[serde(default)]
       pub api_keys: Vec<ApiKeyEntry>,
   }

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct ApiKeyEntry {
       pub name: String,
       pub key: Sensitive<String>,
   }
   ```
4. Add a utility function for redacted serialization (for UI):
   ```rust
   impl<T> Sensitive<T> {
       /// Produce a redacted display string, suitable for UI.
       pub fn redacted_display() -> &'static str {
           "***REDACTED***"
       }
   }
   ```
5. Add unit tests:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_display_redacts() {
           let secret = Sensitive::new("my-api-key".to_string());
           assert_eq!(format!("{}", secret), "***REDACTED***");
       }

       #[test]
       fn test_debug_redacts() {
           let secret = Sensitive::new("my-api-key".to_string());
           assert_eq!(format!("{:?}", secret), "Sensitive(***REDACTED***)");
       }

       #[test]
       fn test_expose_returns_inner() {
           let secret = Sensitive::new("my-api-key".to_string());
           assert_eq!(secret.expose(), "my-api-key");
       }

       #[test]
       fn test_serde_roundtrip() {
           let secret = Sensitive::new("my-api-key".to_string());
           let json = serde_json::to_string(&secret).unwrap();
           assert_eq!(json, "\"my-api-key\"");
           let deserialized: Sensitive<String> = serde_json::from_str(&json).unwrap();
           assert_eq!(deserialized.expose(), "my-api-key");
       }
   }
   ```
6. Run `cargo test -p aisopod-config` to verify all tests pass.

## Dependencies
016

## Acceptance Criteria
- [x] `Sensitive<T>` type exists with `Display` and `Debug` redaction
- [x] `Sensitive<T>` implements `Serialize`, `Deserialize`, `Clone`, and `Default`
- [x] `expose()` method provides access to the inner value
- [x] `AuthConfig` uses `Sensitive<String>` for API keys, tokens, and passwords
- [x] `Display` and `Debug` output of config structs containing sensitive fields does not show secret values
- [x] Serde round-trip (serialize then deserialize) preserves the inner value
- [x] Unit tests verify redaction and round-trip behavior

## Resolution

Issue 022 was implemented by the previous agent and committed in commit e4764d658914421a9e4b68c3d1f39e4e5d8f7e5c.

### Changes Made:
1. Created `crates/aisopod-config/src/sensitive.rs` with `Sensitive<T>` wrapper type
2. Implemented `Display` and `Debug` traits that redact contents
3. Implemented `Serialize` and `Deserialize` for transparent serialization
4. Added `Clone`, `Default`, and `expose()` method
5. Updated `AuthConfig` and related types to use `Sensitive<String>` for secrets
6. Added unit tests for the sensitive wrapper

### Implementation Details:
- `Sensitive<T>` wraps any type `T` and redacts it in `Display` and `Debug`
- Serialization/deserialization is transparent - the inner value is serialized
- The `expose()` method provides safe access to the inner value
- All sensitive fields in `AuthConfig` use `Sensitive<String>`

### Verification:
- Git log shows commit e4764d6 with message "Issue 022: Implement Sensitive Field Handling"
- Files created/modified:
  - `crates/aisopod-config/src/sensitive.rs` - New module with Sensitive wrapper
  - `crates/aisopod-config/src/lib.rs` - Exported Sensitive
  - `crates/aisopod-config/src/types/auth.rs` - Updated to use Sensitive for secrets
  - `crates/aisopod-config/src/types/mod.rs` - Exported Sensitive from types

---
*Created: 2026-02-15*
*Resolved: 2026-02-16*
