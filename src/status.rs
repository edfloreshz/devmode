use std::fmt::Display;

#[derive(Debug)]
pub enum DevmodeStatus {
    NoProjectFound,
    NoSettingsChanged,
    SettingsUpdated,
    FailedToWriteSettings,
    FailedToParseSettings,
    UnableToMapUrl,
    FailedToCloneRepository,
    FailedToSetRemote,
    FailedToGetBranch,
    OpenedProjectWithWarning,
    NoEditorSet,
    AppSettingsNotFound,
}

impl Display for DevmodeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DevmodeStatus::NoProjectFound => write!(f, "No project was found"),
            DevmodeStatus::NoSettingsChanged => write!(f, "No settings were changed"),
            DevmodeStatus::SettingsUpdated => write!(f, "Settings updated"),
            DevmodeStatus::FailedToWriteSettings => write!(f, "Failed to write settings"),
            DevmodeStatus::FailedToParseSettings => write!(f, "Failed to parse settings"),
            DevmodeStatus::UnableToMapUrl => write!(f, "Failed to map url"),
            DevmodeStatus::FailedToCloneRepository => write!(f, "Failed to clone repository"),
            DevmodeStatus::FailedToSetRemote => write!(f, "Failed to set remote repository"),
            DevmodeStatus::FailedToGetBranch => write!(f, "Failed to get branch"),
            DevmodeStatus::OpenedProjectWithWarning => {
                write!(
                    f,
                    "If the editor does not support openning from a path, open it yourself"
                )
            }
            DevmodeStatus::NoEditorSet => {
                write!(f, "No editor set, run `dm config -e` to configure it")
            }
            DevmodeStatus::AppSettingsNotFound => {
                write!(f, "The current app options could not be found.\nRun `dm cf --all` to reconfigure them")
            }
        }
    }
}

pub fn info(status: DevmodeStatus) {
    println!("{}", status.to_string());
}
