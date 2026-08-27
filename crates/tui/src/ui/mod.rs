mod forms;
mod repos;
mod workspaces;

use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::Tabs,
};

use crate::app::{App, Tab, WorkspaceFocus};

const SPINNER: [char; 4] = ['|', '/', '-', '\\'];

pub fn draw(frame: &mut Frame, app: &App) {
    let [tab_bar, body, status_bar] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    let tabs = Tabs::new(vec!["Repos", "Workspaces"])
        .select(match app.active_tab {
            Tab::Repos => 0,
            Tab::Workspaces => 1,
        })
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        );
    frame.render_widget(tabs, tab_bar);

    match app.active_tab {
        Tab::Repos => repos::draw(frame, body, app),
        Tab::Workspaces => workspaces::draw(frame, body, app),
    }

    if let Some(form) = &app.form {
        forms::draw_form(frame, frame.area(), form);
    }
    if let Some(confirm) = &app.confirm {
        forms::draw_confirm(frame, frame.area(), confirm);
    }
    if let Some(picker) = &app.picker {
        forms::draw_picker(frame, frame.area(), picker);
    }

    let status = if app.busy {
        format!(
            "{} {}",
            SPINNER[app.spinner_tick % SPINNER.len()],
            app.status.as_deref().unwrap_or("working...")
        )
    } else if let Some(status) = &app.status {
        status.clone()
    } else if app.filtering {
        "esc/enter: stop filtering".to_string()
    } else if app.form.is_some() || app.confirm.is_some() || app.picker.is_some() {
        String::new()
    } else {
        match app.active_tab {
            Tab::Repos => {
                let base = "q: quit  tab: switch  /: filter  j/k: move  c: clone  n: create  t: track  r: remove";
                if app.drift.is_empty() {
                    base.to_string()
                } else {
                    format!("{base}  l: fix {} layout drift(s)", app.drift.len())
                }
            }
            Tab::Workspaces => match app.workspace_focus {
                WorkspaceFocus::List => {
                    "q: quit  tab: switch  j/k: move  c: create  r: edit  d: delete  a: add member  v: add env  s: switch  enter: focus items"
                        .to_string()
                }
                WorkspaceFocus::Items => {
                    "esc: back to list  j/k: move  x: remove selected item".to_string()
                }
            },
        }
    };
    frame.render_widget(Line::from(status), status_bar);
}
