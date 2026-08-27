//! The Settings screen: devmode's `config.toml`, with a live preview of what
//! the chosen layout actually does to a path.

use std::path::PathBuf;

use iced::widget::{checkbox, column, pick_list, row, space, text};
use iced::{Center, Element, Fill, Task, Theme};

use dm_core::config::{Config, ThemeMode};
use dm_core::layout::PathLayout;

use crate::app::{App, Message as AppMessage};
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
    pub(crate) const ALL: [LayoutChoice; 4] = [
        LayoutChoice::HostOwnerRepo,
        LayoutChoice::OwnerRepo,
        LayoutChoice::Flat,
        LayoutChoice::Custom,
    ];

    pub(crate) fn from_layout(layout: &PathLayout) -> Self {
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
    pub editor: String,
    pub interactive: bool,
    pub layout: Option<LayoutChoice>,
    pub template: String,
    pub theme_mode: ThemeMode,
    pub light_theme: String,
    pub dark_theme: String,
    /// The config as last loaded, to detect unsaved edits.
    saved: Option<Config>,
}

impl State {
    /// Adopts freshly loaded config, unless the user has unsaved edits, a
    /// background reload shouldn't discard something half-typed.
    pub fn sync_from(&mut self, config: &Config) {
        if self.saved.is_some() && self.is_dirty() {
            return;
        }

        self.root = config.repo.root.display().to_string();
        self.editor = config.editor.clone().unwrap_or_default();
        self.interactive = config.interactive;
        self.layout = Some(LayoutChoice::from_layout(&config.repo.layout));
        self.template = match &config.repo.layout {
            PathLayout::Custom { template } => template.clone(),
            other => other.to_config_string(),
        };
        self.theme_mode = config.ui.theme_mode;
        self.light_theme = config.ui.light_theme.clone();
        self.dark_theme = config.ui.dark_theme.clone();
        self.saved = Some(config.clone());
    }

    pub fn is_dirty(&self) -> bool {
        let Some(saved) = &self.saved else {
            return false;
        };

        self.root != saved.repo.root.display().to_string()
            || self.editor != saved.editor.clone().unwrap_or_default()
            || self.interactive != saved.interactive
            || self.layout != Some(LayoutChoice::from_layout(&saved.repo.layout))
            || self.theme_mode != saved.ui.theme_mode
            || self.light_theme != saved.ui.light_theme
            || self.dark_theme != saved.ui.dark_theme
            || (self.layout == Some(LayoutChoice::Custom)
                && !matches!(
                    &saved.repo.layout,
                    PathLayout::Custom { template } if template == &self.template
                ))
    }

    /// The layout the form currently describes, or an error to show inline.
    pub(crate) fn current_layout(&self) -> Result<PathLayout, String> {
        resolve_layout(
            self.layout.unwrap_or(LayoutChoice::HostOwnerRepo),
            &self.template,
        )
    }
}

/// Turns a picker choice plus its (only relevant for `Custom`) template into a
/// `PathLayout`, or an error to show inline.
pub(crate) fn resolve_layout(choice: LayoutChoice, template: &str) -> Result<PathLayout, String> {
    match choice {
        LayoutChoice::HostOwnerRepo => Ok(PathLayout::HostOwnerRepo),
        LayoutChoice::OwnerRepo => Ok(PathLayout::OwnerRepo),
        LayoutChoice::Flat => Ok(PathLayout::Flat),
        LayoutChoice::Custom => {
            let template = template.trim();

            if template.is_empty() {
                return Err("A custom layout needs a template.".to_string());
            }

            Ok(PathLayout::Custom {
                template: template.to_string(),
            })
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    RootChanged(String),
    BrowseRoot,
    RootPicked(Option<PathBuf>),
    EditorChanged(String),
    InteractiveChanged(bool),
    LayoutChanged(LayoutChoice),
    TemplateChanged(String),
    ThemeModeChanged(ThemeMode),
    LightThemeChanged(Theme),
    DarkThemeChanged(Theme),
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
        Message::BrowseRoot => {
            let current = app.settings.root.trim();
            let starting = (!current.is_empty()).then(|| PathBuf::from(current));

            Task::perform(
                crate::task::pick_folder("Choose a folder", starting),
                |picked| wrap(Message::RootPicked(picked)),
            )
        }
        Message::RootPicked(Some(picked)) => {
            app.settings.root = picked.display().to_string();
            Task::none()
        }
        Message::RootPicked(None) => Task::none(),
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
        Message::ThemeModeChanged(mode) => {
            app.settings.theme_mode = mode;
            Task::none()
        }
        Message::LightThemeChanged(theme) => {
            app.settings.light_theme = theme.to_string();
            Task::none()
        }
        Message::DarkThemeChanged(theme) => {
            app.settings.dark_theme = theme.to_string();
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

            let (root, editor, interactive) = (
                app.settings.root.clone(),
                app.settings.editor.clone(),
                app.settings.interactive,
            );
            let appearance = (
                app.settings.theme_mode,
                app.settings.light_theme.clone(),
                app.settings.dark_theme.clone(),
            );

            // Let the reload re-adopt the saved values.
            app.settings.saved = None;

            app.run("Saving settings…", move || {
                save(root, editor, interactive, layout, appearance)
            })
        }
        Message::FixDrift => app.run("Moving repos to match layout…", || {
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
    editor: String,
    interactive: bool,
    layout: PathLayout,
    appearance: (ThemeMode, String, String),
) -> Result<String, String> {
    (|| -> dm_core::Result<String> {
        let mut config = Config::load()?;

        config.set("repo.root", root.trim())?;
        config.set("repo.layout", &layout.to_config_string())?;
        config.set("interactive", &interactive.to_string())?;

        let (theme_mode, light_theme, dark_theme) = appearance;
        config.set("ui.theme_mode", theme_mode.as_str())?;
        config.set("ui.light_theme", &light_theme)?;
        config.set("ui.dark_theme", &dark_theme)?;

        // `Config::set` has no way to clear the editor back to unset, so
        // assign the field directly for the empty case.
        let editor = editor.trim();
        config.editor = (!editor.is_empty()).then(|| editor.to_string());

        config.save()?;

        Ok("Settings saved.".to_string())
    })()
    .map_err(|e| e.to_string())
}

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
        appearance_section(app),
        behaviour_section(app),
    ];

    if state.is_dirty() {
        body = body.push(design::button_row(vec![
            design::secondary_button("Revert", wrap(Message::Revert)),
            design::primary_button("Save changes", wrap(Message::Save)),
        ]));
    }

    design::page(body)
}

fn repo_section(app: &App) -> Element<'_, AppMessage> {
    design::section(
        "Repositories",
        path_input(
            "Repo root",
            "Where clones and new repos land",
            &app.settings.root,
            Message::RootChanged,
            Message::BrowseRoot,
        ),
    )
}

fn layout_section(app: &App) -> Element<'_, AppMessage> {
    let state = &app.settings;

    let picker = design::field(
        "Folder layout",
        pick_list(LayoutChoice::ALL, state.layout, |choice| {
            wrap(Message::LayoutChanged(choice))
        })
        .padding(design::CONTROL_PADDING)
        .text_size(design::CONTROL_TEXT)
        .width(Fill)
        .into(),
    );

    let mut body = column![
        design::muted(text("Where a repo lands under your repo root.").size(design::TEXT_SM)),
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
                layout.render("github.com", "torvalds", "linux").display()
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
    if let Some(snapshot) = app.snapshot()
        && !snapshot.drift.is_empty()
    {
        body = body.push(
            row![
                design::badge("Layout", Tone::Warning),
                text(format!(
                    "{} tracked repo(s) don't match this layout.",
                    snapshot.drift.len()
                ))
                .size(design::TEXT_SM),
                space::horizontal(),
                design::secondary_button("Move them", wrap(Message::FixDrift)),
            ]
            .spacing(design::SM)
            .align_y(Center),
        );
    }

    design::section("Layout", body)
}

/// The built-in theme matching `name`, if there is one.
///
/// Theme names live in the config as plain strings, so an unrecognised one
/// (a hand-edited typo, or a theme from a newer iced) resolves to `None` and
/// the caller falls back rather than failing to start.
pub fn theme_named(name: &str) -> Option<Theme> {
    Theme::ALL
        .iter()
        .find(|theme| theme.to_string() == name)
        .cloned()
}

/// The built-in themes with light palettes, and the ones with dark palettes.
///
/// Splitting them means each picker only offers themes that make sense for
/// the slot, so "follow system" can't end up showing a dark theme in light
/// mode because of a mismatched pick.
pub(crate) fn themes_for(mode: iced::theme::Mode) -> Vec<Theme> {
    use iced::theme::Base;

    Theme::ALL
        .iter()
        .filter(|theme| theme.mode() == mode)
        .cloned()
        .collect()
}

fn appearance_section(app: &App) -> Element<'_, AppMessage> {
    let state = &app.settings;

    let mode = design::field(
        "Theme",
        pick_list(ThemeMode::ALL, Some(state.theme_mode), |mode| {
            wrap(Message::ThemeModeChanged(mode))
        })
        .padding(design::CONTROL_PADDING)
        .text_size(design::CONTROL_TEXT)
        .width(Fill)
        .into(),
    );

    let variant = |label: &'static str,
                   selected: &str,
                   for_mode: iced::theme::Mode,
                   to_message: fn(Theme) -> Message| {
        design::field(
            label,
            pick_list(themes_for(for_mode), theme_named(selected), move |theme| {
                wrap(to_message(theme))
            })
            .padding(design::CONTROL_PADDING)
            .text_size(design::CONTROL_TEXT)
            .width(Fill)
            .into(),
        )
    };

    let hint = match state.theme_mode {
        ThemeMode::System => {
            "Devmode follows your desktop's light and dark setting, switching \
             between these two themes as it changes."
        }
        ThemeMode::Light => "Devmode always uses the light theme below.",
        ThemeMode::Dark => "Devmode always uses the dark theme below.",
    };

    design::section(
        "Appearance",
        column![
            design::muted(text(hint).size(design::TEXT_SM)),
            mode,
            row![
                variant(
                    "Light theme",
                    &state.light_theme,
                    iced::theme::Mode::Light,
                    Message::LightThemeChanged,
                ),
                variant(
                    "Dark theme",
                    &state.dark_theme,
                    iced::theme::Mode::Dark,
                    Message::DarkThemeChanged,
                ),
            ]
            .spacing(design::MD),
            design::muted(
                text("Changes preview straight away; Save keeps them.").size(design::TEXT_SM)
            ),
        ]
        .spacing(design::MD),
    )
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
                .text_size(design::CONTROL_TEXT)
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
        design::input(placeholder, value)
            .on_input(move |value| wrap(on_input(value)))
            .on_submit(wrap(Message::Save))
            .font(design::MONO)
            .width(Fill)
            .into(),
    )
}

fn path_input<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
    on_browse: Message,
) -> Element<'a, AppMessage> {
    design::field(
        label,
        row![
            design::input(placeholder, value)
                .on_input(move |value| wrap(on_input(value)))
                .on_submit(wrap(Message::Save))
                .font(design::MONO)
                .width(Fill),
            design::secondary_button("Browse…", wrap(on_browse)),
        ]
        .spacing(design::SM)
        .align_y(Center)
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
        assert_eq!(state.root, config.repo.root.display().to_string());
        assert_eq!(state.interactive, config.interactive);
    }

    #[test]
    fn detects_edits() {
        let mut state = state_from(&Config::default());
        state.root = "/somewhere/else".to_string();

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
