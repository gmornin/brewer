use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::{V1RenameAccount, V1Response};
use log::*;

use crate::{
    exit_codes::loggedin_only,
    functions::{get_url, post, v1_handle},
    CREDS,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "rename")]
/// Change your account username.
pub struct Rename {
    #[argp(positional)]
    /// Your new username.
    new: String,
}

#[async_trait::async_trait]
impl CommandTrait for Rename {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }
        trace!("Logged in, proceeding with renaming account.");

        let body = V1RenameAccount {
            token: creds.token.clone(),
            new: self.new.clone(),
        };

        let url = get_url("/api/accounts/v1/rename").await;

        let res: V1Response = post(&url, body).await?;
        v1_handle(&res)?;

        Ok(())
    }
}
