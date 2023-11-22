use std::{error::Error, path::PathBuf};

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::V1Response;
use log::*;

use crate::{
    exit_codes::loggedin_only,
    functions::{get_url, upload, v1_handle},
    CREDS,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "upload")]
/// Uploads file to remote.
pub struct Upload {
    #[argp(positional)]
    /// The local (source) path of the file.
    pub source: String,
    #[argp(positional)]
    /// The target (remote) path of new file.
    pub path: String,
    #[argp(switch, short = 'f')]
    /// Allow the overwrite file at remote (if path occupied) or not
    pub force: bool,
}

#[async_trait::async_trait]
impl CommandTrait for Upload {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }

        trace!("Logged in, proceeding with uploading file.");

        let path = self.path.trim_matches('/');

        let url = get_url(&format!(
            "/api/storage/v1/{}/{}/{}",
            if self.force {
                "upload-overwrite"
            } else {
                "upload"
            },
            creds.token,
            path
        ))
        .await;

        let res: V1Response = upload(&url, &PathBuf::from(&self.source)).await?;
        v1_handle(&res)?;

        Ok(())
    }
}
