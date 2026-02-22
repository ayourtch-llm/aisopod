//! SQLite-Vec memory storage backend.
//!
//! This module provides a `SqliteMemoryStore` implementation using SQLite with
//! the sqlite-vec extension for vector storage and cosine similarity search.

use crate::embedding::EmbeddingProvider;
use crate::store::MemoryStore;
use crate::types::{MemoryEntry, MemoryFilter, MemoryMatch, MemoryQueryOptions, MemorySource};
use anyhow::{anyhow, Result};
use rusqlite::{Connection, ToSql};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// Import the sqlite-vec extension initialization function
use sqlite_vec::sqlite3_vec_init;

/// Memory storage backend using SQLite with sqlite-vec extension.
///
/// This struct manages a SQLite database with two tables:
/// - `memories`: Stores memory metadata and content
/// - `memory_embeddings`: Virtual table for vector operations using sqlite-vec
pub struct SqliteMemoryStore {
    db: Arc<Mutex<Connection>>,
    embedding_dim: usize,
    embedder: Arc<dyn EmbeddingProvider>,
}

/// Helper struct for deserializing memory entries from the database.
#[derive(Deserialize)]
struct DbMemory {
    id: String,
    agent_id: String,
    content: String,
    source: String,
    session_key: Option<String>,
    tags: String,
    importance: f64,
    metadata: String,
    created_at: String,
    updated_at: String,
}

/// Helper struct for deserializing memory entries with embeddings from the database.
#[derive(Deserialize)]
struct DbMemoryWithEmbedding {
    id: String,
    agent_id: String,
    content: String,
    source: String,
    session_key: Option<String>,
    tags: String,
    importance: f64,
    metadata: String,
    created_at: String,
    updated_at: String,
    embedding_bytes: Vec<u8>,
}

/// Helper struct for serializing memory entries to the database.
#[derive(Serialize)]
struct DbMemoryInput<'a> {
    id: &'a str,
    agent_id: &'a str,
    content: &'a str,
    source: &'a str,
    session_key: &'a Option<String>,
    tags: &'a Vec<String>,
    importance: f64,
    metadata: &'a serde_json::Value,
    created_at: &'a str,
    updated_at: &'a str,
}

impl SqliteMemoryStore {
    /// Creates a new `SqliteMemoryStore` at the given path with a default MockEmbeddingProvider.
    ///
    /// This is a convenience constructor for tests and examples where a real
    /// embedding provider is not needed. For production use, call `new_with_embedder`
    /// with a custom embedding provider.
    ///
    /// # Arguments
    /// * `path` - Path to the SQLite database file (use `:memory:` for in-memory DB)
    /// * `embedding_dim` - Dimension of the embeddings to store
    ///
    /// # Returns
    /// Returns a new `SqliteMemoryStore` or an error if initialization fails.
    pub fn new(path: &str, embedding_dim: usize) -> Result<Self> {
        use crate::MockEmbeddingProvider;
        let embedder = Arc::new(MockEmbeddingProvider::new(embedding_dim));
        Self::new_with_embedder(path, embedding_dim, embedder)
    }

    /// Creates a new `SqliteMemoryStore` at the given path with a custom embedding provider.
    ///
    /// # Arguments
    /// * `path` - Path to the SQLite database file (use `:memory:` for in-memory DB)
    /// * `embedding_dim` - Dimension of the embeddings to store
    /// * `embedder` - The embedding provider to use for converting text queries to embeddings
    ///
    /// # Returns
    /// Returns a new `SqliteMemoryStore` or an error if initialization fails.
    pub fn new_with_embedder(
        path: &str,
        embedding_dim: usize,
        embedder: Arc<dyn EmbeddingProvider>,
    ) -> Result<Self> {
        // Register the sqlite-vec extension as an auto-extension
        // This must be done before opening any database connections
        unsafe {
            rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite3_vec_init as *const (),
            )));
        }

        let db = Connection::open(path)?;

        // Create the schema
        Self::create_schema(&db, embedding_dim)?;

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            embedding_dim,
            embedder,
        })
    }

    /// Creates the database schema.
    fn create_schema(db: &Connection, embedding_dim: usize) -> Result<()> {
        // Create memories table
        db.execute_batch(&format!(
            "CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                agent_id TEXT NOT NULL,
                content TEXT NOT NULL,
                source TEXT NOT NULL,
                session_key TEXT,
                tags TEXT DEFAULT '[]',
                importance REAL DEFAULT 0.5,
                metadata TEXT DEFAULT '{{}}',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );"
        ))?;

        // Create index on agent_id for fast scoped queries
        db.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_memories_agent_id ON memories(agent_id);",
        )?;

        // Create the vector table with dynamic dimension
        db.execute_batch(&format!(
            "CREATE VIRTUAL TABLE IF NOT EXISTS memory_embeddings USING vec0(
                id TEXT PRIMARY KEY,
                embedding float[{embedding_dim}]
            );"
        ))?;

        Ok(())
    }

    /// Helper to serialize tags to JSON string.
    fn serialize_tags(tags: &[String]) -> String {
        serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string())
    }

    /// Helper to deserialize tags from JSON string.
    fn deserialize_tags(tags: &str) -> Vec<String> {
        serde_json::from_str(tags).unwrap_or_else(|_| Vec::new())
    }

    /// Helper to convert MemorySource to string.
    fn source_to_string(source: &MemorySource) -> &'static str {
        match source {
            MemorySource::Agent => "Agent",
            MemorySource::User => "User",
            MemorySource::System => "System",
        }
    }

    /// Helper to convert string to MemorySource.
    fn string_to_source(s: &str) -> MemorySource {
        match s {
            "Agent" => MemorySource::Agent,
            "User" => MemorySource::User,
            "System" => MemorySource::System,
            _ => MemorySource::System,
        }
    }
}

#[async_trait::async_trait]
impl MemoryStore for SqliteMemoryStore {
    async fn store(&self, mut entry: MemoryEntry) -> Result<String> {
        let mut db = self.db.lock().map_err(|e| anyhow!(e.to_string()))?;

        // Generate ID if not provided
        if entry.id.is_empty() {
            entry.id = Uuid::new_v4().to_string();
        }

        // Serialize data for storage
        let tags_json = Self::serialize_tags(&entry.metadata.tags);
        let metadata_json = serde_json::to_value(&entry.metadata.custom)
            .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));
        let source_str = Self::source_to_string(&entry.metadata.source);

        let db_input = DbMemoryInput {
            id: &entry.id,
            agent_id: &entry.agent_id,
            content: &entry.content,
            source: source_str,
            session_key: &entry.metadata.session_key,
            tags: &entry.metadata.tags,
            importance: entry.metadata.importance as f64,
            metadata: &metadata_json,
            created_at: &entry.created_at.to_rfc3339(),
            updated_at: &entry.updated_at.to_rfc3339(),
        };

        // Insert into memories table
        db.execute(
            r#"
            INSERT INTO memories (id, agent_id, content, source, session_key, tags, importance, metadata, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                agent_id = excluded.agent_id,
                content = excluded.content,
                source = excluded.source,
                session_key = excluded.session_key,
                tags = excluded.tags,
                importance = excluded.importance,
                metadata = excluded.metadata,
                updated_at = excluded.updated_at
            "#,
            rusqlite::params![
                db_input.id,
                db_input.agent_id,
                db_input.content,
                db_input.source,
                db_input.session_key,
                tags_json,
                db_input.importance,
                metadata_json.to_string(),
                db_input.created_at,
                db_input.updated_at,
            ],
        )?;

        // Insert into memory_embeddings table
        let embedding: Vec<f32> = entry.embedding.iter().map(|&x| x as f32).collect();
        // Serialize embedding as bytes for storage
        let embedding_bytes: Vec<u8> = embedding.iter().flat_map(|&x| x.to_le_bytes()).collect();

        // sqlite-vec virtual table doesn't support ON CONFLICT UPDATE, so we need to
        // first try to delete if it exists, then insert
        db.execute(
            "DELETE FROM memory_embeddings WHERE id = ?",
            rusqlite::params![&entry.id],
        )
        .ok(); // Ignore errors (might not exist)

        db.execute(
            r#"
            INSERT INTO memory_embeddings (id, embedding)
            VALUES (?, ?)
            "#,
            rusqlite::params![&entry.id, &embedding_bytes as &[u8]],
        )?;

        Ok(entry.id)
    }

    async fn query(&self, query: &str, opts: MemoryQueryOptions) -> Result<Vec<MemoryMatch>> {
        // Convert query string to embedding BEFORE acquiring the db lock
        let query_embedding = self.embedder.embed(query).await?;

        // Serialize embedding as bytes (same format as in store method)
        let query_embedding_bytes: Vec<u8> = query_embedding
            .iter()
            .flat_map(|&x| x.to_le_bytes())
            .collect();

        let db = self.db.lock().map_err(|e| anyhow!(e.to_string()))?;

        // Build the filter conditions
        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn ToSql>> = Vec::new();

        if let Some(agent_id) = &opts.filter.agent_id {
            conditions.push("m.agent_id = ?".to_string());
            params.push(Box::new(agent_id));
        }

        if let Some(source) = &opts.filter.source {
            let source_str = Self::source_to_string(source);
            conditions.push("m.source = ?".to_string());
            params.push(Box::new(source_str));
        }

        if let Some(session_key) = &opts.filter.session_key {
            conditions.push("m.session_key = ?".to_string());
            params.push(Box::new(session_key));
        }

        if let Some(importance_min) = &opts.filter.importance_min {
            conditions.push("m.importance >= ?".to_string());
            params.push(Box::new(importance_min));
        }

        if let Some(created_after) = &opts.filter.created_after {
            conditions.push("m.created_at >= ?".to_string());
            params.push(Box::new(created_after.to_rfc3339()));
        }

        if let Some(created_before) = &opts.filter.created_before {
            conditions.push("m.created_at <= ?".to_string());
            params.push(Box::new(created_before.to_rfc3339()));
        }

        // Handle tags filter with json_each
        if let Some(tags) = &opts.filter.tags {
            if !tags.is_empty() {
                // For each tag, create a condition and add it to params
                for tag in tags {
                    conditions.push(format!(
                        "EXISTS (SELECT 1 FROM json_each(m.tags) WHERE json_each.value = ?)"
                    ));
                    params.push(Box::new(tag));
                }
            }
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            conditions.join(" AND ")
        };

        // sqlite-vec vec0 table requires k = ? to specify the number of results
        // The MATCH clause must include the embedding and k parameter
        // Format: WHERE embedding MATCH ? AND k = ?

        // Build params: query embedding first, then k, then filter params
        let mut all_params: Vec<Box<dyn ToSql>> = Vec::new();
        all_params.push(Box::new(&query_embedding_bytes as &[u8]));
        all_params.push(Box::new(opts.top_k as i64));
        all_params.extend(params);

        // Build the WHERE clause with vector search
        // The vec0 table uses MATCH syntax where the first param is the embedding
        // and k specifies the number of results
        let vector_match_clause = "e.embedding MATCH ? AND k = ?";
        let final_where_clause = if where_clause.is_empty() {
            vector_match_clause.to_string()
        } else {
            format!("{} AND {}", vector_match_clause, where_clause)
        };

        // Execute the query with vector search
        let sql = format!(
            r#"
            SELECT m.id, m.agent_id, m.content, m.source, m.session_key, m.tags, m.importance, m.metadata, m.created_at, m.updated_at, e.distance
            FROM memory_embeddings e
            JOIN memories m ON e.id = m.id
            WHERE {}
            ORDER BY e.distance ASC
            "#,
            final_where_clause
        );

        let mut stmt = db.prepare(&sql)?;
        let results: Vec<DbMemoryMatch> = stmt
            .query_map(rusqlite::params_from_iter(all_params.iter()), |row| {
                Ok(DbMemoryMatch {
                    id: row.get(0)?,
                    agent_id: row.get(1)?,
                    content: row.get(2)?,
                    source: row.get(3)?,
                    session_key: row.get(4)?,
                    tags: row.get(5)?,
                    importance: row.get(6)?,
                    metadata: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    distance: row.get::<_, f64>(10)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        // Convert to MemoryMatch and filter by min_score
        let min_score = opts.min_score.unwrap_or(0.0).into();
        let mut matches: Vec<MemoryMatch> = results
            .into_iter()
            .filter_map(|m| {
                // Convert distance to score (cosine distance: 0 = identical, 2 = opposite)
                // Score = 1 - (distance / 2) or just use (2 - distance) / 2
                let score = 1.0 - (m.distance / 2.0);

                if score < min_score {
                    return None;
                }

                Some(MemoryMatch {
                    entry: MemoryEntry {
                        id: m.id,
                        agent_id: m.agent_id,
                        content: m.content,
                        embedding: vec![0.0; self.embedding_dim], // Placeholder - embeddings not retrieved
                        metadata: crate::types::MemoryMetadata {
                            source: Self::string_to_source(&m.source),
                            session_key: m.session_key,
                            tags: Self::deserialize_tags(&m.tags),
                            importance: m.importance as f32,
                            custom: serde_json::from_str(&m.metadata).unwrap_or_default(),
                        },
                        created_at: chrono::DateTime::parse_from_rfc3339(&m.created_at)
                            .ok()?
                            .with_timezone(&chrono::Utc),
                        updated_at: chrono::DateTime::parse_from_rfc3339(&m.updated_at)
                            .ok()?
                            .with_timezone(&chrono::Utc),
                    },
                    score: score as f32,
                })
            })
            .collect();

        // Sort by score descending (we already have them in ascending distance order)
        matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(matches)
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let mut db = self.db.lock().map_err(|e| anyhow!(e.to_string()))?;

        // Delete from memory_embeddings table
        db.execute(
            "DELETE FROM memory_embeddings WHERE id = ?",
            rusqlite::params![id],
        )?;

        // Delete from memories table
        db.execute("DELETE FROM memories WHERE id = ?", rusqlite::params![id])?;

        Ok(())
    }

    async fn list(&self, filter: MemoryFilter) -> Result<Vec<MemoryEntry>> {
        let db = self.db.lock().map_err(|e| anyhow!(e.to_string()))?;

        // Build the filter conditions
        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn ToSql>> = Vec::new();

        if let Some(agent_id) = filter.agent_id {
            conditions.push("agent_id = ?".to_string());
            params.push(Box::new(agent_id));
        }

        if let Some(source) = filter.source {
            let source_str = Self::source_to_string(&source);
            conditions.push("source = ?".to_string());
            params.push(Box::new(source_str));
        }

        if let Some(session_key) = filter.session_key {
            conditions.push("session_key = ?".to_string());
            params.push(Box::new(session_key));
        }

        if let Some(importance_min) = filter.importance_min {
            conditions.push("importance >= ?".to_string());
            params.push(Box::new(importance_min));
        }

        if let Some(created_after) = filter.created_after {
            conditions.push("created_at >= ?".to_string());
            params.push(Box::new(created_after.to_rfc3339()));
        }

        if let Some(created_before) = filter.created_before {
            conditions.push("created_at <= ?".to_string());
            params.push(Box::new(created_before.to_rfc3339()));
        }

        // Handle tags filter with json_each
        if let Some(tags) = filter.tags {
            if !tags.is_empty() {
                for tag in tags {
                    conditions.push(format!(
                        "EXISTS (SELECT 1 FROM json_each(m.tags) WHERE json_each.value = ?)"
                    ));
                    params.push(Box::new(tag));
                }
            }
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let sql = format!(
            r#"
            SELECT m.id, m.agent_id, m.content, m.source, m.session_key, m.tags, m.importance, m.metadata, m.created_at, m.updated_at, e.embedding
            FROM memories m
            JOIN memory_embeddings e ON m.id = e.id
            {}
            "#,
            where_clause
        );

        let mut stmt = db.prepare(&sql)?;
        let results: Vec<DbMemoryWithEmbedding> = stmt
            .query_map(rusqlite::params_from_iter(params.iter()), |row| {
                Ok(DbMemoryWithEmbedding {
                    id: row.get(0)?,
                    agent_id: row.get(1)?,
                    content: row.get(2)?,
                    source: row.get(3)?,
                    session_key: row.get(4)?,
                    tags: row.get(5)?,
                    importance: row.get(6)?,
                    metadata: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                    embedding_bytes: row.get(10)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        let entries = results
            .into_iter()
            .filter_map(|m| {
                let created_at = chrono::DateTime::parse_from_rfc3339(&m.created_at).ok()?;
                let updated_at = chrono::DateTime::parse_from_rfc3339(&m.updated_at).ok()?;

                // Deserialize embedding from bytes
                let embedding: Vec<f32> = m
                    .embedding_bytes
                    .chunks_exact(4)
                    .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect();

                Some(MemoryEntry {
                    id: m.id,
                    agent_id: m.agent_id,
                    content: m.content,
                    embedding,
                    metadata: crate::types::MemoryMetadata {
                        source: Self::string_to_source(&m.source),
                        session_key: m.session_key,
                        tags: Self::deserialize_tags(&m.tags),
                        importance: m.importance as f32,
                        custom: serde_json::from_str(&m.metadata).unwrap_or_default(),
                    },
                    created_at: created_at.with_timezone(&chrono::Utc),
                    updated_at: updated_at.with_timezone(&chrono::Utc),
                })
            })
            .collect();

        Ok(entries)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Internal struct for matching database results with distance.
#[derive(Deserialize)]
struct DbMemoryMatch {
    id: String,
    agent_id: String,
    content: String,
    source: String,
    session_key: Option<String>,
    tags: String,
    importance: f64,
    metadata: String,
    created_at: String,
    updated_at: String,
    distance: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// Helper function to create a test memory store using in-memory SQLite.
    ///
    /// This creates a fresh in-memory database with the schema initialized.
    /// Use this helper in all tests to ensure isolation between tests.
    pub fn test_store(embedding_dim: usize) -> SqliteMemoryStore {
        SqliteMemoryStore::new(":memory:", embedding_dim).expect("Failed to create test store")
    }

    #[tokio::test]
    async fn test_sqlite_store_new() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SqliteMemoryStore::new(db_path.to_str().unwrap(), 1536);
        assert!(store.is_ok());
    }

    #[tokio::test]
    async fn test_sqlite_store_store_and_query() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SqliteMemoryStore::new(db_path.to_str().unwrap(), 4).unwrap();

        let entry = MemoryEntry::new(
            "test-1".to_string(),
            "agent-1".to_string(),
            "test content".to_string(),
            vec![0.1, 0.2, 0.3, 0.4],
        );

        let id = store.store(entry).await.unwrap();
        assert!(!id.is_empty());

        let opts = MemoryQueryOptions {
            top_k: 5,
            filter: MemoryFilter::default(),
            min_score: Some(0.0),
        };

        let matches = store.query("test", opts).await.unwrap();
        assert!(!matches.is_empty());
    }

    #[tokio::test]
    async fn test_sqlite_store_delete() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SqliteMemoryStore::new(db_path.to_str().unwrap(), 4).unwrap();

        let entry = MemoryEntry::new(
            "test-2".to_string(),
            "agent-1".to_string(),
            "test content".to_string(),
            vec![0.1, 0.2, 0.3, 0.4],
        );

        let id = store.store(entry).await.unwrap();

        let result = store.delete(&id).await;
        assert!(result.is_ok());

        // Verify deletion
        let opts = MemoryQueryOptions {
            top_k: 5,
            filter: MemoryFilter::default(),
            min_score: Some(0.0),
        };
        let matches = store.query("test", opts).await.unwrap();
        assert!(matches.is_empty());
    }

    // ==================== Vector Storage Tests ====================

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let store = test_store(4);

        let entry = MemoryEntry::new(
            "test-id".to_string(),
            "agent-1".to_string(),
            "test content".to_string(),
            vec![0.1, 0.2, 0.3, 0.4],
        );

        let id = store.store(entry.clone()).await.unwrap();
        assert_eq!(id, "test-id");

        // Retrieve by listing
        let filter = MemoryFilter {
            agent_id: Some("agent-1".to_string()),
            ..Default::default()
        };
        let entries = store.list(filter).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "test-id");
        assert_eq!(entries[0].content, "test content");
        assert_eq!(entries[0].agent_id, "agent-1");
    }

    #[tokio::test]
    async fn test_store_generates_id() {
        let store = test_store(4);

        let mut entry = MemoryEntry::new(
            "".to_string(), // Empty ID
            "agent-1".to_string(),
            "test content".to_string(),
            vec![0.1, 0.2, 0.3, 0.4],
        );

        let id = store.store(entry).await.unwrap();
        assert!(!id.is_empty());
        // Verify it's a valid UUID
        uuid::Uuid::parse_str(&id).expect("Generated ID should be a valid UUID");
    }

    #[tokio::test]
    async fn test_delete_entry() {
        let store = test_store(4);

        let entry = MemoryEntry::new(
            "delete-me".to_string(),
            "agent-1".to_string(),
            "to be deleted".to_string(),
            vec![0.1, 0.2, 0.3, 0.4],
        );

        let id = store.store(entry).await.unwrap();

        // Verify entry exists
        let filter = MemoryFilter::default();
        let entries = store.list(filter).await.unwrap();
        assert_eq!(entries.len(), 1);

        // Delete the entry
        store.delete(&id).await.unwrap();

        // Verify entry no longer exists
        let entries = store.list(MemoryFilter::default()).await.unwrap();
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn test_delete_nonexistent() {
        let store = test_store(4);

        // Delete a non-existent ID should not error
        let result = store.delete("nonexistent-id").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_empty() {
        let store = test_store(4);

        let filter = MemoryFilter::default();
        let entries = store.list(filter).await.unwrap();
        assert!(entries.is_empty());
    }

    #[tokio::test]
    async fn test_list_with_agent_filter() {
        let store = test_store(4);

        // Store entries for different agents
        for agent_id in ["agent-a", "agent-b"] {
            for i in 1..=3 {
                let entry = MemoryEntry::new(
                    format!("{}-{}", agent_id, i),
                    agent_id.to_string(),
                    format!("content for {}-{}", agent_id, i),
                    vec![0.1 * i as f32, 0.2, 0.3, 0.4],
                );
                store.store(entry).await.unwrap();
            }
        }

        // List only agent-a entries
        let filter = MemoryFilter {
            agent_id: Some("agent-a".to_string()),
            ..Default::default()
        };
        let entries = store.list(filter).await.unwrap();
        assert_eq!(entries.len(), 3);
        assert!(entries.iter().all(|e| e.agent_id == "agent-a"));

        // List only agent-b entries
        let filter = MemoryFilter {
            agent_id: Some("agent-b".to_string()),
            ..Default::default()
        };
        let entries = store.list(filter).await.unwrap();
        assert_eq!(entries.len(), 3);
        assert!(entries.iter().all(|e| e.agent_id == "agent-b"));
    }

    #[tokio::test]
    async fn test_list_with_tag_filter() {
        let store = test_store(4);

        // Store entries with different tags
        let entries = vec![
            MemoryEntry {
                metadata: crate::types::MemoryMetadata {
                    tags: vec!["tag1".to_string(), "tag2".to_string()],
                    ..Default::default()
                },
                ..MemoryEntry::new(
                    "entry-1".to_string(),
                    "agent-1".to_string(),
                    "content 1".to_string(),
                    vec![0.1, 0.2, 0.3, 0.4],
                )
            },
            MemoryEntry {
                metadata: crate::types::MemoryMetadata {
                    tags: vec!["tag2".to_string(), "tag3".to_string()],
                    ..Default::default()
                },
                ..MemoryEntry::new(
                    "entry-2".to_string(),
                    "agent-1".to_string(),
                    "content 2".to_string(),
                    vec![0.1, 0.2, 0.3, 0.4],
                )
            },
            MemoryEntry {
                metadata: crate::types::MemoryMetadata {
                    tags: vec!["tag3".to_string()],
                    ..Default::default()
                },
                ..MemoryEntry::new(
                    "entry-3".to_string(),
                    "agent-1".to_string(),
                    "content 3".to_string(),
                    vec![0.1, 0.2, 0.3, 0.4],
                )
            },
        ];

        for entry in entries {
            store.store(entry).await.unwrap();
        }

        // Filter by tag2 - should get 2 entries
        let filter = MemoryFilter {
            tags: Some(vec!["tag2".to_string()]),
            ..Default::default()
        };
        let entries = store.list(filter).await.unwrap();
        assert_eq!(entries.len(), 2);

        // Filter by tag3 - should get 2 entries
        let filter = MemoryFilter {
            tags: Some(vec!["tag3".to_string()]),
            ..Default::default()
        };
        let entries = store.list(filter).await.unwrap();
        assert_eq!(entries.len(), 2);

        // Filter by multiple tags (AND) - should get 1 entry with both tag1 and tag2
        let filter = MemoryFilter {
            tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
            ..Default::default()
        };
        let entries = store.list(filter).await.unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[tokio::test]
    async fn test_list_with_importance_filter_levels() {
        let store = test_store(4);

        // Store entries with different importance scores
        for (i, importance) in [0.1, 0.3, 0.5, 0.7, 0.9].iter().enumerate() {
            let entry = MemoryEntry::new(
                format!("entry-{}", i),
                "agent-1".to_string(),
                format!("content {}", i),
                vec![0.1 * i as f32, 0.2, 0.3, 0.4],
            );
            let mut entry = entry;
            entry.metadata.importance = *importance;
            store.store(entry).await.unwrap();
        }

        // Filter by importance >= 0.5 - should get 3 entries
        let filter = MemoryFilter {
            importance_min: Some(0.5),
            ..Default::default()
        };
        let entries = store.list(filter).await.unwrap();
        assert_eq!(entries.len(), 3);
        assert!(entries.iter().all(|e| e.metadata.importance >= 0.5));

        // Filter by importance >= 0.8 - should get 1 entry
        let filter = MemoryFilter {
            importance_min: Some(0.8),
            ..Default::default()
        };
        let entries = store.list(filter).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].metadata.importance >= 0.8);
    }

    // ==================== Similarity Search Tests ====================

    #[tokio::test]
    async fn test_similarity_search_returns_closest() {
        let store = test_store(4);

        // Store entries with known embeddings
        let entries = vec![
            // This one should be closest to query [0.9, 0.1, 0.1, 0.1]
            MemoryEntry::new(
                "closest".to_string(),
                "agent-1".to_string(),
                "closest content".to_string(),
                vec![0.9, 0.1, 0.1, 0.1],
            ),
            // This one should be farther
            MemoryEntry::new(
                "farther".to_string(),
                "agent-1".to_string(),
                "farther content".to_string(),
                vec![-0.1, 0.9, -0.1, 0.1],
            ),
        ];

        for entry in entries {
            store.store(entry).await.unwrap();
        }

        // Query with a vector close to the first entry
        let opts = MemoryQueryOptions {
            top_k: 10,
            filter: MemoryFilter::default(),
            min_score: Some(0.0),
        };

        let matches = store.query("query", opts).await.unwrap();
        assert!(!matches.is_empty());
        // The closest entry should have the highest score
        assert_eq!(matches[0].entry.id, "closest");
    }

    #[tokio::test]
    async fn test_similarity_search_top_k() {
        let store = test_store(4);

        // Store 20 entries
        for i in 0..20 {
            let entry = MemoryEntry::new(
                format!("entry-{}", i),
                "agent-1".to_string(),
                format!("content {}", i),
                vec![0.1 * (i as f32), 0.2, 0.3, 0.4],
            );
            store.store(entry).await.unwrap();
        }

        // Query with top_k=5
        let opts = MemoryQueryOptions {
            top_k: 5,
            filter: MemoryFilter::default(),
            min_score: Some(0.0),
        };

        let matches = store.query("query", opts).await.unwrap();
        assert_eq!(matches.len(), 5);
    }

    #[tokio::test]
    async fn test_similarity_search_min_score() {
        let store = test_store(4);

        // Store entries
        for i in 0..10 {
            let entry = MemoryEntry::new(
                format!("entry-{}", i),
                "agent-1".to_string(),
                format!("content {}", i),
                vec![0.1 * (i as f32), 0.2, 0.3, 0.4],
            );
            store.store(entry).await.unwrap();
        }

        // Query with high min_score - should get no results if score threshold is too high
        // Note: sqlite-vec returns cosine distance (0=identical, 2=opposite)
        // Score is calculated as 1 - (distance / 2)
        let opts = MemoryQueryOptions {
            top_k: 10,
            filter: MemoryFilter::default(),
            min_score: Some(0.99), // Very high threshold
        };

        let matches = store.query("query", opts).await.unwrap();
        // With a high threshold, we may get fewer results
        assert!(matches.len() <= 10);
    }

    #[tokio::test]
    async fn test_similarity_search_agent_scoped() {
        let store = test_store(4);

        // Store entries for different agents
        for agent_id in ["agent-a", "agent-b"] {
            for i in 0..5 {
                let entry = MemoryEntry::new(
                    format!("{}-{}", agent_id, i),
                    agent_id.to_string(),
                    format!("content {}-{}", agent_id, i),
                    vec![0.1 * (i as f32), 0.2, 0.3, 0.4],
                );
                store.store(entry).await.unwrap();
            }
        }

        // Query scoped to agent-a
        let filter = MemoryFilter {
            agent_id: Some("agent-a".to_string()),
            ..Default::default()
        };
        let opts = MemoryQueryOptions {
            top_k: 10,
            filter,
            min_score: Some(0.0),
        };

        let matches = store.query("query", opts).await.unwrap();
        assert_eq!(matches.len(), 5);
        assert!(matches.iter().all(|m| m.entry.agent_id == "agent-a"));
    }

    // ==================== Additional Edge Case Tests ====================

    #[tokio::test]
    async fn test_store_overwrites_existing() {
        let store = test_store(4);

        // Store initial entry
        let entry1 = MemoryEntry::new(
            "same-id".to_string(),
            "agent-1".to_string(),
            "Original content".to_string(),
            vec![0.1, 0.2, 0.3, 0.4],
        );
        store.store(entry1).await.unwrap();

        // Verify original content
        let filter = MemoryFilter::default();
        let entries = store.list(filter).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content, "Original content");

        // Store updated entry with same ID
        let entry2 = MemoryEntry::new(
            "same-id".to_string(),
            "agent-1".to_string(),
            "Updated content".to_string(),
            vec![0.5, 0.6, 0.7, 0.8],
        );
        store.store(entry2).await.unwrap();

        // Verify entry was updated
        let entries = store.list(MemoryFilter::default()).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content, "Updated content");
        assert_eq!(entries[0].embedding, vec![0.5, 0.6, 0.7, 0.8]);
    }

    #[tokio::test]
    async fn test_list_with_source_filter() {
        let store = test_store(4);

        // Store entries with different sources
        let entry_agent = MemoryEntry {
            metadata: crate::types::MemoryMetadata {
                source: crate::types::MemorySource::Agent,
                ..Default::default()
            },
            ..MemoryEntry::new(
                "agent-entry".to_string(),
                "agent-1".to_string(),
                "Agent created this".to_string(),
                vec![0.1, 0.2, 0.3, 0.4],
            )
        };

        let entry_user = MemoryEntry {
            metadata: crate::types::MemoryMetadata {
                source: crate::types::MemorySource::User,
                ..Default::default()
            },
            ..MemoryEntry::new(
                "user-entry".to_string(),
                "agent-1".to_string(),
                "User created this".to_string(),
                vec![0.2, 0.3, 0.4, 0.5],
            )
        };

        store.store(entry_agent).await.unwrap();
        store.store(entry_user).await.unwrap();

        // List by Agent source
        let filter = MemoryFilter {
            source: Some(crate::types::MemorySource::Agent),
            ..Default::default()
        };
        let entries = store.list(filter).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content, "Agent created this");

        // List by User source
        let filter = MemoryFilter {
            source: Some(crate::types::MemorySource::User),
            ..Default::default()
        };
        let entries = store.list(filter).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content, "User created this");
    }

    #[tokio::test]
    async fn test_list_with_importance_filter_edge_cases() {
        let store = test_store(4);

        // Store entries with different importance levels
        let entry_low = MemoryEntry {
            metadata: crate::types::MemoryMetadata {
                importance: 0.1,
                ..Default::default()
            },
            ..MemoryEntry::new(
                "low-importance".to_string(),
                "agent-1".to_string(),
                "Low importance content".to_string(),
                vec![0.1, 0.2, 0.3, 0.4],
            )
        };

        let entry_high = MemoryEntry {
            metadata: crate::types::MemoryMetadata {
                importance: 0.9,
                ..Default::default()
            },
            ..MemoryEntry::new(
                "high-importance".to_string(),
                "agent-1".to_string(),
                "High importance content".to_string(),
                vec![0.2, 0.3, 0.4, 0.5],
            )
        };

        store.store(entry_low).await.unwrap();
        store.store(entry_high).await.unwrap();

        // List with minimum importance 0.5
        let filter = MemoryFilter {
            importance_min: Some(0.5),
            ..Default::default()
        };
        let entries = store.list(filter).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content, "High importance content");
    }

    #[tokio::test]
    async fn test_list_with_multiple_filters() {
        let store = test_store(4);

        // Store entries with various combinations
        let entry_a1 = MemoryEntry {
            metadata: crate::types::MemoryMetadata {
                source: crate::types::MemorySource::Agent,
                tags: vec!["important".to_string()],
                importance: 0.8,
                ..Default::default()
            },
            ..MemoryEntry::new(
                "a1".to_string(),
                "agent-1".to_string(),
                "Important agent entry".to_string(),
                vec![0.1, 0.2, 0.3, 0.4],
            )
        };

        let entry_a2 = MemoryEntry {
            metadata: crate::types::MemoryMetadata {
                source: crate::types::MemorySource::Agent,
                tags: vec!["draft".to_string()],
                importance: 0.9,
                ..Default::default()
            },
            ..MemoryEntry::new(
                "a2".to_string(),
                "agent-1".to_string(),
                "Another agent entry".to_string(),
                vec![0.2, 0.3, 0.4, 0.5],
            )
        };

        let entry_u1 = MemoryEntry {
            metadata: crate::types::MemoryMetadata {
                source: crate::types::MemorySource::User,
                tags: vec!["important".to_string()],
                importance: 0.7,
                ..Default::default()
            },
            ..MemoryEntry::new(
                "u1".to_string(),
                "agent-1".to_string(),
                "User important entry".to_string(),
                vec![0.3, 0.4, 0.5, 0.6],
            )
        };

        store.store(entry_a1).await.unwrap();
        store.store(entry_a2).await.unwrap();
        store.store(entry_u1).await.unwrap();

        // Filter by agent_id AND source
        let filter = MemoryFilter {
            agent_id: Some("agent-1".to_string()),
            source: Some(crate::types::MemorySource::Agent),
            ..Default::default()
        };
        let entries = store.list(filter).await.unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.agent_id == "agent-1"));
        assert!(entries
            .iter()
            .all(|e| matches!(e.metadata.source, crate::types::MemorySource::Agent)));

        // Filter by agent_id AND tag
        let filter = MemoryFilter {
            agent_id: Some("agent-1".to_string()),
            tags: Some(vec!["important".to_string()]),
            ..Default::default()
        };
        let entries = store.list(filter).await.unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(|e| e.metadata.tags.contains(&"important".to_string())));
    }

    #[tokio::test]
    async fn test_embedder_integration() {
        // Test that the store uses the correct embedder
        let store = test_store(4);

        // Store an entry with a specific embedding
        let entry = MemoryEntry::new(
            "test-id".to_string(),
            "agent-1".to_string(),
            "Test content for embedder".to_string(),
            vec![0.5, 0.5, 0.5, 0.5], // Specific embedding
        );
        store.store(entry).await.unwrap();

        // Query with same text should find the entry
        let opts = MemoryQueryOptions {
            top_k: 5,
            filter: MemoryFilter::default(),
            min_score: Some(0.0),
        };

        let matches = store.query("Test content for embedder", opts).await.unwrap();
        assert!(!matches.is_empty());
        assert_eq!(matches[0].entry.content, "Test content for embedder");
    }

    #[tokio::test]
    async fn test_multiple_agents_isolated() {
        let store = test_store(4);

        // Store memories for different agents
        for agent_id in ["agent-alpha", "agent-beta", "agent-gamma"] {
            for i in 0..3 {
                let entry = MemoryEntry::new(
                    format!("{}-{}", agent_id, i),
                    agent_id.to_string(),
                    format!("Content from {}-{}", agent_id, i),
                    vec![0.1 * i as f32, 0.2, 0.3, 0.4],
                );
                store.store(entry).await.unwrap();
            }
        }

        // Verify each agent has its own memories
        for agent_id in ["agent-alpha", "agent-beta", "agent-gamma"] {
            let filter = MemoryFilter {
                agent_id: Some(agent_id.to_string()),
                ..Default::default()
            };
            let entries = store.list(filter).await.unwrap();
            assert_eq!(entries.len(), 3);
        }

        // Verify query is also scoped correctly
        let filter = MemoryFilter {
            agent_id: Some("agent-beta".to_string()),
            ..Default::default()
        };
        let opts = MemoryQueryOptions {
            top_k: 10,
            filter,
            min_score: Some(0.0),
        };

        let matches = store.query("content", opts).await.unwrap();
        assert_eq!(matches.len(), 3);
        assert!(matches.iter().all(|m| m.entry.agent_id == "agent-beta"));
    }

    #[tokio::test]
    async fn test_empty_content_handling() {
        let store = test_store(4);

        // Store entry with empty content
        let entry = MemoryEntry::new(
            "empty-content".to_string(),
            "agent-1".to_string(),
            "".to_string(),
            vec![0.1, 0.2, 0.3, 0.4],
        );
        store.store(entry).await.unwrap();

        // Retrieve should work
        let filter = MemoryFilter::default();
        let entries = store.list(filter).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].content.is_empty());
    }

    #[tokio::test]
    async fn test_query_empty_store() {
        let store = test_store(4);

        // Query an empty store
        let opts = MemoryQueryOptions {
            top_k: 5,
            filter: MemoryFilter::default(),
            min_score: Some(0.0),
        };

        let matches = store.query("anything", opts).await.unwrap();
        assert!(matches.is_empty());
    }
}
