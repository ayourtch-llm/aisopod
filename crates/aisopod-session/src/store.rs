//! Session store for managing conversation sessions with SQLite persistence.

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Result as SqliteResult};
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::compaction::CompactionStrategy;
use crate::db;
use crate::types::{
    HistoryQuery, Session, SessionFilter, SessionKey, SessionPatch, SessionStatus, SessionSummary,
    StoredMessage,
};

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
/// let session = store.get_or_create("agent_001", &key)?;
///
/// // List sessions with filters
/// let filtered = store.list("agent_001", &SessionFilter::for_agent("agent_001"))?;
///
/// // Update a session
/// let updated = store.patch("agent_001", &key, &SessionPatch::with_status(SessionStatus::Idle))?;
///
/// // Delete a session (also deletes associated messages via CASCADE)
/// store.delete("agent_001", &key)?;
/// ```
#[derive(Debug)]
pub struct SessionStore {
    conn: Arc<Mutex<Connection>>,
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
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Creates a new SessionStore backed by an in-memory SQLite database.
    ///
    /// This is useful for testing where persistence is not needed.
    ///
    /// # Returns
    ///
    /// Returns `Ok(SessionStore)` with the database connection if successful,
    /// or an error if the database cannot be opened or migrations fail.
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        db::run_migrations(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Gets an existing session or creates a new one if it doesn't exist.
    ///
    /// Queries the database for a session matching all five key fields.
    /// If found, returns the existing session. If not found, creates a new
    /// session with default values and returns it.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The agent ID performing the operation (for scope validation).
    /// * `key` - The session key to look up or create.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Session)` with either the existing or newly created session.
    pub fn get_or_create(&self, agent_id: &str, key: &SessionKey) -> Result<Session> {
        // Verify scope: ensure the calling agent matches the session's agent
        Self::verify_scope(agent_id, key)?;

        // First, try to find an existing session
        let session = self.get_by_key(key)?;
        if let Some(session) = session {
            return Ok(session);
        }

        // If not found, create a new session
        let now = Utc::now().to_rfc3339();

        self.conn.lock().unwrap().execute(
            r#"
            INSERT INTO sessions
                (agent_id, channel, account_id, peer_kind, peer_id,
                 created_at, updated_at, message_count, token_usage, metadata, status,
                 compaction_count, last_compacted_at, last_compaction_summary)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
                0i64,
                None::<Option<String>>,
                None::<Option<String>>,
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
        let session: Option<Session> = self
            .conn
            .lock()
            .unwrap()
            .query_row(
                r#"
            SELECT id, agent_id, channel, account_id, peer_kind, peer_id,
                   created_at, updated_at, message_count, token_usage, metadata, status,
                   compaction_count, last_compacted_at, last_compaction_summary
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
            )
            .optional()?;

        Ok(session)
    }

    /// Internal helper to get a session's id by key.
    fn get_session_id(&self, key: &SessionKey) -> Result<Option<i64>> {
        let session_id: Option<i64> = self
            .conn
            .lock()
            .unwrap()
            .query_row(
                r#"
            SELECT id FROM sessions
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
                |row| row.get(0),
            )
            .optional()?;

        Ok(session_id)
    }

    /// Lists sessions matching the given filter.
    ///
    /// Builds a dynamic query based on which filter fields are set.
    /// Supports filtering by agent_id, channel, account_id, peer_kind, peer_id,
    /// status, and date ranges on created_at and updated_at.
    ///
    /// NOTE: This method is agent-scoped. The query ALWAYS includes a filter for
    /// the calling agent_id, regardless of what the SessionFilter specifies. This
    /// ensures agents can only see their own sessions.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The agent ID performing the operation (for scope validation).
    /// * `filter` - The filter criteria. All fields are optional; unset fields are ignored.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Vec<SessionSummary>)` with matching sessions ordered by updated_at DESC,
    /// or an error if the database query fails or the agent_id doesn't match the filter.
    pub fn list(&self, agent_id: &str, filter: &SessionFilter) -> Result<Vec<SessionSummary>> {
        // Verify scope: ensure the calling agent matches the filter's agent_id (if specified)
        // Also validate against the filter's agent_id if present
        if let Some(ref filter_agent_id) = filter.agent_id {
            if filter_agent_id != agent_id {
                return Err(anyhow::anyhow!(
                    "access denied: agent '{}' cannot access sessions owned by agent '{}'",
                    agent_id,
                    filter_agent_id
                ));
            }
        }

        // Build the query with agent_id always included
        let mut query = String::from(
            r#"
            SELECT DISTINCT agent_id, channel, account_id, peer_kind, peer_id,
                   status, message_count, updated_at
            FROM sessions
            WHERE agent_id = ?
            "#,
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(agent_id.to_string())];

        // Add other filter conditions if specified
        if let Some(ref channel) = filter.channel {
            query.push_str(" AND channel = ?");
            params.push(Box::new(channel.clone()));
        }

        if let Some(ref account_id) = filter.account_id {
            query.push_str(" AND account_id = ?");
            params.push(Box::new(account_id.clone()));
        }

        if let Some(ref peer_kind) = filter.peer_kind {
            query.push_str(" AND peer_kind = ?");
            params.push(Box::new(peer_kind.clone()));
        }

        if let Some(ref peer_id) = filter.peer_id {
            query.push_str(" AND peer_id = ?");
            params.push(Box::new(peer_id.clone()));
        }

        if let Some(ref status) = filter.status {
            let status_str = match status {
                SessionStatus::Active => "active",
                SessionStatus::Idle => "idle",
                SessionStatus::Compacted => "compacted",
                SessionStatus::Archived => "archived",
            };
            query.push_str(" AND status = ?");
            params.push(Box::new(status_str.to_string()));
        }

        if let Some(ref created_after) = filter.created_after {
            query.push_str(" AND created_at > ?");
            params.push(Box::new(created_after.to_rfc3339()));
        }

        if let Some(ref created_before) = filter.created_before {
            query.push_str(" AND created_at < ?");
            params.push(Box::new(created_before.to_rfc3339()));
        }

        if let Some(ref updated_after) = filter.updated_after {
            query.push_str(" AND updated_at > ?");
            params.push(Box::new(updated_after.to_rfc3339()));
        }

        if let Some(ref updated_before) = filter.updated_before {
            query.push_str(" AND updated_at < ?");
            params.push(Box::new(updated_before.to_rfc3339()));
        }

        query.push_str(" ORDER BY updated_at DESC");

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&query)?;
        let session_summaries: Vec<SessionSummary> = stmt
            .query_map(rusqlite::params_from_iter(params.iter()), |row| {
                self.row_to_session_summary(row)
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(session_summaries)
    }

    /// Lists all sessions across all agents for admin/monitoring purposes.
    ///
    /// WARNING: This method bypasses agent isolation and can query sessions
    /// from any agent. It should ONLY be used by trusted admin or monitoring
    /// code that has explicit authorization to view cross-agent data.
    ///
    /// # Arguments
    ///
    /// * `filter` - The filter criteria. All fields are optional; unset fields are ignored.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Vec<SessionSummary>)` with matching sessions ordered by updated_at DESC,
    /// or an error if the database query fails.
    pub fn list_all_sessions(&self, filter: &SessionFilter) -> Result<Vec<SessionSummary>> {
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

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&query)?;
        let session_summaries: Vec<SessionSummary> = stmt
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
    /// * `agent_id` - The agent ID performing the operation (for scope validation).
    /// * `key` - The key of the session to update.
    /// * `patch` - The patch containing fields to update.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Session)` with the updated session, or an error if the
    /// session is not found or the database operation fails.
    pub fn patch(&self, agent_id: &str, key: &SessionKey, patch: &SessionPatch) -> Result<Session> {
        // Verify scope: ensure the calling agent matches the session's agent
        Self::verify_scope(agent_id, key)?;

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
            return session.ok_or_else(|| anyhow::anyhow!("Session not found for key {:?}", key));
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

        self.conn
            .lock()
            .unwrap()
            .execute(&sql, rusqlite::params_from_iter(params.iter()))?;

        let session = self.get_by_key(key)?;
        session.ok_or_else(|| anyhow::anyhow!("Session not found for key {:?} after update", key))
    }

    /// Deletes a session and all its associated messages.
    ///
    /// Due to the `ON DELETE CASCADE` constraint on the messages table,
    /// all messages associated with this session are automatically deleted.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The agent ID performing the operation (for scope validation).
    /// * `key` - The key of the session to delete.
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if a session was deleted, `Ok(false)` if no
    /// session matched the key, or an error if the database operation fails.
    pub fn delete(&self, agent_id: &str, key: &SessionKey) -> Result<bool> {
        // Verify scope: ensure the calling agent matches the session's agent
        Self::verify_scope(agent_id, key)?;

        let rows_affected = self.conn.lock().unwrap().execute(
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

    /// Retrieves the current compaction record for a session.
    ///
    /// Reads compaction metadata from the session including:
    /// - compaction_count: how many times the session has been compacted
    /// - last_compacted_at: when compaction last occurred
    /// Appends messages to a session.
    ///
    /// Inserts multiple messages into the messages table within a transaction.
    /// Updates the session's message_count and updated_at after successful insertion.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The agent ID performing the operation (for scope validation).
    /// * `key` - The session key to append messages to.
    /// * `messages` - Slice of messages to append.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if the session doesn't exist
    /// or the database operation fails.
    pub fn append_messages(
        &self,
        agent_id: &str,
        key: &SessionKey,
        messages: &[StoredMessage],
    ) -> Result<()> {
        // Verify scope: ensure the calling agent matches the session's agent
        Self::verify_scope(agent_id, key)?;

        // First, look up the session's id
        let session_id = match self.get_session_id(key)? {
            Some(id) => id,
            None => return Err(anyhow::anyhow!("Session not found for key {:?}", key)),
        };

        // Start a transaction
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        // Insert each message
        for msg in messages {
            let content_str = serde_json::to_string(&msg.content)?;
            let tool_calls_str = match &msg.tool_calls {
                Some(tc) => Some(serde_json::to_string(tc)?),
                None => None,
            };

            tx.execute(
                r#"
                INSERT INTO messages (session_id, role, content, tool_calls, created_at)
                VALUES (?, ?, ?, ?, ?)
                "#,
                params![
                    session_id,
                    msg.role,
                    content_str,
                    tool_calls_str,
                    msg.created_at.to_rfc3339(),
                ],
            )?;
        }

        // Update the session's message_count and updated_at
        let new_count = messages.len() as i64;
        tx.execute(
            r#"
            UPDATE sessions
            SET message_count = message_count + ?, updated_at = ?
            WHERE id = ?
            "#,
            params![new_count, Utc::now().to_rfc3339(), session_id],
        )?;

        tx.commit()?;

        Ok(())
    }

    /// Retrieves message history for a session with optional pagination and filtering.
    ///
    /// Returns messages in chronological order (oldest first). Supports pagination
    /// with limit/offset and timestamp filtering with before/after.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The agent ID performing the operation (for scope validation).
    /// * `key` - The session key to retrieve messages for.
    /// * `query` - The query with pagination and filter options.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Vec<StoredMessage>)` with messages in chronological order,
    /// or an error if the session doesn't exist or the database operation fails.
    pub fn get_history(
        &self,
        agent_id: &str,
        key: &SessionKey,
        query: &HistoryQuery,
    ) -> Result<Vec<StoredMessage>> {
        // Verify scope: ensure the calling agent matches the session's agent
        Self::verify_scope(agent_id, key)?;

        // First, look up the session's id
        let session_id = match self.get_session_id(key)? {
            Some(id) => id,
            None => return Err(anyhow::anyhow!("Session not found for key {:?}", key)),
        };

        let mut query_sql = String::from(
            r#"
            SELECT id, session_id, role, content, tool_calls, created_at
            FROM messages
            WHERE session_id = ?
            "#,
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(session_id)];

        // Add before filter
        if let Some(ref before) = query.before {
            query_sql.push_str(" AND created_at < ?");
            let before_str: String = before.to_rfc3339();
            params.push(Box::new(before_str));
        }

        // Add after filter
        if let Some(ref after) = query.after {
            query_sql.push_str(" AND created_at > ?");
            let after_str: String = after.to_rfc3339();
            params.push(Box::new(after_str));
        }

        // Order by created_at ASC (chronological)
        query_sql.push_str(" ORDER BY created_at ASC");

        // Add limit (default to 1000 if not specified)
        let limit = query.limit.unwrap_or(1000);
        query_sql.push_str(&format!(" LIMIT {}", limit));

        // Add offset (must come after LIMIT)
        if let Some(offset) = query.offset {
            query_sql.push_str(&format!(" OFFSET {}", offset));
        }

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&query_sql)?;
        let messages: Vec<StoredMessage> = stmt
            .query_map(rusqlite::params_from_iter(params.iter()), |row| {
                self.row_to_stored_message(row)
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(messages)
    }

    /// Converts a database row to a StoredMessage struct.
    fn row_to_stored_message(&self, row: &rusqlite::Row) -> SqliteResult<StoredMessage> {
        let id: i64 = row.get(0)?;
        let session_id: i64 = row.get(1)?;
        let role: String = row.get(2)?;
        let content_str: String = row.get(3)?;
        let tool_calls_str: Option<String> = row.get(4)?;
        let created_at_str: String = row.get(5)?;

        let content: serde_json::Value = serde_json::from_str(&content_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e))
        })?;

        let tool_calls: Option<serde_json::Value> = match tool_calls_str {
            Some(s) => Some(serde_json::from_str(&s).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    4,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?),
            None => None,
        };

        let created_at: DateTime<Utc> = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    5,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

        Ok(StoredMessage {
            id,
            session_id,
            role,
            content,
            tool_calls,
            created_at,
        })
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
        let compaction_count: i64 = row.get(12)?;
        let last_compacted_at: Option<String> = row.get(13)?;
        let last_compaction_summary: Option<String> = row.get(14)?;

        let metadata_value: serde_json::Value = serde_json::from_str(&metadata)
            .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));

        let last_compacted_at_dt = last_compacted_at
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        Ok(Session {
            id,
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
            compaction_count: compaction_count as u64,
            last_compacted_at: last_compacted_at_dt,
            last_compaction_summary,
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
        SessionStore::new_in_memory().unwrap()
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
        // Just verify the store can perform operations
        let key = create_test_key();
        let session = store.get_or_create("agent_001", &key).unwrap();
        assert_eq!(session.key.agent_id, key.agent_id);
    }

    #[test]
    fn test_get_or_create_new_session() {
        let store = create_test_store();
        let key = create_test_key();

        let session = store.get_or_create("agent_001", &key).unwrap();

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
        let session1 = store.get_or_create("agent_001", &key).unwrap();
        let initial_updated = session1.updated_at;

        // Sleep briefly to ensure timestamp would be different if it created a new session
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Get the same session again
        let session2 = store.get_or_create("agent_001", &key).unwrap();

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

        store.get_or_create("agent_001", &key1).unwrap();
        store.get_or_create("agent_001", &key2).unwrap();
        store.get_or_create("agent_002", &key3).unwrap();

        // List all sessions
        let sessions = store.list("agent_001", &SessionFilter::new()).unwrap();
        assert_eq!(sessions.len(), 2); // Only sessions for agent_001
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

        store.get_or_create("agent_001", &key1).unwrap();
        store.get_or_create("agent_001", &key2).unwrap();
        store.get_or_create("agent_002", &key3).unwrap();

        let filter = SessionFilter::for_agent("agent_001");
        let sessions = store.list("agent_001", &filter).unwrap();
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

        store.get_or_create("agent_001", &key1).unwrap();
        store.get_or_create("agent_001", &key2).unwrap();

        let filter = SessionFilter::for_channel("discord");
        let sessions = store.list("agent_001", &filter).unwrap();
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

        store.get_or_create("agent_001", &key1).unwrap();
        store.get_or_create("agent_001", &key2).unwrap();

        // Patch key2 to set it to Idle
        store
            .patch(
                "agent_001",
                &key2,
                &SessionPatch::with_status(SessionStatus::Idle),
            )
            .unwrap();

        let mut filter = SessionFilter::new();
        filter.status = Some(SessionStatus::Idle);
        let sessions = store.list("agent_001", &filter).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].status, SessionStatus::Idle);
    }

    #[test]
    fn test_patch_session() {
        let store = create_test_store();
        let key = create_test_key();

        // Create session
        let session1 = store.get_or_create("agent_001", &key).unwrap();
        assert_eq!(session1.status, SessionStatus::Active);

        // Patch the status
        let session2 = store
            .patch(
                "agent_001",
                &key,
                &SessionPatch::with_status(SessionStatus::Idle),
            )
            .unwrap();
        assert_eq!(session2.status, SessionStatus::Idle);

        // Patch message count and token usage
        let patch = SessionPatch {
            message_count: Some(10),
            token_usage: Some(1000),
            ..Default::default()
        };
        let session3 = store.patch("agent_001", &key, &patch).unwrap();
        assert_eq!(session3.message_count, 10);
        assert_eq!(session3.token_usage, 1000);
    }

    #[test]
    fn test_patch_no_changes() {
        let store = create_test_store();
        let key = create_test_key();

        // Create session
        store.get_or_create("agent_001", &key).unwrap();

        // Patch with empty patch (no changes)
        let patch = SessionPatch::new();
        let session = store.patch("agent_001", &key, &patch).unwrap();

        // Should still get the session back
        assert_eq!(session.key, key);
    }

    #[test]
    fn test_delete_session() {
        let store = create_test_store();
        let key = create_test_key();

        // Create session
        store.get_or_create("agent_001", &key).unwrap();

        // Verify session exists
        let session = store.get(&key).unwrap();
        assert!(session.is_some());

        // Delete session
        let result = store.delete("agent_001", &key).unwrap();
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
        let result = store.delete("agent_001", &key).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_delete_cascades_to_messages() {
        let store = create_test_store();
        let key = create_test_key();

        // Create session
        store.get_or_create("agent_001", &key).unwrap();

        // Insert a message directly into the database
        {
            let conn = store.conn.lock().unwrap();
            let session_id: i64 = conn
                .query_row(
                    "SELECT id FROM sessions WHERE agent_id = ? AND channel = ? AND account_id = ? AND peer_kind = ? AND peer_id = ?",
                    params![key.agent_id, key.channel, key.account_id, key.peer_kind, key.peer_id],
                    |row| row.get(0),
                )
                .unwrap();

            conn.execute(
                "INSERT INTO messages (session_id, role, content, created_at) VALUES (?, ?, ?, ?)",
                params![session_id, "user", "\"Hello!\"", "2024-01-01T00:00:00Z"],
            )
            .unwrap();

            // Verify message exists
            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM messages", params![], |row| row.get(0))
                .unwrap();
            assert_eq!(count, 1);
        } // Lock is dropped here

        // Delete session
        store.delete("agent_001", &key).unwrap();

        // Verify message was cascaded deleted
        let conn = store.conn.lock().unwrap();
        let count: i64 = conn
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

        store.get_or_create("agent_001", &key1).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        store.get_or_create("agent_001", &key2).unwrap();

        // Get the created_at of the first session
        let session1 = store.get(&key1).unwrap().unwrap();
        let created_at = session1.created_at;

        // Filter by created_after
        let mut filter = SessionFilter::new();
        filter.created_after = Some(created_at);
        let sessions = store.list("agent_001", &filter).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].key.peer_id, "user_222");
    }

    #[test]
    fn test_append_messages() {
        let store = create_test_store();
        let key = create_test_key();

        // Create session
        store.get_or_create("agent_001", &key).unwrap();

        // Append messages
        let messages = vec![
            StoredMessage::user("Hello!"),
            StoredMessage::assistant("Hi there!"),
            StoredMessage::user("How are you?"),
        ];

        store.append_messages("agent_001", &key, &messages).unwrap();

        // Verify session message_count was updated
        let session = store.get(&key).unwrap().unwrap();
        assert_eq!(session.message_count, 3);
    }

    #[test]
    fn test_append_messages_nonexistent_session() {
        let store = create_test_store();
        let key = create_test_key();

        // Try to append messages to non-existent session
        let messages = vec![StoredMessage::user("Hello")];
        let result = store.append_messages("agent_001", &key, &messages);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_history_empty() {
        let store = create_test_store();
        let key = create_test_key();

        // Create session
        store.get_or_create("agent_001", &key).unwrap();

        // Get history for empty session
        let history = store
            .get_history("agent_001", &key, &HistoryQuery::default())
            .unwrap();
        assert!(history.is_empty());
    }

    #[test]
    fn test_get_history_with_messages() {
        let store = create_test_store();
        let key = create_test_key();

        // Create session
        store.get_or_create("agent_001", &key).unwrap();

        // Append messages
        let messages = vec![
            StoredMessage::user("First"),
            StoredMessage::assistant("Second"),
            StoredMessage::user("Third"),
        ];
        store.append_messages("agent_001", &key, &messages).unwrap();

        // Get history
        let history = store
            .get_history("agent_001", &key, &HistoryQuery::default())
            .unwrap();
        assert_eq!(history.len(), 3);

        // Verify messages are in chronological order
        assert_eq!(history[0].role, "user");
        assert_eq!(history[0].content, serde_json::json!("First"));
        assert_eq!(history[1].role, "assistant");
        assert_eq!(history[1].content, serde_json::json!("Second"));
        assert_eq!(history[2].role, "user");
        assert_eq!(history[2].content, serde_json::json!("Third"));
    }

    #[test]
    fn test_get_history_with_pagination() {
        let store = create_test_store();
        let key = create_test_key();

        // Create session
        store.get_or_create("agent_001", &key).unwrap();

        // Append messages
        let messages: Vec<StoredMessage> = (0..10)
            .map(|i| StoredMessage::user(format!("Message {}", i)))
            .collect();
        store.append_messages("agent_001", &key, &messages).unwrap();

        // Test limit
        let query = HistoryQuery {
            limit: Some(3),
            offset: None,
            before: None,
            after: None,
        };
        let history = store.get_history("agent_001", &key, &query).unwrap();
        assert_eq!(history.len(), 3);

        // Test offset
        let query = HistoryQuery {
            limit: Some(3),
            offset: Some(2),
            before: None,
            after: None,
        };
        let history = store.get_history("agent_001", &key, &query).unwrap();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].content, serde_json::json!("Message 2"));
    }

    #[test]
    fn test_get_history_with_timestamp_filters() {
        let store = create_test_store();
        let key = create_test_key();

        // Create session
        store.get_or_create("agent_001", &key).unwrap();

        // Insert messages directly with specific timestamps using the database
        let now = Utc::now();
        let before_10 = (now - chrono::Duration::minutes(10)).to_rfc3339();
        let before_5 = (now - chrono::Duration::minutes(5)).to_rfc3339();
        let before_2 = (now - chrono::Duration::minutes(2)).to_rfc3339();

        {
            let conn = store.conn.lock().unwrap();
            let session_id: i64 = conn
                .query_row(
                    "SELECT id FROM sessions WHERE agent_id = ? AND channel = ? AND account_id = ? AND peer_kind = ? AND peer_id = ?",
                    params![key.agent_id, key.channel, key.account_id, key.peer_kind, key.peer_id],
                    |row| row.get(0),
                )
                .unwrap();

            conn.execute(
                "INSERT INTO messages (session_id, role, content, tool_calls, created_at) VALUES (?, ?, ?, ?, ?)",
                params![session_id, "user", "\"First\"", None::<String>, before_10],
            ).unwrap();

            conn.execute(
                "INSERT INTO messages (session_id, role, content, tool_calls, created_at) VALUES (?, ?, ?, ?, ?)",
                params![session_id, "assistant", "\"Second\"", None::<String>, before_5],
            ).unwrap();

            conn.execute(
                "INSERT INTO messages (session_id, role, content, tool_calls, created_at) VALUES (?, ?, ?, ?, ?)",
                params![session_id, "user", "\"Third\"", None::<String>, before_2],
            ).unwrap();
        }

        // Update message count
        store
            .patch("agent_001", &key, &SessionPatch::with_message_count(3))
            .unwrap();

        // Test before filter (should return only First - created before 6 minutes ago)
        // Messages at 10 min ago and 5 min ago are both before 6 min ago, but the 5 min ago message
        // has created_at exactly at the boundary, so we need to be careful about comparisons.
        // The 10 min ago message should be returned.
        let query = HistoryQuery {
            limit: None,
            offset: None,
            before: Some(now - chrono::Duration::minutes(6)),
            after: None,
        };
        let history = store.get_history("agent_001", &key, &query).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].content, serde_json::json!("First"));

        // Test after filter (should return only Third - created after 4 minutes ago)
        // Messages at 2 min ago are after 4 min ago
        let query = HistoryQuery {
            limit: None,
            offset: None,
            before: None,
            after: Some(now - chrono::Duration::minutes(4)),
        };
        let history = store.get_history("agent_001", &key, &query).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].content, serde_json::json!("Third"));
    }

    #[test]
    fn test_append_messages_tool_calls() {
        let store = create_test_store();
        let key = create_test_key();

        // Create session
        store.get_or_create("agent_001", &key).unwrap();

        // Append messages with tool calls
        let messages = vec![
            StoredMessage::assistant("Let me check the weather"),
            StoredMessage::with_tool_calls(
                "tool",
                "Weather result",
                serde_json::json!({"city": "New York", "temp": 72}),
            ),
            StoredMessage::assistant("It's 72 degrees in New York"),
        ];
        store.append_messages("agent_001", &key, &messages).unwrap();

        // Get history
        let history = store
            .get_history("agent_001", &key, &HistoryQuery::default())
            .unwrap();
        assert_eq!(history.len(), 3);
        assert!(history[1].tool_calls.is_some());
    }

    #[test]
    fn test_get_compaction_record_initial() {
        let store = create_test_store();
        let key = create_test_key();

        // Create session
        store.get_or_create("agent_001", &key).unwrap();

        // Get initial compaction record
        let record = store.get_compaction_record("agent_001", &key).unwrap();
        assert_eq!(record.compaction_count, 0);
        assert!(record.last_compacted_at.is_none());
        assert!(record.summary.is_none());
    }

    #[test]
    fn test_compact_none() {
        let store = create_test_store();
        let key = create_test_key();

        // Create session
        store.get_or_create("agent_001", &key).unwrap();

        // Set initial compaction metadata via direct database update
        {
            let conn = store.conn.lock().unwrap();
            conn.execute(
                "UPDATE sessions SET compaction_count = 2, last_compacted_at = ? WHERE id = (SELECT id FROM sessions WHERE agent_id = ?)",
                params![Utc::now().to_rfc3339(), key.agent_id],
            ).unwrap();
        }

        // Try to compact with None strategy - should return unchanged record
        let record = store
            .compact(
                "agent_001",
                &key,
                &crate::compaction::CompactionStrategy::None,
                None,
            )
            .unwrap();
        assert_eq!(record.compaction_count, 2);
        assert!(record.last_compacted_at.is_some());
    }

    #[test]
    fn test_compact_sliding_window() {
        let store = create_test_store();
        let key = create_test_key();

        // Create session
        store.get_or_create("agent_001", &key).unwrap();

        // Append messages
        let messages: Vec<StoredMessage> = (0..10)
            .map(|i| StoredMessage::user(format!("Message {}", i)))
            .collect();
        store.append_messages("agent_001", &key, &messages).unwrap();

        // Compact with sliding window of 5
        let record = store
            .compact(
                "agent_001",
                &key,
                &crate::compaction::CompactionStrategy::SlidingWindow { max_messages: 5 },
                Some("summary text"),
            )
            .unwrap();

        // Check compaction record
        assert_eq!(record.compaction_count, 1);
        assert!(record.last_compacted_at.is_some());
        assert_eq!(record.summary, Some("summary text".to_string()));

        // Check message count was updated
        let session = store.get(&key).unwrap().unwrap();
        assert_eq!(session.message_count, 5);

        // Check only 5 messages remain
        let history = store
            .get_history("agent_001", &key, &HistoryQuery::default())
            .unwrap();
        assert_eq!(history.len(), 5);
    }

    #[test]
    fn test_compact_summarize() {
        let store = create_test_store();
        let key = create_test_key();

        // Create session
        store.get_or_create("agent_001", &key).unwrap();

        // Append messages
        let messages = vec![
            StoredMessage::user("Message 1"),
            StoredMessage::assistant("Message 2"),
            StoredMessage::user("Message 3"),
        ];
        store.append_messages("agent_001", &key, &messages).unwrap();

        // Check initial state
        let session = store.get(&key).unwrap().unwrap();
        assert_eq!(
            session.message_count, 3,
            "Initial message count should be 3"
        );

        // Compact with summarize strategy
        let record = store
            .compact(
                "agent_001",
                &key,
                &crate::compaction::CompactionStrategy::Summarize,
                Some("Session summary"),
            )
            .unwrap();

        // Check compaction record
        assert_eq!(record.compaction_count, 1);
        assert!(record.last_compacted_at.is_some());
        assert_eq!(record.summary, Some("Session summary".to_string()));

        // Get fresh session from DB after compact
        let session_after = store.get(&key).unwrap().unwrap();
        assert_eq!(
            session_after.message_count, 1,
            "Message count should be 1 after summarize"
        );

        // Check only summary message remains
        let history = store
            .get_history("agent_001", &key, &HistoryQuery::default())
            .unwrap();
        assert_eq!(
            history.len(),
            1,
            "Should have exactly one message after summarize"
        );
        assert_eq!(
            history[0].role, "system",
            "The summary message should have role 'system'"
        );
    }

    #[test]
    fn test_compact_nonexistent_session() {
        let store = create_test_store();
        let key = SessionKey {
            agent_id: "nonexistent".to_string(),
            channel: "test".to_string(),
            account_id: "bot".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user".to_string(),
        };

        // Try to compact a non-existent session
        let result = store.compact(
            "agent_001",
            &key,
            &crate::compaction::CompactionStrategy::None,
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_compact_sliding_window_noop_when_under_limit() {
        let store = create_test_store();
        let key = create_test_key();

        // Create session
        store.get_or_create("agent_001", &key).unwrap();

        // Append 3 messages
        let messages = vec![
            StoredMessage::user("Message 1"),
            StoredMessage::assistant("Message 2"),
            StoredMessage::user("Message 3"),
        ];
        store.append_messages("agent_001", &key, &messages).unwrap();

        // Try to compact with sliding window of 10 (more than current)
        let record = store
            .compact(
                "agent_001",
                &key,
                &crate::compaction::CompactionStrategy::SlidingWindow { max_messages: 10 },
                None,
            )
            .unwrap();

        // Should still have 3 messages
        let history = store
            .get_history("agent_001", &key, &HistoryQuery::default())
            .unwrap();
        assert_eq!(history.len(), 3);
    }
}

impl SessionStore {
    /// Gets the current compaction record for a session.
    ///
    /// Returns the compaction_count, last_compacted_at, and summary
    /// for the specified session.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The agent ID performing the operation (for scope validation).
    /// * `key` - The session key to get the compaction record for.
    ///
    /// # Returns
    ///
    /// Returns `Ok(CompactionRecord)` with the session's compaction information,
    /// or an error if the session doesn't exist or the database operation fails.
    pub fn get_compaction_record(
        &self,
        agent_id: &str,
        key: &SessionKey,
    ) -> Result<crate::compaction::CompactionRecord> {
        // Verify scope: ensure the calling agent matches the session's agent
        Self::verify_scope(agent_id, key)?;

        let session = self.get(key)?;
        match session {
            Some(s) => Ok(crate::compaction::CompactionRecord {
                compaction_count: s.compaction_count as u32,
                last_compacted_at: s.last_compacted_at,
                summary: s.last_compaction_summary,
            }),
            None => Err(anyhow::anyhow!("Session not found for key {:?}", key)),
        }
    }

    /// Compacts a session's message history according to the specified strategy.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The agent ID performing the operation (for scope validation).
    /// * `key` - The session key to compact.
    /// * `strategy` - The compaction strategy to apply.
    /// * `summary` - Optional summary text to store with the compaction.
    ///
    /// # Returns
    ///
    /// Returns `Ok(CompactionRecord)` with the updated compaction information.
    pub fn compact(
        &self,
        agent_id: &str,
        key: &SessionKey,
        strategy: &crate::compaction::CompactionStrategy,
        summary: Option<&str>,
    ) -> Result<crate::compaction::CompactionRecord> {
        // Verify scope: ensure the calling agent matches the session's agent
        Self::verify_scope(agent_id, key)?;

        // First, get the session
        let session_id = match self.get_session_id(key)? {
            Some(id) => id,
            None => return Err(anyhow::anyhow!("Session not found for key {:?}", key)),
        };

        // Lock the connection
        let conn = self.conn.lock().unwrap();

        match strategy {
            crate::compaction::CompactionStrategy::None => {
                // For None strategy, we need to get the current compaction record
                // We'll fetch it directly from the database to avoid deadlock
                let record: crate::compaction::CompactionRecord = conn.query_row(
                    "SELECT compaction_count, last_compacted_at, last_compaction_summary FROM sessions WHERE id = ?",
                    params![session_id],
                    |row| {
                        let compaction_count: i64 = row.get(0)?;
                        let last_compacted_at: Option<String> = row.get(1)?;
                        let last_compaction_summary: Option<String> = row.get(2)?;

                        let last_compacted_at_dt = last_compacted_at
                            .as_ref()
                            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.with_timezone(&Utc));

                        Ok(crate::compaction::CompactionRecord {
                            compaction_count: compaction_count as u32,
                            last_compacted_at: last_compacted_at_dt,
                            summary: last_compaction_summary,
                        })
                    },
                ).map_err(|e| anyhow::anyhow!("Failed to get compaction record: {}", e))?;

                Ok(record)
            }
            crate::compaction::CompactionStrategy::SlidingWindow { max_messages } => {
                // Delete oldest messages beyond the window
                // Get the count of messages to delete
                let current_count: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM messages WHERE session_id = ?",
                    params![session_id],
                    |row| row.get(0),
                )?;

                let count_to_delete = current_count - *max_messages as i64;

                if count_to_delete > 0 {
                    // Delete the oldest messages (those with smallest ids, since they were created first)
                    conn.execute(
                        "DELETE FROM messages WHERE session_id = ? AND id IN (
                            SELECT id FROM messages WHERE session_id = ?
                            ORDER BY id ASC LIMIT ?
                        )",
                        params![session_id, session_id, count_to_delete],
                    )?;
                }

                // Update message count
                let new_message_count: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM messages WHERE session_id = ?",
                    params![session_id],
                    |row| row.get(0),
                )?;

                // Update session compaction metadata
                conn.execute(
                    "UPDATE sessions
                     SET message_count = ?,
                         compaction_count = compaction_count + 1,
                         last_compacted_at = ?,
                         last_compaction_summary = ?,
                         updated_at = ?,
                         status = 'compacted'
                     WHERE id = ?",
                    params![
                        new_message_count,
                        Utc::now().to_rfc3339(),
                        summary,
                        Utc::now().to_rfc3339(),
                        session_id,
                    ],
                )?;

                // Fetch the updated record directly from the database
                let record: crate::compaction::CompactionRecord = conn.query_row(
                    "SELECT compaction_count, last_compacted_at, last_compaction_summary FROM sessions WHERE id = ?",
                    params![session_id],
                    |row| {
                        let compaction_count: i64 = row.get(0)?;
                        let last_compacted_at: Option<String> = row.get(1)?;
                        let last_compaction_summary: Option<String> = row.get(2)?;

                        let last_compacted_at_dt = last_compacted_at
                            .as_ref()
                            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.with_timezone(&Utc));

                        Ok(crate::compaction::CompactionRecord {
                            compaction_count: compaction_count as u32,
                            last_compacted_at: last_compacted_at_dt,
                            summary: last_compaction_summary,
                        })
                    },
                ).map_err(|e| anyhow::anyhow!("Failed to get compaction record: {}", e))?;

                Ok(record)
            }
            crate::compaction::CompactionStrategy::Summarize => {
                // Delete all messages
                conn.execute(
                    "DELETE FROM messages WHERE session_id = ?",
                    params![session_id],
                )?;

                // Insert a single summary message with role="system"
                let now = Utc::now();
                let summary_text = summary.unwrap_or("Session has been compacted.");

                // Serialize the summary text as JSON string (stored_messages expect JSON content)
                let summary_json = serde_json::Value::String(summary_text.to_string());
                let summary_json_str =
                    serde_json::to_string(&summary_json).expect("Failed to serialize summary JSON");

                // Insert the summary message with role="system"
                conn.execute(
                    "INSERT INTO messages (session_id, role, content, created_at) VALUES (?, ?, ?, ?)",
                    params![
                        session_id,
                        "system",
                        summary_json_str,
                        now.to_rfc3339(),
                    ],
                )?;

                // Update session metadata
                conn.execute(
                    "UPDATE sessions
                     SET message_count = 1,
                         compaction_count = compaction_count + 1,
                         last_compacted_at = ?,
                         last_compaction_summary = ?,
                         updated_at = ?,
                         status = 'compacted'
                     WHERE id = ?",
                    params![now.to_rfc3339(), summary, now.to_rfc3339(), session_id,],
                )?;

                // Fetch the updated record directly from the database
                let record: crate::compaction::CompactionRecord = conn.query_row(
                    "SELECT compaction_count, last_compacted_at, last_compaction_summary FROM sessions WHERE id = ?",
                    params![session_id],
                    |row| {
                        let compaction_count: i64 = row.get(0)?;
                        let last_compacted_at: Option<String> = row.get(1)?;
                        let last_compaction_summary: Option<String> = row.get(2)?;

                        let last_compacted_at_dt = last_compacted_at
                            .as_ref()
                            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.with_timezone(&Utc));

                        Ok(crate::compaction::CompactionRecord {
                            compaction_count: compaction_count as u32,
                            last_compacted_at: last_compacted_at_dt,
                            summary: last_compaction_summary,
                        })
                    },
                ).map_err(|e| anyhow::anyhow!("Failed to get compaction record: {}", e))?;

                Ok(record)
            }
        }
    }

    /// Verifies that the calling agent_id matches the session's agent_id.
    ///
    /// This is a security check to ensure agents cannot access sessions
    /// belonging to other agents.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The agent ID performing the operation.
    /// * `key` - The session key being accessed.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the agent_id matches the key's agent_id,
    /// or an error with a descriptive message if they don't match.
    fn verify_scope(agent_id: &str, key: &SessionKey) -> Result<()> {
        if agent_id != key.agent_id {
            return Err(anyhow::anyhow!(
                "access denied: agent '{}' cannot access sessions owned by agent '{}'",
                agent_id,
                key.agent_id
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod store_tests {
    use super::*;
    use crate::CompactionStrategy;
    use chrono::{DateTime, Duration, Utc};

    /// Creates a fresh in-memory database for testing.
    fn create_test_store() -> SessionStore {
        SessionStore::new_in_memory().unwrap()
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
    fn test_scope_validation_agent_mismatch_on_create() {
        let store = create_test_store();
        let key = create_test_key();

        // Agent_002 should not be able to create session for agent_001
        let result = store.get_or_create("agent_002", &key);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("access denied"));
        assert!(err.contains("agent_002"));
        assert!(err.contains("agent_001"));
    }

    #[test]
    fn test_get_does_not_verify_scope() {
        // Note: get() doesn't have agent_id parameter, so scope isn't checked
        // This is by design - get() is an internal method
        let store = create_test_store();
        let key = create_test_key();
        store.get_or_create("agent_001", &key).unwrap();

        // get() just returns the session if it exists
        let result = store.get(&key).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_scope_validation_agent_mismatch_on_patch() {
        let store = create_test_store();
        let key = create_test_key();
        store.get_or_create("agent_001", &key).unwrap();

        // Agent_002 should not be able to patch session for agent_001
        let patch = SessionPatch::with_status(SessionStatus::Idle);
        let result = store.patch("agent_002", &key, &patch);
        assert!(result.is_err());
    }

    #[test]
    fn test_scope_validation_agent_mismatch_on_delete() {
        let store = create_test_store();
        let key = create_test_key();
        store.get_or_create("agent_001", &key).unwrap();

        // Agent_002 should not be able to delete session for agent_001
        let result = store.delete("agent_002", &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_scope_validation_agent_mismatch_on_list() {
        let store = create_test_store();
        let key = create_test_key();
        store.get_or_create("agent_001", &key).unwrap();

        // Agent_002 should not see agent_001's sessions
        let sessions = store.list("agent_002", &SessionFilter::new()).unwrap();
        assert_eq!(sessions.len(), 0);
    }

    #[test]
    fn test_scope_validation_agent_mismatch_on_append_messages() {
        let store = create_test_store();
        let key = create_test_key();
        store.get_or_create("agent_001", &key).unwrap();

        let msg = StoredMessage::user("Hello");
        // Agent_002 should not be able to append messages
        let result = store.append_messages("agent_002", &key, &vec![msg]);
        assert!(result.is_err());
    }

    #[test]
    fn test_scope_validation_agent_mismatch_on_get_history() {
        let store = create_test_store();
        let key = create_test_key();
        store.get_or_create("agent_001", &key).unwrap();

        // Agent_002 should not be able to get history
        let result = store.get_history("agent_002", &key, &HistoryQuery::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_scope_validation_agent_mismatch_on_compact() {
        let store = create_test_store();
        let key = create_test_key();
        store.get_or_create("agent_001", &key).unwrap();

        // Agent_002 should not be able to compact session
        let result = store.compact("agent_002", &key, &CompactionStrategy::None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_agents_isolation() {
        let store = create_test_store();

        // Agent 1 creates sessions
        let key1 = SessionKey {
            agent_id: "agent_001".to_string(),
            channel: "discord".to_string(),
            account_id: "bot_123".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_111".to_string(),
        };
        let key2 = SessionKey {
            agent_id: "agent_002".to_string(),
            channel: "discord".to_string(),
            account_id: "bot_456".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_222".to_string(),
        };

        store.get_or_create("agent_001", &key1).unwrap();
        store.get_or_create("agent_002", &key2).unwrap();

        // Agent 1 should only see their own session
        let sessions1 = store.list("agent_001", &SessionFilter::new()).unwrap();
        assert_eq!(sessions1.len(), 1);
        assert_eq!(sessions1[0].key.agent_id, "agent_001");

        // Agent 2 should only see their own session
        let sessions2 = store.list("agent_002", &SessionFilter::new()).unwrap();
        assert_eq!(sessions2.len(), 1);
        assert_eq!(sessions2[0].key.agent_id, "agent_002");

        // Note: get() doesn't check scope (no agent_id parameter)
        // But operations with agent_id do check scope
        let result = store.patch(
            "agent_001",
            &key2,
            &SessionPatch::with_status(SessionStatus::Idle),
        );
        assert!(result.is_err()); // Agent 1 can't patch agent 2's session

        let result = store.patch(
            "agent_002",
            &key1,
            &SessionPatch::with_status(SessionStatus::Idle),
        );
        assert!(result.is_err()); // Agent 2 can't patch agent 1's session
    }

    #[test]
    fn test_scope_validation_same_agent_works() {
        let store = create_test_store();
        let key = create_test_key();

        // Same agent should be able to perform all operations
        let session = store.get_or_create("agent_001", &key).unwrap();
        assert_eq!(session.key.agent_id, "agent_001");

        let patch = SessionPatch::with_status(SessionStatus::Idle);
        let updated = store.patch("agent_001", &key, &patch).unwrap();
        assert_eq!(updated.status, SessionStatus::Idle);

        store.delete("agent_001", &key).unwrap();
    }

    #[test]
    fn test_scope_validation_is_strict() {
        // Scope validation is strict - agent_id must match exactly
        let store = create_test_store();
        let key = create_test_key();

        // Create with lowercase
        let session = store.get_or_create("agent_001", &key).unwrap();
        assert_eq!(session.key.agent_id, "agent_001");

        // Access with uppercase fails (strict comparison)
        let result = store.get_or_create("AGENT_001", &key);
        assert!(result.is_err());

        // Access with lowercase succeeds
        let session2 = store.get_or_create("agent_001", &key).unwrap();
        assert_eq!(session2.key, session.key);
    }

    #[test]
    fn test_list_filter_by_account_id() {
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
            account_id: "bot_456".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_222".to_string(),
        };

        store.get_or_create("agent_001", &key1).unwrap();
        store.get_or_create("agent_001", &key2).unwrap();

        let filter = SessionFilter {
            account_id: Some("bot_123".to_string()),
            ..Default::default()
        };
        let sessions = store.list("agent_001", &filter).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].key.account_id, "bot_123");
    }

    #[test]
    fn test_list_filter_by_peer_kind() {
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
            peer_kind: "group".to_string(),
            peer_id: "channel_222".to_string(),
        };

        store.get_or_create("agent_001", &key1).unwrap();
        store.get_or_create("agent_001", &key2).unwrap();

        let filter = SessionFilter {
            peer_kind: Some("dm".to_string()),
            ..Default::default()
        };
        let sessions = store.list("agent_001", &filter).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].key.peer_kind, "dm");
    }

    #[test]
    fn test_list_filter_by_peer_id() {
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

        store.get_or_create("agent_001", &key1).unwrap();
        store.get_or_create("agent_001", &key2).unwrap();

        let filter = SessionFilter {
            peer_id: Some("user_111".to_string()),
            ..Default::default()
        };
        let sessions = store.list("agent_001", &filter).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].key.peer_id, "user_111");
    }

    #[test]
    fn test_patch_multiple_fields() {
        let store = create_test_store();
        let key = create_test_key();
        store.get_or_create("agent_001", &key).unwrap();

        let patch = SessionPatch {
            status: Some(SessionStatus::Idle),
            message_count: Some(10),
            token_usage: Some(500),
            metadata: Some(serde_json::json!({"key": "value"})),
        };
        let updated = store.patch("agent_001", &key, &patch).unwrap();

        assert_eq!(updated.status, SessionStatus::Idle);
        assert_eq!(updated.message_count, 10);
        assert_eq!(updated.token_usage, 500);
        assert_eq!(
            updated.metadata.get("key"),
            Some(&serde_json::json!("value"))
        );
    }

    #[test]
    fn test_append_messages_multiple() {
        let store = create_test_store();
        let key = create_test_key();
        store.get_or_create("agent_001", &key).unwrap();

        let messages = vec![
            StoredMessage::user("Hello"),
            StoredMessage::assistant("Hi there!"),
            StoredMessage::user("How are you?"),
        ];

        store.append_messages("agent_001", &key, &messages).unwrap();

        let history = store
            .get_history("agent_001", &key, &HistoryQuery::default())
            .unwrap();
        assert_eq!(history.len(), 3);
    }

    #[test]
    fn test_get_history_limit_offset() {
        let store = create_test_store();
        let key = create_test_key();
        store.get_or_create("agent_001", &key).unwrap();

        let messages: Vec<StoredMessage> = (0..10)
            .map(|i| StoredMessage::user(format!("Message {}", i)))
            .collect();
        store.append_messages("agent_001", &key, &messages).unwrap();

        // Test limit
        let query = HistoryQuery {
            limit: Some(5),
            ..Default::default()
        };
        let history = store.get_history("agent_001", &key, &query).unwrap();
        assert_eq!(history.len(), 5);

        // Test offset
        let query = HistoryQuery {
            offset: Some(5),
            ..Default::default()
        };
        let history = store.get_history("agent_001", &key, &query).unwrap();
        assert_eq!(history.len(), 5);
        assert_eq!(
            history[0].content,
            serde_json::Value::String("Message 5".to_string())
        );

        // Test combined limit and offset
        let query = HistoryQuery {
            limit: Some(3),
            offset: Some(2),
            ..Default::default()
        };
        let history = store.get_history("agent_001", &key, &query).unwrap();
        assert_eq!(history.len(), 3);
        assert_eq!(
            history[0].content,
            serde_json::Value::String("Message 2".to_string())
        );
    }

    #[test]
    fn test_get_history_timestamp_filters() {
        let store = create_test_store();
        let key = create_test_key();
        store.get_or_create("agent_001", &key).unwrap();

        let now = Utc::now();
        let messages = vec![
            StoredMessage {
                content: serde_json::Value::String("Old".to_string()),
                created_at: now - Duration::hours(3),
                ..Default::default()
            },
            StoredMessage {
                content: serde_json::Value::String("Middle".to_string()),
                created_at: now - Duration::hours(1) - Duration::minutes(30),
                ..Default::default()
            },
            StoredMessage {
                content: serde_json::Value::String("New".to_string()),
                created_at: now,
                ..Default::default()
            },
        ];

        store.append_messages("agent_001", &key, &messages).unwrap();

        // Test before filter - messages older than 2 hours (should get "Old")
        let query = HistoryQuery {
            before: Some(now - Duration::hours(2)),
            ..Default::default()
        };
        let history = store.get_history("agent_001", &key, &query).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(
            history[0].content,
            serde_json::Value::String("Old".to_string())
        );

        // Test before filter - messages older than 1 hour (should get "Old" and "Middle")
        let query = HistoryQuery {
            before: Some(now - Duration::hours(1)),
            ..Default::default()
        };
        let history = store.get_history("agent_001", &key, &query).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(
            history[0].content,
            serde_json::Value::String("Old".to_string())
        );
        assert_eq!(
            history[1].content,
            serde_json::Value::String("Middle".to_string())
        );

        // Test after filter - messages newer than 2 hours ago (should get "Middle" and "New")
        let query = HistoryQuery {
            after: Some(now - Duration::hours(2)),
            ..Default::default()
        };
        let history = store.get_history("agent_001", &key, &query).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(
            history[0].content,
            serde_json::Value::String("Middle".to_string())
        );
        assert_eq!(
            history[1].content,
            serde_json::Value::String("New".to_string())
        );
    }

    #[test]
    fn test_session_status_transitions() {
        let store = create_test_store();
        let key = create_test_key();
        store.get_or_create("agent_001", &key).unwrap();

        let patch = SessionPatch::with_status(SessionStatus::Idle);
        let updated = store.patch("agent_001", &key, &patch).unwrap();
        assert_eq!(updated.status, SessionStatus::Idle);

        let patch = SessionPatch::with_status(SessionStatus::Compacted);
        let updated = store.patch("agent_001", &key, &patch).unwrap();
        assert_eq!(updated.status, SessionStatus::Compacted);

        let patch = SessionPatch::with_status(SessionStatus::Archived);
        let updated = store.patch("agent_001", &key, &patch).unwrap();
        assert_eq!(updated.status, SessionStatus::Archived);
    }

    #[test]
    fn test_patch_empty_metadata() {
        let store = create_test_store();
        let key = create_test_key();
        let session = store.get_or_create("agent_001", &key).unwrap();
        assert!(session.metadata.is_empty());

        let patch = SessionPatch {
            metadata: Some(serde_json::Value::Object(serde_json::Map::new())),
            ..Default::default()
        };
        let updated = store.patch("agent_001", &key, &patch).unwrap();
        assert!(updated.metadata.is_empty());
    }

    #[test]
    fn test_append_messages_empty_list() {
        let store = create_test_store();
        let key = create_test_key();
        store.get_or_create("agent_001", &key).unwrap();

        // Appending empty list should succeed
        store.append_messages("agent_001", &key, &vec![]).unwrap();

        let history = store
            .get_history("agent_001", &key, &HistoryQuery::default())
            .unwrap();
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_append_messages_metadata() {
        let store = create_test_store();
        let key = create_test_key();
        store.get_or_create("agent_001", &key).unwrap();

        let msg = StoredMessage::with_tool_calls(
            "assistant",
            "Response",
            serde_json::json!({"tool": "test", "args": {}}),
        );

        store
            .append_messages("agent_001", &key, &vec![msg])
            .unwrap();

        let history = store
            .get_history("agent_001", &key, &HistoryQuery::default())
            .unwrap();
        assert_eq!(history.len(), 1);
        assert!(history[0].tool_calls.is_some());
    }

    #[test]
    fn test_get_history_nonexistent_session() {
        let store = create_test_store();
        let key = create_test_key();

        let result = store.get_history("agent_001", &key, &HistoryQuery::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_nonexistent_session() {
        let store = create_test_store();
        let key = create_test_key();

        // Deleting nonexistent session should succeed (idempotent)
        store.delete("agent_001", &key).unwrap();
    }

    #[test]
    fn test_patch_nonexistent_session() {
        let store = create_test_store();
        let key = create_test_key();
        let patch = SessionPatch::with_status(SessionStatus::Idle);

        let result = store.patch("agent_001", &key, &patch);
        assert!(result.is_err());
    }

    #[test]
    fn test_scope_validation_error_message() {
        let store = create_test_store();
        let key = create_test_key();

        let result = store.get_or_create("wrong_agent", &key);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("access denied"));
        assert!(err.contains("wrong_agent"));
        assert!(err.contains("agent_001"));
    }

    #[test]
    fn test_message_count_after_append() {
        let store = create_test_store();
        let key = create_test_key();
        let session = store.get_or_create("agent_001", &key).unwrap();
        assert_eq!(session.message_count, 0);

        store
            .append_messages("agent_001", &key, &vec![StoredMessage::user("msg1")])
            .unwrap();
        let session = store.get_or_create("agent_001", &key).unwrap();
        assert_eq!(session.message_count, 1);

        store
            .append_messages("agent_001", &key, &vec![StoredMessage::user("msg2")])
            .unwrap();
        let session = store.get_or_create("agent_001", &key).unwrap();
        assert_eq!(session.message_count, 2);
    }

    #[test]
    fn test_multiple_sessions_same_agent() {
        let store = create_test_store();
        let agent_id = "agent_001";

        // Create multiple sessions for the same agent
        for i in 1..=5 {
            let key = SessionKey {
                agent_id: agent_id.to_string(),
                channel: "discord".to_string(),
                account_id: "bot_123".to_string(),
                peer_kind: "dm".to_string(),
                peer_id: format!("user_{}", i),
            };
            store.get_or_create(agent_id, &key).unwrap();
        }

        // List all sessions for agent
        let sessions = store.list(agent_id, &SessionFilter::new()).unwrap();
        assert_eq!(sessions.len(), 5);
    }

    #[test]
    fn test_session_unique_key_enforcement() {
        let store = create_test_store();
        let key = create_test_key();

        // First creation should succeed
        store.get_or_create("agent_001", &key).unwrap();

        // Second creation with same key should return same session
        let session1 = store.get_or_create("agent_001", &key).unwrap();
        let session2 = store.get_or_create("agent_001", &key).unwrap();

        // They should be the same session (same timestamps)
        assert_eq!(session1.key, session2.key);
        assert_eq!(session1.updated_at, session2.updated_at);
    }

    #[test]
    fn test_session_summary_conversion() {
        let key = create_test_key();
        let mut session = Session::new(key);
        session.message_count = 10;
        session.token_usage = 500;
        session.status = SessionStatus::Idle;

        let summary = SessionSummary::from(&session);
        assert_eq!(summary.message_count, 10);
        assert_eq!(summary.status, SessionStatus::Idle);
    }

    #[test]
    fn test_session_filter_multiple_criteria() {
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
            peer_kind: "group".to_string(),
            peer_id: "user_222".to_string(),
        };

        store.get_or_create("agent_001", &key1).unwrap();
        store.get_or_create("agent_001", &key2).unwrap();

        // Filter by both channel and peer_kind
        let filter = SessionFilter {
            channel: Some("discord".to_string()),
            peer_kind: Some("dm".to_string()),
            ..Default::default()
        };
        let sessions = store.list("agent_001", &filter).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].key.peer_kind, "dm");
    }

    #[test]
    fn test_append_messages_accumulates_count() {
        let store = create_test_store();
        let key = create_test_key();
        store.get_or_create("agent_001", &key).unwrap();

        // First append - message count accumulates
        store
            .append_messages("agent_001", &key, &vec![StoredMessage::user("msg1")])
            .unwrap();
        let session = store.get_or_create("agent_001", &key).unwrap();
        assert_eq!(session.message_count, 1);

        // Second append - message count accumulates again
        store
            .append_messages("agent_001", &key, &vec![StoredMessage::user("msg2")])
            .unwrap();
        let session = store.get_or_create("agent_001", &key).unwrap();
        assert_eq!(session.message_count, 2); // Counts accumulate
    }

    #[test]
    fn test_history_pagination_default_limit() {
        let store = create_test_store();
        let key = create_test_key();
        store.get_or_create("agent_001", &key).unwrap();

        let messages: Vec<StoredMessage> = (0..2000)
            .map(|i| StoredMessage::user(format!("Message {}", i)))
            .collect();
        store.append_messages("agent_001", &key, &messages).unwrap();

        // Without limit, default limit of 1000 is applied
        let query = HistoryQuery::default();
        let history = store.get_history("agent_001", &key, &query).unwrap();
        assert_eq!(history.len(), 1000); // Default limit
    }

    #[test]
    fn test_history_pagination_no_limit() {
        // Test with explicit high limit
        let store = create_test_store();
        let key = create_test_key();
        store.get_or_create("agent_001", &key).unwrap();

        let messages: Vec<StoredMessage> = (0..2000)
            .map(|i| StoredMessage::user(format!("Message {}", i)))
            .collect();
        store.append_messages("agent_001", &key, &messages).unwrap();

        // With high limit, should return all
        let query = HistoryQuery {
            limit: Some(5000),
            ..Default::default()
        };
        let history = store.get_history("agent_001", &key, &query).unwrap();
        assert_eq!(history.len(), 2000);
    }

    #[test]
    fn test_timestamp_filter_edge_cases() {
        let store = create_test_store();
        let key = create_test_key();
        store.get_or_create("agent_001", &key).unwrap();

        let now = Utc::now();
        let messages = vec![
            StoredMessage {
                content: serde_json::Value::String("Old".to_string()),
                created_at: now - Duration::hours(2),
                ..Default::default()
            },
            StoredMessage {
                content: serde_json::Value::String("New".to_string()),
                created_at: now,
                ..Default::default()
            },
        ];

        store.append_messages("agent_001", &key, &messages).unwrap();

        // Test before filter - should get messages older than 1 hour ago
        let query = HistoryQuery {
            before: Some(now - Duration::hours(1)),
            ..Default::default()
        };
        let history = store.get_history("agent_001", &key, &query).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(
            history[0].content,
            serde_json::Value::String("Old".to_string())
        );

        // Test after filter - should get messages newer than 1 hour ago
        let query = HistoryQuery {
            after: Some(now - Duration::hours(1)),
            ..Default::default()
        };
        let history = store.get_history("agent_001", &key, &query).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(
            history[0].content,
            serde_json::Value::String("New".to_string())
        );
    }

    #[test]
    fn test_metadata_replacement() {
        let store = create_test_store();
        let key = create_test_key();
        let session = store.get_or_create("agent_001", &key).unwrap();
        assert!(session.metadata.is_empty());

        // Set initial metadata
        let patch = SessionPatch {
            metadata: Some(serde_json::json!({"key1": "value1"})),
            ..Default::default()
        };
        let updated = store.patch("agent_001", &key, &patch).unwrap();
        assert_eq!(
            updated.metadata.get("key1"),
            Some(&serde_json::json!("value1"))
        );

        // Replace with new metadata
        let patch = SessionPatch {
            metadata: Some(serde_json::json!({"key2": "value2"})),
            ..Default::default()
        };
        let updated = store.patch("agent_001", &key, &patch).unwrap();
        assert_eq!(updated.metadata.get("key1"), None);
        assert_eq!(
            updated.metadata.get("key2"),
            Some(&serde_json::json!("value2"))
        );
    }

    #[test]
    fn test_compact_record_initial_values() {
        let store = create_test_store();
        let key = create_test_key();
        store.get_or_create("agent_001", &key).unwrap();

        let record = store
            .compact("agent_001", &key, &CompactionStrategy::None, None)
            .unwrap();
        assert_eq!(record.compaction_count, 0);
        assert!(record.last_compacted_at.is_none());
        assert!(record.summary.is_none());
    }

    #[test]
    fn test_append_messages_with_system_role() {
        let store = create_test_store();
        let key = create_test_key();
        store.get_or_create("agent_001", &key).unwrap();

        let msg = StoredMessage::system("System initialization complete");
        store
            .append_messages("agent_001", &key, &vec![msg])
            .unwrap();

        let history = store
            .get_history("agent_001", &key, &HistoryQuery::default())
            .unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].role, "system");
    }

    #[test]
    fn test_append_messages_with_tool_role() {
        let store = create_test_store();
        let key = create_test_key();
        store.get_or_create("agent_001", &key).unwrap();

        let msg = StoredMessage::tool("Tool result: success");
        store
            .append_messages("agent_001", &key, &vec![msg])
            .unwrap();

        let history = store
            .get_history("agent_001", &key, &HistoryQuery::default())
            .unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].role, "tool");
    }
}
