use std::path::Path;
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

fn dm(home: &Path) -> Command {
    let mut cmd = Command::cargo_bin("dm").unwrap();
    // `DEVMODE_HOME` isolates config + registry under the test's tempdir on
    // every platform; `HOME` alone doesn't on Windows.
    cmd.env("DEVMODE_HOME", home);
    cmd.env("HOME", home);
    cmd
}

fn init_repo(dir: &Path) {
    std::fs::create_dir_all(dir).unwrap();
    std::process::Command::new("git")
        .args(["init", "-q"])
        .arg(dir)
        .status()
        .unwrap();
}

#[test]
fn config_get_set_roundtrip() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("Developer");

    dm(tmp.path())
        .args(["config", "set", "repo.root", root.to_str().unwrap()])
        .assert()
        .success();
    dm(tmp.path())
        .args(["config", "get", "repo.layout"])
        .assert()
        .success()
        .stdout(predicate::str::contains("host_owner_repo"));
    dm(tmp.path())
        .args(["config", "get", "unknown_key"])
        .assert()
        .failure();
}

#[test]
fn track_list_show_remove() {
    let tmp = tempfile::tempdir().unwrap();
    let repo_dir = tmp.path().join("myrepo");
    init_repo(&repo_dir);

    dm(tmp.path())
        .args(["repo", "track", repo_dir.to_str().unwrap()])
        .assert()
        .success();
    dm(tmp.path())
        .args(["repo", "track", repo_dir.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already tracked"));
    dm(tmp.path())
        .args(["repo", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("myrepo"));
    dm(tmp.path())
        .args(["repo", "show", "myrepo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("name:   myrepo"));
    dm(tmp.path())
        .args(["repo", "remove", "myrepo"])
        .assert()
        .success();
    dm(tmp.path())
        .args(["repo", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("no repos tracked"));
}

#[test]
fn relayout_moves_a_misplaced_tracked_repo() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("Developer");
    dm(tmp.path())
        .args(["config", "set", "repo.root", root.to_str().unwrap()])
        .assert()
        .success();

    let repo_dir = tmp.path().join("misplaced/myrepo");
    init_repo(&repo_dir);
    dm(tmp.path())
        .args([
            "repo",
            "track",
            repo_dir.to_str().unwrap(),
            "--host",
            "example.com",
            "--owner",
            "acme",
        ])
        .assert()
        .success();

    dm(tmp.path())
        .args(["repo", "relayout"])
        .assert()
        .success()
        .stdout(predicate::str::contains("would move"));
    dm(tmp.path())
        .args(["repo", "relayout", "--apply", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("moved 1 repo"));

    assert!(root.join("example.com/acme/myrepo").is_dir());
    dm(tmp.path())
        .args(["repo", "relayout"])
        .assert()
        .success()
        .stdout(predicate::str::contains("already match"));
}

#[test]
fn scan_finds_and_tracks_an_untracked_repo() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("Developer");
    std::fs::create_dir_all(&root).unwrap();
    dm(tmp.path())
        .args(["config", "set", "repo.root", root.to_str().unwrap()])
        .assert()
        .success();

    init_repo(&root.join("found"));

    dm(tmp.path())
        .args(["repo", "scan", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tracked 1 new repo"));
    dm(tmp.path())
        .args(["repo", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("found"));
}

#[test]
fn workspace_lifecycle() {
    let tmp = tempfile::tempdir().unwrap();
    let repo_dir = tmp.path().join("wsrepo");
    init_repo(&repo_dir);
    dm(tmp.path())
        .args(["repo", "track", repo_dir.to_str().unwrap()])
        .assert()
        .success();

    dm(tmp.path())
        .args(["workspace", "create", "demo", "--name", "Demo"])
        .assert()
        .success();
    dm(tmp.path())
        .args(["workspace", "add", "demo", "wsrepo"])
        .assert()
        .success();
    dm(tmp.path())
        .args(["workspace", "show", "demo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("wsrepo"));
    dm(tmp.path())
        .args(["workspace", "switch", "demo", "--cd"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("cd "));
    dm(tmp.path())
        .args(["workspace", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("demo"));

    // Removing a repo that's still a member should refuse without --force.
    dm(tmp.path())
        .args(["repo", "remove", "wsrepo"])
        .assert()
        .failure();
    dm(tmp.path())
        .args(["repo", "remove", "wsrepo", "--force"])
        .assert()
        .success();

    dm(tmp.path())
        .args(["workspace", "delete", "demo", "--force"])
        .assert()
        .success();
    dm(tmp.path())
        .args(["workspace", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("no workspaces"));
}

#[test]
fn ambiguous_repo_name_requires_a_tty_to_disambiguate() {
    let tmp = tempfile::tempdir().unwrap();
    let a = tmp.path().join("a/dup");
    let b = tmp.path().join("b/dup");
    init_repo(&a);
    init_repo(&b);
    dm(tmp.path())
        .args(["repo", "track", a.to_str().unwrap()])
        .assert()
        .success();
    dm(tmp.path())
        .args(["repo", "track", b.to_str().unwrap()])
        .assert()
        .success();

    // Non-interactive test process has no TTY, so the disambiguation
    // prompt should fail cleanly rather than silently pick one.
    dm(tmp.path())
        .args(["repo", "show", "dup"])
        .assert()
        .failure();
}

#[test]
fn top_level_aliases_match_repo_subcommands() {
    let tmp = tempfile::tempdir().unwrap();
    let repo_dir = tmp.path().join("aliased");
    init_repo(&repo_dir);
    dm(tmp.path())
        .args(["repo", "track", repo_dir.to_str().unwrap()])
        .assert()
        .success();

    dm(tmp.path())
        .arg("ls")
        .assert()
        .success()
        .stdout(predicate::str::contains("aliased"));
    dm(tmp.path())
        .args(["find", "alias"])
        .assert()
        .success()
        .stdout(predicate::str::contains("aliased"));
}
