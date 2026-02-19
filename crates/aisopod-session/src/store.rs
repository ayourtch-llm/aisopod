//! Session store for managing conversation sessions.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// A store for managing conversation sessions.
///
/// The `SessionStore` provides a thread-safe registry for storing,
/// retrieving, and managing sessions by their keys.
///
/// # Example
///
/// ```ignore
/// use std::sync::Arc;
/// use aisopod_session::{SessionStore, Session};
///
/// let store = SessionStore::new();
///
/// // Create and store a session
/// let session = Session::new("session_123".to_string());
/// store.insert(session).unwrap();
///
/// // Retrieve a session
/// if let Some(retrieved) = store.get("session_123") {
///     println!("Found session: {}", retrieved.key());
/// }
/// ```
#[derive(Default)]
pub struct SessionStore {
    sessions: HashMap<String, Arc<Session>>,
}

impl SessionStore {
    /// Creates a new empty `SessionStore`.
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Inserts a session into the store.
    ///
    /// Returns `Ok(())` if the session was inserted, or `Err` if
    /// a session with the same key already exists.
    ///
    /// # Arguments
    ///
    /// * `session` - The session to insert.
    pub fn insert(&mut self, session: Arc<Session>) -> Result<(), String> {
        let key = session.key.clone();
        if self.sessions.contains_key(&key) {
            return Err(format!("Session '{}' already exists", key));
        }
        self.sessions.insert(key, session);
        Ok(())
    }

    /// Retrieves a session by its key.
    ///
    /// Returns `Some` with an `Arc` to the session if found, `None` otherwise.
    ///
    /// # Arguments
    ///
    /// * `key` - The session key to look up.
    pub fn get(&self, key: &str) -> Option<Arc<Session>> {
        self.sessions.get(key).cloned()
    }

    /// Removes a session from the store.
    ///
    /// Returns `Some` with the removed session if found, `None` otherwise.
    ///
    /// # Arguments
    ///
    /// * `key` - The session key to remove.
    pub fn remove(&mut self, key: &str) -> Option<Arc<Session>> {
        self.sessions.remove(key)
    }

    /// Returns the number of sessions in the store.
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    /// Returns `true` if the store contains no sessions.
    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }
}

/// A conversation session.
///
/// Represents a single conversation session with a unique key,
/// message history, and associated metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// The unique identifier for this session.
    pub key: String,
    /// Optional metadata associated with the session.
    pub metadata: Option<serde_json::Value>,
}

impl Session {
    /// Creates a new session with the given key.
    ///
    /// # Arguments
    ///
    /// * `key` - The unique identifier for this session.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            metadata: None,
        }
    }

    /// Creates a new session with the given key and metadata.
    ///
    /// # Arguments
    ///
    /// * `key` - The unique identifier for this session.
    /// * `metadata` - Optional metadata for the session.
    pub fn with_metadata(key: impl Into<String>, metadata: serde_json::Value) -> Self {
        Self {
            key: key.into(),
            metadata: Some(metadata),
        }
    }

    /// Returns the session key.
    pub fn key(&self) -> &str {
        &self.key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn test_new_session() {
        let session = Session::new("session_123");
        assert_eq!(session.key(), "session_123");
        assert!(session.metadata.is_none());
    }

    #[test]
    fn test_session_with_metadata() {
        let session = Session::with_metadata("session_456", json!({"user": "test"}));
        assert_eq!(session.key(), "session_456");
        assert!(session.metadata.is_some());
    }

    #[test]
    fn test_new_store_is_empty() {
        let store = SessionStore::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_insert_and_get_session() {
        let mut store = SessionStore::new();
        let session = Arc::new(Session::new("test_session"));
        
        store.insert(Arc::clone(&session)).unwrap();
        
        assert_eq!(store.len(), 1);
        assert!(store.get("test_session").is_some());
    }

    #[test]
    fn test_get_nonexistent_session() {
        let store = SessionStore::new();
        assert!(store.get("nonexistent").is_none());
    }

    #[test]
    fn test_remove_session() {
        let mut store = SessionStore::new();
        let session = Arc::new(Session::new("removable"));
        
        store.insert(Arc::clone(&session)).unwrap();
        assert!(store.remove("removable").is_some());
        assert!(store.is_empty());
    }

    #[test]
    fn test_duplicate_insert_fails() {
        let mut store = SessionStore::new();
        let session = Arc::new(Session::new("duplicate"));
        
        store.insert(Arc::clone(&session)).unwrap();
        assert!(store.insert(session).is_err());
    }
}