use iced::{Element, Task};

use crate::app::{App, Message as AppMessage};

#[derive(Debug, Default)]
pub struct State;

#[derive(Debug, Clone)]
pub enum Message {}

pub fn update(_app: &mut App, _message: Message) -> Task<AppMessage> {
    Task::none()
}

pub fn view(_app: &App) -> Element<'_, AppMessage> {
    iced::widget::text("todo").into()
}

impl State {
    pub fn sync_from(&mut self, _config: &dm_core::config::Config) {}
}
