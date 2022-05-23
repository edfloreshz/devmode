use std::fmt::{Display, Formatter};

use crate::constants::names::*;
use crate::constants::url::{GH_URL, GL_URL};

const VARIANTS: [&str; 6] = ["gh", "gl", "github", "gitlab", "github.com", "gitlab.com"];

#[derive(Clone)]
pub enum Host {
    GitHub,
    GitLab,
    None,
}

impl Host {
    pub fn url<'a>(&self) -> &'a str {
        match self {
            Host::GitHub => GH_URL,
            Host::GitLab => GL_URL,
            Host::None => "",
        }
    }
    pub fn from(text: &str) -> Self {
        match text.to_lowercase().as_str() {
            "github.com" | "github" | "gh" => Host::GitHub,
            "gitlab.com" | "gitlab" | "gl" => Host::GitLab,
            _ => Host::None,
        }
    }
    pub fn get(&self) -> &str {
        match self {
            Host::GitHub => GH_NAME,
            Host::GitLab => GL_NAME,
            Host::None => NONE,
        }
    }
}

pub fn is_host(host: String) -> bool {
    VARIANTS.contains(&host.as_str())
}

impl Display for Host {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}
