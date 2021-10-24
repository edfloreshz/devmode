pub mod patterns {
    pub const GIT_URL: &str = r#"((utils@|http(s)?://)(?P<host>[\w.@]+)([/:]))(?P<owner>[\w,\-_]+)/(?P<repo>[\w,\-_]+)(.utils)?((/)?)"#;
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
    pub const GH_URL: &str = "https://github.com";
    pub const GL_URL: &str = "https://gitlab.com";
}

pub mod commands {
    pub const VSCODE_CMD: &str = "code";
    pub const VIM_CMD: &str = "vim";
}

pub mod messages {
    pub const NO_PROJECT_FOUND: &str = "No project was found. \n\
        If you know this project exists, run `devmode config -m, --map` to refresh the paths file.";
    pub const MORE_PROJECTS_FOUND: &str = "Two or more projects found.";
    pub const NO_SETTINGS_CHANGED: &str = "No settings were changed.";
    pub const SETTINGS_UPDATED: &str = "Settings updated.";
    pub const FAILED_TO_WRITE_CONFIG: &str = "Failed to write changes to `config.toml`.";
    pub const FAILED_TO_PARSE: &str = "Failed to parse app options.";
    pub const UNABLE_TO_MAP_URL: &str = "Could not map url.";
    pub const FAILED_TO_CLONE_REPO: &str = "Failed to clone repository.";
    pub const FAILED_TO_SET_REMOTE: &str = "Failed to set remote.";
    pub const DATA_DIR_NOT_CREATED: &str = "Data directory not yet created.";
    pub const HOME_DIR_NOT_CREATED: &str = "Home directory not yet created.";
    pub const OPENING_WARNING: &str =
        "If the editor does not support openning from a path, you'll have to open it yourself.";
    pub const NO_EDITOR_SET: &str =
        "No editor set, run devmode config -e, --editor to configure it.";
    pub const APP_OPTIONS_NOT_FOUND: &str = "The current app options could not be found.";

    pub fn failed_to(action: &str, obj: &str) -> String {
        format!("Failed to {} `{}`.", action, obj)
    }
}

pub mod paths {
    pub mod folders {
        pub const DEVELOPER_DIR: &str = "Developer";
        pub const DEVMODE_DIR: &str = "devmode";
        pub const CONFIG_DIR: &str = "devmode/config";
        pub const LOGS_DIR: &str = "devmode/logs";
        pub const PATHS_DIR: &str = "devmode/paths";
    }

    pub mod files {
        pub const DEVPATHS_FILE: &str = "devmode/paths/devpaths";
        pub const CONFIG_FILE: &str = "devmode/config/config.toml";
    }
}
