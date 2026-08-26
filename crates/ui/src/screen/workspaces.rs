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
    pub fn reconcile(&mut self, _snapshot: &crate::data::Snapshot) {}
    pub fn selected(&self) -> Option<String> { None }
    pub fn set_detail(&mut self, _detail: crate::data::WorkspaceDetail) {}
}

pub fn on_enter(_app: &mut App) -> Task<AppMessage> { Task::none() }
