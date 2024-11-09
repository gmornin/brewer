use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::{V1Access, V1IdentifierType, V1Response};
use log::*;

use crate::{
    exit_codes::loggedin_only,
    functions::{get_url, post, v1_handle},
    CREDS,
};

use super::AccessType;

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "allow")]
/// Allow an access to another acount.
pub struct Allow {
    #[argp(positional)]
    /// Access you want to allow.
    access: AccessType,
    #[argp(positional)]
    user: String,
}

#[async_trait::async_trait]
impl CommandTrait for Allow {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }

        trace!("Logged in, proceeding with allowing access.");
        let body = V1Access {
            token: creds.token.clone(),
            identifier: self.user.clone(),
            identifier_type: V1IdentifierType::Username,
            r#type: self.access.into(),
        };

        let url = get_url("/api/accounts/v1/allow").await;

        let res: V1Response = post(&url, body).await?;
        v1_handle(&res)?;

        Ok(())
    }
}
