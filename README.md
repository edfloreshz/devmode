# Devmode

**Devmode** is a project management utility for developers. It helps you manage repos and
group them into workspaces — entirely locally, with no GitHub/GitLab API calls and no
shelling out to `git`/`gh`. All git operations run in-process via [`git2`][libgit2].

[libgit2]: https://github.com/rust-lang/git2-rs

## What it does

- **Track repos** you clone, create, or already have on disk in a local SQLite registry,
  so listing and finding them is instant instead of walking the filesystem every time.
- **Lay out cloned repos automatically** under a configurable root, using a built-in
  convention (`host/owner/repo`, `owner/repo`, `flat`) or your own `custom:{owner}/{repo}`
  template — and offers to fix drift (`dm repo relayout`) if you change your mind later.
- **Group repos into workspaces** — a named, non-destructive collection of repos (they
  never move on disk to join one) that can carry its own default editor and environment
  variables, so `dm workspace switch <id>` opens every member repo in one command with
  the right env applied.

## Install

```bash
cargo install --path crates/cli
```

This installs the `dm` binary.

## Quick start

```bash
dm repo clone https://github.com/owner/repo.git   # or: dm clone ...
dm repo list                                       # or: dm ls
dm repo find repo                                  # or: dm find repo

dm workspace create client-x --editor "code -n"
dm workspace add client-x repo
dm workspace switch client-x
```

Run `dm --help` or `dm <command> --help` for the full command reference, and
`dm completions <shell>` to generate shell completions.

## Scope: what devmode deliberately does not do

Because devmode is fully local, a few things GitHub's `gh` CLI does are out of scope,
on purpose:

- **No real "fork".** A GitHub fork is a server-side operation (a copy plus an upstream
  relationship) that requires the GitHub API. Devmode won't pretend a local clone under a
  different name is a fork — if you need one, use `gh repo fork`, then `dm repo track` the
  result.
- **No remote "search".** `dm repo find <query>` searches repos you've already tracked
  locally; it does not search GitHub/GitLab for repos to clone. That stays `gh search
  repos`' job.
- **No issue/PR management.** Devmode manages repos and workspaces, not GitHub-side
  workflow — use `gh` alongside it for that.

## Proposals

If you have a proposal for a new feature, open a new [issue](https://github.com/edfloreshz/devmode/issues).
