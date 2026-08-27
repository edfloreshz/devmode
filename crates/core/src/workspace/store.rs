use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::error::{Error, Result};
use crate::paths;
use crate::registry::RepoId;

use super::model::{NewWorkspace, Workspace};

pub struct WorkspaceStore {
    conn: Connection,
}

impl WorkspaceStore {
    pub fn open(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        crate::db::init_schema(&conn)?;
        Ok(Self { conn })
    }

    /// Opens the workspace store at the standard devmode data directory —
    /// the same SQLite file `RegistryStore` uses, since workspace
    /// membership has a foreign key into `repos`.
    pub fn open_default() -> Result<Self> {
        Self::open(&paths::registry_db_file()?)
    }

    pub fn create(&self, new: NewWorkspace) -> Result<Workspace> {
        if self.find(&new.id)?.is_some() {
            return Err(Error::WorkspaceAlreadyExists(new.id));
        }
        let now = now_unix();
        self.conn.execute(
            "INSERT INTO workspaces (id, name, description, editor, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![new.id, new.name, new.description, new.editor, now],
        )?;
        self.get(&new.id)
    }

    pub fn get(&self, id: &str) -> Result<Workspace> {
        self.find(id)?
            .ok_or_else(|| Error::WorkspaceNotFound(id.to_string()))
    }

    fn find(&self, id: &str) -> Result<Option<Workspace>> {
        self.conn
            .query_row(
                "SELECT * FROM workspaces WHERE id = ?1",
                params![id],
                row_to_workspace,
            )
            .optional()
            .map_err(Error::from)
    }

    pub fn list(&self) -> Result<Vec<Workspace>> {
        let mut stmt = self.conn.prepare("SELECT * FROM workspaces ORDER BY id")?;
        let rows = stmt.query_map([], row_to_workspace)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Error::from)
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let changed = self
            .conn
            .execute("DELETE FROM workspaces WHERE id = ?1", params![id])?;
        if changed == 0 {
            return Err(Error::WorkspaceNotFound(id.to_string()));
        }
        Ok(())
    }

    pub fn get_config(&self, id: &str, key: &str) -> Result<String> {
        let ws = self.get(id)?;
        match key {
            "name" => Ok(ws.name),
            "description" => Ok(ws.description.unwrap_or_default()),
            "editor" => Ok(ws.editor.unwrap_or_default()),
            other => Err(Error::UnknownConfigKey(other.to_string())),
        }
    }

    pub fn set_config(&self, id: &str, key: &str, value: &str) -> Result<()> {
        self.get(id)?; // ensure it exists first, for a clear error
        let column = match key {
            "name" => "name",
            "description" => "description",
            "editor" => "editor",
            other => return Err(Error::UnknownConfigKey(other.to_string())),
        };
        self.conn.execute(
            &format!("UPDATE workspaces SET {column} = ?1 WHERE id = ?2"),
            params![value, id],
        )?;
        Ok(())
    }

    pub fn add_member(&self, workspace_id: &str, repo_id: RepoId) -> Result<()> {
        self.get(workspace_id)?;
        let already_member: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM workspace_members WHERE workspace_id = ?1 AND repo_id = ?2",
            params![workspace_id, repo_id],
            |row| row.get(0),
        )?;
        if already_member > 0 {
            return Err(Error::AlreadyInWorkspace {
                workspace: workspace_id.to_string(),
                repo: repo_id.to_string(),
            });
        }
        self.conn.execute(
            "INSERT INTO workspace_members (workspace_id, repo_id, position)
             VALUES (?1, ?2, (SELECT COALESCE(MAX(position) + 1, 0) FROM workspace_members WHERE workspace_id = ?1))",
            params![workspace_id, repo_id],
        )?;
        Ok(())
    }

    pub fn remove_member(&self, workspace_id: &str, repo_id: RepoId) -> Result<()> {
        let changed = self.conn.execute(
            "DELETE FROM workspace_members WHERE workspace_id = ?1 AND repo_id = ?2",
            params![workspace_id, repo_id],
        )?;
        if changed == 0 {
            return Err(Error::NotInWorkspace {
                workspace: workspace_id.to_string(),
                repo: repo_id.to_string(),
            });
        }
        Ok(())
    }

    pub fn members(&self, workspace_id: &str) -> Result<Vec<RepoId>> {
        let mut stmt = self.conn.prepare(
            "SELECT repo_id FROM workspace_members WHERE workspace_id = ?1 ORDER BY position",
        )?;
        let rows = stmt.query_map(params![workspace_id], |row| row.get(0))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Error::from)
    }

    /// Workspaces that a given repo belongs to — used by `dm repo remove`
    /// to warn before untracking a repo that's still a workspace member.
    pub fn workspaces_containing(&self, repo_id: RepoId) -> Result<Vec<Workspace>> {
        let mut stmt = self.conn.prepare(
            "SELECT w.* FROM workspaces w
             JOIN workspace_members m ON m.workspace_id = w.id
             WHERE m.repo_id = ?1
             ORDER BY w.id",
        )?;
        let rows = stmt.query_map(params![repo_id], row_to_workspace)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Error::from)
    }

    pub fn env_set(&self, workspace_id: &str, key: &str, value: &str) -> Result<()> {
        self.get(workspace_id)?;
        self.conn.execute(
            "INSERT INTO workspace_env (workspace_id, key, value) VALUES (?1, ?2, ?3)
             ON CONFLICT(workspace_id, key) DO UPDATE SET value = excluded.value",
            params![workspace_id, key, value],
        )?;
        Ok(())
    }

    /// Idempotent — unsetting a key that isn't set is not an error, matching
    /// how shell `unset` behaves.
    pub fn env_unset(&self, workspace_id: &str, key: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM workspace_env WHERE workspace_id = ?1 AND key = ?2",
            params![workspace_id, key],
        )?;
        Ok(())
    }

    pub fn env_list(&self, workspace_id: &str) -> Result<Vec<(String, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT key, value FROM workspace_env WHERE workspace_id = ?1 ORDER BY key")?;
        let rows = stmt.query_map(params![workspace_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Error::from)
    }
}

fn row_to_workspace(row: &Row) -> rusqlite::Result<Workspace> {
    Ok(Workspace {
        id: row.get("id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        editor: row.get("editor")?,
        created_at: row.get("created_at")?,
    })
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
