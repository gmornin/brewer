use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::{V1Response, V1SelfFromTo};
use log::*;

use crate::{
    exit_codes::loggedin_only,
    functions::{get_url, post, v1_handle},
    CREDS,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "mv")]
/// Move/rename a file system item.
pub struct Mv {
    #[argp(positional)]
    /// Original path of the file item.
    pub from: String,
    #[argp(positional)]
    /// Target path of the file item.
    pub to: String,
    #[argp(switch, short = 'f')]
    /// Allow overwriting target file.
    pub force: bool,
}

#[async_trait::async_trait]
impl CommandTrait for Mv {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }

        trace!("Logged in, proceeding with regenerating token.");

        let from = self.from.trim_matches('/');
        let to = self.to.trim_matches('/');
        let body = V1SelfFromTo {
            token: creds.token.clone(),
            from: from.to_string(),
            to: to.to_string(),
        };

        let url = get_url(if self.force {
            "/api/storage/v1/move-overwrite"
        } else {
            "/api/storage/v1/move"
        })
        .await;

        let res: V1Response = post(&url, body).await?;
        v1_handle(&res)?;

        Ok(())
    }
}
