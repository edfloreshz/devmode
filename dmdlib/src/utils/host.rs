use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub enum Host {
    GitHub,
    GitLab,
}

impl Host {
    pub fn url<'a>(&self) -> &'a str {
        match self {
            Host::GitHub => "https://github.com",
            Host::GitLab => "https://gitlab.com",
        }
    }
    pub fn from(text: String) -> Option<Self> {
        match text.to_lowercase().as_str() {
            "github.com" | "github" | "gh" => Some(Host::GitHub),
            "gitlab.com" | "gitlab" | "gl" => Some(Host::GitLab),
            _ => None,
        }
    }
    pub fn get(&self) -> &str {
        match self {
            Host::GitHub => "GitHub",
            Host::GitLab => "GitLab"
        }
    }
}

impl Display for Host {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Host::GitHub => write!(f, "{}", self.get()),
            Host::GitLab => write!(f, "{}", self.get()),
        }
    }
}
