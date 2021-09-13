use clap::{Error, ErrorKind};

pub enum Cmd<'a> {
    Clone(Clone<'a>),
    Open(Open<'a>),
    None,
}

impl<'a> Cmd<'a> {
    pub(crate) fn check(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Cmd::Clone(clone) => {
                if clone.host.is_none() {
                    Err(Box::new(Error::with_description(
                        "Host should be one of the following: \n1. GitHub \n2. GitLab",
                        ErrorKind::ArgumentNotFound,
                    )))
                } else if clone.owner.is_none() {
                    Err(Box::new(Error::with_description(
                        "Missing arguments: <owner> <repo>",
                        ErrorKind::ArgumentNotFound,
                    )))
                } else if clone.repo.is_none() {
                    Err(Box::new(Error::with_description(
                        "Missing argument: <repo>",
                        ErrorKind::ArgumentNotFound,
                    )))
                } else {
                    self.clone()
                }
            }
            Cmd::Open(open) => {
                if let Some(_project) = open.project {
                    self.open()
                } else {
                    Err(Box::new(Error::with_description(
                        "The argument <project> was not provided.",
                        ErrorKind::ArgumentNotFound,
                    )))
                }
            }
            Cmd::None => Err(Box::new(Error::with_description(
                "No argument found.",
                ErrorKind::ArgumentNotFound,
            ))),
        }
    }
    fn clone(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Cmd::Clone(clone) = self {
            crate::git::actions::clone(clone)
        } else {
            Ok(())
        }
    }
    fn open(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Cmd::Open(e) = self {
            println!("{}", e.project.unwrap())
        }
        Ok(())
    }
}

pub enum Host<'a> {
    GitHub(&'a str),
    GitLab(&'a str),
}

impl<'a> Host<'a> {
    pub fn get_url(&self) -> &'a str {
        match self {
            Host::GitHub(_) => "https://github.com",
            Host::GitLab(_) => "https://gitlab.com",
        }
    }
    pub fn from(text: &'a str) -> Option<Self> {
        match text {
            "github.com" => Some(Host::GitHub("github")),
            "gitlab.com" => Some(Host::GitHub("gitlab")),
            _ => None
        }
    }
}

pub struct Clone<'a> {
    pub(crate) host: Option<Host<'a>>,
    pub(crate) owner: Option<&'a str>,
    pub(crate) repo: Option<&'a str>,
}

impl<'a> Clone<'a> {
    pub(crate) fn new(
        host: Option<Host<'a>>,
        owner: Option<&'a str>,
        repo: Option<&'a str>,
    ) -> Self {
        Clone { host, owner, repo }
    }
    pub(crate) fn url(&self) -> String {
        format!(
            "{}/{}/{}",
            self.host.as_ref().unwrap().get_url(),
            self.owner.as_ref().unwrap(),
            self.repo.as_ref().unwrap()
        )
    }
}

pub struct Open<'a> {
    pub(crate) project: Option<&'a str>,
}
