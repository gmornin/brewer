use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::{V1FromTo, V1Response};
use log::*;

use crate::{
    exit_codes::loggedin_only,
    functions::{get_url, post, v1_handle},
    CREDS,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "cp")]
/// Copy a file system item.
pub struct Cp {
    #[argp(positional)]
    /// Original path of the file item.
    pub from: String,
    #[argp(positional)]
    /// Target path of the file item.
    pub to: String,
    #[argp(option, short = 'u')]
    /// User to copy from.
    pub user: Option<i64>,
    #[argp(switch, short = 'f')]
    /// Allow overwriting target file.
    pub force: bool,
}

#[async_trait::async_trait]
impl CommandTrait for Cp {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }

        trace!("Logged in, proceeding with copying file.");

        let from = self.from.trim_matches('/');
        let to = self.to.trim_matches('/');
        let body = V1FromTo {
            token: creds.token.clone(),
            from: from.to_string(),
            to: to.to_string(),
            from_userid: self.user.unwrap_or(creds.id),
        };

        let url = get_url(if self.force {
            "/api/storage/v1/copy-overwrite"
        } else {
            "/api/storage/v1/copy"
        })
        .await;

        let res: V1Response = post(&url, body).await?;
        v1_handle(&res)?;

        Ok(())
    }
}
