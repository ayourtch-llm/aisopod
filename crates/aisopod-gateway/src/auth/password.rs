//! Password hashing and verification
//!
//! This module provides functionality for hashing passwords using argon2id
//! and verifying passwords against stored hashes.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// Hash a password using argon2id.
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

/// Verify a password against a stored argon2 hash.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_valid() {
        let hash = hash_password("my-secret").unwrap();
        // Hash should start with $argon2id
        assert!(hash.starts_with("$argon2id"));
    }

    #[test]
    fn test_verify_password_valid() {
        let hash = hash_password("my-secret").unwrap();
        assert!(verify_password("my-secret", &hash).unwrap());
    }

    #[test]
    fn test_verify_password_invalid() {
        let hash = hash_password("my-secret").unwrap();
        assert!(!verify_password("wrong-password", &hash).unwrap());
    }

    #[test]
    fn test_hash_unique_salts() {
        let h1 = hash_password("same-password").unwrap();
        let h2 = hash_password("same-password").unwrap();
        assert_ne!(h1, h2); // Different salts produce different hashes
    }

    #[test]
    fn test_verify_empty_password() {
        let hash = hash_password("").unwrap();
        assert!(verify_password("", &hash).unwrap());
        assert!(!verify_password("non-empty", &hash).unwrap());
    }

    #[test]
    fn test_verify_corrupted_hash() {
        assert!(verify_password("password", "not-a-valid-hash").is_err());
    }
}
