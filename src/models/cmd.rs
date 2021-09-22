use anyhow::bail;
use clap::ArgMatches;
use regex::bytes::Regex;

use crate::cmd::cli::GIT_URL;
use crate::cmd::cli::{clone_setup, config_all, config_editor, config_host, config_owner};
use crate::models::clone::Clone;
use crate::models::config::{AppOptions, ConfigWriter};
use crate::models::host::Host;
use crate::models::project::Project;
use crate::Result;

pub enum Cmd<'a> {
    Clone(Clone<'a>),
    Open(Project<'a>),
    Config(Option<AppOptions>),
    ShowConfig,
    MapPaths,
    None,
}

impl<'a> Cmd<'a> {
    pub fn new(matches: &'a ArgMatches<'a>) -> Result<Cmd<'a>> {
        if let Some(matches) = matches.subcommand_matches("clone") {
            let args = matches
                .values_of("args")
                .unwrap_or_default()
                .collect::<Vec<_>>();
            let url = args.get(0).copied().unwrap_or_default();
            let rx = Regex::new(GIT_URL).unwrap();
            if args.is_empty() {
                clone_setup()
            } else if rx.is_match(url.as_ref()) {
                let clone = Clone::parse_url(url, rx)?;
                Ok(Cmd::Clone(clone))
            } else if let Some(options) = AppOptions::current() {
                let host = Host::from(options.host);
                let owner = Option::from(options.owner);
                let repo = args.get(0).map(|a| a.to_string());
                Ok(Cmd::Clone(Clone::new(host, owner, repo)))
            } else {
                let host = Host::from(url.into());
                let owner = args.get(1).map(|a| a.to_string());
                let repo = args.get(2).map(|a| a.to_string());
                Ok(Cmd::Clone(Clone::new(host, owner, repo)))
            }
        } else if let Some(open) = matches.subcommand_matches("open") {
            Ok(Cmd::Open(Project {
                name: open.value_of("project"),
            }))
        } else if let Some(config) = matches.subcommand_matches("config") {
            if config.is_present("all") {
                config_all()
            } else if config.is_present("map") {
                Ok(Cmd::MapPaths)
            } else if AppOptions::current().is_some() {
                return if config.is_present("editor") {
                    config_editor()
                } else if config.is_present("owner") {
                    config_owner()
                } else if config.is_present("host") {
                    config_host()
                } else if config.is_present("show") {
                    Ok(Cmd::ShowConfig)
                } else {
                    Ok(Cmd::Config(AppOptions::current()))
                };
            } else {
                config_all()
            }
        } else {
            Ok(Cmd::None)
        }
    }
    pub fn check(&self) -> Result<()> {
        match self {
            Cmd::Clone(clone) => {
                if clone.host.is_none() {
                    bail!("You can't do this unless you set your configuration with `devmode config`\n\
                    In the meantime, you can clone by specifying <host> <owner> <repo> \n\n\
                    Host should be one of the following: \n1. GitHub \n2. GitLab")
                } else if clone.owner.is_none() {
                    bail!("Missing arguments: <owner> <repo>")
                } else if clone.repo.is_none() {
                    bail!("Missing arguments: <repo>")
                } else {
                    match self.clone() {
                        Ok(_) => Project::make_dev_paths(),
                        Err(e) => Err(e),
                    }
                }
            }
            Cmd::Open(open) => {
                if let Some(_project) = open.name {
                    self.open()
                } else {
                    bail!("Project name was not provided")
                }
            }
            Cmd::Config(options) => options.as_ref().unwrap().write_to_config(),
            Cmd::ShowConfig => {
                AppOptions::current().unwrap().show();
                Ok(())
            }
            Cmd::None => bail!("No argument found."),
            Cmd::MapPaths => Project::make_dev_paths(),
        }
    }
    fn clone(&self) -> Result<()> {
        if let Cmd::Clone(clone) = self {
            return clone.clone();
        }
        Ok(())
    }
    fn open(&self) -> Result<()> {
        if let Cmd::Open(project) = self {
            return project.open();
        }
        Ok(())
    }
}
