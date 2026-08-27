pub mod discovery;
pub mod repos;
pub mod settings;
pub mod setup;
pub mod workspaces;

use iced::{Element, Task};

use crate::app::{App, Message, Screen};

pub fn view(app: &App, screen: Screen) -> Element<'_, Message> {
    match screen {
        Screen::Repos => repos::view(app),
        Screen::Workspaces => workspaces::view(app),
        Screen::Discovery => discovery::view(app),
        Screen::Settings => settings::view(app),
    }
}

/// Runs when a screen becomes visible, so screens that need extra data (or a
/// focused input) can ask for it at the moment it's actually needed.
pub fn on_enter(app: &mut App, screen: Screen) -> Task<Message> {
    match screen {
        Screen::Repos => repos::on_enter(app),
        Screen::Workspaces => workspaces::on_enter(app),
        Screen::Discovery => discovery::on_enter(app),
        Screen::Settings => Task::none(),
    }
}
