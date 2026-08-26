use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use crate::app::{App, DetailItem, WorkspaceFocus};

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let [list_area, detail_area] =
        Layout::horizontal([Constraint::Percentage(35), Constraint::Percentage(65)]).areas(area);

    let items: Vec<ListItem> = app
        .workspaces_list
        .iter()
        .map(|ws| ListItem::new(ws.id.clone()))
        .collect();

    let list_highlight = if app.workspace_focus == WorkspaceFocus::List {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    };

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Workspaces"))
        .highlight_style(list_highlight);

    let mut state = ListState::default();
    if !app.workspaces_list.is_empty() {
        state.select(Some(app.workspace_selected));
    }
    frame.render_stateful_widget(list, list_area, &mut state);

    let detail_block = Block::default().borders(Borders::ALL).title("Detail");
    let inner = detail_block.inner(detail_area);
    frame.render_widget(detail_block, detail_area);

    let Some(ws) = app.selected_workspace() else {
        frame.render_widget(Paragraph::new("no workspaces tracked"), inner);
        return;
    };

    let [meta_area, items_area] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(inner);

    let meta = format!(
        "name: {}    description: {}    editor: {}",
        ws.name,
        ws.description.as_deref().unwrap_or("-"),
        ws.editor.as_deref().unwrap_or("-"),
    );
    frame.render_widget(Paragraph::new(meta), meta_area);

    let items = app.workspace_detail_items(&ws.id);
    let list_items: Vec<ListItem> = if items.is_empty() {
        vec![ListItem::new("(no members or env vars)")]
    } else {
        items
            .iter()
            .map(|item| match item {
                DetailItem::Member(_, name) => ListItem::new(format!("[repo] {name}")),
                DetailItem::Env(key, value) => ListItem::new(format!("[env]  {key}={value}")),
            })
            .collect()
    };

    let items_highlight = if app.workspace_focus == WorkspaceFocus::Items {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    };

    let items_list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Members & env (enter: focus, x: remove selected)"),
        )
        .highlight_style(items_highlight);

    let mut item_state = ListState::default();
    if !items.is_empty() {
        item_state.select(Some(app.workspace_item_selected.min(items.len() - 1)));
    }
    frame.render_stateful_widget(items_list, items_area, &mut item_state);
}
