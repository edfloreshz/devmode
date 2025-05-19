use std::collections::HashMap;
use std::fs;
use std::io::Write;

use std::path::{Path, PathBuf};

use crate::error::WorkspaceError;

pub fn create_workspace(workspace: &str) -> Result<PathBuf, WorkspaceError> {
    let ws_path = dirs::home_dir()
        .unwrap()
        .join("Developer")
        .join("Workspaces")
        .join(workspace);
    if ws_path.exists() {
        return Err(WorkspaceError::WorkspaceAlreadyExists);
    }
    fs::create_dir_all(&ws_path)?;
    Ok(ws_path)
}

pub fn assign_repo_to_workspace(
    original_path: &Path,
    repository: &str,
    workspace: &str,
) -> Result<(), WorkspaceError> {
    let ws_path = dirs::home_dir()
        .unwrap()
        .join("Developer")
        .join("Workspaces")
        .join(workspace)
        .join(repository);
    fs::rename(original_path, &ws_path)?;
    Ok(())
}

pub fn remove_repo_from_workspace(
    original_path: &Path,
    repository: &str,
    workspace: &str,
) -> Result<(), WorkspaceError> {
    let ws_path = dirs::home_dir()
        .unwrap()
        .join("Developer")
        .join("Workspaces")
        .join(workspace)
        .join(&repository);

    fs::rename(&ws_path, original_path)?;
    // Remove entry from .workspace.ron
    remove_repo_from_metadata(workspace, repository)?;
    Ok(())
}

// Remove a repository entry from the workspace metadata file
fn remove_repo_from_metadata(workspace: &str, repository: &str) -> std::io::Result<()> {
    let meta_path = workspace_metadata_path(workspace);
    if meta_path.exists() {
        let data = fs::read_to_string(&meta_path)?;
        let mut map: HashMap<String, String> = ron::from_str(&data).unwrap_or_default();
        map.remove(repository);
        let mut file = fs::File::create(&meta_path)?;
        file.write_all(ron::to_string(&map).unwrap().as_bytes())?;
    }
    Ok(())
}

// Helper to get the metadata file path for a workspace
pub fn workspace_metadata_path(workspace: &str) -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap()
        .join("Developer")
        .join("Workspaces")
        .join(workspace)
        .join(".workspace.ron")
}

// Helper to update metadata when assigning a repo (RON version)
pub fn update_workspace_metadata_on_assign(
    workspace: &str,
    repository: &str,
    original_path: &std::path::Path,
) -> std::io::Result<()> {
    let meta_path = workspace_metadata_path(workspace);
    let mut map: HashMap<String, String> = if meta_path.exists() {
        let data = fs::read_to_string(&meta_path)?;
        ron::from_str(&data).unwrap_or_default()
    } else {
        HashMap::new()
    };
    map.insert(
        repository.to_string(),
        original_path.to_string_lossy().to_string(),
    );
    let mut file = fs::File::create(&meta_path)?;
    file.write_all(ron::to_string(&map).unwrap().as_bytes())?;
    Ok(())
}

// Helper to get original path when removing a repo (RON version)
pub fn get_original_path_from_metadata(
    workspace: &str,
    repo_name: &str,
) -> Option<std::path::PathBuf> {
    let meta_path = workspace_metadata_path(workspace);
    if meta_path.exists() {
        if let Ok(data) = fs::read_to_string(&meta_path) {
            if let Ok(map) = ron::from_str::<HashMap<String, String>>(&data) {
                if let Some(orig) = map.get(repo_name) {
                    return Some(std::path::PathBuf::from(orig));
                }
            }
        }
    }
    None
}

// Finds existing repositories.
pub fn find_repos(
    initial_path: Option<&PathBuf>,
    repository: &str,
    matches: &mut Vec<std::path::PathBuf>,
) {
    let base_path = dirs::home_dir().unwrap().join("Developer");
    let initial_path = initial_path.unwrap_or(&base_path);

    if let Ok(entries) = std::fs::read_dir(initial_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().map_or(false, |n| n == repository) {
                    matches.push(path.clone());
                }
                find_repos(Some(&path), repository, matches);
            }
        }
    }
}
