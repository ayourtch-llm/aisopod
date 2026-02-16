use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// A wrapper that redacts its contents in Display and Debug output.
/// 
/// This type is used to wrap sensitive values like API keys, tokens, and passwords
/// to prevent accidental exposure in logs, error messages, and debug output.
#[derive(Clone, Default)]
pub struct Sensitive<T>(T);

impl<T> Sensitive<T> {
    /// Create a new Sensitive wrapper around a value.
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// Access the inner value.
    /// 
    /// Use sparingly and only when the actual value is needed (e.g., for making API calls).
    pub fn expose(&self) -> &T {
        &self.0
    }

    /// Produce a redacted display string, suitable for UI.
    pub fn redacted_display() -> &'static str {
        "***REDACTED***"
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

    #[test]
    fn test_default() {
        let secret: Sensitive<String> = Sensitive::default();
        assert_eq!(secret.expose(), "");
    }

    #[test]
    fn test_nested_structure() {
        let config = Sensitive::new("secret-value".to_string());
        assert_eq!(format!("{}", config), "***REDACTED***");
        assert_eq!(format!("{:?}", config), "Sensitive(***REDACTED***)");
        assert_eq!(config.expose(), "secret-value");
    }
}
