//! Reads a repo's current git state — branch, upstream tracking, working
//! tree, last commit — for display, without shelling out to `git`.

use std::path::Path;
use std::time::{Duration, SystemTime};

use git2::{BranchType, Repository, Status, StatusOptions};

use crate::error::Result;

/// A snapshot of a repo's git state at the moment it was read.
#[derive(Debug, Clone)]
pub struct RepoStatus {
    /// `None` for a detached `HEAD` or a repo with no commits yet.
    pub branch: Option<String>,
    pub detached: bool,
    /// Commits ahead of / behind the branch's upstream, if it has one.
    pub ahead: Option<usize>,
    pub behind: Option<usize>,
    pub staged: usize,
    pub modified: usize,
    pub untracked: usize,
    pub last_commit: Option<CommitSummary>,
    pub stash_count: usize,
    pub tag_count: usize,
}

impl RepoStatus {
    pub fn is_clean(&self) -> bool {
        self.staged == 0 && self.modified == 0 && self.untracked == 0
    }
}

#[derive(Debug, Clone)]
pub struct CommitSummary {
    /// The first 7 hex characters of the commit id.
    pub short_id: String,
    pub summary: String,
    pub author: String,
    pub when: SystemTime,
}

/// Cheaply checks whether a repo's working tree has uncommitted changes —
/// staged, modified, or untracked. Meant for running across every tracked
/// repo at once (e.g. a dirty indicator per row in a list), so it skips the
/// extra commit lookups and ahead/behind graph walk `repo_status` does.
/// Not a git repo, or any other error reading it, reads as clean.
pub fn is_dirty(path: &Path) -> bool {
    let Ok(repo) = Repository::open(path) else {
        return false;
    };

    let mut options = StatusOptions::new();
    options.include_untracked(true);

    repo.statuses(Some(&mut options))
        .is_ok_and(|statuses| !statuses.is_empty())
}

/// Reads git state for the repo at `path`. Fails only if `path` isn't a git
/// repo at all; an empty repo or one with no upstream just reports fewer
/// fields.
pub fn repo_status(path: &Path) -> Result<RepoStatus> {
    let repo = Repository::open(path)?;

    // `head()` errors on an unborn branch (no commits yet), so the branch
    // name is read from the symbolic ref directly instead — that still
    // resolves on a brand new repo.
    let detached = repo.head_detached().unwrap_or(false);
    let branch = if detached {
        None
    } else {
        repo.find_reference("HEAD")
            .ok()
            .and_then(|head| head.symbolic_target().map(str::to_string))
            .and_then(|target| target.strip_prefix("refs/heads/").map(str::to_string))
    };

    let head_commit = repo.head().ok().and_then(|head| head.peel_to_commit().ok());

    let (ahead, behind) = branch
        .as_deref()
        .and_then(|name| repo.find_branch(name, BranchType::Local).ok())
        .and_then(|local| local.upstream().ok())
        .and_then(|upstream| upstream.get().target())
        .zip(head_commit.as_ref().map(|commit| commit.id()))
        .and_then(|(upstream_oid, local_oid)| repo.graph_ahead_behind(local_oid, upstream_oid).ok())
        .map_or((None, None), |(ahead, behind)| (Some(ahead), Some(behind)));

    let mut options = StatusOptions::new();
    options.include_untracked(true);
    let statuses = repo.statuses(Some(&mut options))?;

    let mut staged = 0;
    let mut modified = 0;
    let mut untracked = 0;

    for entry in statuses.iter() {
        let status = entry.status();

        if status.intersects(
            Status::INDEX_NEW
                | Status::INDEX_MODIFIED
                | Status::INDEX_DELETED
                | Status::INDEX_RENAMED
                | Status::INDEX_TYPECHANGE,
        ) {
            staged += 1;
        }

        if status.intersects(
            Status::WT_MODIFIED | Status::WT_DELETED | Status::WT_RENAMED | Status::WT_TYPECHANGE,
        ) {
            modified += 1;
        }

        if status.contains(Status::WT_NEW) {
            untracked += 1;
        }
    }

    let last_commit = head_commit.map(|commit| CommitSummary {
        short_id: commit.id().to_string()[..7].to_string(),
        summary: commit.summary().unwrap_or_default().to_string(),
        author: commit.author().name().unwrap_or("unknown").to_string(),
        when: SystemTime::UNIX_EPOCH + Duration::from_secs(commit.time().seconds().max(0) as u64),
    });

    // Neither has a dedicated count on `Repository`, so approximate: a
    // stash is one reflog entry on `refs/stash`, and tags are just the
    // count of matching refs.
    let stash_count = repo.reflog("refs/stash").map(|log| log.len()).unwrap_or(0);
    let tag_count = repo.tag_names(None).map(|tags| tags.len()).unwrap_or(0);

    Ok(RepoStatus {
        branch,
        detached,
        ahead,
        behind,
        staged,
        modified,
        untracked,
        last_commit,
        stash_count,
        tag_count,
    })
}
