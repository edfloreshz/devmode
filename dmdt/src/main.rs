use std::io;

use anyhow::Result;
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
    widgets::{Block, Borders},
};
use tui::style::{Color, Style};
use tui::widgets::{List, ListItem};
use tui::text::Spans;

use dmdlib::models::project::get_projects;

use crate::util::StatefulList;

mod util;

fn main() -> Result<()> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut repo_list = StatefulList::with_items(
        get_projects()?
    );

    loop {
        terminal.draw(|f| {
            // Create two chunks with equal horizontal screen space
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.size());

            let items: Vec<ListItem> = repo_list
                .items
                .iter()
                .map(|i| {
                    let mut lines = Spans::from(i.as_str());
                    ListItem::new(lines).style(Style::default())
                })
                .collect();

            let list_of_repos = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Cloned repositories"))
                .highlight_style(Style::default())
                .highlight_symbol(">> ");
            f.render_stateful_widget(list_of_repos, chunks[0], &mut repo_list.state);
        })?;
    }
    Ok(())
}