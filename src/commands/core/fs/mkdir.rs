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
#[argp(subcommand, name = "mkdir")]
/// Create new directory.
pub struct Mkdir {
    #[argp(positional)]
    /// The path of new directory.
    pub path: String,
}

#[async_trait::async_trait]
impl CommandTrait for Mkdir {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }

        trace!("Logged in, proceeding with creating directory.");

        let path = self.path.trim_matches('/');
        let body = V1PathOnly {
            token: creds.token.clone(),
            path: path.to_string(),
        };

        let url = get_url("/api/storage/v1/mkdir").await;

        let res: V1Response = post(&url, body).await?;
        v1_handle(&res)?;

        Ok(())
    }
}
