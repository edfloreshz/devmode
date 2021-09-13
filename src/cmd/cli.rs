use clap::ArgMatches;
use regex::bytes::Regex;

use crate::cmd::logic::*;
use crate::git::actions::parse_url;

pub fn parse<'a>(matches: &'a ArgMatches<'a>) -> Cmd<'a> {
    if let Some(matches) = matches.subcommand_matches("clone") {
        let args = matches.values_of("args").unwrap().collect::<Vec<_>>();
        let url = args.get(0).map(|v| *v).unwrap();
        let rx = Regex::new(r#"((git@|http(s)?://)(?P<host>[\w.@]+)([/:]))(?P<owner>[\w,\-_]+)/(?P<repo>[\w,\-_]+)(.git)?((/)?)"#).unwrap();
        if rx.is_match(url.as_ref()) {
            let clone = parse_url(url, rx);
            Cmd::Clone(clone)
        } else {
            Cmd::Clone(Clone::new(
                Host::from(args.get(0).map(|v| *v).unwrap()),
                args.get(1).map(|v| *v),
                args.get(2).map(|v| *v),
            ))
        }
    } else if let Some(open) = matches.subcommand_matches("open") {
        Cmd::Open(Open { project: open.value_of("project") })
    } else {
        Cmd::None
    }
}