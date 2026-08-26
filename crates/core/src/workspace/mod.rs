//! The workspace domain model: a named, non-destructive collection of repo
//! references (never physical moves), with optional per-workspace editor
//! and environment variables for `dm workspace switch`.

mod model;
mod store;

pub use model::{NewWorkspace, Workspace, WorkspaceId};
pub use store::WorkspaceStore;
