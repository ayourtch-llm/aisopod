//! Approval RPC data types for aisopod-gateway.
//!
//! This module provides the approval request data structures
//! used by the approval handlers.

use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Storage for pending approval requests
#[derive(Debug, Clone)]
pub struct ApprovalStore {
    inner: Arc<Mutex<HashMap<String, PendingApproval>>>,
}

impl ApprovalStore {
    /// Create a new approval store
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Store a pending approval request
    pub fn store(&self, approval: PendingApproval) {
        let mut store = self.inner.lock().unwrap();
        store.insert(approval.id.clone(), approval);
    }

    /// Get a pending approval by ID
    pub fn get(&self, id: &str) -> Option<PendingApproval> {
        let store = self.inner.lock().unwrap();
        store.get(id).cloned()
    }

    /// Remove and return a pending approval by ID
    pub fn remove(&self, id: &str) -> Option<PendingApproval> {
        let mut store = self.inner.lock().unwrap();
        store.remove(id)
    }

    /// List all pending approvals
    pub fn list(&self) -> Vec<PendingApproval> {
        let store = self.inner.lock().unwrap();
        store.values().cloned().collect()
    }
}

impl Default for ApprovalStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a pending approval request
#[derive(Debug, Clone)]
pub struct PendingApproval {
    /// Unique ID of the approval request
    pub id: String,
    /// Agent ID requesting approval
    pub agent_id: String,
    /// The operation requiring approval
    pub operation: String,
    /// The risk level of the operation
    pub risk_level: String,
    /// Timestamp when the approval was requested
    pub requested_at: u64,
    /// Current status of the approval
    pub status: ApprovalStatus,
}

/// Status of an approval request
#[derive(Debug, Clone, PartialEq)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Denied,
}

impl PendingApproval {
    /// Create a new pending approval request
    pub fn new(agent_id: String, operation: String, risk_level: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            agent_id,
            operation,
            risk_level,
            requested_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            status: ApprovalStatus::Pending,
        }
    }
}

/// Parameters for the approval.request RPC method
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApprovalRequestParams {
    /// The ID of the agent requesting approval
    pub agent_id: String,
    /// The operation requiring approval
    pub operation: String,
    /// The risk level of the operation
    pub risk_level: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_store_new() {
        let store = ApprovalStore::new();
        assert!(store.list().is_empty());
    }

    #[test]
    fn test_approval_store_store_and_get() {
        let store = ApprovalStore::new();
        let approval = PendingApproval::new(
            "agent-123".to_string(),
            "rm -rf /".to_string(),
            "Critical".to_string(),
        );
        let id = approval.id.clone();
        store.store(approval);

        let retrieved = store.get(&id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, id);
    }

    #[test]
    fn test_approval_store_remove() {
        let store = ApprovalStore::new();
        let approval = PendingApproval::new(
            "agent-123".to_string(),
            "rm -rf /".to_string(),
            "Critical".to_string(),
        );
        let id = approval.id.clone();
        store.store(approval);

        let removed = store.remove(&id);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, id);

        // Should be empty after removal
        assert!(store.get(&id).is_none());
    }

    #[test]
    fn test_approval_store_list() {
        let store = ApprovalStore::new();

        let approval1 = PendingApproval::new(
            "agent-1".to_string(),
            "echo hello".to_string(),
            "Low".to_string(),
        );
        let approval2 = PendingApproval::new(
            "agent-2".to_string(),
            "echo world".to_string(),
            "Medium".to_string(),
        );

        store.store(approval1);
        store.store(approval2);

        let approvals = store.list();
        assert_eq!(approvals.len(), 2);
    }

    #[test]
    fn test_approval_request_params_deserialization() {
        let json = r#"{"agent_id":"agent-123","operation":"rm -rf /","risk_level":"Critical"}"#;
        let params: ApprovalRequestParams = serde_json::from_str(json).unwrap();

        assert_eq!(params.agent_id, "agent-123");
        assert_eq!(params.operation, "rm -rf /");
        assert_eq!(params.risk_level, "Critical");
    }

    #[test]
    fn test_approval_request_params_minimal() {
        let json = r#"{"agent_id":"agent-123","operation":"rm -rf /","risk_level":"Critical"}"#;
        let params: ApprovalRequestParams = serde_json::from_str(json).unwrap();

        assert_eq!(params.agent_id, "agent-123");
        assert_eq!(params.operation, "rm -rf /");
        assert_eq!(params.risk_level, "Critical");
    }
}
