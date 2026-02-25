# Issue 148: Implement API Token Generation and Password Hashing

## Summary
Implement secure random API token generation for authentication and password hashing using the argon2 crate. Support token rotation so existing tokens can be replaced without downtime.

## Location
- Crate: `aisopod-gateway`
- File: `crates/aisopod-gateway/src/auth/tokens.rs` (new), `crates/aisopod-gateway/src/auth/password.rs` (new)

## Current Behavior
The gateway authentication system (Issue 031) supports bearer token and password modes, but tokens are manually configured and passwords are stored or compared in plain text. There is no secure token generation or password hashing.

## Expected Behavior
After this issue is completed:
- API tokens are generated using cryptographically secure random bytes, encoded as URL-safe base64 strings.
- Token rotation allows replacing an active token with a new one while briefly accepting both during a grace period.
- Passwords are hashed using argon2id (via the `argon2` crate) before storage.
- Password verification compares a plaintext candidate against a stored argon2 hash.
- A CLI helper or utility function generates new tokens and password hashes for configuration.

## Impact
Without secure token generation, tokens may be predictable. Without password hashing, credentials stored in config files are exposed in plaintext — a critical security risk if config files are leaked or version-controlled.

## Suggested Implementation

1. **Add the `argon2` and `rand` dependencies** to `crates/aisopod-gateway/Cargo.toml`:
   ```toml
   [dependencies]
   argon2 = "0.5"
   rand = "0.8"
   base64 = "0.22"
   ```

2. **Implement token generation** in `crates/aisopod-gateway/src/auth/tokens.rs`:
   ```rust
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
   pub struct TokenStore {
       active_token: String,
       previous_token: Option<String>,
   }

   impl TokenStore {
       pub fn new(token: String) -> Self {
           Self {
               active_token: token,
               previous_token: None,
           }
       }

       /// Rotate to a new token. The old token remains valid
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
   ```

3. **Implement password hashing** in `crates/aisopod-gateway/src/auth/password.rs`:
   ```rust
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
   ```

4. **Wire into the auth system** from Issue 031:
   ```rust
   // In auth middleware, update the password verification path:
   AuthMode::Password { hash } => {
       let (user, pass) = extract_basic_auth(request)?;
       if !password::verify_password(pass, hash)? {
           return Err(AuthError::InvalidCredentials);
       }
   }

   // In auth middleware, update the token verification path:
   AuthMode::Token { store } => {
       let bearer = extract_bearer_token(request)?;
       if !store.validate(bearer) {
           return Err(AuthError::InvalidToken);
       }
   }
   ```

5. **Add unit tests:**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_generate_token_length() {
           let token = generate_token();
           // 32 bytes → 43 chars in base64 (URL_SAFE_NO_PAD)
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

       #[test]
       fn test_password_hash_and_verify() {
           let hash = hash_password("my-secret").unwrap();
           assert!(verify_password("my-secret", &hash).unwrap());
           assert!(!verify_password("wrong-password", &hash).unwrap());
       }

       #[test]
       fn test_password_hash_unique_salts() {
           let h1 = hash_password("same-password").unwrap();
           let h2 = hash_password("same-password").unwrap();
           assert_ne!(h1, h2); // Different salts produce different hashes
       }
   }
   ```

## Dependencies
- Issue 031 (gateway authentication system)

## Acceptance Criteria
- [ ] API tokens are generated using cryptographically secure random bytes (256-bit)
- [ ] Tokens are encoded as URL-safe base64 strings
- [ ] Token rotation supports a grace period where both old and new tokens are valid
- [ ] Previous tokens can be explicitly expired
- [ ] Token comparison uses constant-time equality to prevent timing attacks
- [ ] Passwords are hashed using argon2id with random salts
- [ ] Password verification correctly accepts valid and rejects invalid passwords
- [ ] Unit tests cover token generation, rotation, expiration, and password hashing/verification

---
## Resolution

This issue has been implemented with the following changes:

1. **Added dependencies** to `crates/aisopod-gateway/Cargo.toml`:
   - `argon2 = { version = "0.5", features = ["password-hash"] }`
   - `rand = "0.8"`
   - `base64 = "0.22"` (already present)

2. **Created `tokens.rs`** (`crates/aisopod-gateway/src/auth/tokens.rs`):
   - Implemented `generate_token()` using cryptographically secure random bytes (256-bit)
   - Implemented `TokenStore` struct with:
     - `new()` for initialization
     - `rotate()` for token rotation with grace period
     - `validate()` using constant-time comparison to prevent timing attacks
     - `expire_previous()` to explicitly expire old tokens

3. **Created `password.rs`** (`crates/aisopod-gateway/src/auth/password.rs`):
   - Implemented `hash_password()` using argon2id with random salts
   - Implemented `verify_password()` for password verification against stored hashes
   - Uses `SaltString::generate(&mut OsRng)` for cryptographically secure random salts

4. **Updated `auth.rs`** (`crates/aisopod-gateway/src/auth.rs`):
   - Added module declarations: `mod password;` and `mod tokens;`
   - Re-exported functions: `pub use password::{hash_password, verify_password};` and `pub use tokens::{generate_token, TokenStore};`
   - Added `validate_password_hash()` for validating passwords against stored hashes
   - Added `validate_token_with_store()` for validating tokens using TokenStore

5. **Updated middleware/auth.rs** (`crates/aisopod-gateway/src/middleware/auth.rs`):
   - Imported new modules: `use crate::auth::{hash_password, verify_password, TokenStore};`
   - Added `passwords_hashed` flag to detect if passwords are hashed
   - Updated `validate_basic()` to use `verify_password()` when passwords are hashed
   - Updated `validate_token()` to use `TokenStore` for rotation support
   - Token store is created from the first token credential for rotation support

The implementation supports:
- Cryptographically secure token generation (256-bit random bytes, URL-safe base64 encoded)
- Token rotation with grace period where both old and new tokens are valid
- Explicit expiration of previous tokens
- Constant-time comparison to prevent timing attacks
- Argon2id password hashing with random salts
- Automatic detection of hashed vs plain text passwords

---
*Created: 2026-02-15*
*Resolved: 2026-02-24*
