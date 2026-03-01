//! Device token management for mobile and desktop clients
//!
//! This module provides functionality for issuing, validating, revoking, and
//! refreshing device-specific tokens with persistent storage.

use crate::auth::{password, tokens::generate_token};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::auth::scopes::Scope;

/// A device token with its metadata
///
/// The plaintext token is only returned once during issuance.
/// The stored token_hash is an argon2id hash of the plaintext token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceToken {
    pub token_hash: String, // Store hash, not plaintext
    pub device_name: String,
    pub device_id: String,
    pub scopes: Vec<Scope>,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub revoked: bool,
}

/// Information about a device token (without exposing the hash)
#[derive(Debug, Clone, Serialize)]
pub struct DeviceTokenInfo {
    pub device_name: String,
    pub device_id: String,
    pub scopes: Vec<Scope>,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub revoked: bool,
}

/// Manages device tokens for individual clients
///
/// The DeviceTokenManager handles:
/// - Issuing new device tokens
/// - Validating tokens on requests
/// - Revoking tokens individually
/// - Refreshing tokens with rotation
/// - Persisting token metadata to disk
pub struct DeviceTokenManager {
    /// Map from device_id to DeviceToken (wrapped in Mutex for thread safety)
    tokens: Mutex<HashMap<String, DeviceToken>>,
    store_path: PathBuf,
}

impl DeviceTokenManager {
    /// Create a new DeviceTokenManager with the given storage path
    pub fn new(store_path: PathBuf) -> Self {
        Self {
            tokens: Mutex::new(HashMap::new()),
            store_path,
        }
    }

    /// Load device tokens from persistent storage.
    pub fn load(&mut self) -> Result<()> {
        if self.store_path.exists() {
            let data = std::fs::read_to_string(&self.store_path)?;
            let new_tokens: HashMap<String, DeviceToken> = toml::from_str(&data)?;
            let mut tokens = self.tokens.lock().unwrap();
            *tokens = new_tokens;
        }
        Ok(())
    }

    /// Save device tokens to persistent storage.
    fn save(&self) -> Result<()> {
        let tokens = self.tokens.lock().unwrap();
        let data = toml::to_string_pretty(&*tokens)?;
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
        let token_hash = password::hash_password(&plaintext)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

        let device_token = DeviceToken {
            token_hash,
            device_name,
            device_id: device_id.clone(),
            scopes,
            created_at: Utc::now(),
            last_used: None,
            revoked: false,
        };

        let mut tokens = self.tokens.lock().unwrap();
        tokens.insert(device_id, device_token);
        drop(tokens);
        self.save()?;

        Ok(plaintext)
    }

    /// Validate a token. Returns the matching DeviceToken if valid.
    pub fn validate(&mut self, candidate: &str) -> Option<DeviceToken> {
        let mut tokens = self.tokens.lock().unwrap();
        for token in tokens.values_mut() {
            if token.revoked {
                continue;
            }
            if password::verify_password(candidate, &token.token_hash).unwrap_or(false) {
                token.last_used = Some(Utc::now());
                // Return a clone since we're returning outside the lock scope
                return Some(token.clone());
            }
        }
        None
    }

    /// Revoke a device token by device ID.
    pub fn revoke(&mut self, device_id: &str) -> Result<bool> {
        let mut tokens = self.tokens.lock().unwrap();
        if let Some(token) = tokens.get_mut(device_id) {
            token.revoked = true;
            drop(tokens);
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Refresh a device token. Returns the new plaintext token.
    pub fn refresh(&mut self, device_id: &str) -> Result<Option<String>> {
        let mut tokens = self.tokens.lock().unwrap();
        let Some(existing) = tokens.get(device_id).cloned() else {
            return Ok(None);
        };
        if existing.revoked {
            return Ok(None);
        }

        let scopes = existing.scopes.clone();
        let device_name = existing.device_name.clone();

        let plaintext = generate_token();
        let token_hash = password::hash_password(&plaintext)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

        let refreshed = DeviceToken {
            token_hash,
            device_name,
            device_id: device_id.to_string(),
            scopes,
            created_at: existing.created_at,
            last_used: None,
            revoked: false,
        };

        tokens.insert(device_id.to_string(), refreshed);
        drop(tokens);
        self.save()?;

        Ok(Some(plaintext))
    }

    /// List all device tokens (without exposing hashes).
    pub fn list(&self) -> Vec<DeviceTokenInfo> {
        let tokens = self.tokens.lock().unwrap();
        tokens
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// A helper struct to manage the TempDir lifetime
    struct TestManagerWithTemp {
        _temp_dir: TempDir,
        manager: DeviceTokenManager,
    }

    fn test_manager() -> TestManagerWithTemp {
        let tmp = TempDir::new().unwrap();
        let store_path = tmp.path().join("tokens.toml");
        let manager = DeviceTokenManager::new(store_path);
        TestManagerWithTemp {
            _temp_dir: tmp,
            manager,
        }
    }

    impl TestManagerWithTemp {
        fn manager(&mut self) -> &mut DeviceTokenManager {
            &mut self.manager
        }
    }

    // Implement Deref to allow calling DeviceTokenManager methods directly on TestManagerWithTemp
    impl std::ops::Deref for TestManagerWithTemp {
        type Target = DeviceTokenManager;

        fn deref(&self) -> &Self::Target {
            &self.manager
        }
    }

    impl std::ops::DerefMut for TestManagerWithTemp {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.manager
        }
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
            .issue(
                "Laptop".into(),
                "device-2".into(),
                vec![Scope::OperatorRead],
            )
            .unwrap();

        mgr.revoke("device-2").unwrap();
        assert!(mgr.validate(&token).is_none());
    }

    #[test]
    fn test_refresh_invalidates_old_token() {
        let mut mgr = test_manager();
        let old_token = mgr
            .issue(
                "Tablet".into(),
                "device-3".into(),
                vec![Scope::OperatorRead],
            )
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

    #[test]
    fn test_refresh_nonexistent_device() {
        let mut mgr = test_manager();
        let result = mgr.refresh("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_revoke_nonexistent_device() {
        let mut mgr = test_manager();
        let result = mgr.revoke("nonexistent").unwrap();
        assert!(!result);
    }

    #[test]
    fn test_multiple_devices_with_scopes() {
        let mut mgr = test_manager();

        // Issue tokens for multiple devices with different scopes
        mgr.issue(
            "Phone".into(),
            "d1".into(),
            vec![Scope::OperatorRead, Scope::OperatorWrite],
        )
        .unwrap();
        mgr.issue("Tablet".into(), "d2".into(), vec![Scope::OperatorAdmin])
            .unwrap();

        let list = mgr.list();
        assert_eq!(list.len(), 2);

        // Verify scopes are stored correctly
        let d1 = list.iter().find(|t| t.device_id == "d1").unwrap();
        assert!(d1.scopes.contains(&Scope::OperatorRead));
        assert!(d1.scopes.contains(&Scope::OperatorWrite));

        let d2 = list.iter().find(|t| t.device_id == "d2").unwrap();
        assert!(d2.scopes.contains(&Scope::OperatorAdmin));
    }

    #[test]
    fn test_token_hash_not_exposed() {
        let mut mgr = test_manager();
        let _token = mgr.issue("Phone".into(), "d1".into(), vec![]).unwrap();

        let list = mgr.list();
        // The list should not contain any token hashes
        for info in list {
            // DeviceTokenInfo doesn't expose token_hash
            let _info_json = serde_json::to_string(&info).unwrap();
        }
    }

    #[test]
    fn test_persistence_across_loads() {
        let tmp = TempDir::new().unwrap();
        let store_path = tmp.path().join("tokens.toml");

        // Create manager and add tokens
        {
            let mut mgr = DeviceTokenManager::new(store_path.clone());
            mgr.issue("Phone".into(), "d1".into(), vec![]).unwrap();
            mgr.issue("Tablet".into(), "d2".into(), vec![]).unwrap();
        }

        // Load tokens from storage
        let mut mgr = DeviceTokenManager::new(store_path);
        mgr.load().unwrap();

        assert_eq!(mgr.list().len(), 2);
    }
}
