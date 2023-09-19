use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::{V1Response, V1SetStatus};
use log::*;

use crate::{
    exit_codes::loggedin_only,
    functions::{get_url, post, v1_handle},
    CREDS,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "status")]
/// Change your account wide status.
pub struct Status {
    #[argp(positional, greedy, default = "String::new()")]
    /// Your new status.
    new: String,
}

impl CommandTrait for Status {
    fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }
        trace!("Logged in, proceeding with changing status.");

        let body = V1SetStatus {
            token: creds.token.clone(),
            new: self.new.clone(),
        };

        let url = get_url("/api/accounts/v1/set-status");

        let res: V1Response = post(&url, body)?;
        v1_handle(&res)?;

        Ok(())
    }
}
