use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::{V1ChangePassword, V1Response};
use log::*;

use crate::{
    exit_codes::loggedin_only,
    functions::{get_url, post, v1_handle},
    CREDS,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "pwd")]
/// Change account password.
pub struct Pwd {
    #[argp(option, default = "crate::functions::read_pw_old()", short = 'p')]
    /// You will be prompted to enter your password securely if you skip this option.
    pub old: String,
    #[argp(option, default = "crate::functions::read_pw_confirm()", short = 'n')]
    /// You will be prompted to enter your password securely if you skip this option.
    pub new: String,
}

impl CommandTrait for Pwd {
    fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }

        trace!("Logged in, proceeding with regenerating token.");
        let body = V1ChangePassword {
            token: creds.token.clone(),
            old: self.old.clone(),
            new: self.new.clone(),
        };

        let url = get_url("/api/accounts/v1/change-password");

        let res: V1Response = post(&url, body)?;
        v1_handle(&res)?;

        Ok(())
    }
}
