use {
    crate::models::config::AppOptions,
    crate::models::editor::{Editor, EditorApp},
    crate::models::logic::Clone,
    crate::models::logic::*,
    crate::utils::git::parse_url,
    clap::ArgMatches,
    regex::bytes::Regex,
    requestty::Answer,
};

pub fn parse<'a>(matches: &'a ArgMatches<'a>) -> Cmd<'a> {
    if let Some(matches) = matches.subcommand_matches("clone") {
        let args = matches.values_of("args").unwrap().collect::<Vec<_>>();
        let url = args.get(0).copied().unwrap();
        let rx = Regex::new(r#"((utils@|http(s)?://)(?P<host>[\w.@]+)([/:]))(?P<owner>[\w,\-_]+)/(?P<repo>[\w,\-_]+)(.utils)?((/)?)"#).unwrap();
        if rx.is_match(url.as_ref()) {
            let clone = parse_url(url, rx);
            Cmd::Clone(clone)
        } else if let Some(options) = AppOptions::current() {
            Cmd::Clone(Clone::new(
                Host::from(options.host),
                Option::from(options.owner),
                args.get(0).copied(),
            ))
        } else {
            Cmd::Clone(Clone::new(
                Host::from(url.into()),
                args.get(1).map(|a| a.to_string()),
                args.get(2).copied(),
            ))
        }
    } else if let Some(open) = matches.subcommand_matches("open") {
        Cmd::Open(Open {
            project: open.value_of("project"),
        })
    } else if let Some(config) = matches.subcommand_matches("config") {
        if AppOptions::current().is_some() {
            if config.is_present("editor") {
                let question = requestty::Question::select("editor")
                    .message("Choose your favorite editor:")
                    .choices(vec!["Vim", "Nano", "VSCode"])
                    .build();
                let mut options = AppOptions::current().unwrap();
                if let Answer::ListItem(i) = requestty::prompt_one(question).unwrap() {
                    options.editor = Editor::new(EditorApp::from(&*i.text));
                }
                Cmd::Config(Option::from(options))
            } else if config.is_present("owner") {
                let question = requestty::Question::input("owner")
                    .message("What's your Git username:")
                    .build();
                let mut options = AppOptions::current().unwrap();
                if let Answer::String(owner) = requestty::prompt_one(question).unwrap() {
                    options.owner = owner;
                }
                Cmd::Config(Option::from(options))
            } else if config.is_present("host") {
                let question = requestty::Question::select("host")
                    .message("Choose your Git host:")
                    .choices(vec!["GitHub", "GitLab"])
                    .build();
                let mut options = AppOptions::current().unwrap();
                if let Answer::ListItem(host) = requestty::prompt_one(question).unwrap() {
                    options.host = Host::from(host.text).unwrap().to_string();
                }
                Cmd::Config(Option::from(options))
            } else if config.is_present("show") {
                Cmd::ShowConfig
            } else {
                Cmd::Config(AppOptions::current())
            }
        } else {
            let editor: String = match requestty::prompt_one(
                requestty::Question::select("editor")
                    .message("Choose your favorite editor:")
                    .choices(vec!["Vim", "Nano", "VSCode"])
                    .build(),
            )
                .unwrap()
            {
                Answer::ListItem(editor) => editor.text,
                _ => String::new(),
            };
            let owner: String = match requestty::prompt_one(
                requestty::Question::input("owner")
                    .message("What's your Git username:")
                    .build(),
            )
                .unwrap()
            {
                Answer::String(owner) => owner,
                _ => String::new(),
            };
            let host: String = match requestty::prompt_one(
                requestty::Question::select("host")
                    .message("Choose your Git host:")
                    .choices(vec!["GitHub", "GitLab"])
                    .build(),
            )
                .unwrap()
            {
                Answer::ListItem(host) => host.text,
                _ => String::new(),
            };
            Cmd::Config(Some(AppOptions::new(
                host,
                owner,
                Editor::new(EditorApp::from(editor.as_str())),
            )))
        }
    } else {
        Cmd::None
    }
}
