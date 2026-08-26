use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::error::{Error, Result};
use crate::paths;

use super::model::{NewRepo, Repo, RepoId};

pub struct RegistryStore {
    conn: Connection,
}

impl RegistryStore {
    pub fn open(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        crate::db::init_schema(&conn)?;
        Ok(Self { conn })
    }

    /// Opens the registry at the standard devmode data directory.
    pub fn open_default() -> Result<Self> {
        Self::open(&paths::registry_db_file()?)
    }

    pub fn track(&self, repo: NewRepo) -> Result<Repo> {
        if self.find_by_path(&repo.path)?.is_some() {
            return Err(Error::AlreadyTracked(repo.path));
        }
        let path_str = repo.path.to_string_lossy().to_string();
        let tags = repo.tags.join(",");
        let now = now_unix();
        self.conn.execute(
            "INSERT INTO repos (path, name, remote_url, host, owner, tags, added_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                path_str,
                repo.name,
                repo.remote_url,
                repo.host,
                repo.owner,
                tags,
                now
            ],
        )?;
        self.get(self.conn.last_insert_rowid())
    }

    pub fn get(&self, id: RepoId) -> Result<Repo> {
        self.conn
            .query_row("SELECT * FROM repos WHERE id = ?1", params![id], row_to_repo)
            .optional()?
            .ok_or_else(|| Error::RepoNotFound(id.to_string()))
    }

    pub fn find_by_path(&self, path: &Path) -> Result<Option<Repo>> {
        let path_str = path.to_string_lossy().to_string();
        self.conn
            .query_row(
                "SELECT * FROM repos WHERE path = ?1",
                params![path_str],
                row_to_repo,
            )
            .optional()
            .map_err(Error::from)
    }

    /// Lists tracked repos, optionally filtered by tag and/or host.
    pub fn list(&self, tag: Option<&str>, host: Option<&str>) -> Result<Vec<Repo>> {
        let mut stmt = self.conn.prepare("SELECT * FROM repos ORDER BY name")?;
        let rows = stmt.query_map([], row_to_repo)?;
        let mut repos = Vec::new();
        for row in rows {
            let repo = row?;
            if let Some(tag) = tag {
                if !repo.tags.iter().any(|t| t == tag) {
                    continue;
                }
            }
            if let Some(host) = host {
                if repo.host.as_deref() != Some(host) {
                    continue;
                }
            }
            repos.push(repo);
        }
        Ok(repos)
    }

    /// All tracked repos matching a user-supplied `repo` argument: first by
    /// normalized path (exact), then by name (possibly several). Lets a
    /// caller offer interactive disambiguation instead of hard-erroring —
    /// see `find` for the non-interactive, single-match version.
    pub fn find_matches(&self, identifier: &str) -> Result<Vec<Repo>> {
        let normalized = crate::paths::normalize_path(Path::new(identifier));
        if let Some(repo) = self.find_by_path(&normalized)? {
            return Ok(vec![repo]);
        }
        Ok(self
            .list(None, None)?
            .into_iter()
            .filter(|repo| repo.name == identifier)
            .collect())
    }

    /// Resolves a user-supplied `repo` argument to a single tracked repo:
    /// first as a path (normalized and matched exactly), then as a name.
    pub fn find(&self, identifier: &str) -> Result<Repo> {
        let mut matches = self.find_matches(identifier)?.into_iter();
        match (matches.next(), matches.next()) {
            (Some(repo), None) => Ok(repo),
            (Some(_), Some(_)) => Err(Error::AmbiguousRepo(identifier.to_string())),
            (None, _) => Err(Error::RepoNotFound(identifier.to_string())),
        }
    }

    /// Updates a tracked repo's recorded path, e.g. after it was moved on
    /// disk (see `dm repo relayout`). Does not touch the filesystem itself.
    pub fn update_path(&self, id: RepoId, new_path: &Path) -> Result<()> {
        let path_str = new_path.to_string_lossy().to_string();
        let changed = self.conn.execute(
            "UPDATE repos SET path = ?1 WHERE id = ?2",
            params![path_str, id],
        )?;
        if changed == 0 {
            return Err(Error::RepoNotFound(id.to_string()));
        }
        Ok(())
    }

    /// Updates a tracked repo's recorded remote URL, e.g. after `dm repo
    /// sync` notices the on-disk `origin` no longer matches.
    pub fn update_remote(&self, id: RepoId, remote_url: &str) -> Result<()> {
        let changed = self.conn.execute(
            "UPDATE repos SET remote_url = ?1 WHERE id = ?2",
            params![remote_url, id],
        )?;
        if changed == 0 {
            return Err(Error::RepoNotFound(id.to_string()));
        }
        Ok(())
    }

    /// Updates a tracked repo's remote URL along with the host/owner parsed
    /// from it, so editing the remote also keeps layout checks accurate.
    pub fn update_remote_details(
        &self,
        id: RepoId,
        remote_url: &str,
        host: Option<&str>,
        owner: Option<&str>,
    ) -> Result<()> {
        let changed = self.conn.execute(
            "UPDATE repos SET remote_url = ?1, host = ?2, owner = ?3 WHERE id = ?4",
            params![remote_url, host, owner, id],
        )?;
        if changed == 0 {
            return Err(Error::RepoNotFound(id.to_string()));
        }
        Ok(())
    }

    pub fn remove(&self, id: RepoId) -> Result<()> {
        let changed = self
            .conn
            .execute("DELETE FROM repos WHERE id = ?1", params![id])?;
        if changed == 0 {
            return Err(Error::RepoNotFound(id.to_string()));
        }
        Ok(())
    }
}

fn row_to_repo(row: &Row) -> rusqlite::Result<Repo> {
    let tags_str: String = row.get("tags")?;
    Ok(Repo {
        id: row.get("id")?,
        path: PathBuf::from(row.get::<_, String>("path")?),
        name: row.get("name")?,
        remote_url: row.get("remote_url")?,
        host: row.get("host")?,
        owner: row.get("owner")?,
        tags: if tags_str.is_empty() {
            Vec::new()
        } else {
            tags_str.split(',').map(String::from).collect()
        },
        added_at: row.get("added_at")?,
        last_opened_at: row.get("last_opened_at")?,
    })
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
