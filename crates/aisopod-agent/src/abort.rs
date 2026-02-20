//! Abort mechanism for agent execution.
//!
//! This module provides the `AbortHandle` struct which allows
//! cancelling a running agent execution via a tokio CancellationToken.

use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::types::AgentEvent;

/// An abort handle that can be used to cancel an agent execution.
///
/// Each active agent session has an associated `AbortHandle` that
/// can be used to request cancellation. The handle contains a
/// `CancellationToken` that can be checked at key points in the
/// execution loop.
#[derive(Debug, Clone)]
pub struct AbortHandle {
    token: CancellationToken,
    session_key: String,
}

impl AbortHandle {
    /// Creates a new `AbortHandle` for the given session key.
    pub fn new(session_key: String) -> Self {
        Self {
            token: CancellationToken::new(),
            session_key,
        }
    }

    /// Returns the session key associated with this abort handle.
    pub fn session_key(&self) -> &str {
        &self.session_key
    }

    /// Requests cancellation of the agent execution.
    ///
    /// This will signal the cancellation token, which can be
    /// checked at key points in the execution loop to exit early.
    pub fn abort(&self) {
        self.token.cancel();
    }

    /// Checks if cancellation has been requested.
    ///
    /// Returns `true` if `abort()` has been called on this handle.
    pub fn is_aborted(&self) -> bool {
        self.token.is_cancelled()
    }

    /// Returns a reference to the underlying cancellation token.
    pub fn token(&self) -> &CancellationToken {
        &self.token
    }

    /// Returns an awaitable future that completes when cancellation is requested.
    pub async fn cancelled(&self) {
        self.token.cancelled().await;
    }
}

/// A registry of active agent sessions that can be aborted.
///
/// This struct maintains a map of session keys to `AbortHandle`s,
/// allowing the agent runner to track and abort active executions.
#[derive(Debug, Clone, Default)]
pub struct AbortRegistry {
    sessions: Arc<DashMap<String, AbortHandle>>,
}

impl AbortRegistry {
    /// Creates a new empty `AbortRegistry`.
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
        }
    }

    /// Inserts a new abort handle for the given session key.
    ///
    /// If there was already an entry for this session key, the old
    /// handle is aborted and returned.
    pub fn insert(&self, session_key: &str, handle: AbortHandle) -> Option<AbortHandle> {
        let existing = self.sessions.insert(session_key.to_string(), handle);
        if let Some(old_handle) = &existing {
            old_handle.abort();
        }
        existing
    }

    /// Gets a reference to the abort handle for the given session key.
    pub fn get(&self, session_key: &str) -> Option<dashmap::mapref::one::Ref<'_, String, AbortHandle>> {
        self.sessions.get(session_key)
    }

    /// Removes the abort handle for the given session key.
    pub fn remove(&self, session_key: &str) -> Option<AbortHandle> {
        self.sessions.remove(session_key).map(|(_, handle)| handle)
    }

    /// Checks if there is an active session with the given key.
    pub fn contains_key(&self, session_key: &str) -> bool {
        self.sessions.contains_key(session_key)
    }

    /// Returns the number of active sessions.
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    /// Returns true if there are no active sessions.
    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }
}

/// Notifies subscribers about an abort event.
///
/// Sends an `AgentEvent::Error` to the event channel indicating
/// that the agent execution was aborted.
pub async fn notify_abort(event_tx: &mpsc::Sender<AgentEvent>, session_key: &str) {
    let _ = event_tx
        .send(AgentEvent::Error {
            message: format!("Agent execution aborted for session: {}", session_key),
        })
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abort_handle_new() {
        let handle = AbortHandle::new("test_session".to_string());
        assert_eq!(handle.session_key(), "test_session");
        assert!(!handle.is_aborted());
    }

    #[test]
    fn test_abort_handle_abort() {
        let handle = AbortHandle::new("test_session".to_string());
        handle.abort();
        assert!(handle.is_aborted());
    }

    #[test]
    fn test_abort_registry_insert_get_remove() {
        let registry = AbortRegistry::new();
        let handle = AbortHandle::new("test_session".to_string());

        assert!(registry.get("test_session").is_none());
        assert!(registry.insert("test_session", handle).is_none());
        assert!(registry.get("test_session").is_some());
        assert!(registry.contains_key("test_session"));
        assert_eq!(registry.len(), 1);

        let removed = registry.remove("test_session");
        assert!(removed.is_some());
        assert!(registry.get("test_session").is_none());
        assert!(registry.is_empty());
    }

    #[test]
    fn test_abort_registry_insert_duplicate() {
        let registry = AbortRegistry::new();
        let handle1 = AbortHandle::new("test_session".to_string());
        let handle2 = AbortHandle::new("test_session".to_string());

        let existing = registry.insert("test_session", handle1);
        assert!(existing.is_none());

        let existing = registry.insert("test_session", handle2);
        assert!(existing.is_some());
        assert!(existing.unwrap().is_aborted()); // The old handle was aborted
    }
}
