use anyhow::Context;
use requestty::Answer;

use crate::models::clone::Clone;
use crate::models::cmd::*;
use crate::models::config::AppOptions;
use crate::models::editor::{Editor, EditorApp};
use crate::models::host::Host;
use crate::Result;

pub const GIT_URL: &str = r#"((utils@|http(s)?://)(?P<host>[\w.@]+)([/:]))(?P<owner>[\w,\-_]+)/(?P<repo>[\w,\-_]+)(.utils)?((/)?)"#;

pub fn clone_setup<'a>() -> Result<Cmd<'a>> {
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

pub fn config_all<'a>() -> Result<Cmd<'a>> {
    let editor: String = match requestty::prompt_one(
        requestty::Question::select("editor")
            .message("Choose your favorite editor:")
            .choices(vec!["Vim", "VSCode"])
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

pub fn config_owner<'a>() -> Result<Cmd<'a>> {
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

pub fn config_host<'a>() -> Result<Cmd<'a>> {
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

pub fn config_editor<'a>() -> Result<Cmd<'a>> {
    let question = requestty::Question::select("editor")
        .message("Choose your favorite editor:")
        .choices(vec!["Vim", "VSCode"])
        .build();
    let mut options = AppOptions::current().with_context(|| "Couldn't get current settings.")?;
    if let Answer::ListItem(i) = requestty::prompt_one(question)? {
        options.editor = Editor::new(EditorApp::from(&*i.text));
    }
    Ok(Cmd::Config(Option::from(options)))
}
