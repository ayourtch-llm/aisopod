//! SQLite database module for session persistence.
//!
//! This module provides the database schema and migration system for storing
//! sessions and messages in a SQLite database. It includes functions to open
//! the database, run migrations, and manage the schema version.

use rusqlite::{params, Connection, Result};
use std::path::Path;

/// The current schema version.
const SCHEMA_VERSION: i64 = 1;

/// Opens or creates a SQLite database at the given path.
///
/// This function:
/// - Opens or creates the database file
/// - Enables WAL mode for better concurrency
/// - Enables foreign key support
/// - Runs all pending migrations
///
/// # Arguments
///
/// * `path` - The file system path where the database file should be stored.
///
/// # Returns
///
/// Returns a `Result` containing the `Connection` if successful, or an error.
pub fn open_database(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    
    // Enable WAL mode for better concurrency
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;
    
    // Enable foreign key support
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    
    // Run migrations
    run_migrations(&conn)?;
    
    Ok(conn)
}

/// Runs all pending database migrations.
///
/// This function:
/// - Creates the schema_version table if it doesn't exist
/// - Checks the current schema version
/// - Runs all unapplied migrations in order
/// - Updates the version number after each migration
///
/// # Arguments
///
/// * `conn` - The database connection to use for migrations.
///
/// # Returns
///
/// Returns `Ok(())` if all migrations succeed, or an error.
pub fn run_migrations(conn: &Connection) -> Result<()> {
    // Create schema_version table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY
        )",
        params![],
    )?;

    // Get current version
    let current_version: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version",
        params![],
        |row| row.get(0),
    ).unwrap_or(0);

    if current_version >= SCHEMA_VERSION {
        return Ok(());
    }

    // Define migrations in order
    let migrations = vec![
        create_tables_migration(),
        create_indexes_migration(),
    ];

    // Apply each migration
    for migration_sql in migrations {
        conn.execute_batch(migration_sql)?;
    }

    // Record the new version
    conn.execute(
        "INSERT INTO schema_version (version) VALUES (?)",
        params![SCHEMA_VERSION],
    )?;

    Ok(())
}

/// Returns the SQL statement to create the sessions and messages tables.
fn create_tables_migration() -> &'static str {
    r#"
    -- Create sessions table
    CREATE TABLE IF NOT EXISTS sessions (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        agent_id TEXT NOT NULL,
        channel TEXT NOT NULL,
        account_id TEXT NOT NULL,
        peer_kind TEXT NOT NULL,
        peer_id TEXT NOT NULL,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        message_count INTEGER NOT NULL DEFAULT 0,
        token_usage INTEGER NOT NULL DEFAULT 0,
        metadata TEXT NOT NULL DEFAULT '{}',
        status TEXT NOT NULL DEFAULT 'active',
        UNIQUE(agent_id, channel, account_id, peer_kind, peer_id)
    );

    -- Create messages table
    CREATE TABLE IF NOT EXISTS messages (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
        role TEXT NOT NULL,
        content TEXT NOT NULL,
        tool_calls TEXT,
        created_at TEXT NOT NULL
    );
    "#
}

/// Returns the SQL statements to create all indexes.
fn create_indexes_migration() -> &'static str {
    r#"
    -- Indexes for sessions table
    CREATE INDEX IF NOT EXISTS idx_sessions_agent_id ON sessions(agent_id);
    CREATE INDEX IF NOT EXISTS idx_sessions_channel ON sessions(channel, account_id);
    CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);
    CREATE INDEX IF NOT EXISTS idx_sessions_updated_at ON sessions(updated_at);

    -- Indexes for messages table
    CREATE INDEX IF NOT EXISTS idx_messages_session_id ON messages(session_id);
    CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(session_id, created_at);
    "#}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    /// Creates a fresh in-memory database for testing.
    fn create_test_database() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        
        // Enable foreign keys for testing
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        
        conn
    }

    #[test]
    fn test_create_tables_migration() {
        let conn = create_test_database();
        
        // Apply the tables migration
        conn.execute_batch(create_tables_migration()).unwrap();

        // Verify sessions table exists and has correct structure
        let sessions_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='sessions')",
                params![],
                |row| row.get(0),
            )
            .unwrap();
        assert!(sessions_exists);

        // Verify messages table exists
        let messages_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='messages')",
                params![],
                |row| row.get(0),
            )
            .unwrap();
        assert!(messages_exists);

        // Verify sessions table has the correct columns
        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(sessions)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        
        assert!(columns.contains(&"id".to_string()));
        assert!(columns.contains(&"agent_id".to_string()));
        assert!(columns.contains(&"channel".to_string()));
        assert!(columns.contains(&"account_id".to_string()));
        assert!(columns.contains(&"peer_kind".to_string()));
        assert!(columns.contains(&"peer_id".to_string()));
        assert!(columns.contains(&"created_at".to_string()));
        assert!(columns.contains(&"updated_at".to_string()));
        assert!(columns.contains(&"message_count".to_string()));
        assert!(columns.contains(&"token_usage".to_string()));
        assert!(columns.contains(&"metadata".to_string()));
        assert!(columns.contains(&"status".to_string()));

        // Verify messages table has the correct columns
        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(messages)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        
        assert!(columns.contains(&"id".to_string()));
        assert!(columns.contains(&"session_id".to_string()));
        assert!(columns.contains(&"role".to_string()));
        assert!(columns.contains(&"content".to_string()));
        assert!(columns.contains(&"tool_calls".to_string()));
        assert!(columns.contains(&"created_at".to_string()));
    }

    #[test]
    fn test_create_indexes_migration() {
        let conn = create_test_database();
        
        // First create the tables
        conn.execute_batch(create_tables_migration()).unwrap();
        
        // Then create the indexes
        conn.execute_batch(create_indexes_migration()).unwrap();

        // Verify indexes exist
        let index_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='index' AND name='idx_sessions_agent_id')",
                params![],
                |row| row.get(0),
            )
            .unwrap();
        assert!(index_exists);

        let index_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='index' AND name='idx_sessions_channel')",
                params![],
                |row| row.get(0),
            )
            .unwrap();
        assert!(index_exists);

        let index_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='index' AND name='idx_sessions_status')",
                params![],
                |row| row.get(0),
            )
            .unwrap();
        assert!(index_exists);

        let index_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='index' AND name='idx_sessions_updated_at')",
                params![],
                |row| row.get(0),
            )
            .unwrap();
        assert!(index_exists);

        let index_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='index' AND name='idx_messages_session_id')",
                params![],
                |row| row.get(0),
            )
            .unwrap();
        assert!(index_exists);

        let index_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='index' AND name='idx_messages_created_at')",
                params![],
                |row| row.get(0),
            )
            .unwrap();
        assert!(index_exists);
    }

    #[test]
    fn test_schema_version_table_created() {
        let conn = create_test_database();
        
        // Run migrations (which includes creating schema_version table)
        run_migrations(&conn).unwrap();

        // Verify schema_version table exists
        let table_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='schema_version')",
                params![],
                |row| row.get(0),
            )
            .unwrap();
        assert!(table_exists);
    }

    #[test]
    fn test_run_migrations_is_idempotent() {
        let conn = create_test_database();
        
        // Run migrations twice
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();

        // Verify schema_version is 1
        let version: i64 = conn
            .query_row(
                "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
                params![],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn test_foreign_key_constraint() {
        let conn = create_test_database();
        
        // Run migrations
        run_migrations(&conn).unwrap();

        // Insert a session
        conn.execute(
            "INSERT INTO sessions (agent_id, channel, account_id, peer_kind, peer_id, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                "agent_001",
                "discord",
                "bot_123",
                "dm",
                "user_456",
                "2024-01-01T00:00:00Z",
                "2024-01-01T00:00:00Z",
            ],
        ).unwrap();

        // Insert a message for that session
        conn.execute(
            "INSERT INTO messages (session_id, role, content, created_at) VALUES (?, ?, ?, ?)",
            params![1, "user", "Hello", "2024-01-01T00:00:01Z"],
        ).unwrap();

        // Verify the message was inserted
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM messages", params![], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        // Delete the session - this should cascade to delete the message due to ON DELETE CASCADE
        conn.execute("DELETE FROM sessions WHERE id = ?", params![1]).unwrap();

        // Verify the message was deleted
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM messages", params![], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_unique_constraint_on_sessions() {
        let conn = create_test_database();
        
        // Run migrations
        run_migrations(&conn).unwrap();

        // Insert a session
        conn.execute(
            "INSERT INTO sessions (agent_id, channel, account_id, peer_kind, peer_id, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                "agent_001",
                "discord",
                "bot_123",
                "dm",
                "user_456",
                "2024-01-01T00:00:00Z",
                "2024-01-01T00:00:00Z",
            ],
        ).unwrap();

        // Try to insert a duplicate session - should fail
        let result = conn.execute(
            "INSERT INTO sessions (agent_id, channel, account_id, peer_kind, peer_id, created_at, updated_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                "agent_001",
                "discord",
                "bot_123",
                "dm",
                "user_456",
                "2024-01-01T00:00:00Z",
                "2024-01-01T00:00:00Z",
            ],
        );
        assert!(result.is_err());
    }
}
