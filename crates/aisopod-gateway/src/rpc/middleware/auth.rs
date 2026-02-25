//! Scope-based authorization middleware for RPC methods.
//!
//! This module provides a scope-checking layer that runs before method dispatch
//! and rejects calls where the caller lacks the required scope.

use crate::auth::AuthInfo;
use crate::rpc::jsonrpc::{RpcError, RpcResponse};
use crate::auth::scopes::{required_scope, Scope};

const UNAUTHORIZED_CODE: i64 = -32603;

/// Check if the authenticated user has the required scope for a method.
///
/// This function is used by the RPC middleware to enforce scope-based
/// authorization. It checks if the user's scopes contain the required
/// scope for the given method.
///
/// # Arguments
///
/// * `auth_info` - The authenticated user's info containing their scopes
/// * `method` - The RPC method being called
///
/// # Returns
///
/// * `Ok(())` if the user has the required scope or no scope is required
/// * `Err(RpcResponse)` with an unauthorized error if the user lacks the required scope
pub fn check_scope(auth_info: &AuthInfo, method: &str) -> Result<(), RpcResponse> {
    let Some(required) = required_scope(method) else {
        // No scope requirement defined — allow by default
        return Ok(());
    };

    if has_scope(auth_info, required) {
        Ok(())
    } else {
        Err(RpcResponse::error(
            None,
            UNAUTHORIZED_CODE as i32,
            format!(
                "Insufficient permissions: method '{}' requires scope '{}'",
                method, required
            ),
        ))
    }
}

/// Check if the auth info has the required scope.
///
/// This function checks if the auth info contains the exact scope,
/// or if it has a broader scope that implicitly grants access.
fn has_scope(auth_info: &AuthInfo, required: &Scope) -> bool {
    // Check for exact scope match
    if auth_info.scopes.iter().any(|s| s == required.as_str()) {
        return true;
    }

    // Check if any scope grants broader permissions that include the required scope
    for scope_str in &auth_info.scopes {
        // Parse the scope string to check for broader access
        if let Some(parsed) = parse_scope_string(scope_str) {
            if parsed.allows(required) {
                return true;
            }
        }
    }

    false
}

/// Parse a scope string into a Scope enum.
fn parse_scope_string(scope_str: &str) -> Option<Scope> {
    match scope_str {
        "operator.admin" => Some(Scope::OperatorAdmin),
        "operator.read" => Some(Scope::OperatorRead),
        "operator.write" => Some(Scope::OperatorWrite),
        "operator.approvals" => Some(Scope::OperatorApprovals),
        "operator.pairing" => Some(Scope::OperatorPairing),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthInfo;

    fn auth_info_with_scopes(scopes: Vec<&str>) -> AuthInfo {
        AuthInfo {
            role: "operator".to_string(),
            scopes: scopes.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_check_scope_no_required_scope() {
        let auth_info = auth_info_with_scopes(vec![]);
        // Unknown method - no scope required
        assert!(check_scope(&auth_info, "unknown.method").is_ok());
    }

    #[test]
    fn test_check_scope_with_matching_scope() {
        let auth_info = auth_info_with_scopes(vec!["operator.read"]);
        assert!(check_scope(&auth_info, "agent.list").is_ok());
        assert!(check_scope(&auth_info, "session.get").is_ok());
    }

    #[test]
    fn test_check_scope_without_required_scope() {
        let auth_info = auth_info_with_scopes(vec!["operator.read"]);
        let result = check_scope(&auth_info, "agent.start");
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        assert_eq!(error.error.as_ref().unwrap().code, UNAUTHORIZED_CODE as i32);
        assert!(error.error.as_ref().unwrap().message.contains("Insufficient permissions"));
        assert!(error.error.as_ref().unwrap().message.contains("operator.write"));
    }

    #[test]
    fn test_check_scope_admin_allows_all() {
        let auth_info = auth_info_with_scopes(vec!["operator.admin"]);
        // Admin should be able to access all methods
        assert!(check_scope(&auth_info, "agent.list").is_ok());
        assert!(check_scope(&auth_info, "agent.start").is_ok());
        assert!(check_scope(&auth_info, "admin.shutdown").is_ok());
        assert!(check_scope(&auth_info, "approval.approve").is_ok());
        assert!(check_scope(&auth_info, "pairing.initiate").is_ok());
    }

    #[test]
    fn test_check_scope_write_allows_write_methods() {
        let auth_info = auth_info_with_scopes(vec!["operator.write"]);
        assert!(check_scope(&auth_info, "agent.start").is_ok());
        assert!(check_scope(&auth_info, "chat.send").is_ok());
        assert!(check_scope(&auth_info, "config.update").is_ok());
    }

    #[test]
    fn test_check_scope_write_denies_admin() {
        let auth_info = auth_info_with_scopes(vec!["operator.write"]);
        let result = check_scope(&auth_info, "admin.shutdown");
        assert!(result.is_err());
    }

    #[test]
    fn test_check_scope_approvals() {
        let auth_info = auth_info_with_scopes(vec!["operator.approvals"]);
        assert!(check_scope(&auth_info, "approval.request").is_ok());
        assert!(check_scope(&auth_info, "approval.approve").is_ok());
        assert!(check_scope(&auth_info, "approval.deny").is_ok());
    }

    #[test]
    fn test_check_scope_pairing() {
        let auth_info = auth_info_with_scopes(vec!["operator.pairing"]);
        assert!(check_scope(&auth_info, "pairing.initiate").is_ok());
        assert!(check_scope(&auth_info, "pairing.confirm").is_ok());
    }

    #[test]
    fn test_check_scope_multiple_scopes() {
        let auth_info = auth_info_with_scopes(vec!["operator.read", "operator.write"]);
        assert!(check_scope(&auth_info, "agent.list").is_ok());
        assert!(check_scope(&auth_info, "agent.start").is_ok());
    }

    #[test]
    fn test_check_scope_empty_scopes() {
        let auth_info = auth_info_with_scopes(vec![]);
        // Read methods should fail without any scope
        let result = check_scope(&auth_info, "agent.list");
        assert!(result.is_err());
    }
}
