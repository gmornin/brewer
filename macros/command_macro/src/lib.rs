use std::error::Error;

pub trait CommandTrait {
    fn run(&self) -> Result<(), Box<dyn Error>>;
}
