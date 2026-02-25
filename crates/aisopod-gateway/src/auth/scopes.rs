//! Permission scopes for RPC method access control.
//!
//! This module defines the scope constants and provides a mapping from
//! RPC method names to the required scope for authorization.

use std::collections::HashMap;

/// Permission scopes for RPC method access control.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    /// Admin-level access - allows all operations
    OperatorAdmin,
    /// Read-only access - allows viewing data
    OperatorRead,
    /// Write access - allows modifying data
    OperatorWrite,
    /// Approval access - allows approving/rejecting requests
    OperatorApprovals,
    /// Pairing access - allows device pairing operations
    OperatorPairing,
}

impl Scope {
    /// Get the string representation of this scope.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OperatorAdmin => "operator.admin",
            Self::OperatorRead => "operator.read",
            Self::OperatorWrite => "operator.write",
            Self::OperatorApprovals => "operator.approvals",
            Self::OperatorPairing => "operator.pairing",
        }
    }

    /// Check if this scope grants access to the specified target scope.
    /// Admin scope grants access to all scopes.
    /// Read scope grants access to read scope.
    /// Write scope grants access to read and write scopes.
    /// Approvals scope grants access to read and approvals scopes.
    /// Pairing scope grants access to read and pairing scopes.
    pub fn allows(&self, target_scope: &Scope) -> bool {
        match self {
            Self::OperatorAdmin => true, // Admin can do everything
            Self::OperatorRead => matches!(target_scope, Scope::OperatorRead),
            Self::OperatorWrite => matches!(target_scope, Scope::OperatorRead | Scope::OperatorWrite),
            Self::OperatorApprovals => matches!(target_scope, Scope::OperatorRead | Scope::OperatorApprovals),
            Self::OperatorPairing => matches!(target_scope, Scope::OperatorRead | Scope::OperatorPairing),
        }
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Get the required scope for a method, if any.
///
/// Returns `Some(&Scope)` if the method requires a scope,
/// or `None` if the method is accessible without scope validation.
pub fn required_scope(method: &str) -> Option<&'static Scope> {
    METHOD_SCOPES.get(method).map(|s| s)
}

/// Mapping from RPC method names to required scopes.
///
/// Each RPC method namespace requires a specific scope to access it.
/// Methods not in this map are accessible without scope validation.
static METHOD_SCOPES: std::sync::LazyLock<HashMap<&'static str, Scope>> = std::sync::LazyLock::new(|| {
    let mut m = HashMap::new();

    // Read-only methods (public/list endpoints)
    m.insert("system.ping", Scope::OperatorRead);
    m.insert("system.info", Scope::OperatorRead);
    m.insert("agent.list", Scope::OperatorRead);
    m.insert("agent.get", Scope::OperatorRead);
    m.insert("session.list", Scope::OperatorRead);
    m.insert("session.get", Scope::OperatorRead);
    m.insert("chat.history", Scope::OperatorRead);
    m.insert("tools.list", Scope::OperatorRead);
    m.insert("models.list", Scope::OperatorRead);
    m.insert("channels.list", Scope::OperatorRead);
    m.insert("config.get", Scope::OperatorRead);
    m.insert("health.check", Scope::OperatorRead);
    m.insert("memory.query", Scope::OperatorRead);
    m.insert("approval.list", Scope::OperatorRead);

    // Write methods (create/update endpoints)
    m.insert("agent.start", Scope::OperatorWrite);
    m.insert("agent.stop", Scope::OperatorWrite);
    m.insert("chat.send", Scope::OperatorWrite);
    m.insert("session.create", Scope::OperatorWrite);
    m.insert("session.close", Scope::OperatorWrite);
    m.insert("config.update", Scope::OperatorWrite);

    // Approval methods (approve/reject endpoints)
    m.insert("approval.request", Scope::OperatorApprovals);
    m.insert("approval.approve", Scope::OperatorApprovals);
    m.insert("approval.deny", Scope::OperatorApprovals);

    // Pairing methods (device pairing endpoints)
    m.insert("pairing.initiate", Scope::OperatorPairing);
    m.insert("pairing.confirm", Scope::OperatorPairing);

    // Admin methods (destructive/administrative endpoints)
    m.insert("admin.shutdown", Scope::OperatorAdmin);

    m
});

/// Check if a method requires scope validation.
///
/// Returns `true` if the method is not in the public whitelist.
pub fn requires_scope_validation(method: &str) -> bool {
    METHOD_SCOPES.contains_key(method)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_as_str() {
        assert_eq!(Scope::OperatorAdmin.as_str(), "operator.admin");
        assert_eq!(Scope::OperatorRead.as_str(), "operator.read");
        assert_eq!(Scope::OperatorWrite.as_str(), "operator.write");
        assert_eq!(Scope::OperatorApprovals.as_str(), "operator.approvals");
        assert_eq!(Scope::OperatorPairing.as_str(), "operator.pairing");
    }

    #[test]
    fn test_scope_display() {
        assert_eq!(format!("{}", Scope::OperatorAdmin), "operator.admin");
        assert_eq!(format!("{}", Scope::OperatorRead), "operator.read");
        assert_eq!(format!("{}", Scope::OperatorWrite), "operator.write");
        assert_eq!(format!("{}", Scope::OperatorApprovals), "operator.approvals");
        assert_eq!(format!("{}", Scope::OperatorPairing), "operator.pairing");
    }

    #[test]
    fn test_scope_allows() {
        // Admin scope allows everything
        assert!(Scope::OperatorAdmin.allows(&Scope::OperatorAdmin));
        assert!(Scope::OperatorAdmin.allows(&Scope::OperatorRead));
        assert!(Scope::OperatorAdmin.allows(&Scope::OperatorWrite));
        assert!(Scope::OperatorAdmin.allows(&Scope::OperatorApprovals));
        assert!(Scope::OperatorAdmin.allows(&Scope::OperatorPairing));

        // Read scope allows only read access
        assert!(Scope::OperatorRead.allows(&Scope::OperatorRead));
        assert!(!Scope::OperatorRead.allows(&Scope::OperatorWrite));
        assert!(!Scope::OperatorRead.allows(&Scope::OperatorApprovals));
        assert!(!Scope::OperatorRead.allows(&Scope::OperatorPairing));
        assert!(!Scope::OperatorRead.allows(&Scope::OperatorAdmin));

        // Write scope allows read and write
        assert!(Scope::OperatorWrite.allows(&Scope::OperatorRead));
        assert!(Scope::OperatorWrite.allows(&Scope::OperatorWrite));
        assert!(!Scope::OperatorWrite.allows(&Scope::OperatorAdmin));
        assert!(!Scope::OperatorWrite.allows(&Scope::OperatorApprovals));
        assert!(!Scope::OperatorWrite.allows(&Scope::OperatorPairing));

        // Approvals scope allows read and approvals
        assert!(Scope::OperatorApprovals.allows(&Scope::OperatorRead));
        assert!(Scope::OperatorApprovals.allows(&Scope::OperatorApprovals));
        assert!(!Scope::OperatorApprovals.allows(&Scope::OperatorAdmin));
        assert!(!Scope::OperatorApprovals.allows(&Scope::OperatorWrite));
        assert!(!Scope::OperatorApprovals.allows(&Scope::OperatorPairing));

        // Pairing scope allows read and pairing
        assert!(Scope::OperatorPairing.allows(&Scope::OperatorRead));
        assert!(Scope::OperatorPairing.allows(&Scope::OperatorPairing));
        assert!(!Scope::OperatorPairing.allows(&Scope::OperatorAdmin));
        assert!(!Scope::OperatorPairing.allows(&Scope::OperatorWrite));
        assert!(!Scope::OperatorPairing.allows(&Scope::OperatorApprovals));
    }

    #[test]
    fn test_method_scopes_mapping() {
        // Read-only methods
        assert_eq!(required_scope("system.ping"), Some(&Scope::OperatorRead));
        assert_eq!(required_scope("agent.list"), Some(&Scope::OperatorRead));
        assert_eq!(required_scope("session.get"), Some(&Scope::OperatorRead));

        // Write methods
        assert_eq!(required_scope("agent.start"), Some(&Scope::OperatorWrite));
        assert_eq!(required_scope("chat.send"), Some(&Scope::OperatorWrite));
        assert_eq!(required_scope("config.update"), Some(&Scope::OperatorWrite));

        // Approval methods
        assert_eq!(required_scope("approval.request"), Some(&Scope::OperatorApprovals));
        assert_eq!(required_scope("approval.approve"), Some(&Scope::OperatorApprovals));
        assert_eq!(required_scope("approval.deny"), Some(&Scope::OperatorApprovals));

        // Pairing methods
        assert_eq!(required_scope("pairing.initiate"), Some(&Scope::OperatorPairing));
        assert_eq!(required_scope("pairing.confirm"), Some(&Scope::OperatorPairing));

        // Admin methods
        assert_eq!(required_scope("admin.shutdown"), Some(&Scope::OperatorAdmin));

        // Methods without scope requirement
        assert_eq!(required_scope("unknown.method"), None);
        assert_eq!(required_scope("gateway.event"), None);
    }

    #[test]
    fn test_requires_scope_validation() {
        assert!(requires_scope_validation("agent.list"));
        assert!(requires_scope_validation("chat.send"));
        assert!(requires_scope_validation("admin.shutdown"));

        assert!(!requires_scope_validation("unknown.method"));
        assert!(!requires_scope_validation("gateway.event"));
    }
}
