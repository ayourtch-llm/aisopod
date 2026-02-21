//! Core data types for session management.
//!
//! This module defines the fundamental types used throughout the session management system,
//! including session keys, session state, metadata, filters, and stored messages.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A unique identifier for a conversation session.
///
/// The `SessionKey` uniquely identifies a session by combining:
/// - The agent that owns the session
/// - The channel type and account
/// - The peer (user or group) being conversed with
///
/// # Example
///
/// ```ignore
/// use aisopod_session::SessionKey;
///
/// let key = SessionKey {
///     agent_id: "agent_001".to_string(),
///     channel: "discord".to_string(),
///     account_id: "bot_123".to_string(),
///     peer_kind: "dm".to_string(),
///     peer_id: "user_456".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionKey {
    /// The agent that owns this session.
    pub agent_id: String,
    /// The channel type (e.g., "discord", "slack", "telegram").
    pub channel: String,
    /// The bot/account identifier on the channel.
    pub account_id: String,
    /// The peer kind: "dm" for direct message or "group" for group conversations.
    pub peer_kind: String,
    /// The remote user or group identifier.
    pub peer_id: String,
}

/// The current state of a session in its lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    /// The session is active and accepting messages.
    Active,
    /// The session has been idle for some time.
    Idle,
    /// The session has been compacted (old messages removed or summarized).
    Compacted,
    /// The session has been archived and is no longer active.
    Archived,
}

/// Flexible metadata storage for session properties.
///
/// Uses a HashMap to store arbitrary key-value pairs as JSON values,
/// allowing for dynamic metadata without strict schema requirements.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Inner map of metadata key-value pairs.
    #[serde(flatten)]
    pub inner: HashMap<String, serde_json::Value>,
}

impl SessionMetadata {
    /// Creates a new empty metadata map.
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Gets a value from the metadata by key.
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.inner.get(key)
    }

    /// Sets a value in the metadata.
    pub fn set(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.inner.insert(key.into(), value);
    }

    /// Checks if the metadata contains a specific key.
    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    /// Removes a key from the metadata.
    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        self.inner.remove(key)
    }

    /// Returns true if the metadata contains no entries.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the number of entries in the metadata.
    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

/// A conversation session representing an ongoing or completed conversation.
///
/// The `Session` struct holds all information about a single conversation,
/// including its identity, lifecycle state, statistics, and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// The unique identifier for this session in the database.
    pub id: i64,
    /// The unique key identifying this session.
    pub key: SessionKey,
    /// When this session was created.
    pub created_at: DateTime<Utc>,
    /// When this session was last updated.
    pub updated_at: DateTime<Utc>,
    /// The number of messages in this session.
    pub message_count: u64,
    /// The total token usage for this session.
    pub token_usage: u64,
    /// Arbitrary metadata associated with this session.
    pub metadata: SessionMetadata,
    /// The current lifecycle status of this session.
    pub status: SessionStatus,
    /// The number of times this session has been compacted.
    pub compaction_count: u64,
    /// When this session was last compacted.
    pub last_compacted_at: Option<DateTime<Utc>>,
    /// Optional summary from the most recent compaction.
    pub last_compaction_summary: Option<String>,
}

impl Session {
    /// Creates a new session with the given key.
    ///
    /// The session is initialized with:
    /// - `id` set to 0 (database-generated ID)
    /// - `created_at` and `updated_at` set to the current time
    /// - `message_count` and `token_usage` set to 0
    /// - `metadata` as an empty map
    /// - `status` set to `SessionStatus::Active`
    /// - `compaction_count` set to 0
    /// - `last_compacted_at` set to None
    /// - `last_compaction_summary` set to None
    ///
    /// # Arguments
    ///
    /// * `key` - The unique identifier for this session.
    pub fn new(key: SessionKey) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            key,
            created_at: now,
            updated_at: now,
            message_count: 0,
            token_usage: 0,
            metadata: SessionMetadata::new(),
            status: SessionStatus::Active,
            compaction_count: 0,
            last_compacted_at: None,
            last_compaction_summary: None,
        }
    }

    /// Updates the session's timestamp to the current time.
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Updates the session's status.
    pub fn set_status(&mut self, status: SessionStatus) {
        self.status = status;
        self.touch();
    }

    /// Increments the message count.
    pub fn increment_message_count(&mut self, count: u64) {
        self.message_count += count;
        self.touch();
    }

    /// Adds to the token usage.
    pub fn add_token_usage(&mut self, tokens: u64) {
        self.token_usage += tokens;
        self.touch();
    }

    /// Updates the session's compaction metadata.
    pub fn update_compaction(&mut self, compaction_count: u64, last_compacted_at: Option<DateTime<Utc>>, last_compaction_summary: Option<String>) {
        self.compaction_count = compaction_count;
        self.last_compacted_at = last_compacted_at;
        self.last_compaction_summary = last_compaction_summary;
        self.touch();
    }
}

/// A lightweight view of a session for list endpoints and quick overviews.
///
/// `SessionSummary` provides the most essential information about a session
/// without including the full metadata or message history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    /// The session's unique key.
    pub key: SessionKey,
    /// The current status of the session.
    pub status: SessionStatus,
    /// The number of messages in the session.
    pub message_count: u64,
    /// When the session was last updated.
    pub updated_at: DateTime<Utc>,
}

impl From<&Session> for SessionSummary {
    fn from(session: &Session) -> Self {
        Self {
            key: session.key.clone(),
            status: session.status.clone(),
            message_count: session.message_count,
            updated_at: session.updated_at,
        }
    }
}

/// A filter for querying sessions.
///
/// All fields are optional. A filter matches a session if all provided
/// fields match the corresponding session fields. Unset fields are ignored.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionFilter {
    /// Filter by agent ID (exact match).
    pub agent_id: Option<String>,
    /// Filter by channel type (exact match).
    pub channel: Option<String>,
    /// Filter by account ID (exact match).
    pub account_id: Option<String>,
    /// Filter by peer kind (exact match).
    pub peer_kind: Option<String>,
    /// Filter by peer ID (exact match).
    pub peer_id: Option<String>,
    /// Filter by status.
    pub status: Option<SessionStatus>,
    /// Filter by sessions created after this time.
    pub created_after: Option<DateTime<Utc>>,
    /// Filter by sessions created before this time.
    pub created_before: Option<DateTime<Utc>>,
    /// Filter by sessions updated after this time.
    pub updated_after: Option<DateTime<Utc>>,
    /// Filter by sessions updated before this time.
    pub updated_before: Option<DateTime<Utc>>,
}

impl SessionFilter {
    /// Creates a new empty filter that matches all sessions.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a filter that matches sessions for a specific agent.
    pub fn for_agent(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: Some(agent_id.into()),
            ..Default::default()
        }
    }

    /// Creates a filter that matches sessions on a specific channel.
    pub fn for_channel(channel: impl Into<String>) -> Self {
        Self {
            channel: Some(channel.into()),
            ..Default::default()
        }
    }

    /// Checks if a session matches this filter.
    pub fn matches(&self, session: &Session) -> bool {
        if let Some(ref agent_id) = self.agent_id {
            if session.key.agent_id != *agent_id {
                return false;
            }
        }
        if let Some(ref channel) = self.channel {
            if session.key.channel != *channel {
                return false;
            }
        }
        if let Some(ref account_id) = self.account_id {
            if session.key.account_id != *account_id {
                return false;
            }
        }
        if let Some(ref peer_kind) = self.peer_kind {
            if session.key.peer_kind != *peer_kind {
                return false;
            }
        }
        if let Some(ref peer_id) = self.peer_id {
            if session.key.peer_id != *peer_id {
                return false;
            }
        }
        if let Some(ref status) = self.status {
            if session.status != *status {
                return false;
            }
        }
        if let Some(ref created_after) = self.created_after {
            if session.created_at <= *created_after {
                return false;
            }
        }
        if let Some(ref created_before) = self.created_before {
            if session.created_at >= *created_before {
                return false;
            }
        }
        if let Some(ref updated_after) = self.updated_after {
            if session.updated_at <= *updated_after {
                return false;
            }
        }
        if let Some(ref updated_before) = self.updated_before {
            if session.updated_at >= *updated_before {
                return false;
            }
        }
        true
    }
}

/// A query for retrieving message history with pagination and filtering.
///
/// This struct allows filtering messages by timestamp and paginating results.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistoryQuery {
    /// Maximum number of messages to return.
    pub limit: Option<u32>,
    /// Number of messages to skip (for offset-based pagination).
    pub offset: Option<u32>,
    /// Only return messages created before this timestamp.
    pub before: Option<DateTime<Utc>>,
    /// Only return messages created after this timestamp.
    pub after: Option<DateTime<Utc>>,
}

/// A patch for updating session fields.
///
/// All fields are optional. Only non-None fields are applied when updating
/// a session. This is useful for partial updates without overwriting existing data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionPatch {
    /// The new status for the session.
    pub status: Option<SessionStatus>,
    /// The new message count for the session.
    pub message_count: Option<u64>,
    /// The new token usage for the session.
    pub token_usage: Option<u64>,
    /// Metadata to add or update.
    pub metadata: Option<serde_json::Value>,
}

impl SessionPatch {
    /// Creates a new empty patch that makes no changes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a patch that updates only the status.
    pub fn with_status(status: SessionStatus) -> Self {
        Self {
            status: Some(status),
            ..Default::default()
        }
    }

    /// Creates a patch that updates only the message count.
    pub fn with_message_count(count: u64) -> Self {
        Self {
            message_count: Some(count),
            ..Default::default()
        }
    }

    /// Applies this patch to the given session.
    ///
    /// Only fields that are `Some` are applied to the session.
    pub fn apply(&self, session: &mut Session) {
        if let Some(ref status) = self.status {
            session.status = status.clone();
        }
        if let Some(message_count) = self.message_count {
            session.message_count = message_count;
        }
        if let Some(token_usage) = self.token_usage {
            session.token_usage = token_usage;
        }
        if let Some(ref metadata) = self.metadata {
            // Merge the metadata JSON object into the session's metadata
            if let Some(obj) = metadata.as_object() {
                for (key, value) in obj {
                    session.metadata.set(key.clone(), value.clone());
                }
            }
        }
        session.touch();
    }
}

/// A stored message in a session's history.
///
/// Messages can be from different roles (user, assistant, system, tool)
/// and contain structured content with optional tool call data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StoredMessage {
    /// Auto-incrementing row ID in storage.
    pub id: i64,
    /// The session ID this message belongs to.
    pub session_id: i64,
    /// The role of the message sender: "user", "assistant", "system", or "tool".
    pub role: String,
    /// The message content as a JSON value.
    pub content: serde_json::Value,
    /// Optional tool calls associated with this message.
    pub tool_calls: Option<serde_json::Value>,
    /// When this message was created.
    pub created_at: DateTime<Utc>,
}

impl StoredMessage {
    /// Creates a new user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self::new("user", content)
    }

    /// Creates a new assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new("assistant", content)
    }

    /// Creates a new system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self::new("system", content)
    }

    /// Creates a new tool message.
    pub fn tool(content: impl Into<String>) -> Self {
        Self::new("tool", content)
    }

    /// Creates a new message with the specified role.
    ///
    /// # Arguments
    ///
    /// * `role` - The role of the message sender.
    /// * `content` - The message content.
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: 0,
            session_id: 0,
            role: role.into(),
            content: serde_json::Value::String(content.into()),
            tool_calls: None,
            created_at: Utc::now(),
        }
    }

    /// Creates a new message with the specified role and content.
    ///
    /// # Arguments
    ///
    /// * `role` - The role of the message sender.
    /// * `content` - The message content as a JSON value.
    pub fn with_json_content(role: impl Into<String>, content: serde_json::Value) -> Self {
        Self {
            id: 0,
            session_id: 0,
            role: role.into(),
            content,
            tool_calls: None,
            created_at: Utc::now(),
        }
    }

    /// Creates a new message with tool calls.
    ///
    /// # Arguments
    ///
    /// * `role` - The role of the message sender (typically "assistant" for tool invocation).
    /// * `content` - The message content.
    /// * `tool_calls` - The tool call data as a JSON value.
    pub fn with_tool_calls(
        role: impl Into<String>,
        content: impl Into<String>,
        tool_calls: serde_json::Value,
    ) -> Self {
        Self {
            id: 0,
            session_id: 0,
            role: role.into(),
            content: serde_json::Value::String(content.into()),
            tool_calls: Some(tool_calls),
            created_at: Utc::now(),
        }
    }

    /// Creates a new message with a specific timestamp.
    ///
    /// # Arguments
    ///
    /// * `role` - The role of the message sender.
    /// * `content` - The message content.
    /// * `timestamp` - The timestamp for this message.
    pub fn with_timestamp(role: impl Into<String>, content: impl Into<String>, timestamp: DateTime<Utc>) -> Self {
        Self {
            id: 0,
            session_id: 0,
            role: role.into(),
            content: serde_json::Value::String(content.into()),
            tool_calls: None,
            created_at: timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_key_equality() {
        let key1 = SessionKey {
            agent_id: "agent_1".to_string(),
            channel: "discord".to_string(),
            account_id: "bot_1".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_1".to_string(),
        };
        let key2 = SessionKey {
            agent_id: "agent_1".to_string(),
            channel: "discord".to_string(),
            account_id: "bot_1".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_1".to_string(),
        };
        let key3 = SessionKey {
            agent_id: "agent_2".to_string(),
            ..key1.clone()
        };

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
        assert_ne!(key2, key3);
    }

    #[test]
    fn test_session_key_hash() {
        let key = SessionKey {
            agent_id: "agent_1".to_string(),
            channel: "discord".to_string(),
            account_id: "bot_1".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_1".to_string(),
        };

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash1 = hasher.finish();

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash2 = hasher.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_session_metadata() {
        let mut metadata = SessionMetadata::new();
        assert!(metadata.is_empty());

        metadata.set("key1", serde_json::Value::String("value1".to_string()));
        assert!(metadata.contains_key("key1"));
        assert_eq!(metadata.get("key1"), Some(&serde_json::Value::String("value1".to_string())));

        metadata.remove("key1");
        assert!(!metadata.contains_key("key1"));
    }

    #[test]
    fn test_new_session() {
        let key = SessionKey {
            agent_id: "agent_1".to_string(),
            channel: "discord".to_string(),
            account_id: "bot_1".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_1".to_string(),
        };

        let session = Session::new(key);

        assert_eq!(session.key.agent_id, "agent_1");
        assert_eq!(session.message_count, 0);
        assert_eq!(session.token_usage, 0);
        assert_eq!(session.status, SessionStatus::Active);
    }

    #[test]
    fn test_session_updates() {
        let key = SessionKey {
            agent_id: "agent_1".to_string(),
            channel: "discord".to_string(),
            account_id: "bot_1".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_1".to_string(),
        };

        let mut session = Session::new(key);

        let before = session.updated_at;
        session.touch();
        assert!(session.updated_at > before);

        session.increment_message_count(5);
        assert_eq!(session.message_count, 5);

        session.add_token_usage(100);
        assert_eq!(session.token_usage, 100);

        session.set_status(SessionStatus::Idle);
        assert_eq!(session.status, SessionStatus::Idle);
    }

    #[test]
    fn test_session_filter_matching() {
        let key = SessionKey {
            agent_id: "agent_1".to_string(),
            channel: "discord".to_string(),
            account_id: "bot_1".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_1".to_string(),
        };

        let session = Session::new(key);

        let filter = SessionFilter::for_agent("agent_1");
        assert!(filter.matches(&session));

        let filter = SessionFilter::for_agent("agent_2");
        assert!(!filter.matches(&session));

        let filter = SessionFilter::for_channel("discord");
        assert!(filter.matches(&session));

        let filter = SessionFilter::for_channel("slack");
        assert!(!filter.matches(&session));
    }

    #[test]
    fn test_session_patch() {
        let key = SessionKey {
            agent_id: "agent_1".to_string(),
            channel: "discord".to_string(),
            account_id: "bot_1".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_1".to_string(),
        };

        let mut session = Session::new(key);

        let patch = SessionPatch::with_status(SessionStatus::Compacted);
        patch.apply(&mut session);

        assert_eq!(session.status, SessionStatus::Compacted);

        let metadata = serde_json::json!({"key": "value"});
        let patch = SessionPatch {
            metadata: Some(metadata),
            ..Default::default()
        };
        patch.apply(&mut session);

        assert_eq!(session.metadata.get("key"), Some(&serde_json::json!("value")));
    }

    #[test]
    fn test_stored_message() {
        let msg = StoredMessage::user("Hello!");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, serde_json::Value::String("Hello!".to_string()));

        let msg = StoredMessage::assistant("Hi there!");
        assert_eq!(msg.role, "assistant");

        let msg = StoredMessage::tool("Tool result");
        assert_eq!(msg.role, "tool");

        let msg = StoredMessage::with_json_content(
            "assistant",
            serde_json::json!({"type": "text", "text": "Response"}),
        );
        assert_eq!(msg.role, "assistant");
        assert_eq!(msg.content, serde_json::json!({"type": "text", "text": "Response"}));
    }

    #[test]
    fn test_session_summary_from_session() {
        let key = SessionKey {
            agent_id: "agent_1".to_string(),
            channel: "discord".to_string(),
            account_id: "bot_1".to_string(),
            peer_kind: "dm".to_string(),
            peer_id: "user_1".to_string(),
        };

        let mut session = Session::new(key);
        session.message_count = 10;
        session.token_usage = 500;
        session.status = SessionStatus::Idle;

        let summary = SessionSummary::from(&session);

        assert_eq!(summary.message_count, 10);
        assert_eq!(summary.status, SessionStatus::Idle);
        assert_eq!(summary.key.agent_id, "agent_1");
    }
}
