use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let [list_area, detail_area] =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)]).areas(area);

    let mut items: Vec<ListItem> = app
        .found
        .iter()
        .map(|d| ListItem::new(format!("+ {}", d.name)).style(Style::default().fg(Color::Green)))
        .collect();
    items.extend(app.issues.iter().map(|issue| {
        ListItem::new(format!("⚠ {}", issue.describe())).style(Style::default().fg(Color::Yellow))
    }));
    if items.is_empty() {
        items.push(ListItem::new("(press s to scan the repo root)"));
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Discovery (s: scan  a: track all  enter: track/resolve)"),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let total = app.found.len() + app.issues.len();
    let mut state = ListState::default();
    if total > 0 {
        state.select(Some(app.discovery_selected.min(total - 1)));
    }
    frame.render_stateful_widget(list, list_area, &mut state);

    frame.render_widget(
        Paragraph::new(detail_text(app))
            .block(Block::default().borders(Borders::ALL).title("Detail"))
            .wrap(Wrap { trim: false }),
        detail_area,
    );
}

fn detail_text(app: &App) -> String {
    let idx = app.discovery_selected;

    if let Some(discovered) = app.found.get(idx) {
        let mut lines = vec![
            format!("name: {}", discovered.name),
            format!("path: {}", discovered.path.display()),
            format!(
                "remote: {}",
                discovered.remote_url.as_deref().unwrap_or("(none)")
            ),
        ];
        if let Some(host) = &discovered.host {
            lines.push(format!("host: {host}"));
        }
        if let Some(owner) = &discovered.owner {
            lines.push(format!("owner: {owner}"));
        }
        lines.push(String::new());
        lines.push("enter: start tracking this repo".to_string());
        return lines.join("\n");
    }

    if let Some(issue) = app.issues.get(idx - app.found.len()) {
        return format!("{}\n\nenter: {}", issue.describe(), issue.resolution());
    }

    format!(
        "{} untracked repo(s), {} issue(s)\n\ns: scan the configured repo root for untracked \
         repos and re-check the tracked ones against disk",
        app.found.len(),
        app.issues.len()
    )
}
