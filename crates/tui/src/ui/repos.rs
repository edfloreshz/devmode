use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

use dm_core::git::RepoStatus;
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
            let drifted = app.drift_for(repo.id).is_some();
            let dirty = app.dirty.contains(&repo.id);

            let mut label = String::new();
            if dirty {
                label.push_str("● ");
            }
            if drifted {
                label.push_str("⚠ ");
            }
            label.push_str(&repo.name);

            let style = if drifted {
                Style::default().fg(Color::Yellow)
            } else if dirty {
                Style::default().fg(Color::Magenta)
            } else {
                Style::default()
            };

            ListItem::new(label).style(style)
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
            app.selected_repo_status(),
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

fn format_detail(
    repo: &Repo,
    workspaces: &[String],
    drift: Option<&Candidate>,
    git: Option<&RepoStatus>,
) -> String {
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
        Some(c) => format!("layout: drifted, would move to {} (l: fix)", c.to.display()),
        None => "layout: ok".to_string(),
    });

    lines.push(String::new());
    lines.push("── Git ──".to_string());
    lines.extend(git_lines(git));

    lines.join("\n")
}

/// The "Git card": branch, upstream tracking, working-tree state, and the
/// last commit for the selected repo.
fn git_lines(status: Option<&RepoStatus>) -> Vec<String> {
    let Some(status) = status else {
        return vec!["not a git repo".to_string()];
    };

    let branch = if status.detached {
        "(detached HEAD)".to_string()
    } else {
        status.branch.clone().unwrap_or_else(|| "—".to_string())
    };
    let mut lines = vec![format!("branch: {branch}")];

    if let (Some(ahead), Some(behind)) = (status.ahead, status.behind) {
        lines.push(if ahead == 0 && behind == 0 {
            "upstream: up to date".to_string()
        } else {
            format!("upstream: {ahead} ahead, {behind} behind")
        });
    }

    lines.push(if status.is_clean() {
        "working tree: clean".to_string()
    } else {
        format!(
            "working tree: {} staged, {} modified, {} untracked",
            status.staged, status.modified, status.untracked
        )
    });

    if let Some(commit) = &status.last_commit {
        let summary = if commit.summary.is_empty() {
            "(no summary)"
        } else {
            &commit.summary
        };
        lines.push(format!(
            "last commit: {} {summary} — {}",
            commit.short_id, commit.author
        ));
    }

    if status.tag_count > 0 || status.stash_count > 0 {
        lines.push(format!(
            "{} tag(s), {} stashed",
            status.tag_count, status.stash_count
        ));
    }

    lines
}
