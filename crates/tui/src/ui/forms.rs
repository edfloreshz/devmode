use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};

use crate::app::{ConfirmAction, ConfirmDialog, Form, FormKind, RepoPicker};

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let [_, vertical, _] = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .areas(area);
    let [_, horizontal, _] = Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .areas(vertical);
    horizontal
}

fn title(kind: FormKind) -> &'static str {
    match kind {
        FormKind::Clone => "Clone repo (tab: next field, enter: submit, esc: cancel)",
        FormKind::Create => {
            "Create repo (F2: toggle no-git, tab: next field, enter: submit, esc: cancel)"
        }
        FormKind::Track => "Track repo (tab: next field, enter: submit, esc: cancel)",
        FormKind::WorkspaceCreate => {
            "Create workspace (tab: next field, enter: submit, esc: cancel)"
        }
        FormKind::WorkspaceConfig => "Edit workspace (tab: next field, enter: submit, esc: cancel)",
        FormKind::WorkspaceEnv => "Set env var (tab: next field, enter: submit, esc: cancel)",
        FormKind::ConfigEdit => "Edit setting (enter: save, esc: cancel)",
        FormKind::Setup => "Welcome to devmode, set up (tab: next field, enter: finish)",
    }
}

pub fn draw_form(frame: &mut Frame, area: Rect, form: &Form) {
    let popup = centered_rect(70, 50, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title(form.kind));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let mut constraints: Vec<Constraint> =
        form.fields.iter().map(|_| Constraint::Length(3)).collect();
    if form.kind == FormKind::Create {
        constraints.push(Constraint::Length(1));
    }
    constraints.push(Constraint::Min(0));
    let chunks = Layout::vertical(constraints).split(inner);

    for (i, field) in form.fields.iter().enumerate() {
        let focused = i == form.focus;
        let style = if focused {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let text = format!("{}: {}", form.labels[i], field.value());
        frame.render_widget(
            Paragraph::new(text)
                .style(style)
                .block(Block::default().borders(Borders::ALL)),
            chunks[i],
        );
    }

    if form.kind == FormKind::Create {
        let idx = form.fields.len();
        frame.render_widget(
            Paragraph::new(format!("no-git: {}", form.no_git)),
            chunks[idx],
        );
    }
}

pub fn draw_confirm(frame: &mut Frame, area: Rect, confirm: &ConfirmDialog) {
    let popup = centered_rect(60, 20, area);
    frame.render_widget(Clear, popup);

    let text = match &confirm.action {
        ConfirmAction::RemoveRepo { delete, .. } => {
            format!("{}\n\ndelete from disk: {delete}", confirm.message)
        }
        ConfirmAction::DeleteWorkspace { .. } => confirm.message.clone(),
    };
    frame.render_widget(
        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Confirm"))
            .wrap(Wrap { trim: false }),
        popup,
    );
}

pub fn draw_picker(frame: &mut Frame, area: Rect, picker: &RepoPicker) {
    let popup = centered_rect(60, 50, area);
    frame.render_widget(Clear, popup);

    let title = format!(
        "Add member, type to filter, enter: add, esc: cancel (/{})",
        picker.filter.value()
    );
    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let items: Vec<ListItem> = if picker.filtered.is_empty() {
        vec![ListItem::new("(no matching repos)")]
    } else {
        picker
            .filtered
            .iter()
            .map(|&i| ListItem::new(picker.candidates[i].name.clone()))
            .collect()
    };

    let list = List::new(items).highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = ListState::default();
    if !picker.filtered.is_empty() {
        state.select(Some(picker.selected));
    }
    frame.render_stateful_widget(list, inner, &mut state);
}
