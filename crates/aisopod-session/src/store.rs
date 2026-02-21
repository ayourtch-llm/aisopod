//! Session store for managing conversation sessions with SQLite persistence.

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Result as SqliteResult};
use std::path::Path;

use crate::db;
use crate::types::{Session, SessionFilter, SessionKey, SessionPatch, SessionStatus, SessionSummary, StoredMessage};

/// A store for managing conversation sessions using SQLite.
///
/// The `SessionStore` provides persistent storage for sessions with full CRUD
/// operations: create (via get_or_create), read (get), update (patch), and delete.
///
/// # Example
///
/// ```ignore
/// use std::path::Path;
/// use aisopod_session::{SessionStore, SessionKey};
///
/// let store = SessionStore::new(Path::new("sessions.db"))?;
///
/// // Create or retrieve a session
/// let key = SessionKey {
///     agent_id: "agent_001".to_string(),
///     channel: "discord".to_string(),
///     account_id: "bot_123".to_string(),
///     peer_kind: "dm".to_string(),
///     peer_id: "user_456".to_string(),
/// };
/// let session = store.get_or_create(&key)?;
///
/// // List sessions with filters
/// let filtered = store.list(&SessionFilter::for_agent("agent_001"))?;
///
/// // Update a session
/// let updated = store.patch(&key, &SessionPatch::with_status(SessionStatus::Idle))?;
///
/// // Delete a session (also deletes associated messages via CASCADE)
/// store.delete(&key)?;
/// ```
#[derive(Debug)]
pub struct SessionStore {
    conn: Connection,
}

impl SessionStore {
    /// Creates a new SessionStore backed by a SQLite database.
    ///
    /// Opens or creates the database at the given path and runs any pending
    /// migrations to ensure the schema is up to date.
    ///
    /// # Arguments
    ///
    /// * `path` - The file system path where the database file should be stored.
    ///
    /// # Returns
    ///
    /// Returns `Ok(SessionStore)` with the database connection if successful,
    /// or an error if the database cannot be opened or migrations fail.
    pub fn new(path: &Path) -> Result<Self> {
        let conn = db::open_database(path)?;
        Ok(Self { conn })
    }

    /// Gets an existing session or creates a new one if it doesn't exist.
    ///
    /// Queries the database for a session matching all five key fields.
    /// If found, returns the existing session. If not found, creates a new
    /// session with default values and returns it.
    ///
    /// # Arguments
    ///
    /// * `key` - The session key to look up or create.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Session)` with either the existing or newly created session.
    pub fn get_or_create(&self, key: &SessionKey) -> Result<Session> {
        // First, try to find an existing session
        let session = self.get_by_key(key)?;
        if let Some(session) = session {
            return Ok(session);
        }

        // If not found, create a new session
        let now = Utc::now().to_rfc3339();
        
        self.conn.execute(
            r#"
            INSERT INTO sessions 
                (agent_id, channel, account_id, peer_kind, peer_id, 
                 created_at, updated_at, message_count, token_usage, metadata, status)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                key.agent_id,
                key.channel,
                key.account_id,
                key.peer_kind,
                key.peer_id,
                now,
                now,
                0i64,
                0i64,
                "{}",
                "active",
            ],
        )?;

        // Now fetch the newly created session
        let session = self.get_by_key(key)?;
        session.ok_or_else(|| {
            anyhow::anyhow!("Failed to retrieve newly created session for key {:?}", key)
        })
    }

    /// Retrieves a session by its key.
    ///
    /// # Arguments
    ///
    /// * `key` - The session key to look up.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(Session))` if found, `Ok(None)` if not found,
    /// or an error if the database query fails.
    pub fn get(&self, key: &SessionKey) -> Result<Option<Session>> {
        self.get_by_key(key)
    }

    /// Internal helper to get a session by key.
    fn get_by_key(&self, key: &SessionKey) -> Result<Option<Session>> {
        let session: Option<Session> = self.conn.query_row(
            r#"
            SELECT id, agent_id, channel, account_id, peer_kind, peer_id,
                   created_at, updated_at, message_count, token_usage, metadata, status
            FROM sessions
            WHERE agent_id = ? AND channel = ? AND account_id = ? 
                  AND peer_kind = ? AND peer_id = ?
            "#,
            params![
                key.agent_id,
                key.channel,
                key.account_id,
                key.peer_kind,
                key.peer_id,
            ],
            |row| self.row_to_session(row),
        ).optional()?;

        Ok(session)
    }

    /// Lists sessions matching the given filter.
    ///
    /// Builds a dynamic query based on which filter fields are set.
    /// Supports filtering by agent_id, channel, account_id, peer_kind, peer_id,
    /// status, and date ranges on created_at and updated_at.
    ///
    /// # Arguments
    ///
    /// * `filter` - The filter criteria. All fields are optional; unset fields are ignored.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Vec<SessionSummary>)` with matching sessions ordered by updated_at DESC,
    /// or an error if the database query fails.
    pub fn list(&self, filter: &SessionFilter) -> Result<Vec<SessionSummary>> {
        let mut query = String::from(
            r#"
            SELECT DISTINCT agent_id, channel, account_id, peer_kind, peer_id,
                   status, message_count, updated_at
            FROM sessions
            "#,
        );

        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref agent_id) = filter.agent_id {
            conditions.push("agent_id = ?");
            params.push(Box::new(agent_id.clone()));
        }

        if let Some(ref channel) = filter.channel {
            conditions.push("channel = ?");
            params.push(Box::new(channel.clone()));
        }

        if let Some(ref account_id) = filter.account_id {
            conditions.push("account_id = ?");
            params.push(Box::new(account_id.clone()));
        }

        if let Some(ref peer_kind) = filter.peer_kind {
            conditions.push("peer_kind = ?");
            params.push(Box::new(peer_kind.clone()));
        }

        if let Some(ref peer_id) = filter.peer_id {
            conditions.push("peer_id = ?");
            params.push(Box::new(peer_id.clone()));
        }

        if let Some(ref status) = filter.status {
            let status_str = match status {
                SessionStatus::Active => "active",
                SessionStatus::Idle => "idle",
                SessionStatus::Compacted => "compacted",
                SessionStatus::Archived => "archived",
            };
            conditions.push("status = ?");
            params.push(Box::new(status_str.to_string()));
        }

        if let Some(ref created_after) = filter.created_after {
            conditions.push("created_at > ?");
            params.push(Box::new(created_after.to_rfc3339()));
        }

        if let Some(ref created_before) = filter.created_before {
            conditions.push("created_at < ?");
            params.push(Box::new(created_before.to_rfc3339()));
        }

        if let Some(ref updated_after) = filter.updated_after {
            conditions.push("updated_at > ?");
            params.push(Box::new(updated_after.to_rfc3339()));
        }

        if let Some(ref updated_before) = filter.updated_before {
            conditions.push("updated_at < ?");
            params.push(Box::new(updated_before.to_rfc3339()));
        }

        if !conditions.is_empty() {
            query.push_str("WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(" ORDER BY updated_at DESC");

        let mut stmt = self.conn.prepare(&query)?;
        let session_summaries = stmt
            .query_map(rusqlite::params_from_iter(params.iter()), |row| {
                self.row_to_session_summary(row)
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(session_summaries)
    }

    /// Partially updates a session by key.
    ///
    /// Updates only the fields that are specified in the patch (not None).
    /// Always updates the `updated_at` timestamp to the current time.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the session to update.
    /// * `patch` - The patch containing fields to update.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Session)` with the updated session, or an error if the
    /// session is not found or the database operation fails.
    pub fn patch(&self, key: &SessionKey, patch: &SessionPatch) -> Result<Session> {
        let mut set_clauses = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref status) = patch.status {
            let status_str = match status {
                SessionStatus::Active => "active",
                SessionStatus::Idle => "idle",
                SessionStatus::Compacted => "compacted",
                SessionStatus::Archived => "archived",
            };
            set_clauses.push("status = ?");
            params.push(Box::new(status_str.to_string()));
        }

        if let Some(message_count) = patch.message_count {
            set_clauses.push("message_count = ?");
            params.push(Box::new(message_count as i64));
        }

        if let Some(token_usage) = patch.token_usage {
            set_clauses.push("token_usage = ?");
            params.push(Box::new(token_usage as i64));
        }

        if let Some(ref metadata) = patch.metadata {
            set_clauses.push("metadata = ?");
            params.push(Box::new(metadata.to_string()));
        }

        // Always update updated_at
        set_clauses.push("updated_at = ?");
        params.push(Box::new(Utc::now().to_rfc3339()));

        if set_clauses.is_empty() {
            // No fields to update, just return the current session
            let session = self.get_by_key(key)?;
            return session.ok_or_else(|| {
                anyhow::anyhow!("Session not found for key {:?}", key)
            });
        }

        params.push(Box::new(key.agent_id.clone()));
        params.push(Box::new(key.channel.clone()));
        params.push(Box::new(key.account_id.clone()));
        params.push(Box::new(key.peer_kind.clone()));
        params.push(Box::new(key.peer_id.clone()));

        let sql = format!(
            "UPDATE sessions SET {} WHERE agent_id = ? AND channel = ? AND account_id = ? AND peer_kind = ? AND peer_id = ?",
            set_clauses.join(", ")
        );

        self.conn.execute(&sql, rusqlite::params_from_iter(params.iter()))?;

        let session = self.get_by_key(key)?;
        session.ok_or_else(|| {
            anyhow::anyhow!("Session not found for key {:?} after update", key)
        })
    }

    /// Deletes a session and all its associated messages.
    ///
    /// Due to the `ON DELETE CASCADE` constraint on the messages table,
    /// all messages associated with this session are automatically deleted.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the session to delete.
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if a session was deleted, `Ok(false)` if no
    /// session matched the key, or an error if the database operation fails.
    pub fn delete(&self, key: &SessionKey) -> Result<bool> {
        let rows_affected = self.conn.execute(
            r#"
            DELETE FROM sessions
            WHERE agent_id = ? AND channel = ? AND account_id = ? 
                  AND peer_kind = ? AND peer_id = ?
            "#,
            params![
                key.agent_id,
                key.channel,
                key.account_id,
                key.peer_kind,
                key.peer_id,
            ],
        )?;

        Ok(rows_affected > 0)
    }

    /// Converts a database row to a Session struct.
    fn row_to_session(&self, row: &rusqlite::Row) -> SqliteResult<Session> {
        let id: i64 = row.get(0)?;
        let agent_id: String = row.get(1)?;
        let channel: String = row.get(2)?;
        let account_id: String = row.get(3)?;
        let peer_kind: String = row.get(4)?;
        let peer_id: String = row.get(5)?;
        let created_at: String = row.get(6)?;
        let updated_at: String = row.get(7)?;
        let message_count: i64 = row.get(8)?;
        let token_usage: i64 = row.get(9)?;
        let metadata: String = row.get(10)?;
        let status: String = row.get(11)?;

        let metadata_value: serde_json::Value = serde_json::from_str(&metadata)
            .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));

        Ok(Session {
            key: SessionKey {
                agent_id,
                channel,
                account_id,
                peer_kind,
                peer_id,
            },
            created_at: DateTime::parse_from_rfc3339(&created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            message_count: message_count as u64,
            token_usage: token_usage as u64,
            metadata: crate::types::SessionMetadata {
                inner: metadata_value
                    .as_object()
                    .map(|map| map.clone().into_iter().collect())
                    .unwrap_or_default(),
            },
            status: match status.as_str() {
                "idle" => SessionStatus::Idle,
                "compacted" => SessionStatus::Compacted,
                "archived" => SessionStatus::Archived,
                _ => SessionStatus::Active,
            },
        })
    }

    /// Converts a database row to a SessionSummary struct.
    fn row_to_session_summary(&self, row: &rusqlite::Row) -> SqliteResult<SessionSummary> {
        let agent_id: String = row.get(0)?;
        let channel: String = row.get(1)?;
        let account_id: String = row.get(2)?;
        let peer_kind: String = row.get(3)?;
        let peer_id: String = row.get(4)?;
        let status: String = row.get(5)?;
        let message_count: i64 = row.get(6)?;
        let updated_at: String = row.get(7)?;

        let status_enum = match status.as_str() {
            "idle" => SessionStatus::Idle,
            "compacted" => SessionStatus::Compacted,
            "archived" => SessionStatus::Archived,
            _ => SessionStatus::Active,
        };

        Ok(SessionSummary {
            key: SessionKey {
                agent_id,
                channel,
                account_id,
                peer_kind,
                peer_id,
            },
            status: status_enum,
            message_count: message_count as u64,
            updated_at: DateTime::parse_from_rfc3339(&updated_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Creates a fresh in-memory database for testing.
    fn create_test_store() -> SessionStore {
        let path = PathBuf::from(":memory:");
        SessionStore::new(&path).unwrap()
    }

    fn create_test_key() -> SessionKey {
        SessionKey {
            agent_id: "agent_001".to_string(),
            channel: "discord".to_string(),
            account_id: "bot_123".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_456".to_string(),
        }
    }

    #[test]
    fn test_new_store() {
        let store = create_test_store();
        assert!(std::ptr::addr_of!(store.conn) as *const _ as usize > 0);
    }

    #[test]
    fn test_get_or_create_new_session() {
        let store = create_test_store();
        let key = create_test_key();

        let session = store.get_or_create(&key).unwrap();

        assert_eq!(session.key.agent_id, key.agent_id);
        assert_eq!(session.key.channel, key.channel);
        assert_eq!(session.key.account_id, key.account_id);
        assert_eq!(session.key.peer_kind, key.peer_kind);
        assert_eq!(session.key.peer_id, key.peer_id);
        assert_eq!(session.message_count, 0);
        assert_eq!(session.token_usage, 0);
        assert!(session.metadata.inner.is_empty());
        assert_eq!(session.status, SessionStatus::Active);
    }

    #[test]
    fn test_get_or_create_existing_session() {
        let store = create_test_store();
        let key = create_test_key();

        // Create initial session
        let session1 = store.get_or_create(&key).unwrap();
        let initial_updated = session1.updated_at;

        // Sleep briefly to ensure timestamp would be different if it created a new session
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Get the same session again
        let session2 = store.get_or_create(&key).unwrap();

        assert_eq!(session1.key, session2.key);
        assert_eq!(session1.message_count, session2.message_count);
        assert_eq!(session1.status, session2.status);
        // Should be the same session (not a new one)
        assert_eq!(session1.updated_at, session2.updated_at);
    }

    #[test]
    fn test_get_nonexistent_session() {
        let store = create_test_store();
        let key = create_test_key();

        let result = store.get(&key).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_all_sessions() {
        let store = create_test_store();

        // Create multiple sessions
        let key1 = SessionKey {
            agent_id: "agent_001".to_string(),
            channel: "discord".to_string(),
            account_id: "bot_123".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_111".to_string(),
        };
        let key2 = SessionKey {
            agent_id: "agent_001".to_string(),
            channel: "discord".to_string(),
            account_id: "bot_123".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_222".to_string(),
        };
        let key3 = SessionKey {
            agent_id: "agent_002".to_string(),
            channel: "slack".to_string(),
            account_id: "bot_456".to_string(),
            peer_kind: "group".to_string(),
            peer_id: "channel_789".to_string(),
        };

        store.get_or_create(&key1).unwrap();
        store.get_or_create(&key2).unwrap();
        store.get_or_create(&key3).unwrap();

        // List all sessions
        let sessions = store.list(&SessionFilter::new()).unwrap();
        assert_eq!(sessions.len(), 3);
    }

    #[test]
    fn test_list_filter_by_agent() {
        let store = create_test_store();

        let key1 = SessionKey {
            agent_id: "agent_001".to_string(),
            channel: "discord".to_string(),
            account_id: "bot_123".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_111".to_string(),
        };
        let key2 = SessionKey {
            agent_id: "agent_001".to_string(),
            channel: "discord".to_string(),
            account_id: "bot_123".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_222".to_string(),
        };
        let key3 = SessionKey {
            agent_id: "agent_002".to_string(),
            channel: "slack".to_string(),
            account_id: "bot_456".to_string(),
            peer_kind: "group".to_string(),
            peer_id: "channel_789".to_string(),
        };

        store.get_or_create(&key1).unwrap();
        store.get_or_create(&key2).unwrap();
        store.get_or_create(&key3).unwrap();

        let filter = SessionFilter::for_agent("agent_001");
        let sessions = store.list(&filter).unwrap();
        assert_eq!(sessions.len(), 2);
        for session in sessions {
            assert_eq!(session.key.agent_id, "agent_001");
        }
    }

    #[test]
    fn test_list_filter_by_channel() {
        let store = create_test_store();

        let key1 = SessionKey {
            agent_id: "agent_001".to_string(),
            channel: "discord".to_string(),
            account_id: "bot_123".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_111".to_string(),
        };
        let key2 = SessionKey {
            agent_id: "agent_001".to_string(),
            channel: "slack".to_string(),
            account_id: "bot_123".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_222".to_string(),
        };

        store.get_or_create(&key1).unwrap();
        store.get_or_create(&key2).unwrap();

        let filter = SessionFilter::for_channel("discord");
        let sessions = store.list(&filter).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].key.channel, "discord");
    }

    #[test]
    fn test_list_filter_by_status() {
        let store = create_test_store();

        let key1 = SessionKey {
            agent_id: "agent_001".to_string(),
            channel: "discord".to_string(),
            account_id: "bot_123".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_111".to_string(),
        };
        let key2 = SessionKey {
            agent_id: "agent_001".to_string(),
            channel: "discord".to_string(),
            account_id: "bot_123".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_222".to_string(),
        };

        store.get_or_create(&key1).unwrap();
        store.get_or_create(&key2).unwrap();

        // Patch key2 to set it to Idle
        store.patch(&key2, &SessionPatch::with_status(SessionStatus::Idle)).unwrap();

        let mut filter = SessionFilter::new();
        filter.status = Some(SessionStatus::Idle);
        let sessions = store.list(&filter).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].status, SessionStatus::Idle);
    }

    #[test]
    fn test_patch_session() {
        let store = create_test_store();
        let key = create_test_key();

        // Create session
        let session1 = store.get_or_create(&key).unwrap();
        assert_eq!(session1.status, SessionStatus::Active);

        // Patch the status
        let session2 = store.patch(&key, &SessionPatch::with_status(SessionStatus::Idle)).unwrap();
        assert_eq!(session2.status, SessionStatus::Idle);

        // Patch message count and token usage
        let patch = SessionPatch {
            message_count: Some(10),
            token_usage: Some(1000),
            ..Default::default()
        };
        let session3 = store.patch(&key, &patch).unwrap();
        assert_eq!(session3.message_count, 10);
        assert_eq!(session3.token_usage, 1000);
    }

    #[test]
    fn test_patch_no_changes() {
        let store = create_test_store();
        let key = create_test_key();

        // Create session
        store.get_or_create(&key).unwrap();

        // Patch with empty patch (no changes)
        let patch = SessionPatch::new();
        let session = store.patch(&key, &patch).unwrap();

        // Should still get the session back
        assert_eq!(session.key, key);
    }

    #[test]
    fn test_delete_session() {
        let store = create_test_store();
        let key = create_test_key();

        // Create session
        store.get_or_create(&key).unwrap();

        // Verify session exists
        let session = store.get(&key).unwrap();
        assert!(session.is_some());

        // Delete session
        let result = store.delete(&key).unwrap();
        assert!(result);

        // Verify session no longer exists
        let session = store.get(&key).unwrap();
        assert!(session.is_none());
    }

    #[test]
    fn test_delete_nonexistent_session() {
        let store = create_test_store();
        let key = create_test_key();

        // Try to delete non-existent session
        let result = store.delete(&key).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_delete_cascades_to_messages() {
        let store = create_test_store();
        let key = create_test_key();

        // Create session
        store.get_or_create(&key).unwrap();

        // Insert a message directly into the database
        let session_id: i64 = store
            .conn
            .query_row(
                "SELECT id FROM sessions WHERE agent_id = ? AND channel = ? AND account_id = ? AND peer_kind = ? AND peer_id = ?",
                params![key.agent_id, key.channel, key.account_id, key.peer_kind, key.peer_id],
                |row| row.get(0),
            )
            .unwrap();

        store
            .conn
            .execute(
                "INSERT INTO messages (session_id, role, content, created_at) VALUES (?, ?, ?, ?)",
                params![session_id, "user", "\"Hello!\"", "2024-01-01T00:00:00Z"],
            )
            .unwrap();

        // Verify message exists
        let count: i64 = store.conn
            .query_row("SELECT COUNT(*) FROM messages", params![], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        // Delete session
        store.delete(&key).unwrap();

        // Verify message was cascaded deleted
        let count: i64 = store.conn
            .query_row("SELECT COUNT(*) FROM messages", params![], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_list_with_date_filters() {
        let store = create_test_store();

        let key1 = SessionKey {
            agent_id: "agent_001".to_string(),
            channel: "discord".to_string(),
            account_id: "bot_123".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_111".to_string(),
        };
        let key2 = SessionKey {
            agent_id: "agent_001".to_string(),
            channel: "discord".to_string(),
            account_id: "bot_123".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_222".to_string(),
        };

        store.get_or_create(&key1).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        store.get_or_create(&key2).unwrap();

        // Get the created_at of the first session
        let session1 = store.get(&key1).unwrap().unwrap();
        let created_at = session1.created_at;

        // Filter by created_after
        let mut filter = SessionFilter::new();
        filter.created_after = Some(created_at);
        let sessions = store.list(&filter).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].key.peer_id, "user_222");
    }
}
