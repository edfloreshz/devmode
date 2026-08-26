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
