# 0007 — Session Management

**Master Plan Reference:** Section 3.14 — Session Management  
**Phase:** 3 (Agent Engine)  
**Dependencies:** 0001 (Project Structure), 0002 (Configuration System)

---

## Objective

Implement persistent session management for AI agent conversations, including
session creation, routing, history storage, compaction, and multi-agent isolation.

---

## Deliverables

### 1. Session Store (`aisopod-session`)

Core session storage abstraction:

```rust
pub struct SessionStore {
    db: Arc<Mutex<rusqlite::Connection>>,  // or r2d2 pool for multi-threaded access
}

impl SessionStore {
    /// Create or retrieve a session
    pub async fn get_or_create(&self, key: &SessionKey) -> Result<Session>;

    /// List sessions with optional filters
    pub async fn list(&self, filter: SessionFilter) -> Result<Vec<SessionSummary>>;

    /// Get session history (messages)
    pub async fn get_history(
        &self,
        key: &SessionKey,
        opts: HistoryOptions,
    ) -> Result<Vec<StoredMessage>>;

    /// Append messages to session history
    pub async fn append_messages(
        &self,
        key: &SessionKey,
        messages: Vec<StoredMessage>,
    ) -> Result<()>;

    /// Update session metadata
    pub async fn patch(&self, key: &SessionKey, patch: SessionPatch) -> Result<()>;

    /// Delete a session
    pub async fn delete(&self, key: &SessionKey) -> Result<()>;
}
```

### 2. Session Key Generation

Port the session key system:

```rust
pub struct SessionKey {
    pub agent_id: String,
    pub channel: String,
    pub account_id: String,
    pub peer_kind: PeerKind,     // "dm", "group", "channel"
    pub peer_id: String,
}
```

**Routing rules:**
- DMs collapse to agent `main` session
- Groups are isolated by peer ID (guild/group/channel)
- Agent binding determines which agent handles which channel/peer

### 3. Session State

```rust
pub struct Session {
    pub key: SessionKey,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: u64,
    pub token_usage: TokenUsage,
    pub metadata: SessionMetadata,
    pub status: SessionStatus,
}

pub struct SessionMetadata {
    pub title: Option<String>,
    pub last_model: Option<String>,
    pub compaction_count: u32,
    pub custom: serde_json::Value,
}

pub enum SessionStatus {
    Active,
    Idle,
    Compacted,
    Archived,
}
```

### 4. Message Storage

```rust
pub struct StoredMessage {
    pub id: String,
    pub role: Role,
    pub content: MessageContent,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub token_count: Option<u32>,
    pub metadata: serde_json::Value,
}
```

**Storage backend:**
- SQLite database (via `rusqlite`)
- Messages stored as JSON blobs for flexibility
- Indexed by session key and timestamp
- Efficient pagination for history retrieval

### 5. Database Schema

```sql
CREATE TABLE sessions (
    key TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    channel TEXT NOT NULL,
    account_id TEXT NOT NULL,
    peer_kind TEXT NOT NULL,
    peer_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    message_count INTEGER DEFAULT 0,
    total_input_tokens INTEGER DEFAULT 0,
    total_output_tokens INTEGER DEFAULT 0,
    metadata TEXT DEFAULT '{}',
    status TEXT DEFAULT 'active'
);

CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    session_key TEXT NOT NULL REFERENCES sessions(key),
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    tool_calls TEXT,
    tool_call_id TEXT,
    timestamp TEXT NOT NULL,
    token_count INTEGER,
    metadata TEXT DEFAULT '{}',
    FOREIGN KEY (session_key) REFERENCES sessions(key)
);

CREATE INDEX idx_messages_session ON messages(session_key, timestamp);
```

### 6. Session Compaction Integration

- Hook into agent execution for post-run compaction
- Support compaction strategies from agent config
- Track compaction history (count, last compaction time)
- Preserve compaction summaries in session

### 7. Session Lifecycle

```
Incoming message
  → Generate session key (from channel, peer, agent binding)
  → Get or create session
  → Load history (with pagination/limits)
  → Pass to agent runner
  → Agent streams response
  → Append new messages to history
  → Update session metadata (token usage, timestamps)
  → Compact if needed
```

### 8. Multi-Agent Session Isolation

- Sessions are scoped to agent ID
- Agent binding changes don't affect existing sessions
- Session migration tools for when bindings change
- Cross-agent session listing for admin views

---

## Acceptance Criteria

- [ ] Sessions are created and retrieved by composite key
- [ ] Message history stores and retrieves correctly
- [ ] Pagination works for large histories
- [ ] Session metadata updates correctly
- [ ] Session listing with filters works
- [ ] Session deletion cascades to messages
- [ ] Compaction integrates with session storage
- [ ] Multi-agent isolation prevents cross-agent access
- [ ] Database migrations run on startup
- [ ] Unit tests cover all CRUD operations
- [ ] Performance tests verify history retrieval at scale
