use crate::constants::names::*;
use crate::constants::url::{GH_URL, GL_URL};
use std::fmt::{Display, Formatter};

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
    pub fn from(text: &String) -> Self {
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

pub fn is_host(args: &Vec<String>) -> bool {
    let mut contains = false;
    for arg in args {
        if arg.contains("gh")
            || arg.contains("gl")
            || arg.contains("github")
            || arg.contains("gitlab")
            || arg.contains("github.com")
            || arg.contains("gitlab.com")
        {
            contains = true;
            break;
        }
    }
    contains
}

impl Display for Host {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}