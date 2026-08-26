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

## Three frontends, one core

All of devmode's logic lives in `dm-core`; each frontend only owns how it presents
things. Every one reads and writes the same registry and `config.toml`, so you can move
between them freely — the GUI even refreshes itself when you change something with `dm`
in a terminal.

| Binary  | Crate         | What it's for |
|---------|---------------|---------------|
| `dm`    | `crates/cli`  | Scripting and quick one-off commands |
| `dmtui` | `crates/tui`  | Browsing and editing without leaving the terminal |
| `dmui`  | `crates/ui`   | A desktop window, for batch work and discovery |

## Install

```bash
cargo install --path crates/cli   # dm    — the CLI
cargo install --path crates/tui   # dmtui — the terminal UI
cargo install --path crates/ui    # dmui  — the desktop app
```

## Quick start

```bash
dm repo clone https://github.com/owner/repo.git   # or: dm clone ...
dm repo list                                       # or: dm ls
dm repo find repo                                  # or: dm find repo

dm workspace create client-x --editor "code -n"
dm workspace add client-x repo
dm workspace switch client-x
```

### Scripting

`dm` uses interactive prompts that need a real terminal. To use it from a script, CI, or a
pipe, turn them off — confirmations then read plain stdin, and ambiguous names error
instead of opening a picker:

```bash
dm config set interactive false
echo y | dm repo scan ~/code
```

### The desktop app

```bash
dmui
```

`dmui` covers everything the CLI does for repos and workspaces, plus two things that suit
a window better than a prompt loop:

- **Discovery** scans a folder for untracked repos and shows the whole result set at once
  with checkboxes, so you pick a batch instead of answering one prompt per repo. It also
  checks tracked repos still exist and their remotes still match, with per-issue and
  fix-all actions.
- **Settings** edits the same `config.toml` as `dm config`, previewing where a repo would
  actually land as you change the layout.

It follows your system light/dark setting live, and `ICED_THEME` overrides the theme if
you want a specific one.

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
