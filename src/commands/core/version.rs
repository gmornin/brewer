use std::error::Error;

use argp::FromArgs;

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "version")]
/// Prints CLI version and exits.
pub struct Version {}

impl Version {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        println!(
            "{} {} (git {})",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            env!("GIT_HASH")
        );
        Ok(())
    }
}
