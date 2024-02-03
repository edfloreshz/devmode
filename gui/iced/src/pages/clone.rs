use iced::widget::{button, column, text};

use crate::app::{Message, Repository};

#[derive(Debug)]
pub struct ClonePanel {
    repositories: Vec<Repository>,
    selected: Repository,
    url: String,
}

impl Default for ClonePanel {
    fn default() -> Self {
        let repositories: Vec<Repository> = (0..20)
            .map(|i| Repository {
                name: "Test".to_string(),
                url: i.to_string(),
            })
            .collect();
        Self {
            repositories: repositories.clone(),
            selected: repositories.first().unwrap().to_owned(),
            url: String::default(),
        }
    }
}

impl ClonePanel {
    pub(crate) fn ui(&self) -> iced::Element<'_, Message> {
        let repositories = self
            .repositories
            .iter()
            .map(|repo| {
                button(text(&repo.name))
                    .on_press(Message::SelectRepo(repo.clone()))
                    .width(iced::Length::Fill)
                    .into()
            })
            .collect::<Vec<iced::Element<'_, Message>>>();
        column(repositories).into()
    }
}
