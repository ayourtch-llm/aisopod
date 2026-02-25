//! API token generation and management
//!
//! This module provides functionality for generating cryptographically secure
//! API tokens and managing token rotation with a grace period.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;

const TOKEN_BYTES: usize = 32; // 256-bit tokens

/// Generate a cryptographically secure API token.
pub fn generate_token() -> String {
    let mut bytes = vec![0u8; TOKEN_BYTES];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(&bytes)
}

/// Token store supporting rotation with a grace period.
#[derive(Debug, Clone)]
pub struct TokenStore {
    active_token: String,
    previous_token: Option<String>,
}

impl TokenStore {
    /// Create a new token store with the given initial token.
    pub fn new(token: String) -> Self {
        Self {
            active_token: token,
            previous_token: None,
        }
    }

    /// Generate a new token and rotate to it. The old token remains valid
    /// until the next rotation or until `expire_previous()` is called.
    pub fn rotate(&mut self) -> String {
        let new_token = generate_token();
        self.previous_token = Some(std::mem::replace(
            &mut self.active_token,
            new_token.clone(),
        ));
        new_token
    }

    /// Check if a token is valid (matches active or previous).
    pub fn validate(&self, candidate: &str) -> bool {
        if constant_time_eq(candidate.as_bytes(), self.active_token.as_bytes()) {
            return true;
        }
        if let Some(ref prev) = self.previous_token {
            return constant_time_eq(candidate.as_bytes(), prev.as_bytes());
        }
        false
    }

    /// Expire the previous token so only the active token is valid.
    pub fn expire_previous(&mut self) {
        self.previous_token = None;
    }
}

/// Constant-time byte comparison to prevent timing attacks.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token_length() {
        let token = generate_token();
        // 32 bytes -> 43 chars in base64 (URL_SAFE_NO_PAD)
        assert_eq!(token.len(), 43);
    }

    #[test]
    fn test_generate_token_uniqueness() {
        let t1 = generate_token();
        let t2 = generate_token();
        assert_ne!(t1, t2);
    }

    #[test]
    fn test_token_rotation() {
        let mut store = TokenStore::new("old-token".to_string());
        let new_token = store.rotate();

        assert!(store.validate(&new_token));
        assert!(store.validate("old-token")); // Previous still valid
        assert!(!store.validate("random-token"));

        store.expire_previous();
        assert!(!store.validate("old-token")); // Previous expired
    }
}
