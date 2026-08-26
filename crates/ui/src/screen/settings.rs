//! The Settings screen: devmode's `config.toml`, with a live preview of what
//! the chosen layout actually does to a path.

use iced::widget::{checkbox, column, pick_list, row, space, text, text_input};
use iced::{Center, Element, Fill, Task};

use dm_core::config::Config;
use dm_core::layout::PathLayout;

use crate::app::{self, App, Message as AppMessage};
use crate::design::{self, Tone};

/// The layout choices offered in the picker. `Custom` carries its template in
/// a separate field so switching away and back doesn't lose what was typed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutChoice {
    HostOwnerRepo,
    OwnerRepo,
    Flat,
    Custom,
}

impl LayoutChoice {
    const ALL: [LayoutChoice; 4] = [
        LayoutChoice::HostOwnerRepo,
        LayoutChoice::OwnerRepo,
        LayoutChoice::Flat,
        LayoutChoice::Custom,
    ];

    fn from_layout(layout: &PathLayout) -> Self {
        match layout {
            PathLayout::HostOwnerRepo => LayoutChoice::HostOwnerRepo,
            PathLayout::OwnerRepo => LayoutChoice::OwnerRepo,
            PathLayout::Flat => LayoutChoice::Flat,
            PathLayout::Custom { .. } => LayoutChoice::Custom,
        }
    }
}

impl std::fmt::Display for LayoutChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            LayoutChoice::HostOwnerRepo => "host / owner / repo",
            LayoutChoice::OwnerRepo => "owner / repo",
            LayoutChoice::Flat => "repo",
            LayoutChoice::Custom => "Custom template…",
        };

        f.write_str(label)
    }
}

#[derive(Debug, Default)]
pub struct State {
    pub root: String,
    pub host: String,
    pub editor: String,
    pub interactive: bool,
    pub layout: Option<LayoutChoice>,
    pub template: String,
    /// The config as last loaded, to detect unsaved edits.
    saved: Option<Config>,
}

impl State {
    /// Adopts freshly loaded config, unless the user has unsaved edits — a
    /// background reload shouldn't discard something half-typed.
    pub fn sync_from(&mut self, config: &Config) {
        if self.saved.is_some() && self.is_dirty() {
            return;
        }

        self.root = config.repo.root.display().to_string();
        self.host = config.repo.host.clone();
        self.editor = config.editor.clone().unwrap_or_default();
        self.interactive = config.interactive;
        self.layout = Some(LayoutChoice::from_layout(&config.repo.layout));
        self.template = match &config.repo.layout {
            PathLayout::Custom { template } => template.clone(),
            other => other.to_config_string(),
        };
        self.saved = Some(config.clone());
    }

    pub fn is_dirty(&self) -> bool {
        let Some(saved) = &self.saved else {
            return false;
        };

        self.root != saved.repo.root.display().to_string()
            || self.host != saved.repo.host
            || self.editor != saved.editor.clone().unwrap_or_default()
            || self.interactive != saved.interactive
            || self.layout != Some(LayoutChoice::from_layout(&saved.repo.layout))
            || (self.layout == Some(LayoutChoice::Custom)
                && !matches!(
                    &saved.repo.layout,
                    PathLayout::Custom { template } if template == &self.template
                ))
    }

    /// The layout the form currently describes, or an error to show inline.
    fn current_layout(&self) -> Result<PathLayout, String> {
        match self.layout {
            Some(LayoutChoice::HostOwnerRepo) | None => Ok(PathLayout::HostOwnerRepo),
            Some(LayoutChoice::OwnerRepo) => Ok(PathLayout::OwnerRepo),
            Some(LayoutChoice::Flat) => Ok(PathLayout::Flat),
            Some(LayoutChoice::Custom) => {
                let template = self.template.trim();

                if template.is_empty() {
                    return Err("A custom layout needs a template.".to_string());
                }

                Ok(PathLayout::Custom {
                    template: template.to_string(),
                })
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    RootChanged(String),
    HostChanged(String),
    EditorChanged(String),
    InteractiveChanged(bool),
    LayoutChanged(LayoutChoice),
    TemplateChanged(String),
    Save,
    Revert,
    FixDrift,
}

pub fn update(app: &mut App, message: Message) -> Task<AppMessage> {
    match message {
        Message::RootChanged(root) => {
            app.settings.root = root;
            Task::none()
        }
        Message::HostChanged(host) => {
            app.settings.host = host;
            Task::none()
        }
        Message::EditorChanged(editor) => {
            app.settings.editor = editor;
            Task::none()
        }
        Message::InteractiveChanged(interactive) => {
            app.settings.interactive = interactive;
            Task::none()
        }
        Message::LayoutChanged(choice) => {
            app.settings.layout = Some(choice);
            Task::none()
        }
        Message::TemplateChanged(template) => {
            app.settings.template = template;
            Task::none()
        }
        Message::Revert => {
            // Drop the edits, then adopt the saved config again.
            let saved = app.settings.saved.clone();
            app.settings.saved = None;

            if let Some(config) = saved {
                app.settings.sync_from(&config);
            }

            Task::none()
        }
        Message::Save => {
            let layout = match app.settings.current_layout() {
                Ok(layout) => layout,
                Err(error) => {
                    app.toast_error(error);
                    return Task::none();
                }
            };

            let (root, host, editor, interactive) = (
                app.settings.root.clone(),
                app.settings.host.clone(),
                app.settings.editor.clone(),
                app.settings.interactive,
            );

            // Let the reload re-adopt the saved values.
            app.settings.saved = None;

            app.run(move || save(root, host, editor, interactive, layout))
        }
        Message::FixDrift => app.run(|| {
            let (moved, skipped) = dm_core::relayout::apply_candidates(
                dm_core::relayout::plan().map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;

            Ok(if skipped.is_empty() {
                format!("Moved {moved} repo(s) to match your layout.")
            } else {
                format!(
                    "Moved {moved} repo(s); skipped {} because the target already exists.",
                    skipped.len()
                )
            })
        }),
    }
}

fn save(
    root: String,
    host: String,
    editor: String,
    interactive: bool,
    layout: PathLayout,
) -> Result<String, String> {
    (|| -> dm_core::Result<String> {
        let mut config = Config::load()?;

        config.set("repo.root", root.trim())?;
        config.set("repo.host", host.trim())?;
        config.set("repo.layout", &layout.to_config_string())?;
        config.set("interactive", &interactive.to_string())?;

        // `Config::set` has no way to clear the editor back to unset, so
        // assign the field directly for the empty case.
        let editor = editor.trim();
        config.editor = (!editor.is_empty()).then(|| editor.to_string());

        config.save()?;

        Ok("Settings saved.".to_string())
    })()
    .map_err(|e| e.to_string())
}

// -- view ---------------------------------------------------------------------

pub fn view(app: &App) -> Element<'_, AppMessage> {
    let state = &app.settings;

    let mut body = column![
        design::page_header(
            "Settings",
            Some("These are the same values `dm config` reads and writes.".to_string()),
            None,
        ),
        repo_section(app),
        layout_section(app),
        behaviour_section(app),
    ];

    if state.is_dirty() {
        body = body.push(design::button_row(vec![
            design::small_button("Revert", wrap(Message::Revert)),
            app::primary_button("Save changes", wrap(Message::Save)),
        ]));
    }

    design::page(body)
}

fn repo_section(app: &App) -> Element<'_, AppMessage> {
    design::section(
        "Repositories",
        column![
            input(
                "Repo root",
                "Where clones and new repos land",
                &app.settings.root,
                Message::RootChanged,
            ),
            input(
                "Default host",
                "github.com",
                &app.settings.host,
                Message::HostChanged,
            ),
        ]
        .spacing(design::MD),
    )
}

fn layout_section(app: &App) -> Element<'_, AppMessage> {
    let state = &app.settings;

    let picker = design::field(
        "Folder layout",
        pick_list(LayoutChoice::ALL, state.layout, |choice| {
            wrap(Message::LayoutChanged(choice))
        })
        .padding(design::SM)
        .text_size(design::TEXT_MD)
        .width(Fill)
        .into(),
    );

    let mut body = column![
        design::muted(
            text("Where a repo lands under your repo root.").size(design::TEXT_SM)
        ),
        picker,
    ]
    .spacing(design::MD);

    if state.layout == Some(LayoutChoice::Custom) {
        body = body.push(input(
            "Template",
            "{host}/{owner}/{repo}",
            &state.template,
            Message::TemplateChanged,
        ));
    }

    // A worked example beats explaining the placeholders in prose.
    body = body.push(match state.current_layout() {
        Ok(layout) => design::mono_field(
            "Example",
            format!(
                "{}/{}",
                state.root.trim_end_matches('/'),
                layout
                    .render("github.com", "torvalds", "linux")
                    .display()
            ),
        ),
        Err(error) => row![
            design::badge("Invalid", Tone::Danger),
            text(error).size(design::TEXT_SM),
        ]
        .spacing(design::SM)
        .align_y(Center)
        .into(),
    });

    // Changing the layout never moves anything on its own; surface the drift
    // it creates and offer the fix, exactly as `dm config set` does.
    if let Some(snapshot) = app.snapshot() {
        if !snapshot.drift.is_empty() {
            body = body.push(
                row![
                    design::badge("Layout", Tone::Warning),
                    text(format!(
                        "{} tracked repo(s) don't match this layout.",
                        snapshot.drift.len()
                    ))
                    .size(design::TEXT_SM),
                    space::horizontal(),
                    design::small_button("Move them", wrap(Message::FixDrift)),
                ]
                .spacing(design::SM)
                .align_y(Center),
            );
        }
    }

    design::section("Layout", body)
}

fn behaviour_section(app: &App) -> Element<'_, AppMessage> {
    design::section(
        "Behaviour",
        column![
            input(
                "Editor",
                "code -n",
                &app.settings.editor,
                Message::EditorChanged,
            ),
            design::muted(
                text(
                    "Used to open workspaces that don't set their own editor. \
                     Leave blank for none."
                )
                .size(design::TEXT_SM)
            ),
            checkbox(app.settings.interactive)
                .label("Interactive prompts in the CLI")
                .text_size(design::TEXT_MD)
                .on_toggle(|value| wrap(Message::InteractiveChanged(value))),
            design::muted(
                text(
                    "Turn this off to make `dm` script- and pipe-friendly: it \
                     falls back to plain stdin prompts instead of ones that \
                     need a terminal."
                )
                .size(design::TEXT_SM)
            ),
        ]
        .spacing(design::SM),
    )
}

fn input<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, AppMessage> {
    design::field(
        label,
        text_input(placeholder, value)
            .on_input(move |value| wrap(on_input(value)))
            .on_submit(wrap(Message::Save))
            .padding(design::SM)
            .size(design::TEXT_MD)
            .font(design::MONO)
            .width(Fill)
            .into(),
    )
}

fn wrap(message: Message) -> AppMessage {
    AppMessage::Settings(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_from(config: &Config) -> State {
        let mut state = State::default();
        state.sync_from(config);
        state
    }

    #[test]
    fn adopts_config_without_edits() {
        let config = Config::default();
        let state = state_from(&config);

        assert!(!state.is_dirty());
        assert_eq!(state.layout, Some(LayoutChoice::HostOwnerRepo));
        assert_eq!(state.host, config.repo.host);
        assert_eq!(state.interactive, config.interactive);
    }

    #[test]
    fn detects_edits() {
        let mut state = state_from(&Config::default());
        state.host = "gitlab.com".to_string();

        assert!(state.is_dirty());
    }

    #[test]
    fn a_reload_does_not_discard_unsaved_edits() {
        let mut state = state_from(&Config::default());
        state.editor = "hx".to_string();

        // A background reload lands while the user is mid-edit.
        state.sync_from(&Config::default());

        assert_eq!(state.editor, "hx");
        assert!(state.is_dirty());
    }

    #[test]
    fn custom_layout_requires_a_template() {
        let mut state = state_from(&Config::default());
        state.layout = Some(LayoutChoice::Custom);
        state.template = "   ".to_string();

        assert!(state.current_layout().is_err());

        state.template = "{owner}/{repo}".to_string();
        assert_eq!(
            state.current_layout().unwrap(),
            PathLayout::Custom {
                template: "{owner}/{repo}".to_string()
            }
        );
    }

    #[test]
    fn switching_layout_is_an_edit() {
        let mut state = state_from(&Config::default());
        state.layout = Some(LayoutChoice::Flat);

        assert!(state.is_dirty());
        assert_eq!(state.current_layout().unwrap(), PathLayout::Flat);
    }
}
