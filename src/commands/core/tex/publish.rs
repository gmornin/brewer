use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::{V1Publish, V1Response, V1UpdatePublish};
use log::*;

use crate::{
    exit_codes::{loggedin_only, missing_argument},
    functions::{get_url, post, v1_handle},
    CREDS,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "publish")]
/// Publish tex file.
pub struct Publish {
    #[argp(positional)]
    /// Remote path of file (omit `/tex/`).
    pub path: String,
    #[argp(option, short = 't')]
    /// Title of article
    pub title: Option<String>,
    #[argp(option, short = 'd')]
    /// Description of article
    pub description: Option<String>,
    #[argp(option, short = 'u')]
    /// Update an exsisting article
    pub update: Option<u64>,
}

#[async_trait::async_trait]
impl CommandTrait for Publish {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }

        trace!("Logged in, proceeding with publishing article.");

        let path = self.path.trim_matches('/');

        let res: V1Response = match self.update {
            Some(id) => {
                let body = V1UpdatePublish {
                    token: creds.token.clone(),
                    id: id as i64,
                    path: path.to_string(),
                };
                let url = get_url("/api/publish/v1/update-publish").await;

                post(&url, body).await?
            }
            None => {
                let title = self
                    .title
                    .as_ref()
                    .unwrap_or_else(|| {
                        missing_argument("title");
                        unreachable!()
                    })
                    .to_string();
                let description = self
                    .description
                    .as_ref()
                    .unwrap_or_else(|| {
                        missing_argument("description");
                        unreachable!()
                    })
                    .to_string();

                let body = V1Publish {
                    token: creds.token.clone(),
                    path: path.to_string(),
                    title,
                    desc: description,
                };
                let url = get_url("/api/publish/v1/publish").await;

                post(&url, body).await?
            }
        };

        v1_handle(&res)?;

        Ok(())
    }
}
