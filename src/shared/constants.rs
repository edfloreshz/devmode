pub mod patterns {
    pub const GIT_URL: &str = r#"((utils@|http(s)?://)(?P<host>[\w.@]+)([/:]))(?P<owner>[\w,\-_]+)/(?P<repo>[\w,\-_]+)(.utils)?((/)?)"#;
    pub const REGULAR_GIT_URL: &str = r#"(?:git@|https://)(?P<host>:?github|gitlab)[.]com[:/](?P<owner>[^\s,]+)[/](?P<repo>[^\s,.]+)([.]git)?"#;
    pub const ORG_GIT_URL: &str = r#"(?:git@|https://)(?P<host>gitlab[.][^\s]+[.][^\s]+)[:/](?P<owner>[^\s]+)[/](?P<repo>[^\s,.]+)([.]git)?"#;
}

pub const OS_SLASH: &str = if cfg!(target_os = "windows") {
    "\\"
} else {
    "/"
};

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
