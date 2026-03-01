#![allow(clippy::all)]
//! Authentication module for the gateway
//!
//! This module provides authentication validation functions and the AuthInfo struct
//! that carries user role and scopes through the request pipeline.

mod device_tokens;
mod password;
mod tokens;

use aisopod_config::sensitive::Sensitive;
use aisopod_config::types::{AuthConfig, AuthMode, PasswordCredential, TokenCredential};
use std::collections::HashMap;

pub use device_tokens::{DeviceToken, DeviceTokenInfo, DeviceTokenManager};
pub use password::{hash_password, verify_password};
pub use tokens::{generate_token, TokenStore};

pub mod scopes;
pub use scopes::{required_scope, Scope};

/// Authentication information extracted from a request
///
/// This struct is stored in request extensions and carried through
/// the middleware chain to provide authorization context.
#[derive(Debug, Clone, Default)]
pub struct AuthInfo {
    /// The role of the authenticated user (e.g., "operator", "node")
    pub role: String,
    /// List of permission scopes granted to this user
    pub scopes: Vec<String>,
}

impl AuthInfo {
    /// Check if the authenticated user has a specific scope
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == scope)
    }

    /// Check if the authenticated user has any of the specified scopes
    pub fn has_any_scope(&self, scopes: &[&str]) -> bool {
        scopes.iter().any(|s| self.has_scope(s))
    }

    /// Check if the authenticated user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.role == role
    }
}

/// Validate a Bearer token against the configured token credentials
///
/// Returns Some(AuthInfo) if the token is valid, None otherwise.
pub fn validate_token(token: &str, config: &AuthConfig) -> Option<AuthInfo> {
    if config.gateway_mode != AuthMode::Token {
        return None;
    }

    config.tokens.iter().find_map(|cred| {
        if cred.token == token {
            Some(AuthInfo {
                role: cred.role.clone(),
                scopes: cred.scopes.clone(),
            })
        } else {
            None
        }
    })
}

/// Validate HTTP Basic authentication credentials
///
/// Returns Some(AuthInfo) if the credentials are valid, None otherwise.
pub fn validate_basic(username: &str, password: &str, config: &AuthConfig) -> Option<AuthInfo> {
    if config.gateway_mode != AuthMode::Password {
        return None;
    }

    config.passwords.iter().find_map(|cred| {
        if cred.username == username && cred.password.expose() == password {
            Some(AuthInfo {
                role: cred.role.clone(),
                scopes: cred.scopes.clone(),
            })
        } else {
            None
        }
    })
}

/// Build a credential lookup map for efficient token/password validation
///
/// This creates a HashMap for O(1) lookups instead of iterating through all credentials.
pub fn build_token_map(config: &AuthConfig) -> HashMap<String, AuthInfo> {
    if config.gateway_mode != AuthMode::Token {
        return HashMap::new();
    }

    config
        .tokens
        .iter()
        .map(|cred| {
            (
                cred.token.clone(),
                AuthInfo {
                    role: cred.role.clone(),
                    scopes: cred.scopes.clone(),
                },
            )
        })
        .collect()
}

/// Build a credential lookup map for efficient password validation
pub fn build_password_map(config: &AuthConfig) -> HashMap<String, HashMap<String, AuthInfo>> {
    if config.gateway_mode != AuthMode::Password {
        return HashMap::new();
    }

    let mut map: HashMap<String, HashMap<String, AuthInfo>> = HashMap::new();

    for cred in &config.passwords {
        map.entry(cred.username.clone()).or_default().insert(
            cred.password.expose().to_string(),
            AuthInfo {
                role: cred.role.clone(),
                scopes: cred.scopes.clone(),
            },
        );
    }

    map
}

/// Validate a password against stored password hashes
///
/// This function checks if the provided password matches any of the stored
/// password hashes using argon2 verification.
pub fn validate_password_hash(
    username: &str,
    password: &str,
    config: &AuthConfig,
) -> Option<AuthInfo> {
    if config.gateway_mode != AuthMode::Password {
        return None;
    }

    config.passwords.iter().find_map(|cred| {
        if cred.username == username {
            // Check if password matches the stored hash
            match verify_password(password, &cred.password.expose()) {
                Ok(true) => Some(AuthInfo {
                    role: cred.role.clone(),
                    scopes: cred.scopes.clone(),
                }),
                Ok(false) | Err(_) => None,
            }
        } else {
            None
        }
    })
}

/// Validate a token using TokenStore for rotation support
///
/// This function uses a TokenStore to validate tokens, supporting
/// token rotation with a grace period.
pub fn validate_token_with_store(
    token: &str,
    store: &TokenStore,
    config: &AuthConfig,
) -> Option<AuthInfo> {
    if config.gateway_mode != AuthMode::Token {
        return None;
    }

    if store.validate(token) {
        // Find the token credential to get role and scopes
        config.tokens.iter().find_map(|cred| {
            if cred.token == token {
                Some(AuthInfo {
                    role: cred.role.clone(),
                    scopes: cred.scopes.clone(),
                })
            } else {
                None
            }
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_auth_config(mode: AuthMode) -> AuthConfig {
        AuthConfig {
            gateway_mode: mode,
            tokens: vec![
                TokenCredential {
                    token: "secret-token-1".to_string(),
                    role: "operator".to_string(),
                    scopes: vec!["chat:write".to_string(), "agent:read".to_string()],
                },
                TokenCredential {
                    token: "secret-token-2".to_string(),
                    role: "node".to_string(),
                    scopes: vec!["agent:admin".to_string()],
                },
            ],
            passwords: vec![
                PasswordCredential {
                    username: "admin".to_string(),
                    password: Sensitive::new("admin123".to_string()),
                    role: "operator".to_string(),
                    scopes: vec!["chat:write".to_string(), "agent:admin".to_string()],
                },
                PasswordCredential {
                    username: "monitor".to_string(),
                    password: Sensitive::new("monitor123".to_string()),
                    role: "node".to_string(),
                    scopes: vec!["agent:read".to_string()],
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn test_validate_token_success() {
        let config = create_test_auth_config(AuthMode::Token);

        let result = validate_token("secret-token-1", &config);
        assert!(result.is_some());
        let auth_info = result.unwrap();
        assert_eq!(auth_info.role, "operator");
        assert_eq!(auth_info.scopes, vec!["chat:write", "agent:read"]);
    }

    #[test]
    fn test_validate_token_invalid() {
        let config = create_test_auth_config(AuthMode::Token);

        let result = validate_token("invalid-token", &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_validate_token_wrong_mode() {
        let config = create_test_auth_config(AuthMode::None);

        let result = validate_token("secret-token-1", &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_validate_basic_success() {
        let config = create_test_auth_config(AuthMode::Password);

        let result = validate_basic("admin", "admin123", &config);
        assert!(result.is_some());
        let auth_info = result.unwrap();
        assert_eq!(auth_info.role, "operator");
        assert_eq!(auth_info.scopes, vec!["chat:write", "agent:admin"]);
    }

    #[test]
    fn test_validate_basic_invalid_credentials() {
        let config = create_test_auth_config(AuthMode::Password);

        let result = validate_basic("admin", "wrong-password", &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_validate_basic_invalid_user() {
        let config = create_test_auth_config(AuthMode::Password);

        let result = validate_basic("unknown", "admin123", &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_validate_basic_wrong_mode() {
        let config = create_test_auth_config(AuthMode::None);

        let result = validate_basic("admin", "admin123", &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_build_token_map() {
        let config = create_test_auth_config(AuthMode::Token);
        let token_map = build_token_map(&config);

        assert_eq!(token_map.len(), 2);
        assert!(token_map.contains_key("secret-token-1"));
        assert!(token_map.contains_key("secret-token-2"));

        let auth_info = token_map.get("secret-token-1").unwrap();
        assert_eq!(auth_info.role, "operator");
    }

    #[test]
    fn test_build_password_map() {
        let config = create_test_auth_config(AuthMode::Password);
        let password_map = build_password_map(&config);

        assert_eq!(password_map.len(), 2);
        assert!(password_map.contains_key("admin"));
        assert!(password_map.contains_key("monitor"));

        let passwords = password_map.get("admin").unwrap();
        assert_eq!(passwords.len(), 1);
        assert!(passwords.contains_key("admin123"));
    }

    #[test]
    fn test_auth_info_has_scope() {
        let auth_info = AuthInfo {
            role: "operator".to_string(),
            scopes: vec!["chat:write".to_string(), "agent:read".to_string()],
        };

        assert!(auth_info.has_scope("chat:write"));
        assert!(auth_info.has_scope("agent:read"));
        assert!(!auth_info.has_scope("agent:admin"));
    }

    #[test]
    fn test_auth_info_has_any_scope() {
        let auth_info = AuthInfo {
            role: "operator".to_string(),
            scopes: vec!["chat:write".to_string(), "agent:read".to_string()],
        };

        assert!(auth_info.has_any_scope(&["chat:write", "agent:admin"]));
        assert!(auth_info.has_any_scope(&["agent:admin", "agent:read"]));
        assert!(!auth_info.has_any_scope(&["agent:admin", "data:delete"]));
    }

    #[test]
    fn test_auth_info_has_role() {
        let auth_info = AuthInfo {
            role: "operator".to_string(),
            scopes: vec!["chat:write".to_string()],
        };

        assert!(auth_info.has_role("operator"));
        assert!(!auth_info.has_role("node"));
    }

    #[test]
    fn test_default_auth_info() {
        let auth_info = AuthInfo::default();

        assert_eq!(auth_info.role, "");
        assert!(auth_info.scopes.is_empty());
    }

    #[test]
    fn test_validate_password_hash_not_implemented() {
        // This test is to ensure the function signature compiles
        // Actual hashing tests are in the password module tests
    }

    #[test]
    fn test_validate_token_with_store_not_implemented() {
        // This test is to ensure the function signature compiles
        // Actual token store tests are in the tokens module tests
    }
}
