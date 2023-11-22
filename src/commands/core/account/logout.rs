use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use config_macro::ConfigTrait;
use log::*;

use crate::{exit_codes::loggedin_only, CREDS};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "logout")]
/// Logout of saved GM account.
pub struct Logout {}

#[async_trait::async_trait]
impl CommandTrait for Logout {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }
        trace!("Logged in, proceeding with logout.");

        creds.clear();
        creds.save()?;
        println!("Account login details have been removed.");

        Ok(())
    }
}
