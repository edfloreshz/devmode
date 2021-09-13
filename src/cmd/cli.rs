use clap::{ArgMatches, Error, ErrorKind};
use regex::bytes::Regex;

use crate::cmd::logic::*;

pub fn parse<'a>(matches: &'a ArgMatches<'a>) -> Cmd<'a> {
    if let Some(matches) = matches.subcommand_matches("clone") {
        let values = matches.values_of("args").unwrap().collect::<Vec<_>>();
        if values.len() == 1 {
            let url = values.get(0).map(|v| *v);
            let rx = Regex::new(
                r#"((git@|http(s)?://)([\w.@]+)([/:]))([\w,\-_]+)/([\w,\-_]+)(.git)?((/)?)"#,
            )
                .unwrap();
            let clone = if rx.is_match(url.unwrap().as_ref()) {
                // TODO: Create Clone from url.
                Clone::new(None, None, None)
            } else {
                Clone::new(None, None, None)
            };
            Cmd::Clone(clone)
        } else {
            Cmd::Clone(Clone::new(
                match values.get(0) {
                    None => None,
                    Some(host) => match host.to_lowercase().as_str() {
                        "github" | "gh" => Some(Host::GitHub(host)),
                        "gitlab" | "gl" => Some(Host::GitLab(host)),
                        _ => None,
                    },
                },
                values.get(1).map(|v| *v),
                values.get(2).map(|v| *v),
            ))
        }
    } else if let Some(open) = matches.subcommand_matches("open") {
        Cmd::Open(Open {
            project: open.value_of("project"),
        })
    } else {
        Cmd::None
    }
}
