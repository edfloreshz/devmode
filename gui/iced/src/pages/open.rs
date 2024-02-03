use iced::widget::text;

use crate::app::{Message, Repository};

#[derive(Debug)]
pub struct OpenPanel {
    repositories: Vec<Repository>,
    selected: Repository,
}

impl Default for OpenPanel {
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
        }
    }
}

impl OpenPanel {
    pub(crate) fn ui(&self) -> iced::Element<'_, Message> {
        text("Open").into()
    }
}
