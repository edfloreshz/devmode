use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc;

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

use dm_core::config::Config;
use dm_core::discovery::{self, Discovered, Issue};
use dm_core::git::{self, RepoStatus};
use dm_core::layout::PathLayout;
use dm_core::registry::{RegistryStore, Repo, RepoId};
use dm_core::relayout::Candidate;
use dm_core::workspace::{NewWorkspace, Workspace, WorkspaceId, WorkspaceStore};

use crate::actions::{self, CloneOutcome};
use crate::error::Result;
use crate::event::Mode;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Tab {
    #[default]
    Repos,
    Workspaces,
    Discovery,
    Settings,
}

/// The config keys the Settings tab lists and lets you edit, in display order.
/// Every one round-trips through `Config::get`/`Config::set`.
pub const SETTINGS_KEYS: [&str; 7] = [
    "repo.root",
    "repo.layout",
    "editor",
    "interactive",
    "ui.theme_mode",
    "ui.light_theme",
    "ui.dark_theme",
];

const SETUP_LABELS: &[&str] = &[
    "project root",
    "layout (host_owner_repo, owner_repo, flat, custom:<template>)",
    "editor (optional)",
    "interactive (true/false)",
];

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceFocus {
    #[default]
    List,
    Items,
}

#[derive(Debug, Clone)]
pub enum DetailItem {
    Member(RepoId, String),
    Env(String, String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormKind {
    Clone,
    Create,
    Track,
    WorkspaceCreate,
    WorkspaceConfig,
    WorkspaceEnv,
    /// Editing one config key. `Form::target` holds the key name.
    ConfigEdit,
    /// First-run wizard, shown when there's no config file yet. Can't be
    /// cancelled: it stays up until the config is written.
    Setup,
}

pub struct Form {
    pub kind: FormKind,
    pub labels: &'static [&'static str],
    pub fields: Vec<Input>,
    pub focus: usize,
    pub no_git: bool,
    pub target: Option<WorkspaceId>,
}

impl Form {
    fn clone_form() -> Self {
        Self {
            kind: FormKind::Clone,
            labels: &["url", "path (optional)"],
            fields: vec![Input::default(), Input::default()],
            focus: 0,
            no_git: false,
            target: None,
        }
    }

    fn create_form() -> Self {
        Self {
            kind: FormKind::Create,
            labels: &["name", "path (optional)"],
            fields: vec![Input::default(), Input::default()],
            focus: 0,
            no_git: false,
            target: None,
        }
    }

    fn track_form() -> Self {
        Self {
            kind: FormKind::Track,
            labels: &["path"],
            fields: vec![Input::default()],
            focus: 0,
            no_git: false,
            target: None,
        }
    }

    fn workspace_create_form() -> Self {
        Self {
            kind: FormKind::WorkspaceCreate,
            labels: &["id", "name", "description (optional)", "editor (optional)"],
            fields: vec![
                Input::default(),
                Input::default(),
                Input::default(),
                Input::default(),
            ],
            focus: 0,
            no_git: false,
            target: None,
        }
    }

    fn workspace_config_form(ws: &Workspace) -> Self {
        Self {
            kind: FormKind::WorkspaceConfig,
            labels: &["name", "description", "editor"],
            fields: vec![
                Input::new(ws.name.clone()),
                Input::new(ws.description.clone().unwrap_or_default()),
                Input::new(ws.editor.clone().unwrap_or_default()),
            ],
            focus: 0,
            no_git: false,
            target: Some(ws.id.clone()),
        }
    }

    fn config_edit_form(key: &str, value: &str) -> Self {
        Self {
            kind: FormKind::ConfigEdit,
            labels: &["value"],
            fields: vec![Input::new(value.to_string())],
            focus: 0,
            no_git: false,
            target: Some(key.to_string()),
        }
    }

    fn setup_form(values: &[String]) -> Self {
        Self {
            kind: FormKind::Setup,
            labels: SETUP_LABELS,
            fields: values.iter().map(|v| Input::new(v.clone())).collect(),
            focus: 0,
            no_git: false,
            target: None,
        }
    }

    fn workspace_env_form(ws_id: &str) -> Self {
        Self {
            kind: FormKind::WorkspaceEnv,
            labels: &["key", "value"],
            fields: vec![Input::default(), Input::default()],
            focus: 0,
            no_git: false,
            target: Some(ws_id.to_string()),
        }
    }

    pub fn next_field(&mut self) {
        self.focus = (self.focus + 1) % self.fields.len();
    }

    pub fn active_field(&mut self) -> &mut Input {
        &mut self.fields[self.focus]
    }

    fn field(&self, i: usize) -> Option<PathBuf> {
        let value = self.fields.get(i)?.value();
        if value.is_empty() {
            None
        } else {
            Some(PathBuf::from(value))
        }
    }
}

pub enum ConfirmAction {
    RemoveRepo {
        id: RepoId,
        path: PathBuf,
        delete: bool,
    },
    DeleteWorkspace {
        id: WorkspaceId,
    },
}

pub struct ConfirmDialog {
    pub message: String,
    pub action: ConfirmAction,
}

pub struct RepoPicker {
    pub workspace_id: WorkspaceId,
    pub filter: Input,
    pub candidates: Vec<Repo>,
    pub filtered: Vec<usize>,
    pub selected: usize,
}

fn fuzzy_filter(repos: &[Repo], query: &str) -> Vec<usize> {
    if query.is_empty() {
        return (0..repos.len()).collect();
    }
    let matcher = SkimMatcherV2::default();
    let mut scored: Vec<(i64, usize)> = repos
        .iter()
        .enumerate()
        .filter_map(|(i, repo)| {
            matcher
                .fuzzy_match(&repo.name, query)
                .map(|score| (score, i))
        })
        .collect();
    scored.sort_by_key(|&(score, _)| std::cmp::Reverse(score));
    scored.into_iter().map(|(_, i)| i).collect()
}

pub struct App {
    pub active_tab: Tab,
    pub should_quit: bool,
    pub status: Option<String>,
    pub busy: bool,
    pub spinner_tick: usize,

    pub form: Option<Form>,
    pub confirm: Option<ConfirmDialog>,
    pub picker: Option<RepoPicker>,
    clone_rx: Option<mpsc::Receiver<CloneOutcome>>,
    pub pending_switch: Option<WorkspaceId>,

    registry: RegistryStore,
    workspaces: WorkspaceStore,

    config: Config,
    pub settings_selected: usize,

    pub found: Vec<Discovered>,
    pub issues: Vec<Issue>,
    pub discovery_selected: usize,

    pub repos: Vec<Repo>,
    pub filter: Input,
    pub filtering: bool,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub drift: Vec<Candidate>,
    /// Ids of repos whose working tree has uncommitted changes, refreshed
    /// whenever the repo list reloads. Drives the list's dirty marker.
    pub dirty: HashSet<RepoId>,
    /// Full git state for the currently selected repo, `(id, status)`.
    /// `status` is `None` when the path isn't a git repo. Refreshed on every
    /// selection change so the detail pane's Git card stays current.
    pub repo_status: Option<(RepoId, Option<RepoStatus>)>,

    pub workspaces_list: Vec<Workspace>,
    pub workspace_selected: usize,
    pub workspace_focus: WorkspaceFocus,
    pub workspace_item_selected: usize,
}

impl App {
    pub fn new() -> Result<Self> {
        let registry = RegistryStore::open_default()?;
        let workspaces = WorkspaceStore::open_default()?;
        let repos = registry.list(None, None)?;
        let filtered = (0..repos.len()).collect();
        let workspaces_list = workspaces.list()?;
        let drift = dm_core::relayout::plan().unwrap_or_default();
        let config = Config::load()?;
        let first_run = !Config::is_saved();

        let mut app = Self {
            active_tab: Tab::default(),
            should_quit: false,
            status: None,
            busy: false,
            spinner_tick: 0,
            form: None,
            confirm: None,
            picker: None,
            clone_rx: None,
            pending_switch: None,
            registry,
            workspaces,
            config,
            settings_selected: 0,
            found: Vec::new(),
            issues: Vec::new(),
            discovery_selected: 0,
            repos,
            filter: Input::default(),
            filtering: false,
            filtered,
            selected: 0,
            drift,
            dirty: HashSet::new(),
            repo_status: None,
            workspaces_list,
            workspace_selected: 0,
            workspace_focus: WorkspaceFocus::default(),
            workspace_item_selected: 0,
        };

        if first_run {
            app.form = Some(Form::setup_form(&setup_defaults(&Config::default())));
        }

        app.refresh_dirty();
        app.refresh_repo_status();

        Ok(app)
    }

    pub fn mode(&self) -> Mode {
        if self.confirm.is_some() {
            Mode::Confirm
        } else if self.picker.is_some() {
            Mode::Picker
        } else if self.form.is_some() {
            Mode::Form
        } else if self.filtering {
            Mode::Filtering
        } else {
            Mode::Normal
        }
    }

    pub fn next_tab(&mut self) {
        self.active_tab = match self.active_tab {
            Tab::Repos => Tab::Workspaces,
            Tab::Workspaces => Tab::Discovery,
            Tab::Discovery => Tab::Settings,
            Tab::Settings => Tab::Repos,
        };
    }

    pub fn back(&mut self) {
        if self.active_tab == Tab::Workspaces && self.workspace_focus == WorkspaceFocus::Items {
            self.workspace_focus = WorkspaceFocus::List;
        }
    }

    pub fn reload_repos(&mut self) -> Result<()> {
        self.repos = self.registry.list(None, None)?;
        self.drift = dm_core::relayout::plan()?;
        self.apply_filter();
        self.refresh_dirty();
        Ok(())
    }

    fn refresh_dirty(&mut self) {
        self.dirty = self
            .repos
            .iter()
            .filter(|repo| git::is_dirty(&repo.path))
            .map(|repo| repo.id)
            .collect();
    }

    /// Reads full git state for the selected repo into `repo_status`. Cheap
    /// enough to call on every selection change.
    pub fn refresh_repo_status(&mut self) {
        self.repo_status = self
            .selected_repo()
            .map(|repo| (repo.id, git::repo_status(&repo.path).ok()));
    }

    /// The selected repo's git state, or `None` if the cache is for a
    /// different repo or the path isn't a git repo.
    pub fn selected_repo_status(&self) -> Option<&RepoStatus> {
        let selected = self.selected_repo()?.id;
        match &self.repo_status {
            Some((id, status)) if *id == selected => status.as_ref(),
            _ => None,
        }
    }

    pub fn drift_for(&self, repo_id: RepoId) -> Option<&Candidate> {
        self.drift.iter().find(|c| c.id == repo_id)
    }

    pub fn apply_relayout(&mut self) -> Result<()> {
        if self.active_tab != Tab::Repos || self.drift.is_empty() {
            return Ok(());
        }
        let candidates = self.drift.clone();
        let (moved, skipped) = dm_core::relayout::apply_candidates(candidates)?;
        self.status = Some(if skipped.is_empty() {
            format!("moved {moved} repo(s) to match the current layout")
        } else {
            format!(
                "moved {moved} repo(s), skipped {} (target already exists)",
                skipped.len()
            )
        });
        self.reload_repos()?;
        Ok(())
    }

    pub fn start_filter(&mut self) {
        if self.active_tab == Tab::Repos {
            self.filtering = true;
        }
    }

    pub fn stop_filter(&mut self) {
        self.filtering = false;
    }

    pub fn apply_filter(&mut self) {
        self.filtered = fuzzy_filter(&self.repos, self.filter.value());
        self.selected = 0;
        self.refresh_repo_status();
    }

    pub fn selected_repo(&self) -> Option<&Repo> {
        self.filtered.get(self.selected).map(|&i| &self.repos[i])
    }

    pub fn selected_repo_workspaces(&self) -> Vec<String> {
        let Some(repo) = self.selected_repo() else {
            return Vec::new();
        };
        self.workspaces
            .workspaces_containing(repo.id)
            .map(|ws| ws.into_iter().map(|w| w.name).collect())
            .unwrap_or_default()
    }

    pub fn move_up(&mut self) {
        self.move_delta(-1);
    }

    pub fn move_down(&mut self) {
        self.move_delta(1);
    }

    fn move_delta(&mut self, delta: isize) {
        if let Some(picker) = &mut self.picker {
            move_index(&mut picker.selected, picker.filtered.len(), delta);
            return;
        }
        match self.active_tab {
            Tab::Repos => {
                move_index(&mut self.selected, self.filtered.len(), delta);
                self.refresh_repo_status();
            }
            Tab::Settings => move_index(&mut self.settings_selected, SETTINGS_KEYS.len(), delta),
            Tab::Discovery => move_index(
                &mut self.discovery_selected,
                self.found.len() + self.issues.len(),
                delta,
            ),
            Tab::Workspaces => match self.workspace_focus {
                WorkspaceFocus::List => move_index(
                    &mut self.workspace_selected,
                    self.workspaces_list.len(),
                    delta,
                ),
                WorkspaceFocus::Items => {
                    if let Some(ws) = self.selected_workspace() {
                        let count = self.workspace_detail_items(&ws.id.clone()).len();
                        move_index(&mut self.workspace_item_selected, count, delta);
                    }
                }
            },
        }
    }

    pub fn open_clone_form(&mut self) {
        if self.active_tab == Tab::Repos {
            self.form = Some(Form::clone_form());
        }
    }

    pub fn open_create_form(&mut self) {
        if self.active_tab == Tab::Repos {
            self.form = Some(Form::create_form());
        }
    }

    pub fn open_track_form(&mut self) {
        if self.active_tab == Tab::Repos {
            self.form = Some(Form::track_form());
        }
    }

    pub fn open_remove_confirm(&mut self) {
        if self.active_tab != Tab::Repos {
            return;
        }
        if let Some(repo) = self.selected_repo() {
            self.confirm = Some(ConfirmDialog {
                message: format!(
                    "untrack '{}'? (d: toggle delete-from-disk, y: confirm, n: cancel)",
                    repo.name
                ),
                action: ConfirmAction::RemoveRepo {
                    id: repo.id,
                    path: repo.path.clone(),
                    delete: false,
                },
            });
        }
    }

    pub fn reload_workspaces(&mut self) -> Result<()> {
        self.workspaces_list = self.workspaces.list()?;
        if self.workspace_selected >= self.workspaces_list.len() {
            self.workspace_selected = self.workspaces_list.len().saturating_sub(1);
        }
        Ok(())
    }

    pub fn selected_workspace(&self) -> Option<&Workspace> {
        self.workspaces_list.get(self.workspace_selected)
    }

    pub fn workspace_detail_items(&self, ws_id: &str) -> Vec<DetailItem> {
        let mut items = Vec::new();
        if let Ok(member_ids) = self.workspaces.members(ws_id) {
            for id in member_ids {
                let name = self
                    .repos
                    .iter()
                    .find(|r| r.id == id)
                    .map(|r| r.name.clone())
                    .unwrap_or_else(|| format!("#{id}"));
                items.push(DetailItem::Member(id, name));
            }
        }
        if let Ok(env) = self.workspaces.env_list(ws_id) {
            for (key, value) in env {
                items.push(DetailItem::Env(key, value));
            }
        }
        items
    }

    pub fn toggle_detail_focus(&mut self) {
        if self.active_tab != Tab::Workspaces {
            return;
        }
        self.workspace_focus = match self.workspace_focus {
            WorkspaceFocus::List => {
                self.workspace_item_selected = 0;
                WorkspaceFocus::Items
            }
            WorkspaceFocus::Items => WorkspaceFocus::List,
        };
    }

    pub fn remove_detail_item(&mut self) {
        if self.active_tab != Tab::Workspaces || self.workspace_focus != WorkspaceFocus::Items {
            return;
        }
        let Some(ws) = self.selected_workspace() else {
            return;
        };
        let ws_id = ws.id.clone();
        let items = self.workspace_detail_items(&ws_id);
        let Some(item) = items.get(self.workspace_item_selected) else {
            return;
        };
        let result = match item {
            DetailItem::Member(id, _) => self.workspaces.remove_member(&ws_id, *id),
            DetailItem::Env(key, _) => self.workspaces.env_unset(&ws_id, key),
        };
        match result {
            Ok(()) => self.status = Some("removed".to_string()),
            Err(e) => self.status = Some(format!("error: {e}")),
        }
        let new_len = self.workspace_detail_items(&ws_id).len();
        self.workspace_item_selected = if new_len == 0 {
            0
        } else {
            self.workspace_item_selected.min(new_len - 1)
        };
    }

    pub fn open_workspace_create_form(&mut self) {
        if self.active_tab == Tab::Workspaces {
            self.form = Some(Form::workspace_create_form());
        }
    }

    pub fn open_workspace_config_form(&mut self) {
        if self.active_tab != Tab::Workspaces {
            return;
        }
        if let Some(ws) = self.selected_workspace() {
            self.form = Some(Form::workspace_config_form(ws));
        }
    }

    pub fn open_workspace_env_form(&mut self) {
        if self.active_tab != Tab::Workspaces {
            return;
        }
        if let Some(ws) = self.selected_workspace() {
            let id = ws.id.clone();
            self.form = Some(Form::workspace_env_form(&id));
        }
    }

    pub fn open_workspace_delete_confirm(&mut self) {
        if self.active_tab != Tab::Workspaces {
            return;
        }
        if let Some(ws) = self.selected_workspace() {
            self.confirm = Some(ConfirmDialog {
                message: format!("delete workspace '{}'? (y: confirm, n: cancel)", ws.id),
                action: ConfirmAction::DeleteWorkspace { id: ws.id.clone() },
            });
        }
    }

    /// Requests a clean exit into `dm workspace switch`'s editor-spawn flow.
    /// The main loop notices `pending_switch` once `run()` returns and does
    /// the actual spawn after the terminal has been restored.
    pub fn request_switch(&mut self) {
        if self.active_tab != Tab::Workspaces {
            return;
        }
        if let Some(ws) = self.selected_workspace() {
            self.pending_switch = Some(ws.id.clone());
            self.should_quit = true;
        }
    }

    pub fn open_add_member_picker(&mut self) {
        if self.active_tab != Tab::Workspaces {
            return;
        }
        let Some(ws) = self.selected_workspace() else {
            return;
        };
        let ws_id = ws.id.clone();
        let member_ids = self.workspaces.members(&ws_id).unwrap_or_default();
        let candidates: Vec<Repo> = self
            .repos
            .iter()
            .filter(|r| !member_ids.contains(&r.id))
            .cloned()
            .collect();
        let filtered = (0..candidates.len()).collect();
        self.picker = Some(RepoPicker {
            workspace_id: ws_id,
            filter: Input::default(),
            candidates,
            filtered,
            selected: 0,
        });
    }

    pub fn cancel_picker(&mut self) {
        self.picker = None;
    }

    pub fn submit_picker(&mut self) {
        let Some(picker) = self.picker.take() else {
            return;
        };
        let Some(&idx) = picker.filtered.get(picker.selected) else {
            return;
        };
        let repo = &picker.candidates[idx];
        match self.workspaces.add_member(&picker.workspace_id, repo.id) {
            Ok(()) => self.status = Some(format!("added {} to workspace", repo.name)),
            Err(e) => self.status = Some(format!("error: {e}")),
        }
    }

    pub fn form_next_field(&mut self) {
        if let Some(form) = &mut self.form {
            form.next_field();
        }
    }

    pub fn form_toggle_no_git(&mut self) {
        if let Some(form) = &mut self.form
            && form.kind == FormKind::Create
        {
            form.no_git = !form.no_git;
        }
    }

    pub fn handle_text_input(&mut self, key: crossterm::event::KeyEvent) {
        let event = crossterm::event::Event::Key(key);
        if self.filtering {
            self.filter.handle_event(&event);
            self.apply_filter();
        } else if let Some(form) = &mut self.form {
            form.active_field().handle_event(&event);
        } else if let Some(picker) = &mut self.picker {
            picker.filter.handle_event(&event);
            picker.filtered = fuzzy_filter(&picker.candidates, picker.filter.value());
            picker.selected = 0;
        }
    }

    pub fn cancel_form(&mut self) {
        // The first-run wizard has no config to fall back on, so it can't be
        // dismissed, only completed.
        if matches!(self.form.as_ref().map(|f| f.kind), Some(FormKind::Setup)) {
            return;
        }
        self.form = None;
    }

    pub fn open_settings_edit(&mut self) {
        if self.active_tab != Tab::Settings {
            return;
        }
        let key = SETTINGS_KEYS[self.settings_selected];
        let value = self.config.get(key).unwrap_or_default();
        self.form = Some(Form::config_edit_form(key, &value));
    }

    pub fn settings_rows(&self) -> Vec<(&'static str, String)> {
        SETTINGS_KEYS
            .iter()
            .map(|&key| (key, self.config.get(key).unwrap_or_default()))
            .collect()
    }

    /// Scans the configured repo root for untracked repos and validates the
    /// tracked ones against disk, filling the Discovery tab.
    pub fn run_discovery(&mut self) {
        match discovery::find_untracked(&self.config.repo.root) {
            Ok(found) => self.found = found,
            Err(e) => {
                self.status = Some(format!("scan error: {e}"));
                return;
            }
        }
        match discovery::check() {
            Ok(issues) => self.issues = issues,
            Err(e) => {
                self.status = Some(format!("check error: {e}"));
                return;
            }
        }
        self.discovery_selected = 0;
        self.status = Some(format!(
            "found {} untracked, {} issue(s)",
            self.found.len(),
            self.issues.len()
        ));
    }

    pub fn discovery_track_all(&mut self) -> Result<()> {
        if self.active_tab != Tab::Discovery || self.found.is_empty() {
            return Ok(());
        }
        let found = std::mem::take(&mut self.found);
        match discovery::track_all(found) {
            Ok(n) => {
                self.status = Some(format!("tracked {n} repo(s)"));
                self.reload_repos()?;
            }
            Err(e) => self.status = Some(format!("error: {e}")),
        }
        self.run_discovery();
        Ok(())
    }

    /// Tracks the selected discovered repo, or resolves the selected issue.
    pub fn discovery_activate_selected(&mut self) -> Result<()> {
        if self.active_tab != Tab::Discovery {
            return Ok(());
        }
        let idx = self.discovery_selected;
        if idx < self.found.len() {
            let discovered = self.found.remove(idx);
            let name = discovered.name.clone();
            match discovery::track_all([discovered]) {
                Ok(_) => {
                    self.status = Some(format!("tracked {name}"));
                    self.reload_repos()?;
                }
                Err(e) => self.status = Some(format!("error: {e}")),
            }
        } else if let Some(issue) = self.issues.get(idx - self.found.len()).cloned() {
            match discovery::resolve(&issue) {
                Ok(()) => {
                    self.status = Some("resolved".to_string());
                    self.reload_repos()?;
                }
                Err(e) => self.status = Some(format!("error: {e}")),
            }
        }
        self.run_discovery();
        Ok(())
    }

    pub fn submit_form(&mut self) -> Result<()> {
        let Some(form) = self.form.take() else {
            return Ok(());
        };
        match form.kind {
            FormKind::Clone => {
                let url = form.fields[0].value().to_string();
                if url.is_empty() {
                    self.status = Some("clone: url is required".to_string());
                    return Ok(());
                }
                let path = form.field(1);
                self.busy = true;
                self.status = Some(format!("cloning {url}..."));
                self.clone_rx = Some(actions::spawn_clone(url, path));
            }
            FormKind::Create => {
                let name = form.fields[0].value().to_string();
                if name.is_empty() {
                    self.status = Some("create: name is required".to_string());
                    return Ok(());
                }
                let path = form.field(1);
                match actions::create_repo(name, path, form.no_git) {
                    Ok(name) => {
                        self.status = Some(format!("created {name}"));
                        self.reload_repos()?;
                    }
                    Err(e) => self.status = Some(format!("error: {e}")),
                }
            }
            FormKind::Track => {
                let Some(path) = form.field(0) else {
                    self.status = Some("track: path is required".to_string());
                    return Ok(());
                };
                match actions::track_repo(path) {
                    Ok(name) => {
                        self.status = Some(format!("tracked {name}"));
                        self.reload_repos()?;
                    }
                    Err(e) => self.status = Some(format!("error: {e}")),
                }
            }
            FormKind::WorkspaceCreate => {
                let id = form.fields[0].value().trim().to_string();
                let name = form.fields[1].value().trim().to_string();
                if id.is_empty() || name.is_empty() {
                    self.status = Some("workspace: id and name are required".to_string());
                    return Ok(());
                }
                let non_empty = |v: &str| (!v.is_empty()).then(|| v.to_string());
                let new_workspace = NewWorkspace {
                    id,
                    name,
                    description: non_empty(form.fields[2].value()),
                    editor: non_empty(form.fields[3].value()),
                };
                match self.workspaces.create(new_workspace) {
                    Ok(ws) => {
                        self.status = Some(format!("created workspace {}", ws.id));
                        self.reload_workspaces()?;
                    }
                    Err(e) => self.status = Some(format!("error: {e}")),
                }
            }
            FormKind::WorkspaceConfig => {
                let Some(id) = form.target.clone() else {
                    return Ok(());
                };
                let name = form.fields[0].value();
                let description = form.fields[1].value();
                let editor = form.fields[2].value();
                let result = (|| -> dm_core::error::Result<()> {
                    if !name.is_empty() {
                        self.workspaces.set_config(&id, "name", name)?;
                    }
                    self.workspaces
                        .set_config(&id, "description", description)?;
                    self.workspaces.set_config(&id, "editor", editor)?;
                    Ok(())
                })();
                match result {
                    Ok(()) => {
                        self.status = Some("workspace updated".to_string());
                        self.reload_workspaces()?;
                    }
                    Err(e) => self.status = Some(format!("error: {e}")),
                }
            }
            FormKind::WorkspaceEnv => {
                let Some(id) = form.target.clone() else {
                    return Ok(());
                };
                let key = form.fields[0].value().trim().to_string();
                let value = form.fields[1].value().to_string();
                if key.is_empty() {
                    self.status = Some("env: key is required".to_string());
                    return Ok(());
                }
                match self.workspaces.env_set(&id, &key, &value) {
                    Ok(()) => self.status = Some(format!("set env {key}")),
                    Err(e) => self.status = Some(format!("error: {e}")),
                }
            }
            FormKind::ConfigEdit => {
                let Some(key) = form.target.clone() else {
                    return Ok(());
                };
                let value = form.fields[0].value().trim();
                let result = self
                    .config
                    .set(&key, value)
                    .and_then(|()| self.config.save());
                match result {
                    Ok(()) => {
                        self.status = Some(format!("set {key}"));
                        self.reload_repos()?;
                    }
                    Err(e) => {
                        // Reload so a rejected value doesn't linger in memory.
                        self.config = Config::load().unwrap_or_default();
                        self.status = Some(format!("error: {e}"));
                    }
                }
            }
            FormKind::Setup => {
                let values: Vec<String> =
                    form.fields.iter().map(|f| f.value().to_string()).collect();
                let root = values[0].trim().to_string();

                if root.is_empty() {
                    self.status = Some("setup: project root is required".to_string());
                    self.form = Some(Form::setup_form(&values));
                    return Ok(());
                }

                let layout = match PathLayout::parse(values[1].trim()) {
                    Ok(layout) => layout,
                    Err(e) => {
                        self.status = Some(format!("setup: {e}"));
                        self.form = Some(Form::setup_form(&values));
                        return Ok(());
                    }
                };

                let editor = values[2].trim().to_string();
                let interactive = match values[3].trim() {
                    "" => true,
                    other => other.parse().unwrap_or(true),
                };

                let result = (|| -> dm_core::error::Result<usize> {
                    let mut config = Config::default();
                    config.set("repo.root", &root)?;
                    config.repo.layout = layout;
                    config.editor = (!editor.is_empty()).then(|| editor.clone());
                    config.interactive = interactive;
                    config.save()?;

                    let found = discovery::find_untracked(&config.repo.root)?;
                    discovery::track_all(found)
                })();

                match result {
                    Ok(tracked) => {
                        self.config = Config::load().unwrap_or_default();
                        self.status = Some(match tracked {
                            0 => "setup complete, no repos found to track".to_string(),
                            1 => "setup complete, tracked 1 repo already on disk".to_string(),
                            n => format!("setup complete, tracked {n} repos already on disk"),
                        });
                        self.reload_repos()?;
                    }
                    Err(e) => {
                        self.status = Some(format!("setup error: {e}"));
                        self.form = Some(Form::setup_form(&values));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn poll_clone(&mut self) -> Result<()> {
        let Some(rx) = &self.clone_rx else {
            return Ok(());
        };
        match rx.try_recv() {
            Ok(CloneOutcome::Ok(name)) => {
                self.status = Some(format!("cloned {name}"));
                self.busy = false;
                self.clone_rx = None;
                self.reload_repos()?;
            }
            Ok(CloneOutcome::Err(err)) => {
                self.status = Some(format!("clone failed: {err}"));
                self.busy = false;
                self.clone_rx = None;
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                self.busy = false;
                self.clone_rx = None;
            }
        }
        Ok(())
    }

    pub fn confirm_toggle_delete(&mut self) {
        if let Some(confirm) = &mut self.confirm
            && let ConfirmAction::RemoveRepo { delete, .. } = &mut confirm.action
        {
            *delete = !*delete;
        }
    }

    pub fn cancel_confirm(&mut self) {
        self.confirm = None;
    }

    pub fn accept_confirm(&mut self) -> Result<()> {
        let Some(confirm) = self.confirm.take() else {
            return Ok(());
        };
        match confirm.action {
            ConfirmAction::RemoveRepo { id, path, delete } => {
                match actions::remove_repo(&self.registry, id, &path, delete) {
                    Ok(()) => {
                        self.status = Some("removed".to_string());
                        self.reload_repos()?;
                    }
                    Err(e) => self.status = Some(format!("error: {e}")),
                }
            }
            ConfirmAction::DeleteWorkspace { id } => match self.workspaces.delete(&id) {
                Ok(()) => {
                    self.status = Some(format!("deleted workspace {id}"));
                    self.reload_workspaces()?;
                }
                Err(e) => self.status = Some(format!("error: {e}")),
            },
        }
        Ok(())
    }
}

/// The wizard's fields, prefilled from `config` (on a real first run, the
/// defaults). Order matches `SETUP_LABELS`.
fn setup_defaults(config: &Config) -> Vec<String> {
    vec![
        config.repo.root.display().to_string(),
        config.repo.layout.to_config_string(),
        config.editor.clone().unwrap_or_default(),
        config.interactive.to_string(),
    ]
}

fn move_index(current: &mut usize, len: usize, delta: isize) {
    if len == 0 {
        *current = 0;
        return;
    }
    let idx = (*current as isize + delta).rem_euclid(len as isize);
    *current = idx as usize;
}
