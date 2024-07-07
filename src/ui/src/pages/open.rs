use cosmic::{
    iced::{Alignment, Length},
    theme, widget, Apply, Element,
};
use slotmap::{DefaultKey, SecondaryMap, SlotMap};

#[derive(Debug, Default)]
pub struct Repository {
    url: String,
    selected: bool,
}

#[derive(Debug, Default)]
pub struct OpenPage {
    projects: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Select(String),
}

pub enum Command {}

impl OpenPage {
    pub fn new() -> Self {
        Self::default()
    }

    fn header(&self) -> Element<Message> {
        widget::row::with_capacity(2)
            .push(widget::text::title2("Open"))
            .into()
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        let mut items = widget::list::list_column()
            .style(theme::Container::ContextDrawer)
            .spacing(spacing.space_xxxs)
            .padding([spacing.space_none, spacing.space_xxs]);

        for item in &self.projects {
            let item_text = widget::text(item).width(Length::Fill);

            let row = widget::row::with_capacity(4)
                .align_items(Alignment::Center)
                .spacing(spacing.space_xxs)
                .padding([spacing.space_xxxs, spacing.space_xxs])
                .push(item_text);

            items = items.add(row);
        }

        widget::column::with_capacity(2)
            .push(self.header())
            .push(items)
            .spacing(spacing.space_xxs)
            .apply(widget::container)
            .height(Length::Shrink)
            .apply(widget::scrollable)
            .height(Length::Fill)
            .into()
    }

    pub fn update(&self, message: Message) -> Vec<Command> {
        let mut commands = vec![];
        match message {
            Message::Select(_) => todo!(),
        }
        commands
    }
}
