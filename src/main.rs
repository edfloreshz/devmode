use clap::{App, AppSettings, Arg, SubCommand};

use crate::cmd::cli::parse;
use colored::Colorize;

mod cmd;
mod git;

fn main() {
    let matches = App::new("(Dev)mode")
        .name("(Dev)mode")
        .version("0.1.0")
        .author("Eduardo F. <edfloreshz@gmail.com>")
        .about("Dev(mode) is a project management utility for developers.")
        .settings(&[AppSettings::ArgRequiredElseHelp, AppSettings::ColoredHelp])
        .subcommands(vec![
            SubCommand::with_name("clone")
                .arg(
                    Arg::with_name("args")
                        .help("Provide either a Git <url> or a Git <host> <owner> <repo>.")
                        .required(true)
                        .min_values(1),
                )
                .about("Clones a git repository in a specific folder structure."),
            SubCommand::with_name("open")
                .arg(
                    Arg::with_name("project")
                        .help("Provide a project name.")
                        .takes_value(true)
                        .required(true),
                )
                .about("Opens a project on your selected text editor."),
        ])
        .get_matches();
    let cmd = parse(&matches);
    if let Err(e) = cmd.check() {
        if let Some(e) = e.downcast_ref::<clap::Error>() {
            println!("{}", e.message)
        } else if let Some(e) = e.downcast_ref::<git2::Error>() {
            println!("{} {}", "error:".red(), e.message())
        } else {
            println!("{:?}", e);
        }
    }
}
