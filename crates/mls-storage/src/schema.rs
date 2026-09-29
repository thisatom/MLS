//! Database schema definitions

/// SQL for creating vaults table
pub const CREATE_VAULTS_TABLE: &str = r" 
    CREATE TABLE IF NOT EXISTS vaults (
        id TEXT PRIMARY KEY NOT NULL,
        encrypted_metadata BLOB NOT NULL,
        created_at TEXT NOT NULL
    )
";

/// SQL for creating items table
pub const CREATE_ITEMS_TABLE: &str = r" 
    CREATE TABLE IF NOT EXISTS items (
        id TEXT PRIMARY KEY NOT NULL,
        vault_id TEXT NOT NULL,
        encrypted_blob BLOB NOT NULL,
        version INTEGER NOT NULL,
        updated_at TEXT NOT NULL,
        FOREIGN KEY (vault_id) REFERENCES vaults(id)
    )
";

/// SQL for creating `sync_state` table
pub const CREATE_SYNC_STATE_TABLE: &str = r" 
    CREATE TABLE IF NOT EXISTS sync_state (
        id TEXT PRIMARY KEY NOT NULL,
        vault_id TEXT NOT NULL,
        state_vector BLOB NOT NULL,
        updated_at TEXT NOT NULL,
        FOREIGN KEY (vault_id) REFERENCES vaults(id)
    )
";

/// Initialize database schema
///
/// # Errors
/// Returns an error if any of the SQL statements fail to execute.
pub fn init_schema(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    conn.execute(CREATE_VAULTS_TABLE, [])?;
    conn.execute(CREATE_ITEMS_TABLE, [])?;
    conn.execute(CREATE_SYNC_STATE_TABLE, [])?;
    Ok(())
}
