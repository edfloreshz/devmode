mod logic;

use clap::{App, Arg, SubCommand};
use crate::logic::parse;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("(Dev)mode")
        .name("dm")
        .version("0.1.0")
        .author("Eduardo F. <edfloreshz@gmail.com>")
        .about("Dev(mode) is a project management utility for developers.")
        .subcommands(vec![
            SubCommand::with_name("clone")
                .arg(
                    Arg::with_name("args")
                        .help("Provide either a Git <url> or a Git <provider> <user> <repo>.")
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
    parse(&matches)
}

#[cfg(test)]
mod tests {
    use assert_cmd::prelude::*; // Add methods on commands
    use std::process::Command; // Run programs
    #[test]
    fn open_project() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("devmode")?;

        cmd.arg("open").arg("devmode");
        cmd.assert().success();

        Ok(())
    }
}
