//! The Repos screen: a filterable list of tracked repos beside a detail pane,
//! with clone/create/track dialogs and a layout-drift banner.

use std::path::PathBuf;

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use iced::widget::{
    checkbox, column, container, operation, row, rule, scrollable, space, text,
};
use iced::{Center, Element, Fill, Task};

use dm_core::config::Config;
use dm_core::error::Error as CoreError;
use dm_core::git;
use dm_core::paths;
use dm_core::registry::{NewRepo, RegistryStore, RepoId};
use dm_core::relayout;

use crate::app::{App, Message as AppMessage};
use crate::data::Snapshot;
use crate::design::{self, Tone};

const SEARCH_ID: &str = "repos-search";

/// Which dialog, if any, is open over the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dialog {
    Clone { url: String, path: String },
    Create { name: String, path: String, git: bool },
    Track { path: String },
    Remove { id: RepoId, name: String, delete: bool },
}

#[derive(Debug, Default)]
pub struct State {
    pub query: String,
    pub selected: Option<RepoId>,
    pub dialog: Option<Dialog>,
    /// The selected repo's git status, loaded on demand — `None` while
    /// loading, or if it isn't (or is no longer) a git repo.
    pub git_status: Option<git::RepoStatus>,
}

impl State {
    /// Keeps the selection meaningful across reloads: hold the current repo if
    /// it still exists, otherwise fall back to the first one.
    pub fn reconcile(&mut self, snapshot: &Snapshot) {
        let still_exists = self
            .selected
            .is_some_and(|id| snapshot.repos.iter().any(|repo| repo.id == id));

        if !still_exists {
            self.selected = snapshot.repos.first().map(|repo| repo.id);
            self.git_status = None;
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Search(String),
    FocusSearch,
    Select(RepoId),
    OpenClone,
    OpenCreate,
    OpenTrack,
    OpenRemove(RepoId),
    DialogChanged(Dialog),
    DialogCancel,
    DialogSubmit,
    BrowsePath,
    PathPicked(Option<PathBuf>),
    CopyPath(String),
    FixDrift,
}

pub fn on_enter(app: &mut App) -> Task<AppMessage> {
    // The list is already in the snapshot; only the selected repo's git
    // status is loaded on demand.
    match (app.repos.selected, app.repos.git_status.is_some()) {
        (Some(_), false) => app.refresh_repo_status(),
        _ => Task::none(),
    }
}

pub fn update(app: &mut App, message: Message) -> Task<AppMessage> {
    match message {
        Message::Search(query) => {
            app.repos.query = query;

            // Keep the selection visible: if filtering hid it, select the
            // best remaining match rather than showing an empty detail pane.
            if let Some(snapshot) = app.snapshot() {
                let visible = filter(snapshot, &app.repos.query);

                if !visible.iter().any(|repo| Some(repo.id) == app.repos.selected) {
                    let next = visible.first().map(|repo| repo.id);
                    return match next {
                        Some(id) => select(app, id),
                        None => {
                            app.repos.selected = None;
                            app.repos.git_status = None;
                            Task::none()
                        }
                    };
                }
            }

            Task::none()
        }
        Message::FocusSearch => operation::focus(SEARCH_ID),
        Message::Select(id) => select(app, id),
        Message::OpenClone => {
            app.repos.dialog = Some(Dialog::Clone {
                url: String::new(),
                path: String::new(),
            });
            operation::focus("dialog-first")
        }
        Message::OpenCreate => {
            app.repos.dialog = Some(Dialog::Create {
                name: String::new(),
                path: String::new(),
                git: true,
            });
            operation::focus("dialog-first")
        }
        Message::OpenTrack => {
            app.repos.dialog = Some(Dialog::Track {
                path: String::new(),
            });
            operation::focus("dialog-first")
        }
        Message::OpenRemove(id) => {
            let name = app
                .snapshot()
                .and_then(|snapshot| snapshot.repo(id))
                .map(|repo| repo.name.clone())
                .unwrap_or_default();

            app.repos.dialog = Some(Dialog::Remove {
                id,
                name,
                delete: false,
            });
            Task::none()
        }
        Message::DialogChanged(dialog) => {
            app.repos.dialog = Some(dialog);
            Task::none()
        }
        Message::DialogCancel => {
            app.repos.dialog = None;
            Task::none()
        }
        Message::DialogSubmit => {
            let Some(dialog) = app.repos.dialog.take() else {
                return Task::none();
            };

            match dialog {
                Dialog::Clone { url, path } => {
                    if url.trim().is_empty() {
                        app.toast_error("A repository URL is required.");
                        return Task::none();
                    }

                    app.run(move || clone(&url, optional_path(&path)))
                }
                Dialog::Create { name, path, git } => {
                    if name.trim().is_empty() {
                        app.toast_error("A repository name is required.");
                        return Task::none();
                    }

                    app.run(move || create(&name, optional_path(&path), git))
                }
                Dialog::Track { path } => {
                    let Some(path) = optional_path(&path) else {
                        app.toast_error("A path is required.");
                        return Task::none();
                    };

                    app.run(move || track(path))
                }
                Dialog::Remove { id, name, delete } => {
                    app.run(move || remove(id, &name, delete))
                }
            }
        }
        Message::BrowsePath => {
            let Some(current) = app.repos.dialog.as_ref().and_then(dialog_path) else {
                return Task::none();
            };

            let starting = (!current.trim().is_empty()).then(|| PathBuf::from(current.trim()));

            Task::perform(crate::task::pick_folder("Choose a folder", starting), |picked| {
                wrap(Message::PathPicked(picked))
            })
        }
        Message::PathPicked(Some(picked)) => {
            if let Some(dialog) = app.repos.dialog.as_mut() {
                if let Some(path) = dialog_path_mut(dialog) {
                    *path = picked.display().to_string();
                }
            }

            Task::none()
        }
        Message::PathPicked(None) => Task::none(),
        Message::CopyPath(path) => iced::clipboard::write(path),
        Message::FixDrift => app.run(fix_drift),
    }
}

// -- dm-core calls, each run on a worker thread -------------------------------

fn clone(url: &str, path: Option<PathBuf>) -> Result<String, String> {
    (|| -> dm_core::Result<String> {
        let parsed = git::parse_url(url)?;
        let config = Config::load()?;
        let store = RegistryStore::open_default()?;

        let dest = path.unwrap_or_else(|| {
            config
                .repo
                .root
                .join(config.repo.layout.render(&parsed.host, &parsed.owner, &parsed.name))
        });
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

        Ok(format!("Cloned {}.", repo.name))
    })()
    .map_err(|e| e.to_string())
}

fn create(name: &str, path: Option<PathBuf>, git_init: bool) -> Result<String, String> {
    (|| -> dm_core::Result<String> {
        let config = Config::load()?;
        let store = RegistryStore::open_default()?;

        let dest = path.unwrap_or_else(|| config.repo.root.join("local").join(name));
        let dest = paths::normalize_path(&dest);

        if dest.exists() {
            return Err(CoreError::DestinationExists(dest));
        }

        if git_init {
            git::init(&dest)?;
        } else {
            std::fs::create_dir_all(&dest)?;
        }

        let repo = store.track(NewRepo {
            path: dest,
            name: name.to_string(),
            ..Default::default()
        })?;

        Ok(format!("Created {}.", repo.name))
    })()
    .map_err(|e| e.to_string())
}

fn track(path: PathBuf) -> Result<String, String> {
    (|| -> dm_core::Result<String> {
        if !path.is_dir() {
            return Err(CoreError::NotADirectory(path));
        }

        let path = paths::normalize_path(&path);
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());

        // Pick up host/owner from the remote so the repo participates in
        // layout checks straight away, the same way `dm repo scan` does.
        let remote_url = git::read_origin_url(&path);
        let (host, owner) = match remote_url.as_deref().map(git::parse_url) {
            Some(Ok(parsed)) => (Some(parsed.host), Some(parsed.owner)),
            _ => (None, None),
        };

        let store = RegistryStore::open_default()?;
        let repo = store.track(NewRepo {
            path,
            name,
            remote_url,
            host,
            owner,
            tags: Vec::new(),
        })?;

        Ok(format!("Tracked {}.", repo.name))
    })()
    .map_err(|e| e.to_string())
}

fn remove(id: RepoId, name: &str, delete: bool) -> Result<String, String> {
    (|| -> dm_core::Result<String> {
        let store = RegistryStore::open_default()?;
        let repo = store.get(id)?;

        if delete {
            std::fs::remove_dir_all(&repo.path)?;
        }

        store.remove(id)?;

        Ok(if delete {
            format!("Removed {name} and deleted it from disk.")
        } else {
            format!("Stopped tracking {name}.")
        })
    })()
    .map_err(|e| e.to_string())
}

fn fix_drift() -> Result<String, String> {
    (|| -> dm_core::Result<String> {
        let (moved, skipped) = relayout::apply_candidates(relayout::plan()?)?;

        Ok(if skipped.is_empty() {
            format!("Moved {moved} repo(s) to match your layout.")
        } else {
            format!(
                "Moved {moved} repo(s); skipped {} because the target already exists.",
                skipped.len()
            )
        })
    })()
    .map_err(|e| e.to_string())
}

/// Selects `id` and kicks off a fresh git-status load for it, unless it's
/// already selected.
fn select(app: &mut App, id: RepoId) -> Task<AppMessage> {
    if app.repos.selected == Some(id) {
        return Task::none();
    }

    app.repos.selected = Some(id);
    app.repos.git_status = None;
    app.refresh_repo_status()
}

/// The path field of whichever dialog is open, if it has one.
fn dialog_path(dialog: &Dialog) -> Option<&str> {
    match dialog {
        Dialog::Clone { path, .. } | Dialog::Create { path, .. } | Dialog::Track { path } => Some(path),
        Dialog::Remove { .. } => None,
    }
}

fn dialog_path_mut(dialog: &mut Dialog) -> Option<&mut String> {
    match dialog {
        Dialog::Clone { path, .. } | Dialog::Create { path, .. } | Dialog::Track { path } => Some(path),
        Dialog::Remove { .. } => None,
    }
}

fn optional_path(value: &str) -> Option<PathBuf> {
    let trimmed = value.trim();

    (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
}

// -- view ---------------------------------------------------------------------

/// Fuzzy-matches the query against repo names, best match first. An empty
/// query keeps the registry's own alphabetical order.
fn filter<'a>(snapshot: &'a Snapshot, query: &str) -> Vec<&'a dm_core::registry::Repo> {
    if query.trim().is_empty() {
        return snapshot.repos.iter().collect();
    }

    let matcher = SkimMatcherV2::default();
    let mut scored: Vec<_> = snapshot
        .repos
        .iter()
        .filter_map(|repo| {
            matcher
                .fuzzy_match(&repo.name, query)
                .map(|score| (score, repo))
        })
        .collect();

    scored.sort_by_key(|&(score, _)| std::cmp::Reverse(score));
    scored.into_iter().map(|(_, repo)| repo).collect()
}

pub fn view(app: &App) -> Element<'_, AppMessage> {
    let Some(snapshot) = app.snapshot() else {
        return loading();
    };

    let content: Element<'_, AppMessage> = if snapshot.repos.is_empty() {
        design::empty_state(
            "No repos tracked yet",
            "Clone a repository, create a new one, or point devmode at a folder \
             you already have. Discovery can also find repos already on disk.",
            Some(
                row![
                    design::primary_button("Clone…", wrap(Message::OpenClone)),
                    design::secondary_button("Create…", wrap(Message::OpenCreate)),
                    design::secondary_button("Track a folder…", wrap(Message::OpenTrack)),
                ]
                .spacing(design::SM)
                .into(),
            ),
        )
    } else {
        let visible = filter(snapshot, &app.repos.query);

        let body = row![
            design::pane(list(app, snapshot, &visible), 320.0),
            container(rule::horizontal(0.0)).width(1).height(Fill),
            container(detail(app, snapshot)).width(Fill).height(Fill),
        ]
        .height(Fill);

        column![toolbar(app), drift_banner(snapshot), body]
            .spacing(design::MD)
            .height(Fill)
            .into()
    };

    // Applied to whatever the screen rendered: the empty state is the only
    // way to start a clone when nothing is tracked, so its dialogs have to
    // show over it too.
    match &app.repos.dialog {
        Some(dialog) => modal(content, dialog_view(dialog)),
        None => content,
    }
}

fn loading<'a>() -> Element<'a, AppMessage> {
    container(text("Loading…").size(design::TEXT_MD))
        .width(Fill)
        .height(Fill)
        .center_x(Fill)
        .center_y(Fill)
        .into()
}

fn toolbar(app: &App) -> Element<'_, AppMessage> {
    let search = design::input("Search repos…", &app.repos.query)
        .id(SEARCH_ID)
        .on_input(|query| wrap(Message::Search(query)))
        .width(Fill);

    container(
        row![
            search,
            design::primary_button("Clone…", wrap(Message::OpenClone)),
            design::secondary_button("Create…", wrap(Message::OpenCreate)),
            design::secondary_button("Track…", wrap(Message::OpenTrack)),
        ]
        .spacing(design::SM)
        .align_y(Center),
    )
    .padding(iced::Padding::from([design::MD, design::LG]).bottom(0))
    .into()
}

fn drift_banner<'a>(snapshot: &'a Snapshot) -> Element<'a, AppMessage> {
    if snapshot.drift.is_empty() {
        return space::horizontal().width(0).into();
    }

    let message = format!(
        "{} repo(s) aren't where your current layout says they should be.",
        snapshot.drift.len()
    );

    container(
        container(
            row![
                design::badge("Layout", Tone::Warning),
                text(message).size(design::TEXT_SM),
                space::horizontal(),
                design::secondary_button("Move them", wrap(Message::FixDrift)),
            ]
            .spacing(design::SM)
            .align_y(Center),
        )
        .padding(design::SM)
        .style(|theme: &iced::Theme| container::Style {
            background: Some(theme.extended_palette().warning.weak.color.into()),
            text_color: Some(theme.extended_palette().warning.weak.text),
            border: iced::border::rounded(4),
            ..container::Style::default()
        }),
    )
    .padding(iced::Padding::from([0.0, design::LG]))
    .into()
}

fn list<'a>(
    app: &'a App,
    snapshot: &'a Snapshot,
    visible: &[&'a dm_core::registry::Repo],
) -> Element<'a, AppMessage> {
    if visible.is_empty() {
        return container(design::muted(
            text("No repos match your search.").size(design::TEXT_SM),
        ))
        .padding(design::MD)
        .into();
    }

    let mut items = column![].spacing(2.0).padding(iced::Padding::from([0.0, design::SM]));

    for repo in visible {
        let mut label = row![text(&repo.name).size(design::TEXT_MD)]
            .spacing(design::SM)
            .align_y(Center)
            .width(Fill);

        if snapshot.drift_for(repo.id).is_some() {
            label = label
                .push(space::horizontal())
                .push(design::badge("moved", Tone::Warning));
        }

        items = items.push(design::list_row(
            label,
            app.repos.selected == Some(repo.id),
            wrap(Message::Select(repo.id)),
        ));
    }

    scrollable(items).height(Fill).into()
}

fn detail<'a>(app: &'a App, snapshot: &'a Snapshot) -> Element<'a, AppMessage> {
    let Some(repo) = app.repos.selected.and_then(|id| snapshot.repo(id)) else {
        return design::empty_state("Nothing selected", "Pick a repo to see its details.", None);
    };

    let mut details = column![
        design::mono_field("Path", repo.path.display()),
        design::mono_field("Remote", repo.remote_url.as_deref().unwrap_or("—")),
    ]
    .spacing(design::MD);

    if repo.host.is_some() || repo.owner.is_some() {
        details = details.push(
            row![
                design::field(
                    "Host",
                    text(repo.host.as_deref().unwrap_or("—"))
                        .size(design::TEXT_MD)
                        .into()
                ),
                design::field(
                    "Owner",
                    text(repo.owner.as_deref().unwrap_or("—"))
                        .size(design::TEXT_MD)
                        .into()
                ),
            ]
            .spacing(design::MD),
        );
    }

    let memberships = snapshot.workspaces_for(repo.id);
    let membership_view: Element<'_, AppMessage> = if memberships.is_empty() {
        design::muted(text("Not in any workspace").size(design::TEXT_SM))
    } else {
        let mut chips = row![].spacing(design::XS);
        for name in memberships {
            chips = chips.push(design::badge(name, Tone::Info));
        }
        chips.into()
    };

    details = details.push(design::field("Workspaces", membership_view));

    if let Some(status) = &app.repos.git_status {
        details = details.push(git_section(status));
    }

    if let Some(candidate) = snapshot.drift_for(repo.id) {
        details = details.push(design::section(
            "Layout drift",
            column![
                design::muted(
                    text("Your current layout puts this repo somewhere else.")
                        .size(design::TEXT_SM)
                ),
                design::mono_field("Would move to", candidate.to.display()),
                design::button_row(vec![design::secondary_button(
                    "Move it",
                    wrap(Message::FixDrift),
                )]),
            ]
            .spacing(design::SM),
        ));
    }

    let actions = row![
        design::secondary_button(
            "Copy path",
            wrap(Message::CopyPath(repo.path.display().to_string())),
        ),
        design::secondary_button("Remove…", wrap(Message::OpenRemove(repo.id))),
    ]
    .spacing(design::SM);

    design::page(column![
        design::page_header(&repo.name, None, Some(actions.into())),
        details,
    ])
}

/// Branch, working-tree, and last-commit summary for the selected repo.
fn git_section<'a>(status: &'a git::RepoStatus) -> Element<'a, AppMessage> {
    let branch: Element<'_, AppMessage> = if status.detached {
        design::badge("detached HEAD", Tone::Warning)
    } else {
        text(status.branch.as_deref().unwrap_or("—"))
            .font(design::MONO)
            .size(design::TEXT_MD)
            .into()
    };

    let mut top = row![design::field("Branch", branch)].spacing(design::MD);

    if let (Some(ahead), Some(behind)) = (status.ahead, status.behind) {
        let tracking: Element<'_, AppMessage> = if ahead == 0 && behind == 0 {
            design::muted(text("up to date").size(design::TEXT_SM))
        } else {
            let mut chips = row![].spacing(design::XS);
            if ahead > 0 {
                chips = chips.push(design::badge(format!("↑{ahead}"), Tone::Info));
            }
            if behind > 0 {
                chips = chips.push(design::badge(format!("↓{behind}"), Tone::Warning));
            }
            chips.into()
        };

        top = top.push(design::field("Upstream", tracking));
    }

    let working_tree: Element<'_, AppMessage> = if status.is_clean() {
        design::badge("clean", Tone::Success)
    } else {
        let mut chips = row![].spacing(design::XS);
        if status.staged > 0 {
            chips = chips.push(design::badge(format!("{} staged", status.staged), Tone::Info));
        }
        if status.modified > 0 {
            chips = chips.push(design::badge(format!("{} modified", status.modified), Tone::Warning));
        }
        if status.untracked > 0 {
            chips = chips.push(design::badge(format!("{} untracked", status.untracked), Tone::Neutral));
        }
        chips.into()
    };

    let mut body = column![top, design::field("Working tree", working_tree)].spacing(design::SM);

    if let Some(commit) = &status.last_commit {
        let summary = if commit.summary.is_empty() {
            "(no summary)".to_string()
        } else {
            commit.summary.clone()
        };

        body = body.push(design::field(
            "Last commit",
            column![
                text(format!("{summary} — {}", relative_time(commit.when))).size(design::TEXT_MD),
                design::muted(
                    text(format!("{} by {}", commit.short_id, commit.author))
                        .font(design::MONO)
                        .size(design::TEXT_SM)
                ),
            ]
            .spacing(2.0)
            .into(),
        ));
    }

    if status.tag_count > 0 || status.stash_count > 0 {
        let mut chips = row![].spacing(design::XS);
        if status.tag_count > 0 {
            chips = chips.push(design::badge(format!("{} tags", status.tag_count), Tone::Neutral));
        }
        if status.stash_count > 0 {
            chips = chips.push(design::badge(format!("{} stashed", status.stash_count), Tone::Neutral));
        }
        body = body.push(chips);
    }

    design::section("Git", body)
}

/// A short, human "3 hours ago"-style rendering of a past `SystemTime`.
fn relative_time(when: std::time::SystemTime) -> String {
    let Ok(elapsed) = std::time::SystemTime::now().duration_since(when) else {
        return "just now".to_string();
    };

    let seconds = elapsed.as_secs();

    let (amount, unit) = if seconds < 60 {
        return "just now".to_string();
    } else if seconds < 3600 {
        (seconds / 60, "minute")
    } else if seconds < 86_400 {
        (seconds / 3600, "hour")
    } else if seconds < 604_800 {
        (seconds / 86_400, "day")
    } else if seconds < 2_592_000 {
        (seconds / 604_800, "week")
    } else if seconds < 31_536_000 {
        (seconds / 2_592_000, "month")
    } else {
        (seconds / 31_536_000, "year")
    };

    let plural = if amount == 1 { "" } else { "s" };
    format!("{amount} {unit}{plural} ago")
}

// -- dialogs ------------------------------------------------------------------

/// Overlays `content` with a centred dialog card, dimming what's behind it.
pub fn modal<'a>(
    base: Element<'a, AppMessage>,
    dialog: Element<'a, AppMessage>,
) -> Element<'a, AppMessage> {
    iced::widget::stack![
        base,
        iced::widget::opaque(
            container(iced::widget::opaque(dialog))
                .width(Fill)
                .height(Fill)
                .center_x(Fill)
                .center_y(Fill)
                .style(|_theme: &iced::Theme| container::Style {
                    background: Some(
                        iced::Color {
                            a: 0.7,
                            ..iced::Color::BLACK
                        }
                        .into()
                    ),
                    ..container::Style::default()
                })
        )
    ]
    .into()
}

fn dialog_view(dialog: &Dialog) -> Element<'_, AppMessage> {
    let (title, hint, fields, confirm): (_, _, Element<'_, AppMessage>, &str) = match dialog {
        Dialog::Clone { url, path } => (
            "Clone a repository",
            "The destination is derived from your layout unless you override it.",
            column![
                labelled(
                    "Repository URL",
                    "https://github.com/owner/repo.git",
                    url,
                    true,
                    {
                        let path = path.clone();
                        move |url| Dialog::Clone {
                            url,
                            path: path.clone(),
                        }
                    },
                ),
                labelled_path("Destination (optional)", "Leave blank to use your layout", path, false, {
                    let url = url.clone();
                    move |path| Dialog::Clone {
                        url: url.clone(),
                        path,
                    }
                }),
            ]
            .spacing(design::MD)
            .into(),
            "Clone",
        ),
        Dialog::Create { name, path, git } => (
            "Create a repository",
            "New repos land under `local/` in your repo root unless you set a path.",
            column![
                labelled("Name", "my-project", name, true, {
                    let (path, git) = (path.clone(), *git);
                    move |name| Dialog::Create {
                        name,
                        path: path.clone(),
                        git,
                    }
                }),
                labelled_path("Path (optional)", "Leave blank for the default", path, false, {
                    let (name, git) = (name.clone(), *git);
                    move |path| Dialog::Create {
                        name: name.clone(),
                        path,
                        git,
                    }
                }),
                checkbox(*git)
                    .label("Initialise a git repository")
                    .text_size(design::CONTROL_TEXT)
                    .on_toggle({
                        let (name, path) = (name.clone(), path.clone());
                        move |git| {
                            wrap(Message::DialogChanged(Dialog::Create {
                                name: name.clone(),
                                path: path.clone(),
                                git,
                            }))
                        }
                    }),
            ]
            .spacing(design::MD)
            .into(),
            "Create",
        ),
        Dialog::Track { path } => (
            "Track an existing folder",
            "devmode reads the folder's origin remote to fill in host and owner.",
            column![labelled_path(
                "Path",
                "/path/to/repo",
                path,
                true,
                |path| Dialog::Track { path },
            )]
            .into(),
            "Track",
        ),
        Dialog::Remove { id, name, delete } => (
            "Remove this repo",
            "Removing only stops devmode tracking it, unless you also delete it.",
            column![
                text(format!("“{name}” will no longer be tracked.")).size(design::TEXT_MD),
                checkbox(*delete)
                    .label("Also delete the folder from disk")
                    .text_size(design::CONTROL_TEXT)
                    .style(checkbox::danger)
                    .on_toggle({
                        let (id, name) = (*id, name.clone());
                        move |delete| {
                            wrap(Message::DialogChanged(Dialog::Remove {
                                id,
                                name: name.clone(),
                                delete,
                            }))
                        }
                    }),
            ]
            .spacing(design::MD)
            .into(),
            "Remove",
        ),
    };

    let is_destructive = matches!(dialog, Dialog::Remove { .. });

    let confirm_button = if is_destructive {
        design::danger_button(confirm, wrap(Message::DialogSubmit))
    } else {
        design::primary_button(confirm, wrap(Message::DialogSubmit))
    };

    container(
        column![
            column![
                text(title).size(design::TEXT_LG),
                design::muted(text(hint).size(design::TEXT_SM)),
            ]
            .spacing(design::XS),
            fields,
            design::button_row(vec![
                design::secondary_button("Cancel", wrap(Message::DialogCancel)),
                confirm_button,
            ]),
        ]
        .spacing(design::LG),
    )
    .width(520)
    .padding(design::XL)
    .style(container::rounded_box)
    .into()
}

/// A labelled dialog field. `first` marks the input that receives focus when
/// the dialog opens, and submitting from any field confirms the dialog.
fn labelled<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    first: bool,
    to_dialog: impl Fn(String) -> Dialog + 'a,
) -> Element<'a, AppMessage> {
    let mut input = design::input(placeholder, value)
        .on_input(move |value| wrap(Message::DialogChanged(to_dialog(value))))
        .on_submit(wrap(Message::DialogSubmit))
        .font(design::MONO)
        .width(Fill);

    if first {
        input = input.id("dialog-first");
    }

    design::field(label, input.into())
}

/// A labelled path field paired with a "Browse…" button that opens the
/// native folder picker.
fn labelled_path<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    first: bool,
    to_dialog: impl Fn(String) -> Dialog + 'a,
) -> Element<'a, AppMessage> {
    let mut input = design::input(placeholder, value)
        .on_input(move |value| wrap(Message::DialogChanged(to_dialog(value))))
        .on_submit(wrap(Message::DialogSubmit))
        .font(design::MONO)
        .width(Fill);

    if first {
        input = input.id("dialog-first");
    }

    design::field(
        label,
        row![input, design::secondary_button("Browse…", wrap(Message::BrowsePath))]
            .spacing(design::SM)
            .align_y(Center)
            .into(),
    )
}

fn wrap(message: Message) -> AppMessage {
    AppMessage::Repos(message)
}
