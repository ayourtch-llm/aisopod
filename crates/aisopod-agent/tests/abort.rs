//! Abort mechanism tests for agent engine.
//!
//! This module tests the abort handle and registry functionality
//! for cancelling agent executions.

use aisopod_agent::{notify_abort, AbortHandle, AbortRegistry};
use tokio::sync::mpsc;

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
fn test_abort_handle_session_key() {
    let handle = AbortHandle::new("my_session_123".to_string());
    assert_eq!(handle.session_key(), "my_session_123");
}

#[test]
fn test_abort_handle_token() {
    let handle = AbortHandle::new("test_session".to_string());
    let token = handle.token();
    assert!(!token.is_cancelled());

    handle.abort();
    assert!(token.is_cancelled());
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

    let existing = registry.insert("test_session", handle1.clone());
    assert!(existing.is_none());

    let existing = registry.insert("test_session", handle2);
    assert!(existing.is_some());
    assert!(existing.unwrap().is_aborted()); // The old handle was aborted
    assert!(handle1.is_aborted()); // handle1 shares the same token with the old handle
}

#[test]
fn test_abort_registry_multiple_sessions() {
    let registry = AbortRegistry::new();

    let handle1 = AbortHandle::new("session_1".to_string());
    let handle2 = AbortHandle::new("session_2".to_string());
    let handle3 = AbortHandle::new("session_3".to_string());

    registry.insert("session_1", handle1);
    registry.insert("session_2", handle2);
    registry.insert("session_3", handle3);

    assert_eq!(registry.len(), 3);
    assert!(registry.get("session_1").is_some());
    assert!(registry.get("session_2").is_some());
    assert!(registry.get("session_3").is_some());
}

#[test]
fn test_abort_registry_contains_key() {
    let registry = AbortRegistry::new();

    registry.insert("test_session", AbortHandle::new("test_session".to_string()));

    assert!(registry.contains_key("test_session"));
    assert!(!registry.contains_key("nonexistent"));
}

#[test]
fn test_abort_registry_remove_nonexistent() {
    let registry = AbortRegistry::new();

    let removed = registry.remove("nonexistent");
    assert!(removed.is_none());
    assert!(registry.is_empty());
}

#[test]
fn test_abort_registry_len_is_empty() {
    let registry = AbortRegistry::new();

    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);

    registry.insert("session_1", AbortHandle::new("session_1".to_string()));
    assert!(!registry.is_empty());
    assert_eq!(registry.len(), 1);
}

#[test]
fn test_abort_registry_replace_handle() {
    let registry = AbortRegistry::new();

    let handle1 = AbortHandle::new("session_1".to_string());
    let handle2 = AbortHandle::new("session_1".to_string());

    // First insert
    let replaced = registry.insert("session_1", handle1.clone());
    assert!(replaced.is_none());
    assert!(!handle1.is_aborted());

    // Second insert should abort the first
    let replaced = registry.insert("session_1", handle2.clone());
    assert!(replaced.is_some());
    assert!(replaced.unwrap().is_aborted());
    assert!(!handle2.is_aborted());
}

#[tokio::test]
async fn test_notify_abort() {
    let (tx, mut rx) = mpsc::channel::<aisopod_agent::AgentEvent>(10);

    notify_abort(&tx, "test_session").await;

    let event = rx.recv().await.expect("Expected an event");

    match event {
        aisopod_agent::AgentEvent::Error { message } => {
            assert!(message.contains("test_session"));
            assert!(message.contains("aborted"));
        }
        _ => panic!("Expected AgentEvent::Error"),
    }
}

#[test]
fn test_abort_handle_clone() {
    let handle = AbortHandle::new("test_session".to_string());
    let cloned = handle.clone();

    assert_eq!(handle.session_key(), cloned.session_key());
    assert!(!cloned.is_aborted());

    cloned.abort();
    assert!(handle.is_aborted());
}

#[tokio::test]
async fn test_abort_handle_cancelled_future() {
    let handle = AbortHandle::new("test_session".to_string());

    // Test that cancelled() returns immediately after abort
    handle.abort();
    handle.cancelled().await;
    // If we reach here, the test passed
}

#[test]
fn test_abort_registry_clear_all() {
    let registry = AbortRegistry::new();

    registry.insert("session_1", AbortHandle::new("session_1".to_string()));
    registry.insert("session_2", AbortHandle::new("session_2".to_string()));
    registry.insert("session_3", AbortHandle::new("session_3".to_string()));

    assert_eq!(registry.len(), 3);

    // Remove all
    registry.remove("session_1");
    registry.remove("session_2");
    registry.remove("session_3");

    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}
