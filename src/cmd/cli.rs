use clap::ArgMatches;
use regex::bytes::Regex;
use requestty::Answer;

use crate::models::config::AppOptions;
use crate::models::editor::{Editor, EditorApp};
use crate::models::logic::*;
use crate::utils::git::parse_url;

pub fn parse<'a>(matches: &'a ArgMatches<'a>) -> Cmd<'a> {
    if let Some(matches) = matches.subcommand_matches("clone") {
        let args = matches.values_of("args").unwrap().collect::<Vec<_>>();
        let url = args.get(0).map(|v| *v).unwrap();
        let rx = Regex::new(r#"((utils@|http(s)?://)(?P<host>[\w.@]+)([/:]))(?P<owner>[\w,\-_]+)/(?P<repo>[\w,\-_]+)(.utils)?((/)?)"#).unwrap();
        if rx.is_match(url.as_ref()) {
            let clone = parse_url(url, rx);
            Cmd::Clone(clone)
        } else {
            Cmd::Clone(Clone::new(
                Host::from(url),
                args.get(1).map(|v| *v),
                args.get(2).map(|v| *v),
            ))
        }
    } else if let Some(open) = matches.subcommand_matches("open") {
        Cmd::Open(Open {
            project: open.value_of("project"),
        })
    } else if let Some(config) = matches.subcommand_matches("config") {
        if config.is_present("editor") {
            let question = requestty::Question::expand("overwrite")
                .message("Choose your favorite editor:")
                .choices(vec![('v', "Vim"), ('c', "VSCode"), ('n', "Nano")])
                .default_separator()
                .choice('x', "Abort")
                .build();
            let answer = match requestty::prompt_one(question).unwrap() {
                Answer::ExpandItem(exp) => exp.key,
                _ => ' ',
            };
            Cmd::Config(AppOptions::new(Editor::new(EditorApp::from(answer))))
        } else {
            Cmd::Config(AppOptions::new(Editor::new(EditorApp::Vim)))
        }
    } else {
        Cmd::None
    }
}
