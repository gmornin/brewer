use std::error::Error;

use argp::FromArgs;
use goodmorning_bindings::services::v1::{V1All3, V1Response};

use crate::{
    functions::{get_url, post, v1_handle},
    CREDS,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "create")]
/// Create a new GM account.
pub struct Create {
    #[argp(positional, short = 'u')]
    /// Identify yourself.
    pub username: String,
    #[argp(positional, short = 'e')]
    /// Email for verification.
    pub email: String,
    #[argp(option, default = "crate::functions::read_pw_confirm()")]
    /// You will be prompted to enter your password securely if you skip this option.
    pub password: String,
    #[argp(option, short = 'i')]
    /// Instance domain or IP.
    pub instance: String,
}

impl Create {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        unsafe { CREDS.get().unwrap().is_loggedin() };
        let body = V1All3 {
            username: self.username.clone(),
            email: self.email.clone(),
            password: self.password.clone(),
        };
        let url = get_url("/api/accounts/v1/create");

        let res: V1Response = post(&url, body)?;
        v1_handle(&res)?;

        Ok(())
    }
}
