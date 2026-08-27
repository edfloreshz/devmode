use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

use crate::app::App;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let rows = app.settings_rows();

    let items: Vec<ListItem> = rows
        .iter()
        .map(|(key, value)| {
            let value = if value.is_empty() {
                "(unset)"
            } else {
                value.as_str()
            };
            ListItem::new(format!("{key:<16}  {value}"))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Settings (e/enter: edit selected)"),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = ListState::default();
    if !rows.is_empty() {
        state.select(Some(app.settings_selected.min(rows.len() - 1)));
    }
    frame.render_stateful_widget(list, area, &mut state);
}
