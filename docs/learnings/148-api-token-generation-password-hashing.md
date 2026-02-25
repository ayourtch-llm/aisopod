# Issue 148: API Token Generation and Password Hashing - Learnings

**Issue Number:** 148  
**Date:** 2026-02-25  
**Category:** Security, Authentication, Cryptography

---

## Overview

This document captures key learnings from implementing Issue #148: API Token Generation and Password Hashing. The implementation focuses on secure authentication mechanisms including cryptographically secure token generation, token rotation, and password hashing with argon2id.

---

## Key Learnings

### 1. Token Generation: Randomness and Encoding

#### Learnings

1. **Use `rand::RngCore` for Cryptographically Secure Tokens**
   - `rand::thread_rng().fill_bytes()` provides cryptographically secure randomness
   - Delegates to OS random source (`/dev/urandom` on Linux, `CryptoRandom` on Windows)
   - Preferred over `rng.gen_range()` for key generation

2. **URL-Safe Base64 Encoding for Tokens**
   - Standard base64 uses `+` and `/` characters which need URL encoding
   - Standard base64 pads with `=` which can cause issues in URLs
   - `base64::URL_SAFE_NO_PAD` is ideal for:
     - API tokens in headers (no padding needed)
     - Tokens in URLs (no special characters)
     - Tokens in cookies (no encoding required)

3. **Token Length Considerations**
   - 256-bit tokens (32 bytes) = 43 characters (URL-safe base64)
   - Provides 2^256 possible tokens ( astronomically large)
   - 128-bit tokens (16 bytes) = 22 characters (sufficient for most cases)
   - 512-bit tokens (64 bytes) = 86 characters (overkill, use 256-bit)

#### Best Practices

```rust
// ✅ GOOD: Cryptographically secure token generation
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;

const TOKEN_BYTES: usize = 32; // 256-bit tokens

pub fn generate_token() -> String {
    let mut bytes = vec![0u8; TOKEN_BYTES];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(&bytes)
}

// ❌ BAD: Using random string generation
pub fn generate_token_bad() -> String {
    // This is NOT cryptographically secure!
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
        .chars()
        .collect();
    let mut rng = rand::thread_rng();
    (0..32)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect()
}
```

---

### 2. Token Rotation: Grace Period Design

#### Learnings

1. **Grace Period Enables Smooth Rotation**
   - Allows clients to update tokens without downtime
   - Critical for production systems with long-running sessions
   - Two valid tokens simultaneously during transition

2. **Simple State Management**
   - Store only `active_token` and `previous_token` (optional)
   - No need for complex rotation history
   - `Option<String>` handles first token case (no previous)

3. **Explicit Expiration Control**
   - `expire_previous()` allows immediate rotation completion
   - Useful when you know all clients have updated
   - Prevents indefinite accumulation of valid tokens

#### Implementation Pattern

```rust
pub struct TokenStore {
    active_token: String,
    previous_token: Option<String>,
}

impl TokenStore {
    /// Create new store with initial token
    pub fn new(token: String) -> Self {
        Self {
            active_token: token,
            previous_token: None,
        }
    }

    /// Rotate to new token, keeping old for grace period
    pub fn rotate(&mut self) -> String {
        let new_token = generate_token();
        self.previous_token = Some(std::mem::replace(
            &mut self.active_token,
            new_token.clone(),
        ));
        new_token
    }

    /// Check if token matches either active or previous
    pub fn validate(&self, candidate: &str) -> bool {
        if constant_time_eq(candidate.as_bytes(), self.active_token.as_bytes()) {
            return true;
        }
        if let Some(ref prev) = self.previous_token {
            return constant_time_eq(candidate.as_bytes(), prev.as_bytes());
        }
        false
    }

    /// Expire previous token immediately
    pub fn expire_previous(&mut self) {
        self.previous_token = None;
    }
}
```

#### Use Cases

1. **Rolling Tokens (Default):**
   ```rust
   let mut store = TokenStore::new(initial_token);
   // Use store.validate() for all requests
   // At any time, call store.rotate() to get new token
   ```

2. **Immediate Rotation:**
   ```rust
   store.rotate(); // New token active
   store.expire_previous(); // Old token invalid immediately
   ```

3. **Scheduled Rotation:**
   ```rust
   // In background task
   tokio::spawn(async move {
       loop {
           tokio::time::sleep(Duration::from_secs(3600)).await;
           store.rotate();
       }
   });
   ```

---

### 3. Constant-Time Comparison: Preventing Timing Attacks

#### Learnings

1. **Standard String Comparison is NOT Safe**
   ```rust
   // ❌ BAD: Early exit on first mismatch
   if a == b { ... } // Leaks timing information
   ```

2. **Timing Attack Vulnerability**
   ```
   Attacker sends tokens: "aaaa...", "aaab...", "aaac..."
   If "aaaax..." takes longer than "aaaay...", attacker knows first 3 chars are "aaa"
   Repeatedly, attacker can recover entire token
   ```

3. **Constant-Time Comparison Implementation**
   ```rust
   fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
       if a.len() != b.len() {
           return false;
       }
       // Always iterate through ALL bytes
       a.iter()
           .zip(b.iter())
           .fold(0u8, |acc, (x, y)| acc | (x ^ y))
           == 0
   }
   ```

#### Why This Works

1. **Length Check First:**
   - Different lengths return immediately (safe - no timing info leaked)
   - Same length ensures same iteration count

2. **XOR and Fold:**
   - XOR returns 0 for same bytes, non-zero for different
   - Bitwise OR accumulates any differences
   - Always processes all bytes (no early exit)

3. **No Branching on Data:**
   - Comparison result doesn't affect loop execution
   - CPU cannot predict branch based on token contents

#### Library Alternatives

```toml
# Option 1: Use constant-time-eq crate
[dependencies]
constant-time-eq = "0.2"

use constant_time_eq::constant_time_eq;

# Option 2: Use crypto-mac crate
[dependencies]
crypto-mac = "0.11"

use crypto_mac::GenericAuth;

# Option 3: Use hmac crate (if already dependency)
[dependencies]
hmac = "0.12"

use hmac::SimpleHmac;
use sha2::Sha256;
```

**Why Implement Manually:**
- No additional dependencies
- Full control over implementation
- Understanding of the security properties
- Simpler than adding crate for single function

---

### 4. Password Hashing: argon2id Best Practices

#### Learnings

1. **argon2id is the Current Standard**
   - Combines argon2i (resistant to side-channel) and argon2d (resistant to GPU)
   - Default in argon2 crate
   - Recommended by PHC (Password Hashing Competition)

2. **Salt is Critical**
   - Must be unique per password
   - Must be cryptographically random
   - `SaltString::generate(&mut OsRng)` provides this
   - Salt embedded in hash string (no separate storage)

3. **Hash Format Standardization**
   - argon2 uses standard format: `$argon2id$v=19$m=65536,t=3,p=1$salt$hash`
   - `$argon2id` - Algorithm identifier
   - `$v=19` - Version (19 = current)
   - `$m=65536` - Memory cost (64 MB)
   - `$t=3` - Time cost (3 iterations)
   - `$p=1` - Parallelism (1 thread)
   - `$salt` - Base64-encoded salt
   - `$hash` - Base64-encoded hash

#### Recommended Configuration

```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Algorithm, Params, Version,
};

// Default configuration (used in implementation)
let argon2 = Argon2::default();

// Custom configuration for higher security (if needed)
let argon2_custom = Argon2::new(
    Algorithm::Argon2id,
    Version::V0x19,
    Params::new(65536, 3, 1, None).unwrap(), // 64 MB, 3 iterations, 1 thread
);

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default(); // Or Argon2::custom(...);
    let hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}
```

#### Memory and Time Costs

| Configuration | Memory | Time | Security | Performance |
|---------------|--------|------|----------|-------------|
| Default | 64 MB | 3 iter | Good | ~100ms |
| High | 256 MB | 4 iter | Better | ~500ms |
| Very High | 512 MB | 5 iter | Best | ~1s |

**Recommendation:** Use default for most cases. Adjust based on:
- Server resources available
- Acceptable login latency
- Security requirements

---

### 5. Password Verification: Error Handling

#### Learnings

1. **Parse Hash Before Verification**
   ```rust
   pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
       let parsed_hash = PasswordHash::new(hash)?; // Parse first
       Ok(Argon2::default()
           .verify_password(password.as_bytes(), &parsed_hash)
           .is_ok())
   }
   ```

2. **Three Possible Outcomes:**
   - `Ok(true)` - Password matches
   - `Ok(false)` - Password doesn't match (but hash is valid)
   - `Err(_)` - Hash format is invalid (corrupted or old format)

3. **Error Cases:**
   - Old argon2i/argon2d hashes (version 0x10 instead of 0x13)
   - Corrupted hashes (truncated, invalid base64)
   - Wrong algorithm identifier
   - Invalid parameter format

#### Best Practice: Hash Migration

```rust
// During login, check if hash needs migration
pub fn verify_and_migrate(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    match verify_password(password, hash) {
        Ok(true) => {
            // Password correct, check if hash should be upgraded
            if needs_upgrade(hash) {
                let new_hash = hash_password(password)?;
                // Update stored hash (implement your update logic)
                save_new_hash(username, &new_hash).await?;
            }
            Ok(true)
        }
        Ok(false) => Ok(false),
        Err(e) => Err(e),
    }
}

fn needs_upgrade(hash: &str) -> bool {
    // Check for old versions or parameters
    !hash.starts_with("$argon2id$v=19$")
}
```

---

### 6. Integration with Existing Auth System

#### Learnings

1. **Backward Compatibility is Key**
   ```rust
   // Detect if passwords are hashed by prefix
   let passwords_hashed = config.passwords.iter().any(|cred| {
       cred.password.expose().starts_with("$argon2")
   });
   ```

2. **Graceful Degradation**
   - If hashed: Use `verify_password()`
   - If plain: Use direct comparison
   - Allows gradual migration

3. **TokenStore Integration**
   ```rust
   // In middleware, use TokenStore for rotation support
   if config.gateway_mode == AuthMode::Token {
       if let Some(ref store) = self.token_store {
           if store.validate(token) {
               // Token valid (active or previous)
           }
       }
   }
   ```

#### Migration Strategy

1. **Phase 1: Implement Hashing**
   - Add `hash_password()` to auth module
   - Update configuration to use hashed passwords
   - New passwords are hashed automatically

2. **Phase 2: Migration on Login**
   - On successful login with plain password
   - Hash the password and update storage
   - Subsequent logins use hashed verification

3. **Phase 3: Force Migration**
   - Set deadline for migration
   - Require password reset for users who haven't migrated
   - Disable plain password storage

---

## Common Pitfalls and Solutions

### Pitfall 1: Using Non-Cryptographic Randomness

```rust
// ❌ BAD: rand::Rng for non-crypto (not cryptographically secure)
use rand::seq::IteratorRandom;

let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
let token: String = (0..32)
    .map(|_| chars.chars().choose(&mut rng).unwrap())
    .collect();

// ✅ GOOD: Use rand::RngCore for bytes
use rand::RngCore;

let mut bytes = [0u8; 32];
rand::thread_rng().fill_bytes(&mut bytes);
```

### Pitfall 2: Not Checking Hash Format

```rust
// ❌ BAD: Direct verification without parsing
pub fn verify_password_bad(password: &str, hash: &str) -> bool {
    Argon2::default()
        .verify_password(password.as_bytes(), hash.as_bytes()) // Wrong!
        .is_ok()
}

// ✅ GOOD: Parse hash first
pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?; // Validates format
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
```

### Pitfall 3: Timing Attacks in Comparison

```rust
// ❌ BAD: Standard string comparison
pub fn validate(token: &str, stored: &str) -> bool {
    token == stored // Leaks timing information
}

// ✅ GOOD: Constant-time comparison
pub fn validate(token: &str, stored: &str) -> bool {
    constant_time_eq(token.as_bytes(), stored.as_bytes())
}
```

### Pitfall 4: Reusing Salts

```rust
// ❌ BAD: Fixed or predictable salt
static SALT: &[u8] = b"fixed-salt";

pub fn hash_password_bad(password: &str) -> String {
    let salt = SaltString::from_b64("fixed-salt").unwrap(); // Same salt for all!
    Argon2::default().hash_password(password.as_bytes(), &salt).unwrap()
}

// ✅ GOOD: Random salt per password
pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng); // Unique salt per password
    Argon2::default().hash_password(password.as_bytes(), &salt).unwrap()
}
```

---

## Testing Strategies

### Token Generation Tests

```rust
#[test]
fn test_generate_token_length() {
    let token = generate_token();
    assert_eq!(token.len(), 43); // 32 bytes -> 43 chars (URL-safe base64)
}

#[test]
fn test_generate_token_uniqueness() {
    let tokens: Vec<String> = (0..1000).map(|_| generate_token()).collect();
    let unique: HashSet<&String> = tokens.iter().collect();
    assert_eq!(tokens.len(), unique.len()); // All unique
}

#[test]
fn test_generate_token_valid_characters() {
    let token = generate_token();
    assert!(token.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
}
```

### Token Rotation Tests

```rust
#[test]
fn test_token_rotation() {
    let mut store = TokenStore::new("old-token".to_string());
    let new_token = store.rotate();

    // New token is valid
    assert!(store.validate(&new_token));

    // Old token still valid (grace period)
    assert!(store.validate("old-token"));

    // Random token invalid
    assert!(!store.validate("random-token"));

    // Expire previous
    store.expire_previous();

    // Old token no longer valid
    assert!(!store.validate("old-token"));

    // New token still valid
    assert!(store.validate(&new_token));
}
```

### Password Hashing Tests

```rust
#[test]
fn test_hash_password_valid() {
    let hash = hash_password("my-secret").unwrap();
    assert!(hash.starts_with("$argon2id"));
    assert!(hash.split('$').nth(4).is_some()); // Has salt
}

#[test]
fn test_verify_password_valid() {
    let password = "my-secret";
    let hash = hash_password(password).unwrap();
    assert!(verify_password(password, &hash).unwrap());
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
    assert_ne!(h1, h2); // Different salts = different hashes
    assert!(verify_password("same-password", &h1).unwrap());
    assert!(verify_password("same-password", &h2).unwrap());
}
```

---

## Security Checklist

When implementing token/password security:

- [ ] Use `rand::RngCore` for cryptographic randomness
- [ ] Encode tokens as URL-safe base64 (no padding)
- [ ] Use constant-time comparison for sensitive strings
- [ ] Implement token rotation with grace period
- [ ] Use argon2id (not argon2i or argon2d) for password hashing
- [ ] Generate random salt per password (use `OsRng`)
- [ ] Parse hash before verification (catch format errors)
- [ ] Include comprehensive unit tests
- [ ] Document security properties in code comments
- [ ] Consider performance impact on login flow
- [ ] Plan for hash algorithm migration (versioned format)
- [ ] Log security events (failed logins, token rotations)

---

## References

### Documentation

1. **argon2 crate documentation**
   - https://docs.rs/argon2/latest/argon2/
   - https://docs.rs/argon2/latest/argon2/password_hash/index.html

2. **base64 crate documentation**
   - https://docs.rs/base64/latest/base64/

3. **rand crate documentation**
   - https://docs.rs/rand/latest/rand/

### Security Standards

1. **PHC String Format**
   - https://github.com/P-H-C/phc-string-format/blob/master/phc-sf-spec.md
   - Standard format for password hashes

2. **OWASP Password Storage Cheat Sheet**
   - https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html
   - Best practices for password storage

3. **OWASP Token-Based Authentication**
   - https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html
   - Token security best practices

### Related Issues

- Issue #031: Gateway Authentication System (integration target)
- Issue #059: Approval Workflow (similar security patterns)
- Issue #147: Execution Approval Workflow (authorization patterns)

---

## Conclusion

Issue #148 implementation demonstrates:

1. **Security-First Approach**
   - Cryptographically secure randomness
   - Timing-attack prevention
   - Modern password hashing (argon2id)

2. **Clean API Design**
   - Simple, focused functions
   - Proper error handling
   - Clear separation of concerns

3. **Test Coverage**
   - Comprehensive unit tests
   - Integration tests
   - Edge case handling

4. **Production Ready**
   - Backward compatible integration
   - Graceful degradation
   - Migration path available

The implementation is a strong foundation for secure authentication in the aisopod platform and demonstrates best practices for token and password security.

---

**Document Created:** 2026-02-25  
**Last Updated:** 2026-02-25  
**Maintained By:** Development Team
