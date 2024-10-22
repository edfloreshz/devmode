use std::fmt::Display;

#[derive(Debug)]
pub enum DevmodeStatus {
    RepositoryCloned,
    RepositoryUpdated(String),
    NoSettingsChanged,
    SettingsUpdated,
    UnableToMapUrl,
    NoEditorSet,
}

impl Display for DevmodeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DevmodeStatus::RepositoryCloned => write!(f, "Repository cloned successfully! 🎉️"),
            DevmodeStatus::RepositoryUpdated(name) => write!(f, "Updating project: {}...", name),
            DevmodeStatus::NoSettingsChanged => write!(f, "No settings were changed"),
            DevmodeStatus::SettingsUpdated => write!(f, "Settings updated"),
            DevmodeStatus::UnableToMapUrl => write!(f, "Failed to map url"),
            DevmodeStatus::NoEditorSet => {
                write!(f, "No editor set, run `dm config -e` to configure it")
            }
        }
    }
}

pub fn report(status: DevmodeStatus) {
    println!("{}", status.to_string());
}
