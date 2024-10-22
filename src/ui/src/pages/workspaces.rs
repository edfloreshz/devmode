use cosmic::{
    iced::{Alignment, Length},
    theme, widget, Apply, Element,
};
use slotmap::{DefaultKey, SecondaryMap, SlotMap};

#[derive(Debug, Default)]
pub struct WorkspacesPage {
    workspaces: SlotMap<DefaultKey, String>,
    editing: SecondaryMap<DefaultKey, bool>,
    workspace_input_ids: SecondaryMap<DefaultKey, widget::Id>,
}

#[derive(Debug, Clone)]
pub enum Message {
    EditMode(DefaultKey, bool),
    TitleUpdate(DefaultKey, String),
    TitleSubmit(DefaultKey),
}

pub enum Command {}

impl WorkspacesPage {
    pub fn new() -> Self {
        let mut workspaces = SlotMap::new();
        let mut workspace_input_ids = SecondaryMap::new();
        let id = workspaces.insert("tasks".into());
        workspace_input_ids.insert(id, widget::Id::unique());
        Self {
            workspaces,
            workspace_input_ids,
            ..Default::default()
        }
    }

    fn header(&self) -> Element<Message> {
        widget::row::with_capacity(2)
            .push(widget::text::title2("Workspaces"))
            .into()
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        let mut items = widget::list::list_column()
            .style(theme::Container::ContextDrawer)
            .spacing(spacing.space_xxxs)
            .padding([spacing.space_none, spacing.space_xxs]);

        for (id, item) in &self.workspaces {
            let item_text = widget::editable_input(
                "",
                item,
                *self.editing.get(id).unwrap_or(&false),
                move |editing| Message::EditMode(id, editing),
            )
            .id(self.workspace_input_ids[id].clone())
            .on_submit(Message::TitleSubmit(id))
            .on_input(move |text| Message::TitleUpdate(id, text))
            .width(Length::Fill);

            let row = widget::row::with_capacity(4)
                .align_y(Alignment::Center)
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
        match message {
            Message::TitleSubmit(_) => todo!(),
            Message::TitleUpdate(_, _) => todo!(),
            Message::EditMode(_, _) => todo!(),
        }
    }
}
