use anyhow::Result;

pub trait Action {
    fn run(&mut self) -> Result<()>;
}
