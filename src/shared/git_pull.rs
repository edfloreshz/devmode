use git2::Error as GitError;
use git2::ErrorCode;
use git2::Repository;
use git2::SubmoduleIgnore;
use std::io;
use std::io::Write;
use std::path::Path;

use crate::error::Error;
use crate::DevmodeError;

pub fn pull(repo_path: &Path) -> Result<(), Error> {
    let remote_name = "origin";
    let repo = Repository::open(repo_path)?;
    let remote_branch = get_branch(&repo)?;
    let remote_branch = remote_branch.as_str();
    let mut remote = repo.find_remote(remote_name)?;
    let fetch_commit = fetch(&repo, &[remote_branch], &mut remote)?;
    merge(&repo, remote_branch, fetch_commit)?;

    Ok(())
}

fn merge<'a>(
    repo: &'a Repository,
    remote_branch: &str,
    fetch_commit: git2::AnnotatedCommit<'a>,
) -> Result<(), GitError> {
    let analysis = repo.merge_analysis(&[&fetch_commit])?;
    if analysis.0.is_fast_forward() {
        println!("Doing fast forward");
        let refname = format!("refs/heads/{}", remote_branch);
        match repo.find_reference(&refname) {
            Ok(mut r) => {
                fast_forward(repo, &mut r, &fetch_commit)?;
            }
            Err(_) => {
                repo.reference(
                    &refname,
                    fetch_commit.id(),
                    true,
                    &format!("Setting {} to {}", remote_branch, fetch_commit.id()),
                )?;
                repo.set_head(&refname)?;
                repo.checkout_head(Some(
                    git2::build::CheckoutBuilder::default()
                        .allow_conflicts(true)
                        .conflict_style_merge(true)
                        .force(),
                ))?;
            }
        }
    } else if analysis.0.is_normal() {
        let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
        normal_merge(repo, &head_commit, &fetch_commit)?;
    } else {
        println!("Nothing to do...");
    }

    Ok(())
}

fn normal_merge(
    repo: &Repository,
    local: &git2::AnnotatedCommit,
    remote: &git2::AnnotatedCommit,
) -> Result<(), GitError> {
    let local_tree = repo.find_commit(local.id())?.tree()?;
    let remote_tree = repo.find_commit(local.id())?.tree()?;
    let ancestor = repo
        .find_commit(repo.merge_base(local.id(), remote.id())?)?
        .tree()?;
    let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

    if idx.has_conflicts() {
        println!("Merge conficts detected...");
        repo.checkout_index(Some(&mut idx), None)?;
        return Ok(());
    }
    let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;
    let msg = format!("Merge: {} info {}", remote.id(), local.id());
    let sig = repo.signature()?;
    let local_commit = repo.find_commit(local.id())?;
    let remote_commit = repo.find_commit(remote.id())?;
    let _merge_comit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &result_tree,
        &[&local_commit, &remote_commit],
    );
    repo.checkout_head(None)?;

    Ok(())
}

fn fast_forward(
    repo: &Repository,
    lb: &mut git2::Reference,
    rc: &git2::AnnotatedCommit,
) -> Result<(), GitError> {
    let name = match lb.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
    };
    let msg = format!("Fast-Forward: Setting {} to id: {}", name, rc.id());
    println!("{}", msg);
    lb.set_target(rc.id(), &msg)?;
    repo.set_head(&name)?;
    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
    Ok(())
}

fn get_branch(repo: &Repository) -> Result<String, Error> {
    let head = match repo.head() {
        Ok(head) => Some(head),
        Err(ref e) if e.code() == ErrorCode::UnbornBranch || e.code() == ErrorCode::NotFound => {
            None
        }
        Err(e) => return Err(Error::Git(e)),
    };
    let head = head.as_ref().and_then(|h| h.shorthand());
    match head {
        Some(branch) => Ok(String::from(branch)),
        None => Err(Error::Devmode(DevmodeError::FailedToGetBranch)),
    }
}

fn fetch<'a>(
    repo: &'a git2::Repository,
    refs: &[&str],
    remote: &'a mut git2::Remote,
) -> Result<git2::AnnotatedCommit<'a>, GitError> {
    let mut cb = git2::RemoteCallbacks::new();

    cb.transfer_progress(|stats| {
        if stats.received_objects() == stats.total_objects() {
            print!(
                "Resolving deltas {}/{}\r",
                stats.indexed_deltas(),
                stats.total_deltas()
            );
        } else if stats.total_objects() > 0 {
            print!(
                "Received {}/{} objects ({}) in {} bytes\r",
                stats.received_objects(),
                stats.total_objects(),
                stats.indexed_deltas(),
                stats.received_bytes()
            );
        }
        io::stdout().flush().unwrap();
        true
    });

    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(cb);
    fo.download_tags(git2::AutotagOption::All);
    println!("Fetching {} for repo", remote.name().unwrap());
    remote.fetch(refs, Some(&mut fo), None)?;

    let stats = remote.stats();
    if stats.local_objects() > 0 {
        println!(
            "\rReceived {}/{} objects in {} bytes (used {} local \
             objects)",
            stats.indexed_objects(),
            stats.total_objects(),
            stats.received_bytes(),
            stats.local_objects()
        );
    } else {
        println!(
            "\rReceived {}/{} objects in {} bytes",
            stats.indexed_objects(),
            stats.total_objects(),
            stats.received_bytes()
        );
    }

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    repo.reference_to_annotated_commit(&fetch_head)
}

pub fn status_short(path: String) -> Result<(), GitError> {
    let repo = Repository::open(&path)?;
    if repo.is_bare() {
        return Err(GitError::from_str(
            "Cannot report status on bare repository.",
        ));
    }
    let statuses = repo.statuses(None)?;

    let branch = get_branch(&repo);
    log::info!(
        "Branch: {}",
        branch.unwrap_or_else(|_| "HEAD (no branch)".to_string()),
    );

    let modules = repo.submodules()?;
    for sm in modules {
        log::info!(
            "Submodule '{}' at {}",
            sm.name().unwrap(),
            sm.path().display()
        );
    }

    for entry in statuses
        .iter()
        .filter(|e| e.status() != git2::Status::CURRENT)
    {
        let mut istatus = match entry.status() {
            s if s.contains(git2::Status::INDEX_NEW) => 'A',
            s if s.contains(git2::Status::INDEX_MODIFIED) => 'M',
            s if s.contains(git2::Status::INDEX_DELETED) => 'D',
            s if s.contains(git2::Status::INDEX_RENAMED) => 'R',
            s if s.contains(git2::Status::INDEX_TYPECHANGE) => 'T',
            _ => ' ',
        };
        let mut wstatus = match entry.status() {
            s if s.contains(git2::Status::WT_NEW) => {
                if istatus == ' ' {
                    istatus = '?';
                }
                '?'
            }
            s if s.contains(git2::Status::WT_MODIFIED) => 'M',
            s if s.contains(git2::Status::WT_DELETED) => 'D',
            s if s.contains(git2::Status::WT_RENAMED) => 'R',
            s if s.contains(git2::Status::WT_TYPECHANGE) => 'T',
            _ => ' ',
        };

        if entry.status().contains(git2::Status::IGNORED) {
            istatus = '!';
            wstatus = '!';
        }
        if istatus == '?' && wstatus == '?' {
            continue;
        }
        let mut extra = "";

        let status = entry.index_to_workdir().and_then(|diff| {
            let ignore = SubmoduleIgnore::Unspecified;
            diff.new_file()
                .path_bytes()
                .and_then(|s| std::str::from_utf8(s).ok())
                .and_then(|name| repo.submodule_status(name, ignore).ok())
        });
        if let Some(status) = status {
            if status.contains(git2::SubmoduleStatus::WD_MODIFIED) {
                extra = " (new commits)";
            } else if status.contains(git2::SubmoduleStatus::WD_INDEX_MODIFIED)
                || status.contains(git2::SubmoduleStatus::WD_WD_MODIFIED)
            {
                extra = " (modified content)";
            } else if status.contains(git2::SubmoduleStatus::WD_UNTRACKED) {
                extra = " (untracked content)";
            }
        }

        let (mut a, mut b, mut c) = (None, None, None);
        if let Some(diff) = entry.head_to_index() {
            a = diff.old_file().path();
            b = diff.new_file().path();
        }
        if let Some(diff) = entry.index_to_workdir() {
            a = a.or_else(|| diff.old_file().path());
            b = b.or_else(|| diff.old_file().path());
            c = diff.new_file().path();
        }

        match (istatus, wstatus) {
            ('R', 'R') => println!(
                "RR {} {} {}{}",
                a.unwrap().display(),
                b.unwrap().display(),
                c.unwrap().display(),
                extra
            ),
            ('R', w) => println!(
                "R{} {} {}{}",
                w,
                a.unwrap().display(),
                b.unwrap().display(),
                extra
            ),
            (i, 'R') => println!(
                "{}R {} {}{}",
                i,
                a.unwrap().display(),
                c.unwrap().display(),
                extra
            ),
            (i, w) => println!("{}{} {}{}", i, w, a.unwrap().display(), extra),
        }
    }

    for entry in statuses
        .iter()
        .filter(|e| e.status() == git2::Status::WT_NEW)
    {
        println!(
            "?? {}",
            entry
                .index_to_workdir()
                .unwrap()
                .old_file()
                .path()
                .unwrap()
                .display()
        );
    }
    Ok(())
}
