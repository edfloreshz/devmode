use crate::Error;

pub trait Action {
    fn run(&mut self) -> Result<(), Error>;
}
