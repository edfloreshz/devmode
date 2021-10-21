use std::fmt::{Display, Formatter};

#[derive(Copy, Clone)]
pub enum Host<'a> {
    GitHub(&'a str),
    GitLab(&'a str),
}

impl<'a> Host<'a> {
    pub fn url(&self) -> &'a str {
        match self {
            Host::GitHub(_) => "https://github.com",
            Host::GitLab(_) => "https://gitlab.com",
        }
    }
    pub fn from(text: String) -> Option<Self> {
        match text.to_lowercase().as_str() {
            "github.com" | "github" | "gh" => Some(Host::GitHub("GitHub")),
            "gitlab.com" | "gitlab" | "gl" => Some(Host::GitLab("GitLab")),
            _ => None,
        }
    }
}

impl<'a> Display for Host<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Host::GitHub(host) => write!(f, "{}", host),
            Host::GitLab(host) => write!(f, "{}", host),
        }
    }
}
