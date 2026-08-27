//! The Discovery screen: finding repos on disk devmode doesn't know about,
//! and reconciling entries that no longer match reality.
//!
//! This is where a GUI genuinely beats the CLI. `dm repo scan` walks you
//! through one yes/no prompt per repo; here the whole result set arrives at
//! once, pre-selected, and you tick off what you want before committing to
//! anything.

use std::collections::HashSet;
use std::path::PathBuf;

use iced::widget::{checkbox, column, row, scrollable, space, text};
use iced::{Center, Element, Fill, Task};

use dm_core::discovery::{self, Discovered, Issue};

use crate::app::{App, Message as AppMessage};
use crate::design::{self, Tone};

/// Where the scan half of the screen is in its lifecycle.
#[derive(Debug, Default)]
pub enum Scan {
    #[default]
    Idle,
    Running,
    Done {
        found: Vec<Discovered>,
        /// Indices of `found` the user wants to track.
        selected: HashSet<usize>,
    },
}

#[derive(Debug, Default)]
pub enum Check {
    #[default]
    Idle,
    Running,
    Done(Vec<Issue>),
}

#[derive(Debug, Default)]
pub struct State {
    pub root: String,
    /// Set once from config so the field starts at the user's repo root.
    root_initialised: bool,
    pub scan: Scan,
    pub check: Check,
}

#[derive(Debug, Clone)]
pub enum Message {
    RootChanged(String),
    BrowseRoot,
    RootPicked(Option<PathBuf>),
    StartScan,
    ScanFinished(Result<Vec<Discovered>, String>),
    ToggleFound(usize, bool),
    SelectAll(bool),
    TrackSelected,
    StartCheck,
    CheckFinished(Result<Vec<Issue>, String>),
    Resolve(usize),
    ResolveAll,
}

pub fn on_enter(app: &mut App) -> Task<AppMessage> {
    // Default the scan root to the configured repo root the first time the
    // screen is opened, without clobbering anything typed since.
    if !app.discovery.root_initialised
        && let Some(snapshot) = app.snapshot()
    {
        app.discovery.root = snapshot.config.repo.root.display().to_string();
        app.discovery.root_initialised = true;
    }

    Task::none()
}

pub fn update(app: &mut App, message: Message) -> Task<AppMessage> {
    match message {
        Message::RootChanged(root) => {
            app.discovery.root = root;
            Task::none()
        }
        Message::BrowseRoot => {
            let current = app.discovery.root.trim();
            let starting = (!current.is_empty()).then(|| PathBuf::from(current));

            Task::perform(
                crate::task::pick_folder("Choose a folder", starting),
                |picked| wrap(Message::RootPicked(picked)),
            )
        }
        Message::RootPicked(Some(picked)) => {
            app.discovery.root = picked.display().to_string();
            Task::none()
        }
        Message::RootPicked(None) => Task::none(),
        Message::StartScan => {
            let root = PathBuf::from(app.discovery.root.trim());

            if root.as_os_str().is_empty() {
                app.toast_error("Choose a folder to scan.");
                return Task::none();
            }

            app.discovery.scan = Scan::Running;

            Task::perform(
                crate::task::blocking(move || {
                    discovery::find_untracked(&root).map_err(|e| e.to_string())
                }),
                |result| wrap(Message::ScanFinished(result)),
            )
        }
        Message::ScanFinished(Ok(found)) => {
            // Everything found is worth tracking by default, the user is
            // here because they want these in the registry.
            let selected = (0..found.len()).collect();
            app.discovery.scan = Scan::Done { found, selected };
            Task::none()
        }
        Message::ScanFinished(Err(error)) => {
            app.discovery.scan = Scan::Idle;
            app.toast_error(error);
            Task::none()
        }
        Message::ToggleFound(index, checked) => {
            if let Scan::Done { selected, .. } = &mut app.discovery.scan {
                if checked {
                    selected.insert(index);
                } else {
                    selected.remove(&index);
                }
            }

            Task::none()
        }
        Message::SelectAll(checked) => {
            if let Scan::Done { found, selected } = &mut app.discovery.scan {
                *selected = if checked {
                    (0..found.len()).collect()
                } else {
                    HashSet::new()
                };
            }

            Task::none()
        }
        Message::TrackSelected => {
            let Scan::Done { found, selected } = &app.discovery.scan else {
                return Task::none();
            };

            let batch: Vec<Discovered> = found
                .iter()
                .enumerate()
                .filter(|(index, _)| selected.contains(index))
                .map(|(_, repo)| repo.clone())
                .collect();

            if batch.is_empty() {
                app.toast_error("Nothing selected to track.");
                return Task::none();
            }

            app.discovery.scan = Scan::Idle;

            let label = format!("Tracking {} repo(s)…", batch.len());
            app.run(label, move || {
                let tracked = discovery::track_all(batch).map_err(|e| e.to_string())?;

                Ok(format!("Tracked {tracked} repo(s)."))
            })
        }
        Message::StartCheck => {
            app.discovery.check = Check::Running;

            Task::perform(
                crate::task::blocking(|| discovery::check().map_err(|e| e.to_string())),
                |result| wrap(Message::CheckFinished(result)),
            )
        }
        Message::CheckFinished(Ok(issues)) => {
            app.discovery.check = Check::Done(issues);
            Task::none()
        }
        Message::CheckFinished(Err(error)) => {
            app.discovery.check = Check::Idle;
            app.toast_error(error);
            Task::none()
        }
        Message::Resolve(index) => {
            let Check::Done(issues) = &app.discovery.check else {
                return Task::none();
            };

            let Some(issue) = issues.get(index).cloned() else {
                return Task::none();
            };

            app.discovery.check = Check::Idle;

            let label = format!("Resolving: {}…", issue.describe());
            app.run(label, move || {
                discovery::resolve(&issue).map_err(|e| e.to_string())?;

                Ok(format!("Resolved: {}", issue.describe()))
            })
        }
        Message::ResolveAll => {
            let Check::Done(issues) = &app.discovery.check else {
                return Task::none();
            };

            let issues = issues.clone();

            if issues.is_empty() {
                return Task::none();
            }

            app.discovery.check = Check::Idle;

            let label = format!("Resolving {} issue(s)…", issues.len());
            app.run(label, move || {
                let total = issues.len();

                for issue in &issues {
                    discovery::resolve(issue).map_err(|e| e.to_string())?;
                }

                Ok(format!("Resolved {total} issue(s)."))
            })
        }
    }
}

pub fn view(app: &App) -> Element<'_, AppMessage> {
    design::page(column![
        design::page_header(
            "Discovery",
            Some(
                "Find repos already on disk, and check that what devmode tracks \
                 still matches reality."
                    .to_string()
            ),
            None,
        ),
        scan_section(app),
        check_section(app),
    ])
}

fn scan_section(app: &App) -> Element<'_, AppMessage> {
    let is_running = matches!(app.discovery.scan, Scan::Running);

    let controls = row![
        design::input("/path/to/your/code", &app.discovery.root)
            .on_input(|root| wrap(Message::RootChanged(root)))
            .on_submit(wrap(Message::StartScan))
            .font(design::MONO)
            .width(Fill),
        design::secondary_button("Browse…", wrap(Message::BrowseRoot)),
        if is_running {
            design::secondary_button("Scanning…", None)
        } else {
            design::primary_button("Scan", wrap(Message::StartScan))
        },
    ]
    .spacing(design::SM)
    .align_y(Center);

    let body: Element<'_, AppMessage> = match &app.discovery.scan {
        Scan::Idle => column![
            design::muted(
                text(
                    "Walks the folder for git repos devmode isn't tracking yet. \
                     Nothing is added until you choose."
                )
                .size(design::TEXT_SM)
            ),
            controls,
        ]
        .spacing(design::MD)
        .into(),
        Scan::Running => column![
            controls,
            design::muted(text("Walking the folder…").size(design::TEXT_SM)),
        ]
        .spacing(design::MD)
        .into(),
        Scan::Done { found, selected } => {
            if found.is_empty() {
                column![
                    controls,
                    design::muted(
                        text("No untracked repos found there, everything is already tracked.")
                            .size(design::TEXT_SM)
                    ),
                ]
                .spacing(design::MD)
                .into()
            } else {
                let all_selected = selected.len() == found.len();

                let mut rows = column![].spacing(design::XS);

                for (index, repo) in found.iter().enumerate() {
                    let remote = repo
                        .remote_url
                        .clone()
                        .unwrap_or_else(|| "no remote".to_string());

                    rows = rows.push(
                        row![
                            checkbox(selected.contains(&index)).on_toggle(move |checked| wrap(
                                Message::ToggleFound(index, checked)
                            )),
                            column![
                                text(&repo.name).size(design::TEXT_MD),
                                design::muted(
                                    text(repo.path.display().to_string())
                                        .size(design::TEXT_SM)
                                        .font(design::MONO)
                                ),
                                design::muted(
                                    text(remote).size(design::TEXT_SM).font(design::MONO)
                                ),
                            ]
                            .spacing(1.0)
                            .width(Fill),
                        ]
                        .spacing(design::SM)
                        .align_y(Center),
                    );
                }

                column![
                    controls,
                    row![
                        checkbox(all_selected)
                            .label(format!("{} of {} selected", selected.len(), found.len()))
                            .text_size(design::CONTROL_TEXT)
                            .on_toggle(|checked| wrap(Message::SelectAll(checked))),
                        space::horizontal(),
                        design::primary_button("Track selected", wrap(Message::TrackSelected)),
                    ]
                    .align_y(Center),
                    scrollable(rows).height(260.0),
                ]
                .spacing(design::MD)
                .into()
            }
        }
    };

    design::section("Find untracked repos", body)
}

fn check_section(app: &App) -> Element<'_, AppMessage> {
    let body: Element<'_, AppMessage> = match &app.discovery.check {
        Check::Idle => column![
            design::muted(
                text(
                    "Checks every tracked repo still exists where devmode thinks \
                     it does, and that its remote hasn't changed."
                )
                .size(design::TEXT_SM)
            ),
            design::button_row(vec![design::primary_button(
                "Run check",
                wrap(Message::StartCheck),
            )]),
        ]
        .spacing(design::MD)
        .into(),
        Check::Running => design::muted(text("Checking…").size(design::TEXT_SM)),
        Check::Done(issues) if issues.is_empty() => row![
            design::badge("Healthy", Tone::Success),
            text("Every tracked repo checks out.").size(design::TEXT_SM),
            space::horizontal(),
            design::secondary_button("Run again", wrap(Message::StartCheck)),
        ]
        .spacing(design::SM)
        .align_y(Center)
        .into(),
        Check::Done(issues) => {
            let mut rows = column![].spacing(design::SM);

            for (index, issue) in issues.iter().enumerate() {
                let tone = match issue {
                    Issue::Missing { .. } => Tone::Danger,
                    Issue::RemoteChanged { .. } => Tone::Warning,
                };

                let label = match issue {
                    Issue::Missing { .. } => "Missing",
                    Issue::RemoteChanged { .. } => "Remote",
                };

                rows = rows.push(
                    row![
                        design::badge(label, tone),
                        column![
                            text(issue.describe()).size(design::TEXT_SM),
                            design::muted(text(issue.resolution()).size(design::TEXT_SM)),
                        ]
                        .spacing(1.0)
                        .width(Fill),
                        design::secondary_button("Fix", wrap(Message::Resolve(index))),
                    ]
                    .spacing(design::SM)
                    .align_y(Center),
                );
            }

            column![
                row![
                    text(format!("{} issue(s) found.", issues.len())).size(design::TEXT_SM),
                    space::horizontal(),
                    design::secondary_button("Run again", wrap(Message::StartCheck)),
                    design::primary_button("Fix all", wrap(Message::ResolveAll)),
                ]
                .spacing(design::SM)
                .align_y(Center),
                rows,
            ]
            .spacing(design::MD)
            .into()
        }
    };

    design::section("Check tracked repos", body)
}

fn wrap(message: Message) -> AppMessage {
    AppMessage::Discovery(message)
}
