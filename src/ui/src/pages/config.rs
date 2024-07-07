use cosmic::{iced::Length, theme, widget, Apply, Element};

#[derive(Debug, Default)]
pub struct ConfigPage {}

#[derive(Debug, Clone)]
pub enum Message {}

pub enum Command {}

impl ConfigPage {
    pub fn new() -> Self {
        Self::default()
    }

    fn header(&self) -> Element<Message> {
        widget::row::with_capacity(2)
            .push(widget::text::title2("Config"))
            .into()
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        widget::column::with_capacity(2)
            .push(self.header())
            .spacing(spacing.space_xxs)
            .apply(widget::container)
            .height(Length::Shrink)
            .apply(widget::scrollable)
            .height(Length::Fill)
            .into()
    }

    pub fn update(&self, message: Message) -> Vec<Command> {
        let mut commands = vec![];
        match message {}
        commands
    }
}
