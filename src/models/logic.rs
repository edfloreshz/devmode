use crate::error::custom::ArgumentNotFound;
use crate::models::config::{AppOptions, ConfigWriter};
use crate::Result;
use crate::utils::project;
use crate::utils::project::make_dev_paths;
use std::fmt::{Display, Formatter};

pub enum Cmd<'a> {
    Clone(Clone<'a>),
    Open(Open<'a>),
    Config(Option<AppOptions>),
    ShowConfig,
    None,
}

impl<'a> Cmd<'a> {
    pub fn check(&self) -> Result<()> {
        match self {
            Cmd::Clone(clone) => {
                if clone.host.is_none() {
                    Err(ArgumentNotFound::from(
                        "Host should be one of the following: \n1. GitHub \n2. GitLab",
                    ))
                } else if clone.owner.is_none() {
                    Err(ArgumentNotFound::from("Missing arguments: <owner> <repo>"))
                } else if clone.repo.is_none() {
                    Err(ArgumentNotFound::from("Missing argument: <repo>"))
                } else {
                    match self.clone() {
                        Ok(_) => make_dev_paths(),
                        Err(e) => Err(e),
                    }
                }
            }
            Cmd::Open(open) => {
                if let Some(_project) = open.project {
                    self.open()
                } else {
                    Err(ArgumentNotFound::from(
                        "The argument <project> was not provided",
                    ))
                }
            }
            Cmd::Config(options) => options.as_ref().unwrap().write_to_config(),
            Cmd::ShowConfig => Ok(AppOptions::current().unwrap().show()),
            Cmd::None => Err(ArgumentNotFound::from("No argument found")),
        }
    }
    fn clone(&self) -> Result<()> {
        if let Cmd::Clone(clone) = self {
            crate::utils::git::clone(clone)
        } else {
            Ok(())
        }
    }
    fn open(&self) -> Result<()> {
        if let Cmd::Open(open) = self {
            return project::open(open.project.unwrap());
        }
        Ok(())
    }
}

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

pub struct Clone<'a> {
    pub host: Option<Host<'a>>,
    pub owner: Option<String>,
    pub repo: Option<&'a str>,
}

impl<'a> Clone<'a> {
    pub fn new(host: Option<Host<'a>>, owner: Option<String>, repo: Option<&'a str>) -> Self {
        Clone { host, owner, repo }
    }
    pub fn url(&self) -> String {
        format!(
            "{}/{}/{}",
            self.host.as_ref().unwrap().url(),
            self.owner.as_ref().unwrap(),
            self.repo.as_ref().unwrap()
        )
    }
}

pub struct Open<'a> {
    pub(crate) project: Option<&'a str>,
}
