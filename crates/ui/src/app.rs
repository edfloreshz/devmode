//! The application shell: navigation, global state, and the message loop.

use std::time::{Duration, SystemTime};

use iced::widget::{column, container, row, rule, space, text};
use iced::{Center, Element, Fill, Subscription, Task, Theme};

use dm_core::config::ThemeMode;

use crate::data::{self, Snapshot, WorkspaceDetail};
use crate::design::{self, Tone};
use crate::screen::{self, discovery, repos, settings, workspaces};
use crate::task::blocking;

/// `modifiers.command()` is Cmd on macOS and Ctrl elsewhere, so the hint
/// text has to follow suit.
#[cfg(target_os = "macos")]
const MODIFIER: &str = "⌘";
#[cfg(not(target_os = "macos"))]
const MODIFIER: &str = "Ctrl+";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Screen {
    Repos,
    Workspaces,
    Discovery,
    Settings,
}

impl Screen {
    pub fn title(self) -> &'static str {
        match self {
            Screen::Repos => "Repos",
            Screen::Workspaces => "Workspaces",
            Screen::Discovery => "Discovery",
            Screen::Settings => "Settings",
        }
    }
}

/// A transient message in the bottom bar. Successes fade on their own;
/// failures stay until dismissed, because an error the user never saw is
/// worse than one that lingers.
#[derive(Debug, Clone)]
pub struct Toast {
    pub body: String,
    pub tone: Tone,
    pub shown_at: SystemTime,
}

impl Toast {
    fn success(body: impl Into<String>) -> Self {
        Self {
            body: body.into(),
            tone: Tone::Success,
            shown_at: SystemTime::now(),
        }
    }

    fn error(body: impl Into<String>) -> Self {
        Self {
            body: body.into(),
            tone: Tone::Danger,
            shown_at: SystemTime::now(),
        }
    }

    fn is_transient(&self) -> bool {
        self.tone != Tone::Danger
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Navigate(Screen),
    Reload,
    Loaded(Result<Snapshot, String>),
    WorkspaceDetailLoaded(Result<WorkspaceDetail, String>),
    /// A completed mutation: a message to show, then a reload.
    Completed(Result<String, String>),
    DismissToast,
    ToastTick,
    StateChanged(Option<(SystemTime, SystemTime)>),
    SystemTheme(iced::theme::Mode),
    Repos(repos::Message),
    Workspaces(workspaces::Message),
    Discovery(discovery::Message),
    Settings(settings::Message),
}

pub struct App {
    screen: Screen,
    snapshot: Option<Snapshot>,
    toast: Option<Toast>,
    loading: bool,
    fingerprint: Option<(SystemTime, SystemTime)>,
    /// The desktop's current preference, kept live by `theme_changes`.
    system_mode: iced::theme::Mode,
    pub repos: repos::State,
    pub workspaces: workspaces::State,
    pub discovery: discovery::State,
    pub settings: settings::State,
}

impl App {
    pub fn boot() -> (Self, Task<Message>) {
        let app = Self {
            screen: Screen::Repos,
            snapshot: None,
            toast: None,
            loading: true,
            fingerprint: None,
            system_mode: iced::theme::Mode::default(),
            repos: repos::State::default(),
            workspaces: workspaces::State::default(),
            discovery: discovery::State::default(),
            settings: settings::State::default(),
        };

        let boot = Task::batch([
            Task::perform(blocking(data::load), Message::Loaded),
            // The initial read; `theme_changes` in `subscription` keeps it
            // current if the user flips appearance while the app is open.
            iced::system::theme().map(Message::SystemTheme),
        ]);

        (app, boot)
    }

    pub fn title(&self) -> String {
        format!("Devmode — {}", self.screen.title())
    }

    /// Resolves the theme from the user's settings and, when they've chosen
    /// to follow the system, its live light/dark preference.
    ///
    /// Reads the Settings screen's working copy rather than the saved config
    /// so picking a theme previews immediately, and reverting puts it back.
    pub fn theme(&self) -> Theme {
        use iced::theme::Base;

        let mode = match self.settings.theme_mode {
            ThemeMode::Light => iced::theme::Mode::Light,
            ThemeMode::Dark => iced::theme::Mode::Dark,
            ThemeMode::System => self.system_mode,
        };

        let name = match mode {
            iced::theme::Mode::Dark => &self.settings.dark_theme,
            _ => &self.settings.light_theme,
        };

        settings::theme_named(name).unwrap_or_else(|| Theme::default(mode))
    }

    pub fn snapshot(&self) -> Option<&Snapshot> {
        self.snapshot.as_ref()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Navigate(screen) => {
                self.screen = screen;
                screen::on_enter(self, screen)
            }
            Message::Reload => self.reload(),
            Message::Loaded(Ok(snapshot)) => {
                self.loading = false;
                self.settings.sync_from(&snapshot.config);
                self.repos.reconcile(&snapshot);
                self.workspaces.reconcile(&snapshot);
                self.snapshot = Some(snapshot);
                self.fingerprint = data::state_fingerprint();

                // Selecting a workspace loads its members lazily, so a
                // reload has to refresh whatever is currently open.
                self.workspaces
                    .selected()
                    .map(|id| self.load_workspace_detail(id))
                    .unwrap_or(Task::none())
            }
            Message::Loaded(Err(error)) => {
                self.loading = false;
                self.toast = Some(Toast::error(error));
                Task::none()
            }
            Message::WorkspaceDetailLoaded(Ok(detail)) => {
                self.workspaces.set_detail(detail);
                Task::none()
            }
            Message::WorkspaceDetailLoaded(Err(error)) => {
                self.toast = Some(Toast::error(error));
                Task::none()
            }
            Message::Completed(Ok(body)) => {
                self.toast = Some(Toast::success(body));
                self.reload()
            }
            Message::Completed(Err(error)) => {
                self.toast = Some(Toast::error(error));
                self.loading = false;
                Task::none()
            }
            Message::DismissToast => {
                self.toast = None;
                Task::none()
            }
            Message::ToastTick => {
                if let Some(toast) = &self.toast {
                    let elapsed = toast.shown_at.elapsed().unwrap_or_default();

                    if toast.is_transient() && elapsed > Duration::from_secs(4) {
                        self.toast = None;
                    }
                }

                Task::none()
            }
            Message::StateChanged(fingerprint) => {
                // `dm` or `dmtui` changed the registry or config underneath
                // us — pick the change up instead of showing stale data.
                if fingerprint.is_some() && fingerprint != self.fingerprint {
                    self.fingerprint = fingerprint;
                    return self.reload();
                }

                Task::none()
            }
            Message::SystemTheme(mode) => {
                self.system_mode = mode;
                Task::none()
            }
            Message::Repos(message) => repos::update(self, message),
            Message::Workspaces(message) => workspaces::update(self, message),
            Message::Discovery(message) => discovery::update(self, message),
            Message::Settings(message) => settings::update(self, message),
        }
    }

    /// Builds an app without touching the real registry, for tests.
    #[cfg(test)]
    pub fn for_test() -> Self {
        Self {
            screen: Screen::Repos,
            snapshot: None,
            toast: None,
            loading: false,
            fingerprint: None,
            system_mode: iced::theme::Mode::Light,
            repos: repos::State::default(),
            workspaces: workspaces::State::default(),
            discovery: discovery::State::default(),
            settings: settings::State::default(),
        }
    }

    /// The state-adoption half of handling `Message::Loaded`, without the
    /// follow-up task, so tests can seed a snapshot directly.
    #[cfg(test)]
    pub fn apply_snapshot(&mut self, snapshot: Snapshot) {
        self.loading = false;
        self.settings.sync_from(&snapshot.config);
        self.repos.reconcile(&snapshot);
        self.workspaces.reconcile(&snapshot);
        self.snapshot = Some(snapshot);
    }

    pub fn reload(&mut self) -> Task<Message> {
        self.loading = true;
        Task::perform(blocking(data::load), Message::Loaded)
    }

    pub fn load_workspace_detail(&self, id: String) -> Task<Message> {
        Task::perform(
            blocking(move || data::load_workspace_detail(id)),
            Message::WorkspaceDetailLoaded,
        )
    }

    /// Runs a mutation on a worker thread, reporting either a success message
    /// (which triggers a reload) or the error's `Display`.
    pub fn run<F>(&mut self, f: F) -> Task<Message>
    where
        F: FnOnce() -> Result<String, String> + Send + 'static,
    {
        self.loading = true;
        Task::perform(blocking(f), Message::Completed)
    }

    pub fn toast_error(&mut self, error: impl Into<String>) {
        self.toast = Some(Toast::error(error));
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![
            keyboard_shortcuts(self.screen),
            iced::system::theme_changes().map(Message::SystemTheme),
        ];

        // Only poll while there's something to notice: a visible transient
        // toast, or an idle window that external tools might change.
        if self.toast.as_ref().is_some_and(Toast::is_transient) {
            subscriptions
                .push(iced::time::every(Duration::from_millis(500)).map(|_| Message::ToastTick));
        }

        if !self.loading {
            subscriptions.push(
                iced::time::every(Duration::from_secs(2))
                    .map(|_| Message::StateChanged(data::state_fingerprint())),
            );
        }

        Subscription::batch(subscriptions)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let body = row![
            self.sidebar(),
            container(screen::view(self, self.screen))
                .width(Fill)
                .height(Fill),
        ]
        .height(Fill);

        column![body, self.status_bar()].into()
    }

    fn sidebar(&self) -> Element<'_, Message> {
        let counts = self.snapshot.as_ref();

        let destination = |screen: Screen, count: Option<usize>| -> Element<'_, Message> {
            let is_active = self.screen == screen;

            let mut label = row![text(screen.title()).size(design::TEXT_MD)]
                .spacing(design::SM)
                .align_y(Center)
                .width(Fill);

            if let Some(count) = count {
                label = label
                    .push(space::horizontal())
                    .push(design::badge(
                        count,
                        if is_active { Tone::Info } else { Tone::Neutral },
                    ));
            }

            design::list_row(label, is_active, Message::Navigate(screen))
        };

        let nav = column![
            destination(Screen::Repos, counts.map(|s| s.repos.len())),
            destination(Screen::Workspaces, counts.map(|s| s.workspaces.len())),
            destination(Screen::Discovery, None),
            destination(Screen::Settings, None),
        ]
        .spacing(design::XS);

        container(
            column![
                container(text("Devmode").size(design::TEXT_LG))
                    .padding(iced::Padding::from([design::SM, design::MD])),
                nav,
            ]
            .spacing(design::MD),
        )
        .width(220)
        .height(Fill)
        .padding(design::MD)
        .style(|theme: &Theme| container::Style {
            background: Some(theme.extended_palette().background.weakest.color.into()),
            ..container::Style::default()
        })
        .into()
    }

    fn status_bar(&self) -> Element<'_, Message> {
        let content: Element<'_, Message> = match &self.toast {
            Some(toast) => row![
                design::badge(
                    match toast.tone {
                        Tone::Danger => "Error",
                        _ => "Done",
                    },
                    toast.tone,
                ),
                text(&toast.body).size(design::TEXT_SM),
                space::horizontal(),
                design::secondary_button("Dismiss", Message::DismissToast),
            ]
            .spacing(design::SM)
            .align_y(Center)
            .into(),
            None if self.loading => row![
                text("Working…").size(design::TEXT_SM),
                space::horizontal(),
            ]
            .align_y(Center)
            .into(),
            None => row![
                design::muted(
                    text(format!(
                        "{MODIFIER}1–4 switch · {MODIFIER}F search · {MODIFIER}R refresh"
                    ))
                    .size(design::TEXT_SM)
                ),
                space::horizontal(),
            ]
            .align_y(Center)
            .into(),
        };

        column![
            rule::horizontal(1.0),
            container(content)
                .padding(iced::Padding::from([design::SM, design::MD]))
                .width(Fill),
        ]
        .into()
    }
}

/// Global shortcuts. `keyboard::listen` only reports events no focused widget
/// consumed, so these can't fire while the user is typing in a text field.
fn keyboard_shortcuts(screen: Screen) -> Subscription<Message> {
    use iced::keyboard::{self, Key, key};

    // `with` carries the screen into the closure: iced requires subscription
    // closures to be non-capturing so it can identify them across rebuilds.
    keyboard::listen().with(screen).filter_map(|(screen, event)| {
        let keyboard::Event::KeyPressed { key, modifiers, .. } = event else {
            return None;
        };

        if !modifiers.command() {
            return match key {
                Key::Named(key::Named::Escape) => Some(Message::DismissToast),
                _ => None,
            };
        }

        match key.as_ref() {
            Key::Character("1") => Some(Message::Navigate(Screen::Repos)),
            Key::Character("2") => Some(Message::Navigate(Screen::Workspaces)),
            Key::Character("3") => Some(Message::Navigate(Screen::Discovery)),
            Key::Character("4") => Some(Message::Navigate(Screen::Settings)),
            Key::Character("r") => Some(Message::Reload),
            // Only the repo list has a search field to focus.
            Key::Character("f") if screen == Screen::Repos => {
                Some(Message::Repos(repos::Message::FocusSearch))
            }
            _ => None,
        }
    })
}
