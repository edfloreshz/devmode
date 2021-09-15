pub mod custom {
    pub struct ArgumentNotFound {}

    impl ArgumentNotFound {
        pub fn from(msj: &str) -> Box<dyn std::error::Error> {
            Box::new(clap::Error::with_description(
                msj,
                clap::ErrorKind::ArgumentNotFound,
            ))
        }
    }
}
