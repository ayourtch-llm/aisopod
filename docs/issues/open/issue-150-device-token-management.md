# Issue 150: Implement Device Token Management

## Summary
Implement a device token system for mobile and desktop clients. Support issuing, validating, revoking, and refreshing device tokens, with storage in the configuration file or a separate token database.

## Location
- Crate: `aisopod-gateway`
- File: `crates/aisopod-gateway/src/auth/device_tokens.rs` (new)

## Current Behavior
The gateway supports bearer tokens and password authentication (Issue 031), and Issue 148 adds secure token generation. However, there is no mechanism for issuing per-device tokens that can be independently revoked, refreshed, or tracked. Clients share a single API token.

## Expected Behavior
After this issue is completed:
- Device tokens are issued to individual mobile/desktop clients via a pairing or registration flow.
- Each device token is associated with a device name/ID, creation timestamp, and scopes.
- Tokens can be validated on every request to confirm they are active and not revoked.
- Tokens can be revoked individually without affecting other devices.
- Token refresh generates a new token for the device, invalidating the old one.
- Device token metadata is persisted (in config TOML or a lightweight file-based store).

## Impact
Without per-device tokens, revoking access for a lost or compromised device requires rotating the shared API token, which disconnects all clients. Device tokens enable granular access control and secure multi-device setups.

## Suggested Implementation

1. **Define the `DeviceToken` type** in `crates/aisopod-gateway/src/auth/device_tokens.rs`:
   ```rust
   use chrono::{DateTime, Utc};
   use serde::{Deserialize, Serialize};
   use crate::auth::scopes::Scope;

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct DeviceToken {
       pub token_hash: String,       // Store hash, not plaintext
       pub device_name: String,
       pub device_id: String,
       pub scopes: Vec<Scope>,
       pub created_at: DateTime<Utc>,
       pub last_used: Option<DateTime<Utc>>,
       pub revoked: bool,
   }
   ```

2. **Implement the `DeviceTokenManager`:**
   ```rust
   use crate::auth::tokens::generate_token;
   use crate::auth::password::hash_password;
   use std::collections::HashMap;

   pub struct DeviceTokenManager {
       /// Map from device_id to DeviceToken
       tokens: HashMap<String, DeviceToken>,
       store_path: PathBuf,
   }

   impl DeviceTokenManager {
       pub fn new(store_path: PathBuf) -> Self {
           Self {
               tokens: HashMap::new(),
               store_path,
           }
       }

       /// Load device tokens from persistent storage.
       pub fn load(&mut self) -> Result<()> {
           if self.store_path.exists() {
               let data = std::fs::read_to_string(&self.store_path)?;
               self.tokens = toml::from_str(&data)?;
           }
           Ok(())
       }

       /// Save device tokens to persistent storage.
       fn save(&self) -> Result<()> {
           let data = toml::to_string_pretty(&self.tokens)?;
           std::fs::write(&self.store_path, data)?;
           Ok(())
       }

       /// Issue a new device token. Returns the plaintext token
       /// (only returned once; the hash is stored).
       pub fn issue(
           &mut self,
           device_name: String,
           device_id: String,
           scopes: Vec<Scope>,
       ) -> Result<String> {
           let plaintext = generate_token();
           let token_hash = hash_password(&plaintext)?;

           let device_token = DeviceToken {
               token_hash,
               device_name,
               device_id: device_id.clone(),
               scopes,
               created_at: Utc::now(),
               last_used: None,
               revoked: false,
           };

           self.tokens.insert(device_id, device_token);
           self.save()?;

           Ok(plaintext)
       }

       /// Validate a token. Returns the matching DeviceToken if valid.
       pub fn validate(&mut self, candidate: &str) -> Option<&DeviceToken> {
           for token in self.tokens.values_mut() {
               if token.revoked {
                   continue;
               }
               if crate::auth::password::verify_password(candidate, &token.token_hash)
                   .unwrap_or(false)
               {
                   token.last_used = Some(Utc::now());
                   return Some(token);
               }
           }
           None
       }

       /// Revoke a device token by device ID.
       pub fn revoke(&mut self, device_id: &str) -> Result<bool> {
           if let Some(token) = self.tokens.get_mut(device_id) {
               token.revoked = true;
               self.save()?;
               Ok(true)
           } else {
               Ok(false)
           }
       }

       /// Refresh a device token. Returns the new plaintext token.
       pub fn refresh(
           &mut self,
           device_id: &str,
       ) -> Result<Option<String>> {
           let Some(existing) = self.tokens.get(device_id) else {
               return Ok(None);
           };
           if existing.revoked {
               return Ok(None);
           }

           let scopes = existing.scopes.clone();
           let device_name = existing.device_name.clone();

           let plaintext = generate_token();
           let token_hash = hash_password(&plaintext)?;

           let refreshed = DeviceToken {
               token_hash,
               device_name,
               device_id: device_id.to_string(),
               scopes,
               created_at: existing.created_at,
               last_used: None,
               revoked: false,
           };

           self.tokens.insert(device_id.to_string(), refreshed);
           self.save()?;

           Ok(Some(plaintext))
       }

       /// List all device tokens (without exposing hashes).
       pub fn list(&self) -> Vec<DeviceTokenInfo> {
           self.tokens
               .values()
               .map(|t| DeviceTokenInfo {
                   device_name: t.device_name.clone(),
                   device_id: t.device_id.clone(),
                   scopes: t.scopes.clone(),
                   created_at: t.created_at,
                   last_used: t.last_used,
                   revoked: t.revoked,
               })
               .collect()
       }
   }

   #[derive(Debug, Serialize)]
   pub struct DeviceTokenInfo {
       pub device_name: String,
       pub device_id: String,
       pub scopes: Vec<Scope>,
       pub created_at: DateTime<Utc>,
       pub last_used: Option<DateTime<Utc>>,
       pub revoked: bool,
   }
   ```

3. **Add unit tests:**
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       use tempfile::TempDir;

       fn test_manager() -> DeviceTokenManager {
           let tmp = TempDir::new().unwrap();
           DeviceTokenManager::new(tmp.path().join("tokens.toml"))
       }

       #[test]
       fn test_issue_and_validate() {
           let mut mgr = test_manager();
           let token = mgr
               .issue("Phone".into(), "device-1".into(), vec![Scope::OperatorRead])
               .unwrap();

           assert!(mgr.validate(&token).is_some());
           assert!(mgr.validate("wrong-token").is_none());
       }

       #[test]
       fn test_revoke_prevents_validation() {
           let mut mgr = test_manager();
           let token = mgr
               .issue("Laptop".into(), "device-2".into(), vec![Scope::OperatorRead])
               .unwrap();

           mgr.revoke("device-2").unwrap();
           assert!(mgr.validate(&token).is_none());
       }

       #[test]
       fn test_refresh_invalidates_old_token() {
           let mut mgr = test_manager();
           let old_token = mgr
               .issue("Tablet".into(), "device-3".into(), vec![Scope::OperatorRead])
               .unwrap();

           let new_token = mgr.refresh("device-3").unwrap().unwrap();
           assert!(mgr.validate(&new_token).is_some());
           assert!(mgr.validate(&old_token).is_none());
       }

       #[test]
       fn test_list_devices() {
           let mut mgr = test_manager();
           mgr.issue("Phone".into(), "d1".into(), vec![]).unwrap();
           mgr.issue("Laptop".into(), "d2".into(), vec![]).unwrap();

           let list = mgr.list();
           assert_eq!(list.len(), 2);
       }
   }
   ```

## Dependencies
- Issue 148 (API token generation and password hashing)
- Issue 031 (gateway authentication system)

## Acceptance Criteria
- [ ] Device tokens are issued with a unique plaintext token returned to the client
- [ ] Only the hash of the token is stored; plaintext is never persisted
- [ ] Token validation correctly identifies active, non-revoked tokens
- [ ] Revoked tokens are rejected during validation
- [ ] Token refresh generates a new token and invalidates the previous one
- [ ] Device token metadata (name, scopes, timestamps) is persisted to disk
- [ ] `list()` returns token info without exposing hashes
- [ ] Unit tests cover issue, validate, revoke, refresh, and list operations

---
*Created: 2026-02-15*
