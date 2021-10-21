use std::io;

use anyhow::Result;
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::style::Style;
use tui::text::Spans;
use tui::widgets::{List, ListItem};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
    Terminal,
};

use dmdlib::models::project::get_projects;

use crate::util::event::{Event, Events};
use crate::util::StatefulList;

mod util;

fn main() -> Result<()> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    let mut list = StatefulList::with_items(get_projects()?);

    loop {
        terminal.draw(|f| {
            // Create two chunks with equal horizontal screen space
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.size());

            let items: Vec<ListItem> = list
                .items
                .iter()
                .map(|i| {
                    let lines = Spans::from(i.as_str());
                    ListItem::new(lines).style(Style::default())
                })
                .collect();

            let list_of_repos = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Cloned repositories"),
                )
                .highlight_style(Style::default())
                .highlight_symbol(">> ");
            f.render_stateful_widget(list_of_repos, chunks[0], &mut list.state);
        })?;
        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Left => {
                    list.unselect();
                }
                Key::Down => {
                    list.next();
                }
                Key::Up => {
                    list.previous();
                }
                _ => {}
            },
            Event::Tick => {
                // list.advance();
            }
        }
    }
    Ok(())
}
