//! The Workspaces screen: workspaces beside a detail pane holding their
//! members, environment variables, and editor override.

use iced::widget::{column, container, operation, row, rule, scrollable, space, text};
use iced::{Center, Element, Fill, Task};

use dm_core::config::Config;
use dm_core::registry::{RegistryStore, RepoId};
use dm_core::workspace::{NewWorkspace, WorkspaceStore};

use crate::app::{App, Message as AppMessage};
use crate::data::{Snapshot, WorkspaceDetail};
use crate::design::{self, Tone};

/// A dialog layered over the screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dialog {
    Create {
        id: String,
        name: String,
        description: String,
        editor: String,
    },
    AddMember {
        workspace: String,
        query: String,
    },
    SetEnv {
        workspace: String,
        key: String,
        value: String,
    },
    Delete {
        id: String,
        members: usize,
    },
}

#[derive(Debug, Default)]
pub struct State {
    pub selected: Option<String>,
    pub detail: Option<WorkspaceDetail>,
    pub dialog: Option<Dialog>,
    /// The inline "edit this workspace" form, when open.
    pub editing: Option<Editing>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Editing {
    pub name: String,
    pub description: String,
    pub editor: String,
}

impl State {
    pub fn reconcile(&mut self, snapshot: &Snapshot) {
        let still_exists = self
            .selected
            .as_ref()
            .is_some_and(|id| snapshot.workspaces.iter().any(|w| &w.id == id));

        if !still_exists {
            self.selected = snapshot.workspaces.first().map(|w| w.id.clone());
            self.detail = None;
            self.editing = None;
        }
    }

    pub fn selected(&self) -> Option<String> {
        self.selected.clone()
    }

    pub fn set_detail(&mut self, detail: WorkspaceDetail) {
        // A stale response from a workspace the user already navigated away
        // from would otherwise overwrite the current one.
        if self.selected.as_deref() == Some(detail.id.as_str()) {
            self.detail = Some(detail);
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Select(String),
    OpenCreate,
    OpenDelete,
    OpenAddMember,
    OpenSetEnv,
    DialogChanged(Dialog),
    DialogCancel,
    DialogSubmit,
    StartEditing,
    EditingChanged(Editing),
    CancelEditing,
    SaveEditing,
    RemoveMember(RepoId),
    UnsetEnv(String),
    Switch,
}

pub fn on_enter(app: &mut App) -> Task<AppMessage> {
    // The list is already in the snapshot; only the selected workspace's
    // members and env are loaded on demand.
    match (app.workspaces.selected(), app.workspaces.detail.is_some()) {
        (Some(id), false) => app.load_workspace_detail(id),
        _ => Task::none(),
    }
}

pub fn update(app: &mut App, message: Message) -> Task<AppMessage> {
    match message {
        Message::Select(id) => {
            if app.workspaces.selected.as_deref() == Some(id.as_str()) {
                return Task::none();
            }

            app.workspaces.selected = Some(id.clone());
            app.workspaces.detail = None;
            app.workspaces.editing = None;
            app.load_workspace_detail(id)
        }
        Message::OpenCreate => {
            app.workspaces.dialog = Some(Dialog::Create {
                id: String::new(),
                name: String::new(),
                description: String::new(),
                editor: String::new(),
            });
            operation::focus("ws-dialog-first")
        }
        Message::OpenDelete => {
            let Some(id) = app.workspaces.selected() else {
                return Task::none();
            };

            app.workspaces.dialog = Some(Dialog::Delete {
                id,
                members: app
                    .workspaces
                    .detail
                    .as_ref()
                    .map(|detail| detail.members.len())
                    .unwrap_or(0),
            });
            Task::none()
        }
        Message::OpenAddMember => {
            let Some(workspace) = app.workspaces.selected() else {
                return Task::none();
            };

            app.workspaces.dialog = Some(Dialog::AddMember {
                workspace,
                query: String::new(),
            });
            operation::focus("ws-dialog-first")
        }
        Message::OpenSetEnv => {
            let Some(workspace) = app.workspaces.selected() else {
                return Task::none();
            };

            app.workspaces.dialog = Some(Dialog::SetEnv {
                workspace,
                key: String::new(),
                value: String::new(),
            });
            operation::focus("ws-dialog-first")
        }
        Message::DialogChanged(dialog) => {
            app.workspaces.dialog = Some(dialog);
            Task::none()
        }
        Message::DialogCancel => {
            app.workspaces.dialog = None;
            Task::none()
        }
        Message::DialogSubmit => submit_dialog(app),
        Message::StartEditing => {
            let Some(workspace) = current_workspace(app) else {
                return Task::none();
            };

            app.workspaces.editing = Some(Editing {
                name: workspace.name.clone(),
                description: workspace.description.clone().unwrap_or_default(),
                editor: workspace.editor.clone().unwrap_or_default(),
            });
            operation::focus("ws-edit-name")
        }
        Message::EditingChanged(editing) => {
            app.workspaces.editing = Some(editing);
            Task::none()
        }
        Message::CancelEditing => {
            app.workspaces.editing = None;
            Task::none()
        }
        Message::SaveEditing => {
            let (Some(id), Some(editing)) =
                (app.workspaces.selected(), app.workspaces.editing.take())
            else {
                return Task::none();
            };

            if editing.name.trim().is_empty() {
                app.toast_error("A workspace needs a name.");
                return Task::none();
            }

            app.run(move || save_config(&id, editing))
        }
        Message::RemoveMember(repo) => {
            let Some(id) = app.workspaces.selected() else {
                return Task::none();
            };

            app.run(move || {
                let store = WorkspaceStore::open_default().map_err(|e| e.to_string())?;
                store.remove_member(&id, repo).map_err(|e| e.to_string())?;

                Ok("Removed from workspace.".to_string())
            })
        }
        Message::UnsetEnv(key) => {
            let Some(id) = app.workspaces.selected() else {
                return Task::none();
            };

            app.run(move || {
                let store = WorkspaceStore::open_default().map_err(|e| e.to_string())?;
                store.env_unset(&id, &key).map_err(|e| e.to_string())?;

                Ok(format!("Unset {key}."))
            })
        }
        Message::Switch => {
            let Some(id) = app.workspaces.selected() else {
                return Task::none();
            };

            app.run(move || switch(&id))
        }
    }
}

fn submit_dialog(app: &mut App) -> Task<AppMessage> {
    let Some(dialog) = app.workspaces.dialog.take() else {
        return Task::none();
    };

    match dialog {
        Dialog::Create {
            id,
            name,
            description,
            editor,
        } => {
            let id = id.trim().to_string();

            if id.is_empty() {
                app.toast_error("A workspace needs an id.");
                return Task::none();
            }

            // A blank name is a nuisance to fix later; default it to the id.
            let name = if name.trim().is_empty() {
                id.clone()
            } else {
                name.trim().to_string()
            };

            let new = NewWorkspace {
                id,
                name,
                description: non_empty(&description),
                editor: non_empty(&editor),
            };

            app.workspaces.selected = Some(new.id.clone());
            app.run(move || {
                let store = WorkspaceStore::open_default().map_err(|e| e.to_string())?;
                let workspace = store.create(new).map_err(|e| e.to_string())?;

                Ok(format!("Created workspace {}.", workspace.id))
            })
        }
        Dialog::AddMember { workspace, query } => {
            // The picker submits the top match, so an empty query is a no-op
            // rather than an error.
            let Some(snapshot) = app.snapshot() else {
                return Task::none();
            };

            let Some(repo) = candidates(app, snapshot, &query).first().map(|repo| (repo.id, repo.name.clone()))
            else {
                app.toast_error("No repo matches that search.");
                return Task::none();
            };

            let (repo_id, repo_name) = repo;

            app.run(move || {
                let store = WorkspaceStore::open_default().map_err(|e| e.to_string())?;

                match store.add_member(&workspace, repo_id) {
                    Ok(()) => Ok(format!("Added {repo_name} to {workspace}.")),
                    Err(dm_core::Error::AlreadyInWorkspace { .. }) => {
                        Ok(format!("{repo_name} is already in {workspace}."))
                    }
                    Err(e) => Err(e.to_string()),
                }
            })
        }
        Dialog::SetEnv {
            workspace,
            key,
            value,
        } => {
            let key = key.trim().to_string();

            if key.is_empty() {
                app.toast_error("An environment variable needs a name.");
                return Task::none();
            }

            app.run(move || {
                let store = WorkspaceStore::open_default().map_err(|e| e.to_string())?;
                store
                    .env_set(&workspace, &key, &value)
                    .map_err(|e| e.to_string())?;

                Ok(format!("Set {key}."))
            })
        }
        Dialog::Delete { id, .. } => app.run(move || {
            let store = WorkspaceStore::open_default().map_err(|e| e.to_string())?;
            store.delete(&id).map_err(|e| e.to_string())?;

            Ok(format!("Deleted workspace {id}."))
        }),
    }
}

fn save_config(id: &str, editing: Editing) -> Result<String, String> {
    (|| -> dm_core::Result<String> {
        let store = WorkspaceStore::open_default()?;

        store.set_config(id, "name", editing.name.trim())?;
        store.set_config(id, "description", editing.description.trim())?;
        store.set_config(id, "editor", editing.editor.trim())?;

        Ok("Workspace updated.".to_string())
    })()
    .map_err(|e| e.to_string())
}

/// Opens the workspace's member repos in its editor.
///
/// Unlike `dm workspace switch`, the GUI spawns the editor detached instead
/// of waiting on it — the window stays usable, and a long-lived editor
/// shouldn't pin the app open.
fn switch(id: &str) -> Result<String, String> {
    (|| -> dm_core::Result<String> {
        let registry = RegistryStore::open_default()?;
        let workspaces = WorkspaceStore::open_default()?;
        let workspace = workspaces.get(id)?;

        let repos = workspaces
            .members(id)?
            .into_iter()
            .map(|repo_id| registry.get(repo_id))
            .collect::<dm_core::Result<Vec<_>>>()?;

        if repos.is_empty() {
            return Ok(format!("{id} has no members to open yet."));
        }

        let editor = workspace.editor.clone().or(Config::load()?.editor);

        let Some(editor) = editor else {
            return Ok(format!(
                "No editor set for {id}. Add one in this workspace's settings, \
                 or set a global editor in Settings."
            ));
        };

        let mut parts = editor.split_whitespace();

        let Some(program) = parts.next() else {
            return Ok(format!("The editor command for {id} is empty."));
        };

        let mut command = std::process::Command::new(program);
        command.args(parts);

        for repo in &repos {
            command.arg(&repo.path);
        }

        for (key, value) in workspaces.env_list(id)? {
            command.env(key, value);
        }

        command.spawn()?;

        Ok(format!("Opened {} repo(s) in {editor}.", repos.len()))
    })()
    .map_err(|e| e.to_string())
}

fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();

    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn current_workspace(app: &App) -> Option<&dm_core::workspace::Workspace> {
    let id = app.workspaces.selected.as_deref()?;

    app.snapshot()?.workspace(id)
}

/// Repos not already in the workspace, filtered by the picker's query.
fn candidates<'a>(
    app: &'a App,
    snapshot: &'a Snapshot,
    query: &str,
) -> Vec<&'a dm_core::registry::Repo> {
    use fuzzy_matcher::FuzzyMatcher;
    use fuzzy_matcher::skim::SkimMatcherV2;

    let members: Vec<RepoId> = app
        .workspaces
        .detail
        .as_ref()
        .map(|detail| detail.members.iter().map(|repo| repo.id).collect())
        .unwrap_or_default();

    let available = snapshot
        .repos
        .iter()
        .filter(|repo| !members.contains(&repo.id));

    if query.trim().is_empty() {
        return available.collect();
    }

    let matcher = SkimMatcherV2::default();
    let mut scored: Vec<_> = available
        .filter_map(|repo| {
            matcher
                .fuzzy_match(&repo.name, query)
                .map(|score| (score, repo))
        })
        .collect();

    scored.sort_by_key(|&(score, _)| std::cmp::Reverse(score));
    scored.into_iter().map(|(_, repo)| repo).collect()
}

// -- view ---------------------------------------------------------------------

pub fn view(app: &App) -> Element<'_, AppMessage> {
    let Some(snapshot) = app.snapshot() else {
        return container(text("Loading…").size(design::TEXT_MD))
            .width(Fill)
            .height(Fill)
            .center_x(Fill)
            .center_y(Fill)
            .into();
    };

    let content: Element<'_, AppMessage> = if snapshot.workspaces.is_empty() {
        design::empty_state(
            "No workspaces yet",
            "A workspace groups repos you work on together, with its own editor \
             and environment variables. Repos never move on disk when you group them.",
            Some(design::primary_button(
                "New workspace…",
                wrap(Message::OpenCreate),
            )),
        )
    } else {
        let body = row![
            container(design::pane(list(app, snapshot), 280.0))
                .padding(iced::Padding::default().left(design::XL)),
            container(rule::horizontal(0.0)).width(1).height(Fill),
            container(detail(app, snapshot)).width(Fill).height(Fill),
        ]
        .height(Fill);

        body.into()
    };

    let actions = (!snapshot.workspaces.is_empty())
        .then(|| design::primary_button("New workspace…", wrap(Message::OpenCreate)));

    let page: Element<'_, AppMessage> = column![header(actions), content]
        .spacing(design::MD)
        .height(Fill)
        .into();

    // The empty state offers the only "New workspace" button when there are
    // none, so its dialog has to render over it.
    match &app.workspaces.dialog {
        Some(dialog) => crate::screen::repos::modal(page, dialog_view(app, dialog)),
        None => page,
    }
}

fn header<'a>(actions: impl Into<Option<Element<'a, AppMessage>>>) -> Element<'a, AppMessage> {
    container(design::page_header(
        "Workspaces",
        Some(
            "Group repos you work on together, with a shared editor and environment."
                .to_string(),
        ),
        actions,
    ))
    .padding(iced::Padding::from([design::XL, design::XL]).bottom(0))
    .into()
}

fn list<'a>(app: &'a App, snapshot: &'a Snapshot) -> Element<'a, AppMessage> {
    let mut items = column![]
        .spacing(2.0)
        .padding(iced::Padding::from([0.0, design::SM]));

    for workspace in &snapshot.workspaces {
        let label = column![
            text(&workspace.name).size(design::TEXT_MD),
            design::muted(text(&workspace.id).size(design::TEXT_SM)),
        ]
        .spacing(1.0)
        .width(Fill);

        items = items.push(design::list_row(
            label,
            app.workspaces.selected.as_deref() == Some(workspace.id.as_str()),
            wrap(Message::Select(workspace.id.clone())),
        ));
    }

    scrollable(items).height(Fill).into()
}

fn detail<'a>(app: &'a App, snapshot: &'a Snapshot) -> Element<'a, AppMessage> {
    let Some(workspace) = current_workspace(app) else {
        return design::empty_state(
            "Nothing selected",
            "Pick a workspace to see its repos and environment.",
            None,
        );
    };

    let actions = row![
        design::primary_button("Open in editor", wrap(Message::Switch)),
        design::secondary_button("Edit…", wrap(Message::StartEditing)),
        design::destructive_button("Delete…", wrap(Message::OpenDelete)),
    ]
    .spacing(design::SM);

    let mut page = column![design::page_header(
        &workspace.name,
        Some(workspace.description.clone().unwrap_or_else(|| {
            "No description".to_string()
        })),
        Some(actions.into()),
    )];

    if let Some(editing) = &app.workspaces.editing {
        page = page.push(editing_form(editing));
    } else {
        page = page.push(design::section(
            "Editor",
            column![
                design::muted(
                    text(
                        "Used when opening this workspace. Falls back to your \
                         global editor when blank."
                    )
                    .size(design::TEXT_SM)
                ),
                text(workspace.editor.clone().unwrap_or_else(|| "—".to_string()))
                    .font(design::MONO)
                    .size(design::TEXT_MD),
            ]
            .spacing(design::XS),
        ));
    }

    page = page.push(members_section(app, snapshot));
    page = page.push(env_section(app));

    design::page(page)
}

fn editing_form(editing: &Editing) -> Element<'_, AppMessage> {
    let field = |label: &'static str,
                 placeholder: &'static str,
                 value: &str,
                 id: Option<&'static str>,
                 to_editing: Box<dyn Fn(String) -> Editing>| {
        let mut input = design::input(placeholder, value)
            .on_input(move |value| wrap(Message::EditingChanged(to_editing(value))))
            .on_submit(wrap(Message::SaveEditing))
            .width(Fill);

        if let Some(id) = id {
            input = input.id(id);
        }

        design::field(label, input.into())
    };

    design::section(
        "Edit workspace",
        column![
            field(
                "Name",
                "Display name",
                &editing.name,
                Some("ws-edit-name"),
                {
                    let editing = editing.clone();
                    Box::new(move |name| Editing {
                        name,
                        ..editing.clone()
                    })
                },
            ),
            field("Description", "What this workspace is for", &editing.description, None, {
                let editing = editing.clone();
                Box::new(move |description| Editing {
                    description,
                    ..editing.clone()
                })
            }),
            field("Editor", "code -n", &editing.editor, None, {
                let editing = editing.clone();
                Box::new(move |editor| Editing {
                    editor,
                    ..editing.clone()
                })
            }),
            design::button_row(vec![
                design::secondary_button("Cancel", wrap(Message::CancelEditing)),
                design::primary_button("Save", wrap(Message::SaveEditing)),
            ]),
        ]
        .spacing(design::MD),
    )
}

fn members_section<'a>(app: &'a App, snapshot: &'a Snapshot) -> Element<'a, AppMessage> {
    let add = design::secondary_button("Add repo…", wrap(Message::OpenAddMember));

    let Some(detail) = &app.workspaces.detail else {
        return design::section(
            "Repos",
            design::muted(text("Loading…").size(design::TEXT_SM)),
        );
    };

    if detail.members.is_empty() {
        return design::section(
            "Repos",
            column![
                design::muted(
                    text("No repos in this workspace yet.").size(design::TEXT_SM)
                ),
                design::button_row(vec![add]),
            ]
            .spacing(design::SM),
        );
    }

    let mut rows = column![].spacing(design::XS);

    for repo in &detail.members {
        let drift: Element<'_, AppMessage> = match snapshot.drift_for(repo.id) {
            Some(_) => design::badge("moved", Tone::Warning),
            None => space::horizontal().width(0).into(),
        };

        rows = rows.push(
            row![
                column![
                    text(&repo.name).size(design::TEXT_MD),
                    design::muted(
                        text(repo.path.display().to_string())
                            .size(design::TEXT_SM)
                            .font(design::MONO)
                    ),
                ]
                .spacing(1.0)
                .width(Fill),
                drift,
                design::secondary_button("Remove", wrap(Message::RemoveMember(repo.id))),
            ]
            .spacing(design::SM)
            .align_y(Center),
        );
    }

    design::section(
        "Repos",
        column![rows, design::button_row(vec![add])].spacing(design::MD),
    )
}

fn env_section(app: &App) -> Element<'_, AppMessage> {
    let add = design::secondary_button("Add variable…", wrap(Message::OpenSetEnv));

    let Some(detail) = &app.workspaces.detail else {
        return design::section(
            "Environment",
            design::muted(text("Loading…").size(design::TEXT_SM)),
        );
    };

    if detail.env.is_empty() {
        return design::section(
            "Environment",
            column![
                design::muted(
                    text("These are applied to the editor process when you open \
                          this workspace.")
                    .size(design::TEXT_SM)
                ),
                design::button_row(vec![add]),
            ]
            .spacing(design::SM),
        );
    }

    let mut rows = column![].spacing(design::XS);

    for (key, value) in &detail.env {
        rows = rows.push(
            row![
                text(format!("{key}={value}"))
                    .font(design::MONO)
                    .size(design::TEXT_MD)
                    .width(Fill),
                design::secondary_button("Unset", wrap(Message::UnsetEnv(key.clone()))),
            ]
            .spacing(design::SM)
            .align_y(Center),
        );
    }

    design::section(
        "Environment",
        column![rows, design::button_row(vec![add])].spacing(design::MD),
    )
}

// -- dialogs ------------------------------------------------------------------

fn dialog_view<'a>(app: &'a App, dialog: &'a Dialog) -> Element<'a, AppMessage> {
    let (title, hint, fields, confirm, destructive): (_, _, Element<'a, AppMessage>, _, bool) =
        match dialog {
            Dialog::Create {
                id,
                name,
                description,
                editor,
            } => (
                "New workspace",
                "The id is how you refer to this workspace from the CLI.",
                column![
                    dialog_field("Id", "work", id, Some("ws-dialog-first"), {
                        let d = dialog.clone();
                        Box::new(move |id| match d.clone() {
                            Dialog::Create {
                                name,
                                description,
                                editor,
                                ..
                            } => Dialog::Create {
                                id,
                                name,
                                description,
                                editor,
                            },
                            other => other,
                        })
                    }),
                    dialog_field("Name (optional)", "Defaults to the id", name, None, {
                        let d = dialog.clone();
                        Box::new(move |name| match d.clone() {
                            Dialog::Create {
                                id,
                                description,
                                editor,
                                ..
                            } => Dialog::Create {
                                id,
                                name,
                                description,
                                editor,
                            },
                            other => other,
                        })
                    }),
                    dialog_field(
                        "Description (optional)",
                        "What this workspace is for",
                        description,
                        None,
                        {
                            let d = dialog.clone();
                            Box::new(move |description| match d.clone() {
                                Dialog::Create {
                                    id, name, editor, ..
                                } => Dialog::Create {
                                    id,
                                    name,
                                    description,
                                    editor,
                                },
                                other => other,
                            })
                        }
                    ),
                    dialog_field("Editor (optional)", "code -n", editor, None, {
                        let d = dialog.clone();
                        Box::new(move |editor| match d.clone() {
                            Dialog::Create {
                                id,
                                name,
                                description,
                                ..
                            } => Dialog::Create {
                                id,
                                name,
                                description,
                                editor,
                            },
                            other => other,
                        })
                    }),
                ]
                .spacing(design::MD)
                .into(),
                "Create",
                false,
            ),
            Dialog::AddMember { workspace, query } => {
                let matches = app
                    .snapshot()
                    .map(|snapshot| candidates(app, snapshot, query))
                    .unwrap_or_default();

                let mut results = column![].spacing(2.0);

                for repo in matches.iter().take(8) {
                    results = results.push(
                        container(
                            column![
                                text(&repo.name).size(design::TEXT_MD),
                                design::muted(
                                    text(repo.path.display().to_string())
                                        .size(design::TEXT_SM)
                                        .font(design::MONO)
                                ),
                            ]
                            .spacing(1.0),
                        )
                        .padding(iced::Padding::from([design::XS, design::SM]))
                        .width(Fill),
                    );
                }

                let body: Element<'a, AppMessage> = if matches.is_empty() {
                    design::muted(
                        text("Every tracked repo is already in this workspace.")
                            .size(design::TEXT_SM),
                    )
                } else {
                    column![
                        dialog_field(
                            "Search",
                            "Type to narrow, Enter adds the top match",
                            query,
                            Some("ws-dialog-first"),
                            {
                                let workspace = workspace.clone();
                                Box::new(move |query| Dialog::AddMember {
                                    workspace: workspace.clone(),
                                    query,
                                })
                            }
                        ),
                        design::muted(text("Matches").size(design::TEXT_SM)),
                        results,
                    ]
                    .spacing(design::SM)
                    .into()
                };

                (
                    "Add a repo",
                    "Repos can belong to any number of workspaces, and never move on disk.",
                    body,
                    "Add",
                    false,
                )
            }
            Dialog::SetEnv {
                workspace,
                key,
                value,
            } => (
                "Set an environment variable",
                "Applied to the editor process when you open this workspace.",
                column![
                    dialog_field("Name", "DATABASE_URL", key, Some("ws-dialog-first"), {
                        let (workspace, value) = (workspace.clone(), value.clone());
                        Box::new(move |key| Dialog::SetEnv {
                            workspace: workspace.clone(),
                            key,
                            value: value.clone(),
                        })
                    }),
                    dialog_field("Value", "postgres://localhost/dev", value, None, {
                        let (workspace, key) = (workspace.clone(), key.clone());
                        Box::new(move |value| Dialog::SetEnv {
                            workspace: workspace.clone(),
                            key: key.clone(),
                            value,
                        })
                    }),
                ]
                .spacing(design::MD)
                .into(),
                "Set",
                false,
            ),
            Dialog::Delete { id, members } => (
                "Delete this workspace",
                "The repos themselves are untouched — only the grouping goes away.",
                column![text(format!(
                    "“{id}” and its {members} membership(s) will be deleted."
                ))
                .size(design::TEXT_MD)]
                .into(),
                "Delete",
                true,
            ),
        };

    let confirm_button = if destructive {
        design::destructive_button(confirm, wrap(Message::DialogSubmit))
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
    .width(560)
    .padding(design::XL)
    .style(container::rounded_box)
    .into()
}

fn dialog_field<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    id: Option<&'static str>,
    to_dialog: Box<dyn Fn(String) -> Dialog + 'a>,
) -> Element<'a, AppMessage> {
    let mut input = design::input(placeholder, value)
        .on_input(move |value| wrap(Message::DialogChanged(to_dialog(value))))
        .on_submit(wrap(Message::DialogSubmit))
        .width(Fill);

    if let Some(id) = id {
        input = input.id(id);
    }

    design::field(label, input.into())
}

fn wrap(message: Message) -> AppMessage {
    AppMessage::Workspaces(message)
}
