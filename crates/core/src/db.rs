use rusqlite::Connection;

use crate::error::Result;

/// Creates every table devmode's stores use, if they don't already exist.
/// `RegistryStore` and `WorkspaceStore` each open their own `Connection` to
/// the same underlying SQLite file, so either one can be opened first
/// without the other's tables — or the foreign keys between them — missing.
pub(crate) fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;

         CREATE TABLE IF NOT EXISTS repos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            remote_url TEXT,
            host TEXT,
            owner TEXT,
            tags TEXT NOT NULL DEFAULT '',
            added_at INTEGER NOT NULL,
            last_opened_at INTEGER
         );

         CREATE TABLE IF NOT EXISTS workspaces (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            editor TEXT,
            created_at INTEGER NOT NULL
         );

         CREATE TABLE IF NOT EXISTS workspace_members (
            workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
            repo_id INTEGER NOT NULL REFERENCES repos(id) ON DELETE CASCADE,
            position INTEGER NOT NULL,
            PRIMARY KEY (workspace_id, repo_id)
         );

         CREATE TABLE IF NOT EXISTS workspace_env (
            workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
            key TEXT NOT NULL,
            value TEXT NOT NULL,
            PRIMARY KEY (workspace_id, key)
         );",
    )?;
    Ok(())
}
