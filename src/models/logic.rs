use {
    crate::error::custom::ArgumentNotFound,
    crate::models::config::{AppOptions, ConfigWriter},
    crate::utils::project,
    crate::utils::project::make_dev_paths,
    crate::Result,
    std::fmt::{Display, Formatter},
};

pub enum Cmd<'a> {
    Clone(Clone<'a>),
    Open(Project<'a>),
    Edit(Project<'a>),
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
                        "You can't do this unless you set your configuration with `devmode config`\n\
                        In the meantime, you can clone by specifying <host> <owner> <repo> \n\n\
                        Host should be one of the following: \n1. GitHub \n2. GitLab",
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
                if let Some(_project) = open.name {
                    self.open()
                } else {
                    Err(ArgumentNotFound::from(
                        "The argument <project> was not provided",
                    ))
                }
            }
            Cmd::Config(options) => options.as_ref().unwrap().write_to_config(),
            Cmd::ShowConfig => {
                AppOptions::current().unwrap().show();
                Ok(())
            }
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
            return project::open(open.name.unwrap());
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
    pub repo: Option<String>,
}

impl<'a> Clone<'a> {
    pub fn new(host: Option<Host<'a>>, owner: Option<String>, repo: Option<String>) -> Self {
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

pub struct Project<'a> {
    pub(crate) name: Option<&'a str>,
}
