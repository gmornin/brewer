use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::{V1ChangeEmail, V1Response};
use log::*;

use crate::{
    exit_codes::loggedin_only,
    functions::{doasisay, get_url, post, v1_handle},
    CREDS,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "email")]
/// Change account email address.
pub struct Email {
    #[argp(positional)]
    /// Your new linked email address.
    pub new: String,
    #[argp(option, default = "crate::functions::read_pw()", short = 'p')]
    /// You will be prompted to enter your password securely if you skip this option.
    pub password: String,
}

impl CommandTrait for Email {
    fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }

        doasisay("changing email");

        trace!("Logged in, proceeding with regenerating token.");
        let body = V1ChangeEmail {
            token: creds.token.clone(),
            password: self.password.clone(),
            new: self.new.clone(),
        };

        let url = get_url("/api/accounts/v1/change-email");

        let res: V1Response = post(&url, body)?;
        v1_handle(&res)?;

        Ok(())
    }
}
