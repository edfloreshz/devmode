use std::fmt::{Display, Formatter};

use crate::constants::names::*;
use crate::constants::url::{GH_URL, GL_URL};

#[derive(Debug, Clone)]
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
        let text = text.to_lowercase();
        if text.contains("github") || text.eq(&"gh") {
            Host::GitHub
        } else if text.contains("gitlab") || text.eq(&"gl") {
            Host::GitLab
        } else {
            Host::None
        }
    }
}

impl Display for Host {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Host::GitHub => GH_NAME,
                Host::GitLab => GL_NAME,
                Host::None => NONE,
            }
        )
    }
}
