use iced::{
    widget::{button, column, container, horizontal_rule, row, text},
    Length, Theme,
};
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::pages::{ClonePanel, OpenPanel, PreferencesPanel, WorkspacesPanel};

#[derive(Debug, Default)]
pub struct Devmode {
    theme: Theme,
    pages: Vec<Page>,
    selected: Page,
    clone: ClonePanel,
    open: OpenPanel,
    workspaces: WorkspacesPanel,
    preferences: PreferencesPanel,
}

#[derive(Debug, Clone)]
pub enum Message {
    SwitchPage(Page),
    SelectRepo(Repository),
}

#[derive(Debug, Default, PartialEq, EnumIter, Display, Clone)]
enum Page {
    #[default]
    Clone,
    Open,
    Workspaces,
    Preferences,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Repository {
    pub name: String,
    pub url: String,
}

impl iced::Sandbox for Devmode {
    type Message = Message;

    fn new() -> Self {
        Devmode {
            theme: Theme::Dark,
            pages: Page::iter().collect(),
            selected: Page::default(),
            ..Default::default()
        }
    }

    fn title(&self) -> String {
        String::from("Devmode")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::SwitchPage(page) => self.selected = page,
            Message::SelectRepo(repo) => print!("Selected repo: {repo:?}"),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        container(column!(self.tab_bar(), horizontal_rule(1), self.content()).spacing(10.0))
            .padding(10.0)
            .into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

impl Devmode {
    fn tab_bar(&self) -> iced::Element<'_, Message> {
        let pages = self
            .pages
            .iter()
            .map(|page| {
                button(text(page.to_string()))
                    .on_press(Message::SwitchPage(page.clone()))
                    .width(iced::Length::Fill)
                    .into()
            })
            .collect::<Vec<iced::Element<'_, Message>>>();

        container(row(pages).spacing(5.0))
            .width(Length::Fill)
            .into()
    }

    fn content(&self) -> iced::Element<'_, Message> {
        match self.selected {
            Page::Clone => self.clone.ui(),
            Page::Open => self.open.ui(),
            Page::Workspaces => self.workspaces.ui(),
            Page::Preferences => self.preferences.ui(),
        }
    }
}
