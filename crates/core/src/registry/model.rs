use std::path::PathBuf;

use serde::Serialize;

pub type RepoId = i64;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Repo {
    pub id: RepoId,
    pub path: PathBuf,
    pub name: String,
    pub remote_url: Option<String>,
    pub host: Option<String>,
    pub owner: Option<String>,
    pub tags: Vec<String>,
    pub added_at: i64,
    pub last_opened_at: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct NewRepo {
    pub path: PathBuf,
    pub name: String,
    pub remote_url: Option<String>,
    pub host: Option<String>,
    pub owner: Option<String>,
    pub tags: Vec<String>,
}
