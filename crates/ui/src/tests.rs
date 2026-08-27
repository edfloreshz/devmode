//! Headless UI tests.
//!
//! `iced_test::simulator` lays out a real widget tree and lets us query and
//! click it without a window, so these assert what the screens actually
//! render — not just that the app starts.

use std::path::PathBuf;

use iced_test::simulator;

use dm_core::config::Config;
use dm_core::layout::PathLayout;
use dm_core::registry::Repo;
use dm_core::relayout::Candidate;
use dm_core::workspace::Workspace;

use crate::app::{App, Message};
use crate::data::{Snapshot, WorkspaceDetail};
use crate::screen::{discovery, repos, settings, workspaces};

fn repo(id: i64, name: &str) -> Repo {
    Repo {
        id,
        path: PathBuf::from(format!("/code/{name}")),
        name: name.to_string(),
        remote_url: Some(format!("https://github.com/acme/{name}.git")),
        host: Some("github.com".to_string()),
        owner: Some("acme".to_string()),
        tags: Vec::new(),
        added_at: 0,
        last_opened_at: None,
    }
}

fn workspace(id: &str) -> Workspace {
    Workspace {
        id: id.to_string(),
        name: format!("{id} workspace"),
        description: None,
        editor: None,
        created_at: 0,
    }
}

fn snapshot(repos: Vec<Repo>, workspaces: Vec<Workspace>) -> Snapshot {
    let memberships = repos.iter().map(|repo| (repo.id, Vec::new())).collect();

    Snapshot {
        repos,
        workspaces,
        config: Config::default(),
        drift: Vec::new(),
        memberships,
        dirty: Default::default(),
    }
}

/// An app already past its initial load, so screens render their real state.
fn app_with(snapshot: Snapshot) -> App {
    let mut app = App::for_test();
    app.apply_snapshot(snapshot);
    app
}

#[test]
fn repos_empty_state_offers_a_way_forward() {
    let app = app_with(snapshot(Vec::new(), Vec::new()));
    let mut ui = simulator(repos::view(&app));

    assert!(ui.find("No repos tracked yet").is_ok());
    assert!(
        ui.find("Clone…").is_ok(),
        "an empty list should offer the action that fills it"
    );
}

#[test]
fn repos_list_shows_names_and_selected_detail() {
    let app = app_with(snapshot(vec![repo(1, "alpha"), repo(2, "beta")], Vec::new()));
    let mut ui = simulator(repos::view(&app));

    assert!(ui.find("alpha").is_ok());
    assert!(ui.find("beta").is_ok());
    // reconcile() selects the first repo, so its path is in the detail pane.
    assert!(ui.find("/code/alpha").is_ok());
}

#[test]
fn repos_search_narrows_the_list() {
    let mut app = app_with(snapshot(vec![repo(1, "alpha"), repo(2, "beta")], Vec::new()));

    let _ = repos::update(&mut app, repos::Message::Search("beta".to_string()));

    let mut ui = simulator(repos::view(&app));
    assert!(ui.find("beta").is_ok());
    assert!(
        ui.find("alpha").is_err(),
        "a non-matching repo should be filtered out"
    );
}

#[test]
fn repos_search_moves_the_selection_to_a_visible_repo() {
    let mut app = app_with(snapshot(vec![repo(1, "alpha"), repo(2, "beta")], Vec::new()));
    assert_eq!(app.repos.selected, Some(1));

    let _ = repos::update(&mut app, repos::Message::Search("beta".to_string()));

    assert_eq!(
        app.repos.selected,
        Some(2),
        "filtering out the selected repo should select a visible one instead"
    );
}

#[test]
fn repos_drift_banner_appears_only_when_repos_have_drifted() {
    let mut snapshot = snapshot(vec![repo(1, "alpha")], Vec::new());

    let clean = app_with(snapshot.clone());
    let mut ui = simulator(repos::view(&clean));
    assert!(ui.find("Move them").is_err());

    snapshot.drift.push(Candidate {
        id: 1,
        name: "alpha".to_string(),
        from: PathBuf::from("/code/alpha"),
        to: PathBuf::from("/code/github.com/acme/alpha"),
    });

    let drifted = app_with(snapshot);
    let mut ui = simulator(repos::view(&drifted));
    assert!(ui.find("Move them").is_ok());
    assert!(ui.find("/code/github.com/acme/alpha").is_ok());
}

#[test]
fn clicking_clone_opens_the_clone_dialog() {
    let mut app = app_with(snapshot(vec![repo(1, "alpha")], Vec::new()));

    let mut ui = simulator(repos::view(&app));
    ui.click("Clone…").expect("the toolbar has a Clone button");

    for message in ui.into_messages() {
        let _ = app.update(message);
    }

    assert!(matches!(app.repos.dialog, Some(repos::Dialog::Clone { .. })));

    // The dialog is now on screen with its fields.
    let mut ui = simulator(repos::view(&app));
    assert!(ui.find("Clone a repository").is_ok());
    assert!(ui.find("Cancel").is_ok());
}

#[test]
fn workspaces_empty_state_explains_what_a_workspace_is() {
    let app = app_with(snapshot(Vec::new(), Vec::new()));
    let mut ui = simulator(workspaces::view(&app));

    assert!(ui.find("No workspaces yet").is_ok());
    assert!(ui.find("New workspace…").is_ok());
}

#[test]
fn workspaces_detail_lists_members_and_env() {
    let mut app = app_with(snapshot(vec![repo(1, "alpha")], vec![workspace("work")]));

    app.workspaces.set_detail(WorkspaceDetail {
        id: "work".to_string(),
        members: vec![repo(1, "alpha")],
        env: vec![("API_URL".to_string(), "http://localhost".to_string())],
    });

    let mut ui = simulator(workspaces::view(&app));

    assert!(ui.find("work workspace").is_ok());
    assert!(ui.find("alpha").is_ok());
    assert!(ui.find("API_URL=http://localhost").is_ok());
    assert!(ui.find("Open in editor").is_ok());
}

#[test]
fn a_stale_workspace_detail_is_discarded() {
    let mut app = app_with(snapshot(Vec::new(), vec![workspace("work")]));

    // A response arrives for a workspace the user already navigated away from.
    app.workspaces.set_detail(WorkspaceDetail {
        id: "other".to_string(),
        members: vec![repo(9, "stale")],
        env: Vec::new(),
    });

    assert!(
        app.workspaces.detail.is_none(),
        "a detail for a different workspace must not overwrite the current pane"
    );
}

#[test]
fn discovery_offers_both_halves() {
    let app = app_with(snapshot(Vec::new(), Vec::new()));
    let mut ui = simulator(discovery::view(&app));

    assert!(ui.find("Find untracked repos").is_ok());
    assert!(ui.find("Check tracked repos").is_ok());
    assert!(ui.find("Scan").is_ok());
    assert!(ui.find("Run check").is_ok());
}

#[test]
fn settings_previews_the_selected_layout() {
    let mut app = app_with(snapshot(Vec::new(), Vec::new()));
    app.settings.root = "/code".to_string();

    {
        let mut ui = simulator(settings::view(&app));
        assert!(
            ui.find("/code/github.com/torvalds/linux").is_ok(),
            "the default layout should preview as host/owner/repo"
        );
    }

    let _ = settings::update(
        &mut app,
        settings::Message::LayoutChanged(settings::LayoutChoice::Flat),
    );

    let mut ui = simulator(settings::view(&app));
    assert!(
        ui.find("/code/linux").is_ok(),
        "switching to a flat layout should update the preview"
    );
}

#[test]
fn settings_save_controls_appear_only_when_something_changed() {
    let mut app = app_with(snapshot(Vec::new(), Vec::new()));

    {
        let mut ui = simulator(settings::view(&app));
        assert!(ui.find("Save changes").is_err());
    }

    let _ = settings::update(
        &mut app,
        settings::Message::HostChanged("gitlab.com".to_string()),
    );

    let mut ui = simulator(settings::view(&app));
    assert!(ui.find("Save changes").is_ok());
    assert!(ui.find("Revert").is_ok());
}

#[test]
fn settings_rejects_an_empty_custom_template() {
    let mut app = app_with(snapshot(Vec::new(), Vec::new()));

    let _ = settings::update(
        &mut app,
        settings::Message::LayoutChanged(settings::LayoutChoice::Custom),
    );
    let _ = settings::update(&mut app, settings::Message::TemplateChanged(String::new()));

    let mut ui = simulator(settings::view(&app));
    assert!(ui.find("Invalid").is_ok());
}

#[test]
fn a_custom_template_previews_like_the_built_ins() {
    let mut app = app_with(snapshot(Vec::new(), Vec::new()));
    app.settings.root = "/code".to_string();

    let _ = settings::update(
        &mut app,
        settings::Message::LayoutChanged(settings::LayoutChoice::Custom),
    );
    let _ = settings::update(
        &mut app,
        settings::Message::TemplateChanged("{owner}-{repo}".to_string()),
    );

    let mut ui = simulator(settings::view(&app));
    assert!(ui.find("/code/torvalds-linux").is_ok());

    assert_eq!(
        app.settings.current_layout().unwrap(),
        PathLayout::Custom {
            template: "{owner}-{repo}".to_string()
        }
    );
}

#[test]
fn the_shell_renders_every_destination() {
    let app = app_with(snapshot(vec![repo(1, "alpha")], vec![workspace("work")]));
    let mut ui = simulator(app.view());

    for destination in ["Repos", "Workspaces", "Discovery", "Settings"] {
        assert!(
            ui.find(destination).is_ok(),
            "the sidebar should show {destination}"
        );
    }
}

#[test]
fn navigating_switches_the_visible_screen() {
    let mut app = app_with(snapshot(Vec::new(), Vec::new()));

    let _ = app.update(Message::Navigate(crate::app::Screen::Discovery));

    let mut ui = simulator(app.view());
    assert!(ui.find("Find untracked repos").is_ok());
}

#[test]
fn a_dialog_opened_from_the_empty_state_is_visible() {
    // The empty state is the *only* place to start a clone when nothing is
    // tracked yet, so the dialog has to render over it.
    let mut app = app_with(snapshot(Vec::new(), Vec::new()));

    let _ = repos::update(&mut app, repos::Message::OpenClone);

    let mut ui = simulator(repos::view(&app));
    assert!(
        ui.find("Clone a repository").is_ok(),
        "the clone dialog should render over the empty state"
    );
}

#[test]
fn a_workspace_dialog_opened_from_the_empty_state_is_visible() {
    let mut app = app_with(snapshot(Vec::new(), Vec::new()));

    let _ = workspaces::update(&mut app, workspaces::Message::OpenCreate);

    let mut ui = simulator(workspaces::view(&app));
    assert!(
        ui.find("New workspace").is_ok(),
        "the create dialog should render over the empty state"
    );
}

// -- appearance ---------------------------------------------------------------

use dm_core::config::ThemeMode;
use iced::Theme;

/// Sets the app's idea of the desktop preference, as `theme_changes` would.
fn with_system_mode(app: &mut App, mode: iced::theme::Mode) {
    let _ = app.update(Message::SystemTheme(mode));
}

#[test]
fn following_the_system_switches_between_the_two_chosen_variants() {
    let mut app = app_with(snapshot(Vec::new(), Vec::new()));

    app.settings.theme_mode = ThemeMode::System;
    app.settings.light_theme = "Solarized Light".to_string();
    app.settings.dark_theme = "Tokyo Night".to_string();

    with_system_mode(&mut app, iced::theme::Mode::Light);
    assert_eq!(app.theme(), Theme::SolarizedLight);

    with_system_mode(&mut app, iced::theme::Mode::Dark);
    assert_eq!(
        app.theme(),
        Theme::TokyoNight,
        "a system switch to dark should use the configured dark variant"
    );
}

#[test]
fn an_explicit_mode_ignores_the_system_preference() {
    let mut app = app_with(snapshot(Vec::new(), Vec::new()));

    app.settings.theme_mode = ThemeMode::Dark;
    app.settings.dark_theme = "Dracula".to_string();
    app.settings.light_theme = "Light".to_string();

    // The desktop says light; the explicit choice must win.
    with_system_mode(&mut app, iced::theme::Mode::Light);
    assert_eq!(app.theme(), Theme::Dracula);
}

#[test]
fn an_unknown_theme_name_falls_back_instead_of_failing() {
    let mut app = app_with(snapshot(Vec::new(), Vec::new()));

    app.settings.theme_mode = ThemeMode::Light;
    app.settings.light_theme = "Not A Real Theme".to_string();

    assert_eq!(app.theme(), Theme::Light);
}

#[test]
fn picking_a_theme_previews_before_saving() {
    let mut app = app_with(snapshot(Vec::new(), Vec::new()));
    app.settings.theme_mode = ThemeMode::Dark;

    let _ = settings::update(
        &mut app,
        settings::Message::DarkThemeChanged(Theme::Nord),
    );

    assert_eq!(
        app.theme(),
        Theme::Nord,
        "the picked theme should apply immediately, before Save"
    );
    assert!(app.settings.is_dirty());
}

#[test]
fn reverting_restores_the_saved_theme() {
    let mut app = app_with(snapshot(Vec::new(), Vec::new()));

    // Whatever the saved config resolves to, before any edits.
    let saved = app.theme();

    let _ = settings::update(
        &mut app,
        settings::Message::ThemeModeChanged(ThemeMode::Dark),
    );
    let _ = settings::update(
        &mut app,
        settings::Message::DarkThemeChanged(Theme::Dracula),
    );
    assert_eq!(app.theme(), Theme::Dracula);

    let _ = settings::update(&mut app, settings::Message::Revert);

    assert_eq!(app.theme(), saved, "Revert should restore the saved theme");
    assert!(!app.settings.is_dirty());
}

#[test]
fn the_appearance_section_offers_both_variants() {
    let app = app_with(snapshot(Vec::new(), Vec::new()));
    let mut ui = simulator(settings::view(&app));

    assert!(ui.find("Appearance").is_ok());
    assert!(ui.find("Light theme").is_ok());
    assert!(ui.find("Dark theme").is_ok());
}

#[test]
fn changing_the_theme_mode_marks_settings_dirty() {
    let mut app = app_with(snapshot(Vec::new(), Vec::new()));
    assert!(!app.settings.is_dirty());

    let _ = settings::update(
        &mut app,
        settings::Message::ThemeModeChanged(ThemeMode::Dark),
    );

    assert!(app.settings.is_dirty());
    let mut ui = simulator(settings::view(&app));
    assert!(ui.find("Save changes").is_ok());
}
