use iced::widget::text;

use crate::app::Message;

#[derive(Debug)]
pub struct WorkspacesPanel {
    workspaces: Vec<Workspace>,
    selected: Workspace,
}

impl Default for WorkspacesPanel {
    fn default() -> Self {
        let workspaces: Vec<Workspace> = (0..20)
            .map(|i| Workspace {
                name: i.to_string(),
            })
            .collect();
        Self {
            workspaces: workspaces.clone(),
            selected: workspaces.first().unwrap().to_owned(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Workspace {
    name: String,
}

impl WorkspacesPanel {
    pub(crate) fn ui(&self) -> iced::Element<'_, Message> {
        text("Workspaces").into()
    }
}
