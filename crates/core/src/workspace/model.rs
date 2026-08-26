use serde::Serialize;

pub type WorkspaceId = String;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub name: String,
    pub description: Option<String>,
    /// Command used to open member repos on `dm workspace switch`, e.g.
    /// `"code -n"`. Falls back to the global `editor` config when unset.
    pub editor: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct NewWorkspace {
    pub id: WorkspaceId,
    pub name: String,
    pub description: Option<String>,
    pub editor: Option<String>,
}
