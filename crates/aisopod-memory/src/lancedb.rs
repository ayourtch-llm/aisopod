//! LanceDB memory storage backend.
//!
//! This module provides a `LanceDbMemoryStore` implementation using LanceDB
//! for vector storage and similarity search, offering an alternative to the
//! SQLite-Vec backend.

#![cfg(feature = "lancedb")]

use crate::store::MemoryStore;
use crate::types::{MemoryEntry, MemoryFilter, MemoryMatch, MemoryQueryOptions, MemorySource};
use anyhow::{anyhow, Result};
use arrow_array::Array;
use chrono::DateTime;
use futures_util::stream::TryStreamExt;
use lancedb::arrow::RecordBatchStream;
use lancedb::connection::Connection as LanceDbConnection;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::table::Table as LanceDbTable;
use lancedb::Result as LanceDbResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// Re-export arrow types needed for the lancedb feature
#[cfg(feature = "arrow-array")]
pub use arrow_array as arrow_array_internal;
#[cfg(feature = "arrow-array")]
pub use arrow_array::types::Float32Type;
#[cfg(feature = "arrow-schema")]
pub use arrow_schema as arrow_schema_internal;

/// Memory storage backend using LanceDB.
///
/// This struct manages a LanceDB connection and table for storing and retrieving
/// memory entries with vector embeddings.
pub struct LanceDbMemoryStore {
    db: LanceDbConnection,
    table: LanceDbTable,
    table_name: String,
    embedding_dim: usize,
}

/// Helper struct for deserializing memory entries from LanceDB.
#[derive(Debug, Deserialize, Serialize)]
struct DbMemory {
    id: String,
    agent_id: String,
    content: String,
    source: String,
    session_key: Option<String>,
    tags: Vec<String>,
    importance: f64,
    metadata: HashMap<String, serde_json::Value>,
    created_at: i64,
    updated_at: i64,
    embedding: Vec<f32>,
}

impl LanceDbMemoryStore {
    /// Creates a new `LanceDbMemoryStore` at the given path.
    ///
    /// # Arguments
    /// * `path` - Path to the LanceDB dataset directory
    /// * `embedding_dim` - Dimension of the embeddings to store
    ///
    /// # Returns
    /// Returns a new `LanceDbMemoryStore` or an error if initialization fails.
    pub async fn new(path: &str, embedding_dim: usize) -> Result<Self> {
        let db = lancedb::connect(path)
            .execute()
            .await
            .map_err(|e| anyhow!("Failed to connect to LanceDB: {}", e))?;

        // Try to open existing table or create new one
        let table_name = "memories";

        let table = match db.open_table(table_name).execute().await {
            Ok(tbl) => tbl,
            Err(_) => {
                // Create schema with lancedb's arrow module
                let schema = Arc::new(lancedb::arrow::arrow_schema::Schema::new(vec![
                    lancedb::arrow::arrow_schema::Field::new(
                        "id",
                        lancedb::arrow::arrow_schema::DataType::Utf8,
                        false,
                    ),
                    lancedb::arrow::arrow_schema::Field::new(
                        "agent_id",
                        lancedb::arrow::arrow_schema::DataType::Utf8,
                        false,
                    ),
                    lancedb::arrow::arrow_schema::Field::new(
                        "content",
                        lancedb::arrow::arrow_schema::DataType::Utf8,
                        false,
                    ),
                    lancedb::arrow::arrow_schema::Field::new(
                        "source",
                        lancedb::arrow::arrow_schema::DataType::Utf8,
                        false,
                    ),
                    lancedb::arrow::arrow_schema::Field::new(
                        "session_key",
                        lancedb::arrow::arrow_schema::DataType::Utf8,
                        true,
                    ),
                    lancedb::arrow::arrow_schema::Field::new(
                        "tags",
                        lancedb::arrow::arrow_schema::DataType::Utf8,
                        false,
                    ),
                    lancedb::arrow::arrow_schema::Field::new(
                        "importance",
                        lancedb::arrow::arrow_schema::DataType::Float64,
                        false,
                    ),
                    lancedb::arrow::arrow_schema::Field::new(
                        "metadata",
                        lancedb::arrow::arrow_schema::DataType::Utf8,
                        false,
                    ),
                    lancedb::arrow::arrow_schema::Field::new(
                        "created_at",
                        lancedb::arrow::arrow_schema::DataType::Int64,
                        false,
                    ),
                    lancedb::arrow::arrow_schema::Field::new(
                        "updated_at",
                        lancedb::arrow::arrow_schema::DataType::Int64,
                        false,
                    ),
                    lancedb::arrow::arrow_schema::Field::new(
                        "embedding",
                        lancedb::arrow::arrow_schema::DataType::FixedSizeList(
                            Arc::new(lancedb::arrow::arrow_schema::Field::new(
                                "item",
                                lancedb::arrow::arrow_schema::DataType::Float32,
                                true,
                            )),
                            embedding_dim as i32,
                        ),
                        true,
                    ),
                ]));

                // Create empty table with schema
                db.create_empty_table(table_name, schema)
                    .execute()
                    .await
                    .map_err(|e| anyhow!("Failed to create table: {}", e))?
            }
        };

        Ok(Self {
            db,
            table,
            table_name: table_name.to_string(),
            embedding_dim,
        })
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
impl MemoryStore for LanceDbMemoryStore {
    async fn store(&self, mut entry: MemoryEntry) -> Result<String> {
        // Generate ID if not provided
        if entry.id.is_empty() {
            entry.id = uuid::Uuid::new_v4().to_string();
        }

        // Serialize data for storage
        let tags_json = Self::serialize_tags(&entry.metadata.tags);
        let metadata_json = serde_json::to_value(&entry.metadata.custom)
            .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));
        let source_str = Self::source_to_string(&entry.metadata.source);

        // Convert timestamps to i64 (Unix timestamp in seconds)
        let created_at = entry.created_at.timestamp();
        let updated_at = entry.updated_at.timestamp();

        let db_memory = DbMemory {
            id: entry.id.clone(),
            agent_id: entry.agent_id.clone(),
            content: entry.content.clone(),
            source: source_str.to_string(),
            session_key: entry.metadata.session_key.clone(),
            tags: entry.metadata.tags.clone(),
            importance: entry.metadata.importance as f64,
            metadata: entry.metadata.custom.clone(),
            created_at,
            updated_at,
            embedding: entry.embedding.clone(),
        };

        // Convert to record batch for insertion
        let record = Self::db_memory_to_record(&db_memory, self.embedding_dim)?;

        // Wrap in RecordBatchIterator for IntoArrow
        let schema = record.schema();
        let batch_reader =
            arrow_array::RecordBatchIterator::new(vec![record].into_iter().map(Ok), schema);

        // Add to table
        self.table
            .add(batch_reader)
            .execute()
            .await
            .map_err(|e| anyhow!("Failed to insert record: {}", e))?;

        Ok(entry.id)
    }

    async fn query(&self, query: &str, opts: MemoryQueryOptions) -> Result<Vec<MemoryMatch>> {
        // Build filter string
        let mut filters = Vec::new();

        if let Some(agent_id) = &opts.filter.agent_id {
            filters.push(format!("agent_id = '{}'", agent_id.replace("'", "''")));
        }

        if let Some(source) = &opts.filter.source {
            let source_str = Self::source_to_string(source);
            filters.push(format!("source = '{}'", source_str.replace("'", "''")));
        }

        if let Some(session_key) = &opts.filter.session_key {
            filters.push(format!(
                "session_key = '{}'",
                session_key.replace("'", "''")
            ));
        }

        if let Some(importance_min) = &opts.filter.importance_min {
            filters.push(format!("importance >= {}", importance_min));
        }

        if let Some(created_after) = &opts.filter.created_after {
            filters.push(format!("created_at >= {}", created_after.timestamp()));
        }

        if let Some(created_before) = &opts.filter.created_before {
            filters.push(format!("created_at <= {}", created_before.timestamp()));
        }

        // Handle tags filter
        if let Some(tags) = &opts.filter.tags {
            for tag in tags {
                let escaped_tag = tag.replace("'", "''");
                filters.push(format!("tags LIKE '%{}%'", escaped_tag));
            }
        }

        let where_clause = if filters.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", filters.join(" AND "))
        };

        // Create a dummy query vector (LanceDB requires a vector for vector search)
        let dummy_vector: Vec<f32> = vec![0.0; self.embedding_dim];

        // Execute vector search
        let mut query = self
            .table
            .query()
            .nearest_to(dummy_vector)?
            .limit(opts.top_k as usize);

        if !where_clause.is_empty() {
            query = query.only_if(&where_clause);
        }

        // Execute the query and get results as a stream
        let results = query
            .execute()
            .await
            .map_err(|e| anyhow!("Query failed: {}", e))?;

        // Convert stream to Vec<RecordBatch>
        let batches: Vec<arrow_array::RecordBatch> = results
            .try_collect()
            .await
            .map_err(|e| anyhow!("Result collection failed: {:?}", e))?;

        // Convert to MemoryMatch
        let min_score = opts.min_score.unwrap_or(0.0);
        let mut matches: Vec<MemoryMatch> = Vec::new();

        for record in batches.iter() {
            // Access columns by index
            let id = record
                .column(0)
                .as_any()
                .downcast_ref::<arrow_array::StringArray>()
                .unwrap()
                .value(0)
                .to_string();
            let agent_id = record
                .column(1)
                .as_any()
                .downcast_ref::<arrow_array::StringArray>()
                .unwrap()
                .value(0)
                .to_string();
            let content = record
                .column(2)
                .as_any()
                .downcast_ref::<arrow_array::StringArray>()
                .unwrap()
                .value(0)
                .to_string();
            let source = record
                .column(3)
                .as_any()
                .downcast_ref::<arrow_array::StringArray>()
                .unwrap()
                .value(0)
                .to_string();
            // Handle nullable session_key using is_null() + value() pattern
            let session_key_arr = record
                .column(4)
                .as_any()
                .downcast_ref::<arrow_array::StringArray>()
                .unwrap();
            let session_key = if session_key_arr.is_null(0) {
                None
            } else {
                Some(session_key_arr.value(0).to_string())
            };
            let tags_json = record
                .column(5)
                .as_any()
                .downcast_ref::<arrow_array::StringArray>()
                .unwrap()
                .value(0)
                .to_string();
            let importance = record
                .column(6)
                .as_any()
                .downcast_ref::<arrow_array::Float64Array>()
                .unwrap()
                .value(0);
            let metadata_json = record
                .column(7)
                .as_any()
                .downcast_ref::<arrow_array::StringArray>()
                .unwrap()
                .value(0)
                .to_string();
            let created_at = record
                .column(8)
                .as_any()
                .downcast_ref::<arrow_array::Int64Array>()
                .unwrap()
                .value(0);
            let updated_at = record
                .column(9)
                .as_any()
                .downcast_ref::<arrow_array::Int64Array>()
                .unwrap()
                .value(0);
            // The distance column should be available
            let distance = record
                .column(11)
                .as_any()
                .downcast_ref::<arrow_array::Float32Array>()
                .unwrap()
                .value(0);

            // Parse JSON values
            let tags: Vec<String> =
                serde_json::from_str(tags_json.as_str()).unwrap_or_else(|_| Vec::new());
            let metadata: HashMap<String, serde_json::Value> =
                serde_json::from_str(metadata_json.as_str()).unwrap_or_else(|_| HashMap::new());

            // Convert distance to score (LanceDB uses cosine distance by default)
            let score = 1.0 - (distance / 2.0);

            if score < min_score {
                continue;
            }

            let entry = MemoryEntry {
                id,
                agent_id,
                content,
                embedding: vec![0.0; self.embedding_dim], // We'll need to extract from the embedding column
                metadata: crate::types::MemoryMetadata {
                    source: Self::string_to_source(&source),
                    session_key,
                    tags,
                    importance: importance as f32,
                    custom: metadata,
                },
                created_at: DateTime::from_timestamp_millis(created_at * 1000)
                    .ok_or_else(|| anyhow!("Invalid timestamp"))?
                    .with_timezone(&chrono::Utc),
                updated_at: DateTime::from_timestamp_millis(updated_at * 1000)
                    .ok_or_else(|| anyhow!("Invalid timestamp"))?
                    .with_timezone(&chrono::Utc),
            };

            matches.push(MemoryMatch {
                entry,
                score: score as f32,
            });
        }

        // Sort by score descending
        matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(matches)
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let sql = format!("id = '{}'", id.replace("'", "''"));
        self.table
            .delete(&sql)
            .await
            .map_err(|e| anyhow!("Delete failed: {}", e))?;

        Ok(())
    }

    async fn list(&self, filter: MemoryFilter) -> Result<Vec<MemoryEntry>> {
        // Build filter string
        let mut filters = Vec::new();

        if let Some(agent_id) = &filter.agent_id {
            filters.push(format!("agent_id = '{}'", agent_id.replace("'", "''")));
        }

        if let Some(source) = &filter.source {
            let source_str = Self::source_to_string(source);
            filters.push(format!("source = '{}'", source_str.replace("'", "''")));
        }

        if let Some(session_key) = &filter.session_key {
            filters.push(format!(
                "session_key = '{}'",
                session_key.replace("'", "''")
            ));
        }

        if let Some(importance_min) = &filter.importance_min {
            filters.push(format!("importance >= {}", importance_min));
        }

        if let Some(created_after) = &filter.created_after {
            filters.push(format!("created_at >= {}", created_after.timestamp()));
        }

        if let Some(created_before) = &filter.created_before {
            filters.push(format!("created_at <= {}", created_before.timestamp()));
        }

        // Handle tags filter
        if let Some(tags) = &filter.tags {
            for tag in tags {
                let escaped_tag = tag.replace("'", "''");
                filters.push(format!("tags LIKE '%{}%'", escaped_tag));
            }
        }

        let where_clause = if filters.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", filters.join(" AND "))
        };

        // Execute query with filter
        let mut query = self.table.query().select(lancedb::query::Select::All);

        if !where_clause.is_empty() {
            query = query.only_if(&where_clause);
        }

        let results = query
            .execute()
            .await
            .map_err(|e| anyhow!("List query failed: {}", e))?;

        let batches: Vec<arrow_array::RecordBatch> = results
            .try_collect()
            .await
            .map_err(|e| anyhow!("{:?}", e))?;

        let mut entries = Vec::new();

        for record in batches.iter() {
            // Manually extract columns from RecordBatch (try_get doesn't exist in arrow-array v53)
            let id = record
                .column(0)
                .as_any()
                .downcast_ref::<arrow_array::StringArray>()
                .unwrap()
                .value(0)
                .to_string();
            let agent_id = record
                .column(1)
                .as_any()
                .downcast_ref::<arrow_array::StringArray>()
                .unwrap()
                .value(0)
                .to_string();
            let content = record
                .column(2)
                .as_any()
                .downcast_ref::<arrow_array::StringArray>()
                .unwrap()
                .value(0)
                .to_string();
            let source = record
                .column(3)
                .as_any()
                .downcast_ref::<arrow_array::StringArray>()
                .unwrap()
                .value(0)
                .to_string();
            let session_key_arr = record
                .column(4)
                .as_any()
                .downcast_ref::<arrow_array::StringArray>()
                .unwrap();
            let session_key = if session_key_arr.is_null(0) {
                None
            } else {
                Some(session_key_arr.value(0).to_string())
            };
            let tags_json = record
                .column(5)
                .as_any()
                .downcast_ref::<arrow_array::StringArray>()
                .unwrap()
                .value(0)
                .to_string();
            let importance = record
                .column(6)
                .as_any()
                .downcast_ref::<arrow_array::Float64Array>()
                .unwrap()
                .value(0);
            let metadata_json = record
                .column(7)
                .as_any()
                .downcast_ref::<arrow_array::StringArray>()
                .unwrap()
                .value(0)
                .to_string();
            let created_at = record
                .column(8)
                .as_any()
                .downcast_ref::<arrow_array::Int64Array>()
                .unwrap()
                .value(0);
            let updated_at = record
                .column(9)
                .as_any()
                .downcast_ref::<arrow_array::Int64Array>()
                .unwrap()
                .value(0);
            // Extract embedding from the FixedSizeListArray
            let embedding_arr = record
                .column(10)
                .as_any()
                .downcast_ref::<arrow_array::FixedSizeListArray>()
                .unwrap();
            let embedding_values = embedding_arr.values();
            let embedding_f32 = embedding_values
                .as_any()
                .downcast_ref::<arrow_array::Float32Array>()
                .unwrap();
            let embedding: Vec<f32> = embedding_f32.values().to_vec();

            let tags: Vec<String> =
                serde_json::from_str(tags_json.as_str()).unwrap_or_else(|_| Vec::new());
            let metadata: HashMap<String, serde_json::Value> =
                serde_json::from_str(metadata_json.as_str()).unwrap_or_else(|_| HashMap::new());

            let entry = MemoryEntry {
                id,
                agent_id,
                content,
                embedding,
                metadata: crate::types::MemoryMetadata {
                    source: Self::string_to_source(&source),
                    session_key,
                    tags,
                    importance: importance as f32,
                    custom: metadata,
                },
                created_at: DateTime::from_timestamp_millis(created_at * 1000)
                    .ok_or_else(|| anyhow!("Invalid timestamp"))?
                    .with_timezone(&chrono::Utc),
                updated_at: DateTime::from_timestamp_millis(updated_at * 1000)
                    .ok_or_else(|| anyhow!("Invalid timestamp"))?
                    .with_timezone(&chrono::Utc),
            };

            entries.push(entry);
        }

        Ok(entries)
    }
}

impl LanceDbMemoryStore {
    fn db_memory_to_record(
        memory: &DbMemory,
        embedding_dim: usize,
    ) -> Result<arrow_array::RecordBatch> {
        use arrow_array::builder::ArrayBuilder;
        use arrow_array::{Float32Array, Int64Array, RecordBatch, StringArray};
        use arrow_schema::{DataType, Field, Schema};

        let embedding_array = arrow_array::FixedSizeListArray::new(
            Arc::new(Field::new("item", DataType::Float32, true)),
            embedding_dim as i32,
            Arc::new(Float32Array::from_iter_values(
                memory.embedding.iter().cloned(),
            )) as arrow_array::ArrayRef,
            None,
        );

        let schema = Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("agent_id", DataType::Utf8, false),
            Field::new("content", DataType::Utf8, false),
            Field::new("source", DataType::Utf8, false),
            Field::new("session_key", DataType::Utf8, true),
            Field::new("tags", DataType::Utf8, false),
            Field::new("importance", DataType::Float64, false),
            Field::new("metadata", DataType::Utf8, false),
            Field::new("created_at", DataType::Int64, false),
            Field::new("updated_at", DataType::Int64, false),
            Field::new(
                "embedding",
                DataType::FixedSizeList(
                    Arc::new(Field::new("item", DataType::Float32, true)),
                    embedding_dim as i32,
                ),
                true,
            ),
        ]);

        let tags_json = serde_json::to_string(&memory.tags).unwrap_or_else(|_| "[]".to_string());
        let metadata_json =
            serde_json::to_string(&memory.metadata).unwrap_or_else(|_| "{}".to_string());

        RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(StringArray::from(vec![memory.id.as_str()])),
                Arc::new(StringArray::from(vec![memory.agent_id.as_str()])),
                Arc::new(StringArray::from(vec![memory.content.as_str()])),
                Arc::new(StringArray::from(vec![memory.source.as_str()])),
                // Use StringArray::from for nullable strings (Vec<Option<&str>>)
                Arc::new(StringArray::from(vec![memory
                    .session_key
                    .as_ref()
                    .map(|s| s.as_str())])),
                Arc::new(StringArray::from(vec![tags_json.as_str()])),
                Arc::new(arrow_array::Float64Array::from(vec![memory.importance])),
                Arc::new(StringArray::from(vec![metadata_json.as_str()])),
                Arc::new(Int64Array::from(vec![memory.created_at])),
                Arc::new(Int64Array::from(vec![memory.updated_at])),
                Arc::new(embedding_array),
            ],
        )
        .map_err(|e| anyhow!("Failed to create record batch: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_lancedb_store_new() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_lancedb");
        let store = LanceDbMemoryStore::new(db_path.to_str().unwrap(), 1536).await;
        assert!(store.is_ok());
    }

    #[tokio::test]
    async fn test_lancedb_store_store_and_query() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_lancedb");
        let store = LanceDbMemoryStore::new(db_path.to_str().unwrap(), 4)
            .await
            .unwrap();

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
    async fn test_lancedb_store_delete() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_lancedb");
        let store = LanceDbMemoryStore::new(db_path.to_str().unwrap(), 4)
            .await
            .unwrap();

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
