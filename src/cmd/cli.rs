use anyhow::Context;
use {
    crate::models::config::AppOptions,
    crate::models::editor::{Editor, EditorApp},
    crate::models::logic::Clone,
    crate::models::logic::*,
    crate::utils::git::parse_url,
    crate::Result,
    clap::ArgMatches,
    regex::bytes::Regex,
    requestty::Answer,
};

const GIT_URL: &str = r#"((utils@|http(s)?://)(?P<host>[\w.@]+)([/:]))(?P<owner>[\w,\-_]+)/(?P<repo>[\w,\-_]+)(.utils)?((/)?)"#;

pub fn parse<'a>(matches: &'a ArgMatches<'a>) -> Result<Cmd<'a>> {
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
            let clone = parse_url(url, rx)?;
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

fn clone_setup<'a>() -> Result<Cmd<'a>> {
    let mut clone = Clone::new(None, None, None);
    let question = requestty::Question::select("host")
        .message("Choose your Git host:")
        .choices(vec!["GitHub", "GitLab"])
        .build();
    if let Answer::ListItem(host) = requestty::prompt_one(question)? {
        clone.host = Host::from(host.text);
    }
    let question = requestty::Question::input("owner")
        .message("Git username:")
        .validate(|owner, _previous| {
            if owner.is_empty() {
                Err("Please enter a Git username.".to_owned())
            } else {
                Ok(())
            }
        })
        .build();
    if let Answer::String(owner) = requestty::prompt_one(question)? {
        clone.owner = Option::from(owner);
    }
    let question = requestty::Question::input("repo")
        .message("Git repo name:")
        .validate(|owner, _previous| {
            if owner.is_empty() {
                Err("Please enter a Git repo name.".to_owned())
            } else {
                Ok(())
            }
        })
        .build();
    if let Answer::String(repo) = requestty::prompt_one(question)? {
        clone.repo = Option::from(repo);
    }
    Ok(Cmd::Clone(clone))
}

fn config_all<'a>() -> Result<Cmd<'a>> {
    let editor: String = match requestty::prompt_one(
        requestty::Question::select("editor")
            .message("Choose your favorite editor:")
            .choices(vec!["Vim", "Nano", "VSCode"])
            .build(),
    )? {
        Answer::ListItem(editor) => editor.text,
        _ => String::new(),
    };
    let owner: String = match requestty::prompt_one(
        requestty::Question::input("owner")
            .message("What's your Git username:")
            .validate(|owner, _previous| {
                if owner.is_empty() {
                    Err("Please enter a Git username.".to_owned())
                } else {
                    Ok(())
                }
            })
            .build(),
    )? {
        Answer::String(owner) => owner,
        _ => String::new(),
    };
    let host: String = match requestty::prompt_one(
        requestty::Question::select("host")
            .message("Choose your Git host:")
            .choices(vec!["GitHub", "GitLab"])
            .build(),
    )? {
        Answer::ListItem(host) => host.text,
        _ => String::new(),
    };
    Ok(Cmd::Config(Some(AppOptions::new(
        host,
        owner,
        Editor::new(EditorApp::from(editor.as_str())),
    ))))
}

fn config_owner<'a>() -> Result<Cmd<'a>> {
    let question = requestty::Question::input("owner")
        .message("What's your Git username:")
        .validate(|owner, _previous| {
            if owner.is_empty() {
                Err("Please enter a Git username.".to_owned())
            } else {
                Ok(())
            }
        })
        .build();
    let mut options = AppOptions::current().with_context(|| "Couldn't get current settings.")?;
    if let Answer::String(owner) = requestty::prompt_one(question)? {
        options.owner = owner;
    }
    Ok(Cmd::Config(Option::from(options)))
}

fn config_host<'a>() -> Result<Cmd<'a>> {
    let question = requestty::Question::select("host")
        .message("Choose your Git host:")
        .choices(vec!["GitHub", "GitLab"])
        .build();
    let mut options = AppOptions::current().with_context(|| "Couldn't get current settings.")?;
    if let Answer::ListItem(host) = requestty::prompt_one(question)? {
        options.host = Host::from(host.text)
            .with_context(|| "Couldn't get a host.")?
            .to_string();
    }
    Ok(Cmd::Config(Option::from(options)))
}

fn config_editor<'a>() -> Result<Cmd<'a>> {
    let question = requestty::Question::select("editor")
        .message("Choose your favorite editor:")
        .choices(vec!["Vim", "Nano", "VSCode"])
        .build();
    let mut options = AppOptions::current().with_context(|| "Couldn't get current settings.")?;
    if let Answer::ListItem(i) = requestty::prompt_one(question)? {
        options.editor = Editor::new(EditorApp::from(&*i.text));
    }
    Ok(Cmd::Config(Option::from(options)))
}
