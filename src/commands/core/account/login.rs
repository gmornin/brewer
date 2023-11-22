use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::{V1IdentifierType, V1PasswordId, V1Response};
use log::*;

use crate::{
    exit_codes::loggedin_not_allowed,
    functions::{get_url, post, v1_handle},
    CREDS, INSTANCE,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "login")]
/// Login to an existing GM account.
pub struct Login {
    #[argp(positional)]
    /// Username or email address.
    pub identifier: String,
    #[argp(positional)]
    /// Instance domain or IP.
    pub instance: String,
    #[argp(option, default = "crate::functions::read_pw()", short = 'p')]
    /// You will be prompted to enter your password securely if you skip this option.
    pub password: String,
}

#[async_trait::async_trait]
impl CommandTrait for Login {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        if unsafe { CREDS.get().unwrap().is_loggedin() } {
            loggedin_not_allowed()
        }

        trace!("Not logged in, proceeding with login.");
        unsafe { INSTANCE.set(self.instance.clone()).unwrap() };
        let r#type = if self.identifier.contains('@') {
            debug!("Identifier is an email address");
            V1IdentifierType::Email
        } else {
            debug!("Identifier is a username");
            V1IdentifierType::Username
        };

        let body = V1PasswordId {
            identifier: self.identifier.clone(),
            identifier_type: r#type,
            password: self.password.clone(),
        };

        let url = get_url("/api/accounts/v1/login").await;

        let res: V1Response = post(&url, body).await?;
        v1_handle(&res)?;

        Ok(())
    }
}
