pub mod patterns {
    pub const GIT_URL: &str = r#"((utils@|http(s)?://)(?P<host>[\w.@]+)([/:]))(?P<owner>[\w,\-_]+)/(?P<repo>[\w,\-_]+)(.utils)?((/)?)"#;
    pub const REGULAR_GIT_URL: &str = r#"(?:git@|https://)(?P<host>:?github|gitlab)[.]com[:/](?P<owner>[^\s,]+)[/](?P<repo>[^\s,.]+)([.]git)?"#;
    pub const ORG_GIT_URL: &str = r#"(?:git@|https://)(?P<host>gitlab[.][^\s]+[.][^\s]+)[:/](?P<owner>[^\s]+)[/](?P<repo>[^\s,.]+)([.]git)?"#;
}

pub mod names {
    pub const VSCODE_NAME: &str = "Visual Studio Code";
    pub const VIM_NAME: &str = "Vim";
    pub const CUSTOM_NAME: &str = "Custom";
    pub const GH_NAME: &str = "GitHub";
    pub const GL_NAME: &str = "GitLab";
    pub const NONE: &str = "None";
}

pub mod url {
    pub const GH_URL: &str = "github.com";
    pub const GL_URL: &str = "gitlab.com";
}

pub mod commands {
    pub const VSCODE_CMD: &str = "code";
    pub const VIM_CMD: &str = "vim";
}

pub mod messages {
    pub const NO_PROJECT_FOUND: &str = "No project was found. \n\
        If you know this project exists, run `dm config -m` to refresh the paths file.";
    pub const MORE_PROJECTS_FOUND: &str = "Two or more projects found.";
    pub const NO_SETTINGS_CHANGED: &str = "No settings were changed.";
    pub const SETTINGS_UPDATED: &str = "Settings updated.";
    pub const FAILED_TO_WRITE_CONFIG: &str = "Failed to write changes to `settings.toml`.";
    pub const FAILED_TO_PARSE: &str = "Failed to parse app options.";
    pub const UNABLE_TO_MAP_URL: &str = "Could not map url.";
    pub const FAILED_TO_CLONE_REPO: &str = "Failed to clone repository.";
    pub const FAILED_TO_SET_REMOTE: &str = "Failed to set remote.";
    pub const FAILED_TO_GET_BRANCH: &str = "Failed to get branch.";
    pub const OPENING_WARNING: &str =
        "If the editor does not support openning from a path, you'll have to open it yourself.";
    pub const NO_EDITOR_SET: &str = "No editor set, run `dm config -e` to configure it.";
    pub const APP_OPTIONS_NOT_FOUND: &str =
        "The current app options could not be found.\nRun `dm cf --all` to reconfigure them.";

    pub fn _failed_to(action: &str, obj: &str) -> String {
        format!("Failed to {} `{}`.", action, obj)
    }
}
