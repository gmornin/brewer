use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::{V1Response, V1TokenOnly};
use log::*;

use crate::{
    exit_codes::loggedin_only,
    functions::{get_url, post, v1_handle},
    CREDS,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "invite")]
/// Create an invite code.
pub struct Invite {}

#[async_trait::async_trait]
impl CommandTrait for Invite {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }

        trace!("Logged in, proceeding with creating invite.");
        let body = V1TokenOnly {
            token: creds.token.clone(),
        };

        let url = get_url("/api/accounts/v1/invite").await;

        let res: V1Response = post(&url, body).await?;
        v1_handle(&res)?;

        Ok(())
    }
}
