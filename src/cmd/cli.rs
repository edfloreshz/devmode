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
        let url = args.get(0).copied().unwrap();
        let rx = Regex::new(r#"((utils@|http(s)?://)(?P<host>[\w.@]+)([/:]))(?P<owner>[\w,\-_]+)/(?P<repo>[\w,\-_]+)(.utils)?((/)?)"#).unwrap();
        if rx.is_match(url.as_ref()) {
            let clone = parse_url(url, rx);
            Cmd::Clone(clone)
        } else {
            Cmd::Clone(Clone::new(
                Host::from(url),
                args.get(1).copied(),
                args.get(2).copied(),
            ))
        }
    } else if let Some(open) = matches.subcommand_matches("open") {
        Cmd::Open(Open {
            project: open.value_of("project"),
        })
    } else if let Some(config) = matches.subcommand_matches("config") {
        if config.is_present("editor") {
            let question = requestty::Question::select("editor")
                .message("Choose your favorite editor:")
                .choices(vec!["Vim", "Nano", "VSCode"])
                .build();
            if let Answer::ListItem(i) = requestty::prompt_one(question).unwrap() {
                let mut options = AppOptions::current();
                options.editor = Editor::new(EditorApp::from(&*i.text));
                return Cmd::Config(options);
            }
            Cmd::Config(AppOptions::current())
        } else if config.is_present("owner") {
            let question = requestty::Question::input("owner")
                .message("What's your Git username:")
                .build();
            if let Answer::String(owner) = requestty::prompt_one(question).unwrap() {
                let mut options = AppOptions::current();
                options.owner = owner;
                return Cmd::Config(options);
            }
            Cmd::Config(AppOptions::current())
        } else if config.is_present("host") {
            let question = requestty::Question::select("host")
                .message("Choose your Git host:")
                .choices(vec!["GitHub", "GitLab"])
                .build();
            if let Answer::ListItem(host) = requestty::prompt_one(question).unwrap() {
                let mut options = AppOptions::current();
                options.host = Host::from(&*host.text).unwrap().display().parse().unwrap();
                return Cmd::Config(options);
            }
            Cmd::Config(AppOptions::current())
        } else {
            // let questions = vec![
            //     requestty::Question::select("editor")
            //         .message("Choose your favorite editor:")
            //         .choices(vec!["Vim", "Nano", "VSCode"])
            //         .build(),
            //     requestty::Question::input("owner")
            //         .message("What's your Git username:")
            //         .build(),
            //     requestty::Question::select("host")
            //         .message("Choose your Git host:")
            //         .choices(vec!["GitHub", "GitLab"])
            //         .build(),
            // ];
            Cmd::Config(AppOptions::current())
        }
    } else {
        Cmd::None
    }
}
