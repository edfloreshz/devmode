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
pub struct ClonePage {
    workspace: String,
    repository: Repository,
    repositories: SlotMap<DefaultKey, Repository>,
    editing: SecondaryMap<DefaultKey, bool>,
    repository_input_ids: SecondaryMap<DefaultKey, widget::Id>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Clone,
    Select(DefaultKey, bool),
    TitleSubmit(DefaultKey),
    TitleUpdate(DefaultKey, String),
    EditMode(DefaultKey, bool),
}

pub enum Command {
    Clone(String, String),
}

impl ClonePage {
    pub fn new() -> Self {
        let mut repositories = SlotMap::new();
        let mut repository_input_ids = SecondaryMap::new();
        let id = repositories.insert(Repository {
            url: "https://github.com/edfloreshz/tasks".into(),
            selected: false,
        });
        repository_input_ids.insert(id, widget::Id::unique());
        Self {
            repositories,
            repository_input_ids,
            ..Default::default()
        }
    }

    fn header(&self) -> Element<Message> {
        widget::row::with_capacity(2)
            .push(widget::text::title2("Clone"))
            .into()
    }

    pub fn view(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        let mut items = widget::list::list_column()
            .style(theme::Container::ContextDrawer)
            .spacing(spacing.space_xxxs)
            .padding([spacing.space_none, spacing.space_xxs]);

        for (id, item) in &self.repositories {
            let item_checkbox = widget::checkbox("", item.selected)
                .on_toggle(move |value| Message::Select(id, value));

            let item_text = widget::editable_input(
                "",
                &item.url,
                *self.editing.get(id).unwrap_or(&false),
                move |editing| Message::EditMode(id, editing),
            )
            .id(self.repository_input_ids[id].clone())
            .on_submit(Message::TitleSubmit(id))
            .on_input(move |text| Message::TitleUpdate(id, text))
            .width(Length::Fill);

            let row = widget::row::with_capacity(4)
                .align_y(Alignment::Center)
                .spacing(spacing.space_xxs)
                .padding([spacing.space_xxxs, spacing.space_xxs])
                .push(item_checkbox)
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
            Message::Clone => commands.push(Command::Clone(
                self.repository.url.clone(),
                self.workspace.clone(),
            )),
            Message::Select(_, _) => todo!(),
            Message::TitleSubmit(_) => todo!(),
            Message::TitleUpdate(_, _) => todo!(),
            Message::EditMode(_, _) => todo!(),
        }
        commands
    }
}
