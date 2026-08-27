//! First-run setup.
//!
//! Shown once, while there's no config file yet: a short wizard for the repo
//! root, folder layout, editor, and theme. Finishing writes the config and
//! runs a single discovery pass over the chosen root, tracking everything it
//! finds. After that there's a config file, so this never runs again.

use std::path::PathBuf;

use iced::widget::{column, container, pick_list, row, space, text};
use iced::{Center, Element, Fill, Task, Theme};

use dm_core::config::{Config, ThemeMode};
use dm_core::discovery;
use dm_core::layout::PathLayout;

use crate::app::{App, Message as AppMessage, Screen};
use crate::design::{self, Tone};
use crate::screen::settings::{self, LayoutChoice};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Location,
    Editor,
    Appearance,
}

impl Step {
    const ORDER: [Step; 3] = [Step::Location, Step::Editor, Step::Appearance];

    fn index(self) -> usize {
        Self::ORDER.iter().position(|step| *step == self).unwrap()
    }

    fn next(self) -> Option<Step> {
        Self::ORDER.get(self.index() + 1).copied()
    }

    fn prev(self) -> Option<Step> {
        self.index().checked_sub(1).map(|i| Self::ORDER[i])
    }

    fn title(self) -> &'static str {
        match self {
            Step::Location => "Where your projects live",
            Step::Editor => "Your editor",
            Step::Appearance => "Appearance",
        }
    }
}

#[derive(Debug)]
pub struct State {
    step: Step,
    pub root: String,
    pub layout: LayoutChoice,
    pub template: String,
    pub editor: String,
    pub theme_mode: ThemeMode,
    pub light_theme: String,
    pub dark_theme: String,
}

impl State {
    /// Seeded from `config`, which on a genuine first run is `Config::default`.
    pub fn new(config: &Config) -> Self {
        Self {
            step: Step::Location,
            root: config.repo.root.display().to_string(),
            layout: LayoutChoice::from_layout(&config.repo.layout),
            template: match &config.repo.layout {
                PathLayout::Custom { template } => template.clone(),
                other => other.to_config_string(),
            },
            editor: config.editor.clone().unwrap_or_default(),
            theme_mode: config.ui.theme_mode,
            light_theme: config.ui.light_theme.clone(),
            dark_theme: config.ui.dark_theme.clone(),
        }
    }

    fn layout(&self) -> Result<PathLayout, String> {
        settings::resolve_layout(self.layout, &self.template)
    }

    /// Whether the current step is complete enough to advance.
    fn step_ready(&self) -> bool {
        match self.step {
            Step::Location => !self.root.trim().is_empty() && self.layout().is_ok(),
            Step::Editor | Step::Appearance => true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Back,
    Next,
    RootChanged(String),
    BrowseRoot,
    RootPicked(Option<PathBuf>),
    LayoutChanged(LayoutChoice),
    TemplateChanged(String),
    EditorChanged(String),
    ThemeModeChanged(ThemeMode),
    LightThemeChanged(Theme),
    DarkThemeChanged(Theme),
    Finish,
}

pub fn update(app: &mut App, message: Message) -> Task<AppMessage> {
    let Some(state) = app.setup.as_mut() else {
        return Task::none();
    };

    match message {
        Message::Back => {
            if let Some(prev) = state.step.prev() {
                state.step = prev;
            }
            Task::none()
        }
        Message::Next => {
            if state.step_ready()
                && let Some(next) = state.step.next()
            {
                state.step = next;
            }
            Task::none()
        }
        Message::RootChanged(root) => {
            state.root = root;
            Task::none()
        }
        Message::BrowseRoot => {
            let current = state.root.trim();
            let starting = (!current.is_empty()).then(|| PathBuf::from(current));

            Task::perform(
                crate::task::pick_folder("Choose a folder", starting),
                |picked| wrap(Message::RootPicked(picked)),
            )
        }
        Message::RootPicked(Some(picked)) => {
            state.root = picked.display().to_string();
            Task::none()
        }
        Message::RootPicked(None) => Task::none(),
        Message::LayoutChanged(choice) => {
            state.layout = choice;
            Task::none()
        }
        Message::TemplateChanged(template) => {
            state.template = template;
            Task::none()
        }
        Message::EditorChanged(editor) => {
            state.editor = editor;
            Task::none()
        }
        Message::ThemeModeChanged(mode) => {
            state.theme_mode = mode;
            Task::none()
        }
        Message::LightThemeChanged(theme) => {
            state.light_theme = theme.to_string();
            Task::none()
        }
        Message::DarkThemeChanged(theme) => {
            state.dark_theme = theme.to_string();
            Task::none()
        }
        Message::Finish => {
            let layout = match state.layout() {
                Ok(layout) => layout,
                Err(error) => {
                    app.toast_error(error);
                    return Task::none();
                }
            };

            let root = state.root.trim().to_string();
            let editor = state.editor.trim().to_string();
            let appearance = (
                state.theme_mode,
                state.light_theme.clone(),
                state.dark_theme.clone(),
            );

            app.setup = None;
            app.screen = Screen::Repos;

            app.run("Setting up devmode…", move || {
                finish(root, editor, layout, appearance)
            })
        }
    }
}

/// Writes the config, then scans the chosen root once and tracks whatever it
/// finds.
fn finish(
    root: String,
    editor: String,
    layout: PathLayout,
    appearance: (ThemeMode, String, String),
) -> Result<String, String> {
    (|| -> dm_core::Result<String> {
        let mut config = Config::load()?;

        config.set("repo.root", &root)?;
        config.set("repo.layout", &layout.to_config_string())?;
        config.editor = (!editor.is_empty()).then(|| editor.clone());

        let (theme_mode, light_theme, dark_theme) = appearance;
        config.set("ui.theme_mode", theme_mode.as_str())?;
        config.set("ui.light_theme", &light_theme)?;
        config.set("ui.dark_theme", &dark_theme)?;

        config.save()?;

        let found = discovery::find_untracked(&config.repo.root)?;
        let tracked = discovery::track_all(found)?;

        Ok(match tracked {
            0 => "Setup complete. No repos found to track yet.".to_string(),
            1 => "Setup complete. Tracked 1 repo already on disk.".to_string(),
            n => format!("Setup complete. Tracked {n} repos already on disk."),
        })
    })()
    .map_err(|e| e.to_string())
}

pub fn view(state: &State) -> Element<'_, AppMessage> {
    let steps = Step::ORDER.len();
    let progress = format!("Step {} of {}", state.step.index() + 1, steps);

    let body = match state.step {
        Step::Location => location_step(state),
        Step::Editor => editor_step(state),
        Step::Appearance => appearance_step(state),
    };

    let mut actions = row![].spacing(design::SM);
    if state.step.prev().is_some() {
        actions = actions.push(design::secondary_button("Back", wrap(Message::Back)));
    }
    actions = actions.push(space::horizontal());
    actions = actions.push(if state.step.next().is_some() {
        design::primary_button("Next", state.step_ready().then(|| wrap(Message::Next)))
    } else {
        design::primary_button("Finish", state.step_ready().then(|| wrap(Message::Finish)))
    });

    let card = design::section(
        state.step.title(),
        column![
            design::muted(text(progress).size(design::TEXT_SM)),
            body,
            actions,
        ]
        .spacing(design::LG),
    );

    container(
        column![
            text("Welcome to devmode").size(design::TEXT_XL),
            design::muted(
                text(
                    "A few choices and you're set. You can change any of these later in Settings."
                )
                .size(design::TEXT_SM)
            ),
            card,
        ]
        .spacing(design::MD)
        .max_width(560),
    )
    .width(Fill)
    .height(Fill)
    .center_x(Fill)
    .padding(design::XL)
    .into()
}

fn location_step(state: &State) -> Element<'_, AppMessage> {
    let root = design::field(
        "Project root",
        row![
            design::input("/home/you/Developer", &state.root)
                .on_input(|value| wrap(Message::RootChanged(value)))
                .on_submit(wrap(Message::Next))
                .font(design::MONO)
                .width(Fill),
            design::secondary_button("Browse…", wrap(Message::BrowseRoot)),
        ]
        .spacing(design::SM)
        .align_y(Center)
        .into(),
    );

    let layout = design::field(
        "Folder layout",
        pick_list(LayoutChoice::ALL, Some(state.layout), |choice| {
            wrap(Message::LayoutChanged(choice))
        })
        .padding(design::CONTROL_PADDING)
        .text_size(design::CONTROL_TEXT)
        .width(Fill)
        .into(),
    );

    let mut body = column![
        design::muted(
            text("Where devmode keeps every repo it clones or discovers.").size(design::TEXT_SM)
        ),
        root,
        layout,
    ]
    .spacing(design::MD);

    if state.layout == LayoutChoice::Custom {
        body = body.push(design::field(
            "Template",
            design::input("{host}/{owner}/{repo}", &state.template)
                .on_input(|value| wrap(Message::TemplateChanged(value)))
                .font(design::MONO)
                .width(Fill)
                .into(),
        ));
    }

    body = body.push(match state.layout() {
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

    body.into()
}

fn editor_step(state: &State) -> Element<'_, AppMessage> {
    column![
        design::muted(
            text(
                "The command devmode runs to open a workspace's repos. Leave it \
                 blank to skip, workspaces can set their own."
            )
            .size(design::TEXT_SM)
        ),
        design::field(
            "Editor command",
            design::input("code -n", &state.editor)
                .on_input(|value| wrap(Message::EditorChanged(value)))
                .on_submit(wrap(Message::Next))
                .font(design::MONO)
                .width(Fill)
                .into(),
        ),
    ]
    .spacing(design::MD)
    .into()
}

fn appearance_step(state: &State) -> Element<'_, AppMessage> {
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
            pick_list(
                settings::themes_for(for_mode),
                settings::theme_named(selected),
                move |theme| wrap(to_message(theme)),
            )
            .padding(design::CONTROL_PADDING)
            .text_size(design::CONTROL_TEXT)
            .width(Fill)
            .into(),
        )
    };

    let hint = match state.theme_mode {
        ThemeMode::System => {
            "Devmode follows your desktop's light and dark setting, using each \
             theme below in turn."
        }
        ThemeMode::Light => "Devmode always uses the light theme below.",
        ThemeMode::Dark => "Devmode always uses the dark theme below.",
    };

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
        design::muted(text("The window updates as you pick.").size(design::TEXT_SM)),
    ]
    .spacing(design::MD)
    .into()
}

fn wrap(message: Message) -> AppMessage {
    AppMessage::Setup(message)
}
