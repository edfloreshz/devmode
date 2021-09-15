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

const GIT_URL: &str = r#"((utils@|http(s)?://)(?P<host>[\w.@]+)([/:]))(?P<owner>[\w,\-_]+)/(?P<repo>[\w,\-_]+)(.utils)?((/)?)"#;

pub fn parse<'a>(matches: &'a ArgMatches<'a>) -> Cmd<'a> {
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
            let clone = parse_url(url, rx);
            Cmd::Clone(clone)
        } else if let Some(options) = AppOptions::current() {
            let host = Host::from(options.host);
            let owner = Option::from(options.owner);
            let repo = args.get(0).map(|a| a.to_string());
            Cmd::Clone(Clone::new(host, owner, repo))
        } else {
            let host = Host::from(url.into());
            let owner = args.get(1).map(|a| a.to_string());
            let repo = args.get(2).map(|a| a.to_string());
            Cmd::Clone(Clone::new(host, owner, repo))
        }
    } else if let Some(open) = matches.subcommand_matches("open") {
        Cmd::Open(Open { project: open.value_of("project") })
    } else if let Some(config) = matches.subcommand_matches("config") {
        if config.is_present("all") {
            config_all()
        } else if AppOptions::current().is_some() {
            return if config.is_present("editor") {
                config_editor()
            } else if config.is_present("owner") {
                config_owner()
            } else if config.is_present("host") {
                config_host()
            } else if config.is_present("show") {
                Cmd::ShowConfig
            } else {
                Cmd::Config(AppOptions::current())
            };
        } else {
            config_all()
        }
    } else {
        Cmd::None
    }
}

fn clone_setup<'a>() -> Cmd<'a> {
    let mut clone = Clone::new(None, None, None);
    let question = requestty::Question::select("host")
        .message("Choose your Git host:")
        .choices(vec!["GitHub", "GitLab"])
        .build();
    if let Answer::ListItem(host) = requestty::prompt_one(question).unwrap() {
        clone.host = Host::from(host.text);
    }
    let question = requestty::Question::input("owner")
        .message("Git username:")
        .build();
    if let Answer::String(owner) = requestty::prompt_one(question).unwrap() {
        clone.owner = Option::from(owner);
    }
    let question = requestty::Question::input("repo")
        .message("Git repo name:")
        .build();
    if let Answer::String(repo) = requestty::prompt_one(question).unwrap() {
        clone.repo = Option::from(repo);
    }
    Cmd::Clone(clone)
}

fn config_all<'a>() -> Cmd<'a> {
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

fn config_owner<'a>() -> Cmd<'a> {
    let question = requestty::Question::input("owner")
        .message("What's your Git username:")
        .build();
    let mut options = AppOptions::current().unwrap();
    if let Answer::String(owner) = requestty::prompt_one(question).unwrap() {
        options.owner = owner;
    }
    Cmd::Config(Option::from(options))
}

fn config_host<'a>() -> Cmd<'a> {
    let question = requestty::Question::select("host")
        .message("Choose your Git host:")
        .choices(vec!["GitHub", "GitLab"])
        .build();
    let mut options = AppOptions::current().unwrap();
    if let Answer::ListItem(host) = requestty::prompt_one(question).unwrap() {
        options.host = Host::from(host.text).unwrap().to_string();
    }
    Cmd::Config(Option::from(options))
}

fn config_editor<'a>() -> Cmd<'a> {
    let question = requestty::Question::select("editor")
        .message("Choose your favorite editor:")
        .choices(vec!["Vim", "Nano", "VSCode"])
        .build();
    let mut options = AppOptions::current().unwrap();
    if let Answer::ListItem(i) = requestty::prompt_one(question).unwrap() {
        options.editor = Editor::new(EditorApp::from(&*i.text));
    }
    Cmd::Config(Option::from(options))
}