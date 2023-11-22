use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::{V1Response, V1TokenPassword};
use log::*;

use crate::{
    exit_codes::loggedin_only,
    functions::{doasisay, get_url, post, v1_handle},
    CREDS,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "delete")]
/// Delete an existing GM account.
pub struct Delete {
    #[argp(option, default = "crate::functions::read_pw()", short = 'p')]
    /// You will be prompted to enter your password securely if you skip this option.
    pub password: String,
}

#[async_trait::async_trait]
impl CommandTrait for Delete {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }
        trace!("Logged in, proceeding with deletion.");

        doasisay("delete account").await;

        let body = V1TokenPassword {
            token: creds.token.clone(),
            password: self.password.clone(),
        };

        let url = get_url("/api/accounts/v1/delete").await;

        let res: V1Response = post(&url, body).await?;
        v1_handle(&res)?;

        Ok(())
    }
}
