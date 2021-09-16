use {
    thiserror::Error,
    crate::Result,
    colored::Colorize,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("The argument '{0}' was not provided")]
    ArgumentMissing(String),

    #[error("You can't do this unless you set your configuration with `devmode config`\n \
    In the meantime, you can clone by specifying <host> <owner> <repo> \n\n\
    Host should be one of the following: \n1. GitHub \n2. GitLab")]
    CloneWithNoConfig,

    #[error("Missing arguments: <owner> <repo>")]
    MissingCloneOwnerRepo,

    #[error("Missing argument: <repo>")]
    MissingCloneRepo,

    // #[error("There was a problem cloning : {0}")]
    // CloneError(String),

    #[error("{0}")]
    GenericFailure(String),
}

pub fn downcast_err(result: Result<()>) {
    if let Err(e) = result {
        if let Some(e) = e.downcast_ref::<clap::Error>() {
            println!("{}", e.message)
        } else if let Some(e) = e.downcast_ref::<git2::Error>() {
            println!("{} {}", "error:".red(), e.message())
        } else if let Some(e) = e.downcast_ref::<std::io::Error>() {
            println!("{} {}", "error:".red(), e.to_string())
        } else {
            println!("{:?}", e);
        }
    }
}
