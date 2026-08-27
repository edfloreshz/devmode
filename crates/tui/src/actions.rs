use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use dm_core::config::Config;
use dm_core::error::{Error as CoreError, Result as CoreResult};
use dm_core::git;
use dm_core::paths;
use dm_core::registry::{NewRepo, RegistryStore, RepoId};
use dm_core::workspace::WorkspaceStore;

use crate::error::Result;

/// Result of a background `dm_core::git::clone`, sent back to the main loop
/// over an mpsc channel so the UI thread never blocks on the network.
pub enum CloneOutcome {
    Ok(String),
    Err(String),
}

pub fn spawn_clone(url: String, path: Option<PathBuf>) -> mpsc::Receiver<CloneOutcome> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let outcome = match clone_and_track(&url, path) {
            Ok(name) => CloneOutcome::Ok(name),
            Err(e) => CloneOutcome::Err(e.to_string()),
        };
        let _ = tx.send(outcome);
    });
    rx
}

fn clone_and_track(url: &str, path: Option<PathBuf>) -> CoreResult<String> {
    let parsed = git::parse_url(url)?;
    let config = Config::load()?;
    let store = RegistryStore::open_default()?;

    let dest = match path {
        Some(path) => path,
        None => config.repo.root.join(config.repo.layout.render(
            &parsed.host,
            &parsed.owner,
            &parsed.name,
        )),
    };
    let dest = paths::normalize_path(&dest);

    if dest.exists() {
        return Err(CoreError::DestinationExists(dest));
    }

    git::clone(url, &dest)?;

    let repo = store.track(NewRepo {
        path: dest,
        name: parsed.name,
        remote_url: Some(url.to_string()),
        host: Some(parsed.host),
        owner: Some(parsed.owner),
        tags: Vec::new(),
    })?;
    Ok(repo.name)
}

pub fn create_repo(name: String, path: Option<PathBuf>, no_git: bool) -> Result<String> {
    let config = Config::load()?;
    let store = RegistryStore::open_default()?;

    let dest = path.unwrap_or_else(|| config.repo.root.join("local").join(&name));
    let dest = paths::normalize_path(&dest);

    if dest.exists() {
        return Err(CoreError::DestinationExists(dest).into());
    }

    if no_git {
        std::fs::create_dir_all(&dest).map_err(CoreError::from)?;
    } else {
        git::init(&dest)?;
    }

    let repo = store.track(NewRepo {
        path: dest,
        name,
        ..Default::default()
    })?;
    Ok(repo.name)
}

pub fn track_repo(path: PathBuf) -> Result<String> {
    if !path.is_dir() {
        return Err(CoreError::NotADirectory(path).into());
    }
    let path = paths::normalize_path(&path);
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());

    let store = RegistryStore::open_default()?;
    let repo = store.track(NewRepo {
        path,
        name,
        ..Default::default()
    })?;
    Ok(repo.name)
}

pub fn remove_repo(registry: &RegistryStore, id: RepoId, path: &Path, delete: bool) -> Result<()> {
    if delete {
        std::fs::remove_dir_all(path).map_err(CoreError::from)?;
    }
    registry.remove(id)?;
    Ok(())
}

/// Spawns the workspace's editor (or the global `editor` config) with every
/// member repo's path as an argument and the workspace's env vars applied,
/// blocking until it exits, same behavior as `dm workspace switch`. Meant
/// to run after the TUI has already restored the terminal, since this takes
/// over stdio for the duration of the editor process.
pub fn switch_workspace(id: &str) -> Result<()> {
    let registry = RegistryStore::open_default()?;
    let workspaces = WorkspaceStore::open_default()?;
    let ws = workspaces.get(id)?;

    let repos = workspaces
        .members(&ws.id)?
        .into_iter()
        .map(|repo_id| registry.get(repo_id))
        .collect::<CoreResult<Vec<_>>>()?;

    if repos.is_empty() {
        println!(
            "workspace '{}' has no members yet, add one with `dm workspace add {} <repo>`",
            ws.id, ws.id
        );
        return Ok(());
    }

    let global_config = Config::load()?;
    let editor = ws.editor.clone().or(global_config.editor.clone());
    let Some(editor) = editor else {
        println!(
            "no editor configured for '{}' (or globally), member repos:",
            ws.id
        );
        for repo in &repos {
            println!("  {}", repo.path.display());
        }
        println!(
            "\nset one with `dm workspace config set {} editor <cmd>` or `dm config set editor <cmd>`",
            ws.id
        );
        return Ok(());
    };

    let mut parts = editor.split_whitespace();
    let Some(program) = parts.next() else {
        println!("editor command for '{}' is empty", ws.id);
        return Ok(());
    };

    let mut command = std::process::Command::new(program);
    command.args(parts);
    for repo in &repos {
        command.arg(&repo.path);
    }
    for (key, value) in workspaces.env_list(&ws.id)? {
        command.env(key, value);
    }

    let status = command.status().map_err(CoreError::from)?;
    if !status.success() {
        eprintln!("'{editor}' exited with a non-zero status");
    }
    Ok(())
}
