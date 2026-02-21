//! SQLite-Vec memory storage backend.
//!
//! This module provides a `SqliteMemoryStore` implementation using SQLite with
//! the sqlite-vec extension for vector storage and cosine similarity search.

use crate::store::MemoryStore;
use crate::types::{
    MemoryEntry, MemoryFilter, MemoryMatch, MemoryQueryOptions, MemorySource,
};
use anyhow::{anyhow, Result};
use rusqlite::{Connection, ToSql};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Memory storage backend using SQLite with sqlite-vec extension.
///
/// This struct manages a SQLite database with two tables:
/// - `memories`: Stores memory metadata and content
/// - `memory_embeddings`: Virtual table for vector operations using sqlite-vec
pub struct SqliteMemoryStore {
    db: Arc<Mutex<Connection>>,
    embedding_dim: usize,
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
    /// Creates a new `SqliteMemoryStore` at the given path.
    ///
    /// # Arguments
    /// * `path` - Path to the SQLite database file (use `:memory:` for in-memory DB)
    /// * `embedding_dim` - Dimension of the embeddings to store
    ///
    /// # Returns
    /// Returns a new `SqliteMemoryStore` or an error if initialization fails.
    pub fn new(path: &str, embedding_dim: usize) -> Result<Self> {
        let db = Connection::open(path)?;

        // Load the sqlite-vec extension (unsafe function)
        unsafe {
            db.load_extension("vec0", None).map_err(|e| anyhow!(e))?;
        }

        // Create the schema
        Self::create_schema(&db, embedding_dim)?;

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            embedding_dim,
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
            VALUES (:id, :agent_id, :content, :source, :session_key, :tags, :importance, :metadata, :created_at, :updated_at)
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
                &tags_json,  // For tags
                metadata_json.to_string(),
                db_input.created_at,
                db_input.updated_at,
            ],
        )?;

        // Insert into memory_embeddings table
        let embedding: Vec<f32> = entry.embedding.iter().map(|&x| x as f32).collect();
        // Serialize embedding as bytes for storage
        let embedding_bytes: Vec<u8> = embedding
            .iter()
            .flat_map(|&x| x.to_le_bytes())
            .collect();

        db.execute(
            r#"
            INSERT INTO memory_embeddings (id, embedding)
            VALUES (:id, :embedding)
            ON CONFLICT(id) DO UPDATE SET
                embedding = excluded.embedding
            "#,
            rusqlite::params![
                &entry.id,
                &embedding_bytes as &[u8]
            ],
        )?;

        Ok(entry.id)
    }

    async fn query(&self, query: &str, opts: MemoryQueryOptions) -> Result<Vec<MemoryMatch>> {
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
            format!("WHERE {}", conditions.join(" AND "))
        };

        // Execute the query
        let sql = format!(
            r#"
            SELECT m.id, m.agent_id, m.content, m.source, m.session_key, m.tags, m.importance, m.metadata, m.created_at, m.updated_at, e.distance
            FROM memory_embeddings e
            JOIN memories m ON e.id = m.id
            {}
            ORDER BY e.distance ASC
            LIMIT {}
            "#,
            where_clause, opts.top_k
        );

        let mut stmt = db.prepare(&sql)?;
        let results: Vec<DbMemoryMatch> = stmt
            .query_map(rusqlite::params_from_iter(params.iter()), |row| {
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
        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(matches)
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let mut db = self.db.lock().map_err(|e| anyhow!(e.to_string()))?;

        // Delete from memory_embeddings table
        db.execute("DELETE FROM memory_embeddings WHERE id = ?", rusqlite::params![id])?;

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
                        "EXISTS (SELECT 1 FROM json_each(memories.tags) WHERE json_each.value = ?)"
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
            SELECT id, agent_id, content, source, session_key, tags, importance, metadata, created_at, updated_at
            FROM memories
            {}
            "#,
            where_clause
        );

        let mut stmt = db.prepare(&sql)?;
        let results: Vec<DbMemory> = stmt
            .query_map(rusqlite::params_from_iter(params.iter()), |row| {
                Ok(DbMemory {
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
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        let entries = results
            .into_iter()
            .filter_map(|m| {
                let created_at = chrono::DateTime::parse_from_rfc3339(&m.created_at).ok()?;
                let updated_at = chrono::DateTime::parse_from_rfc3339(&m.updated_at).ok()?;

                Some(MemoryEntry {
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
                    created_at: created_at.with_timezone(&chrono::Utc),
                    updated_at: updated_at.with_timezone(&chrono::Utc),
                })
            })
            .collect();

        Ok(entries)
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

    #[test]
    fn test_sqlite_store_new() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SqliteMemoryStore::new(db_path.to_str().unwrap(), 1536);
        assert!(store.is_ok());
    }

    #[test]
    fn test_sqlite_store_store_and_query() {
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

    #[test]
    fn test_sqlite_store_delete() {
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
}
