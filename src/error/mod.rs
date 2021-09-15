pub mod custom {
    use {
        crate::models::logic::Cmd,
        colored::Colorize,
    };

    pub struct ArgumentNotFound {}

    impl ArgumentNotFound {
        pub fn from(msj: &str) -> Box<dyn std::error::Error> {
            Box::new(clap::Error::with_description(
                msj,
                clap::ErrorKind::ArgumentNotFound,
            ))
        }
    }

    pub fn downcast_err(cmd: Cmd) {
        if let Err(e) = cmd.check() {
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
}
