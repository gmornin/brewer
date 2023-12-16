use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::{V1PathOnly, V1PathVisibility, V1Response};
use log::*;

use crate::{
    exit_codes::loggedin_only,
    functions::{get_url, post, v1_handle},
    structs::Visibility,
    CREDS,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "vis")]
/// Change file item visibility.
pub struct Vis {
    #[argp(positional)]
    /// The path of file item.
    pub path: String,
    #[argp(positional)]
    /// New visibility.
    pub visibility: Visibility,
}

#[async_trait::async_trait]
impl CommandTrait for Vis {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }

        trace!("Logged in, proceeding with changing visibility.");

        let path = self.path.trim_matches('/');

        let res: V1Response = if self.visibility == Visibility::Inherit {
            let body = V1PathOnly {
                token: creds.token.clone(),
                path: path.to_string(),
            };

            let url = get_url("/api/storage/v1/remove-visibility").await;
            post(&url, body).await?
        } else {
            let body = V1PathVisibility {
                token: creds.token.clone(),
                path: path.to_string(),
                visibility: self.visibility.into(),
            };

            let url = get_url("/api/storage/v1/set-visibility").await;
            post(&url, body).await?
        };

        v1_handle(&res)?;

        Ok(())
    }
}
