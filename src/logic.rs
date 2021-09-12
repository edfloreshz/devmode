use clap::{ArgMatches, Error, ErrorKind, Values};

pub enum Cmd {
    Clone(Clone),
    Open(Open),
}

enum Clone {
    URL(Option<String>),
    Provider(Option<String>),
    User(Option<String>),
    Repository(Option<String>),
}

enum Open {
    Project(Option<String>),
}

pub fn parse(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(clone) = matches.subcommand_matches("clone") {
        let values = clone.values_of("args").unwrap().collect::<Vec<_>>();
        if values.len() == 1 {
            Ok(println!("{}", values[0]))
        } else if values.len() == 3 {
            Ok(println!("{} - {} - {}", values[0], values[1], values[2]))
        } else {
            Err(Box::new(Error::with_description(
                "Possibly missing values: <provider> <user> <repo>",
                ErrorKind::ArgumentNotFound,
            )))
        }
    } else if let Some(open) = matches.subcommand_matches("open") {
        match open.value_of("project") {
            None => Err(Box::new(Error::with_description(
                "No project name was provided",
                ErrorKind::ArgumentNotFound,
            ))),
            Some(name) => Ok(println!("{}", name)),
        }
    } else {
        Err(Box::new(Error::with_description(
            "Missing arguments",
            ErrorKind::ArgumentNotFound,
        )))
    }
}
