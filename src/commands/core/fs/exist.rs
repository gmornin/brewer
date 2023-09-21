use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::{V1PathOnly, V1Response};
use log::*;

use crate::{
    exit_codes::loggedin_only,
    functions::{get_url, post, v1_handle},
    CREDS,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "exist")]
/// Checks if file item exists.
pub struct Exist {
    #[argp(positional)]
    /// Path to file system item.
    pub path: String,
}

impl CommandTrait for Exist {
    fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }

        trace!("Logged in, proceeding with checking item existence.");

        let path = self.path.trim_matches('/');
        let body = V1PathOnly {
            token: creds.token.clone(),
            path: path.to_string(),
        };

        let url = get_url("/api/storage/v1/exists");

        let res: V1Response = post(&url, body)?;
        v1_handle(&res)?;

        Ok(())
    }
}
