use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

use dm_core::registry::Repo;
use dm_core::relayout::Candidate;

use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let [list_area, detail_area] =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).areas(area);

    let items: Vec<ListItem> = app
        .filtered
        .iter()
        .map(|&i| {
            let repo = &app.repos[i];
            if app.drift_for(repo.id).is_some() {
                ListItem::new(format!("⚠ {}", repo.name)).style(Style::default().fg(Color::Yellow))
            } else {
                ListItem::new(repo.name.clone())
            }
        })
        .collect();

    let list_title = if app.filtering || !app.filter.value().is_empty() {
        format!("Repos (/{})", app.filter.value())
    } else {
        "Repos".to_string()
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(list_title))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = ListState::default();
    if !app.filtered.is_empty() {
        state.select(Some(app.selected));
    }
    frame.render_stateful_widget(list, list_area, &mut state);

    let detail = match app.selected_repo() {
        Some(repo) => format_detail(
            repo,
            &app.selected_repo_workspaces(),
            app.drift_for(repo.id),
        ),
        None => "no repos tracked".to_string(),
    };
    frame.render_widget(
        Paragraph::new(detail)
            .block(Block::default().borders(Borders::ALL).title("Detail"))
            .wrap(Wrap { trim: false }),
        detail_area,
    );
}

fn format_detail(repo: &Repo, workspaces: &[String], drift: Option<&Candidate>) -> String {
    let mut lines = vec![
        format!("name: {}", repo.name),
        format!("path: {}", repo.path.display()),
    ];
    if let Some(remote) = &repo.remote_url {
        lines.push(format!("remote: {remote}"));
    }
    if let Some(host) = &repo.host {
        lines.push(format!("host: {host}"));
    }
    if let Some(owner) = &repo.owner {
        lines.push(format!("owner: {owner}"));
    }
    if !repo.tags.is_empty() {
        lines.push(format!("tags: {}", repo.tags.join(", ")));
    }
    lines.push(format!(
        "workspaces: {}",
        if workspaces.is_empty() {
            "(none)".to_string()
        } else {
            workspaces.join(", ")
        }
    ));
    lines.push(match drift {
        Some(c) => format!(
            "layout: drifted — would move to {} (l: fix)",
            c.to.display()
        ),
        None => "layout: ok".to_string(),
    });
    lines.join("\n")
}
