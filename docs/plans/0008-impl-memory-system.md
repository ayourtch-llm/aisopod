# 0008 — Memory System

**Master Plan Reference:** Section 3.13 — Memory System  
**Phase:** 3 (Agent Engine)  
**Dependencies:** 0001 (Project Structure), 0002 (Configuration), 0007 (Session Management)

---

## Objective

Implement the memory system (QMD — query-memory database) providing persistent
context retention for AI agents via vector-based semantic search over SQLite.

---

## Deliverables

### 1. Memory Store (`aisopod-memory`)

Core memory abstraction:

```rust
pub struct MemoryStore {
    db: Arc<Mutex<rusqlite::Connection>>,  // rusqlite with sqlite-vec extension loaded
}

impl MemoryStore {
    /// Store a memory entry with embedding
    pub async fn store(&self, entry: MemoryEntry) -> Result<String>;

    /// Query memories by semantic similarity
    pub async fn query(
        &self,
        query: &str,
        opts: MemoryQueryOptions,
    ) -> Result<Vec<MemoryMatch>>;

    /// Delete a memory entry
    pub async fn delete(&self, id: &str) -> Result<()>;

    /// List memories with filters
    pub async fn list(&self, filter: MemoryFilter) -> Result<Vec<MemoryEntry>>;
}
```

### 2. Memory Entry Types

```rust
pub struct MemoryEntry {
    pub id: String,
    pub agent_id: String,
    pub content: String,
    pub embedding: Vec<f32>,
    pub metadata: MemoryMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct MemoryMetadata {
    pub source: MemorySource,       // "agent", "user", "system"
    pub session_key: Option<String>,
    pub tags: Vec<String>,
    pub importance: f32,            // 0.0 - 1.0
    pub custom: serde_json::Value,
}
```

### 3. Vector Storage with SQLite-Vec

- Use `rusqlite` with `sqlite-vec` extension for vector operations
- Store embeddings as float arrays
- Cosine similarity search
- Configurable embedding dimensions

**Database schema:**
```sql
CREATE TABLE memories (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    content TEXT NOT NULL,
    source TEXT NOT NULL,
    session_key TEXT,
    tags TEXT DEFAULT '[]',
    importance REAL DEFAULT 0.5,
    metadata TEXT DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- sqlite-vec virtual table for embeddings
CREATE VIRTUAL TABLE memory_embeddings USING vec0(
    id TEXT PRIMARY KEY,
    embedding float[1536]
);
```

### 4. Embedding Generation

- Abstract embedding provider trait
- OpenAI embeddings API integration (`text-embedding-3-small`)
- Local embedding support (via Ollama or similar)
- Cached embeddings to avoid redundant API calls
- Configurable embedding model and dimensions

### 5. Memory Query Pipeline

```
User/agent query
  → Generate query embedding
  → Vector similarity search (top-K)
  → Filter by agent_id, tags, importance threshold
  → Re-rank results
  → Format for context injection
  → Return matched memories
```

### 6. Automatic Memory Management

- Extract key facts from conversations for storage
- Importance scoring (frequency, recency, explicit marking)
- Memory consolidation (merge similar entries)
- Memory expiration (time-based, importance-based)
- Storage quota management per agent

### 7. Memory Integration with Agent

- Pre-query memories before agent run using conversation context
- Inject relevant memories into system prompt
- Agent tool for explicit memory operations (store, query, delete)
- Post-run memory extraction from conversation

### 8. LanceDB Alternative

- Optional LanceDB backend for more advanced vector operations
- Feature-gated compilation (`--features lancedb`)
- Same trait interface, different storage backend

---

## Acceptance Criteria

- [ ] Memory entries store with embeddings in SQLite-Vec
- [ ] Semantic similarity search returns relevant results
- [ ] Agent-scoped memory isolation works correctly
- [ ] Embedding generation works via OpenAI API
- [ ] Memory query pipeline integrates with agent execution
- [ ] Automatic memory extraction works from conversations
- [ ] Memory expiration and quota management functions
- [ ] LanceDB backend works as an alternative (if implemented)
- [ ] Unit tests cover CRUD and similarity search
- [ ] Integration tests verify end-to-end memory flow
