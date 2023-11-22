use std::error::Error;

#[async_trait::async_trait]
pub trait CommandTrait {
    async fn run(&self) -> Result<(), Box<dyn Error>>;
}
