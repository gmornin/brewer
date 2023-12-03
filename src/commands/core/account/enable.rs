use std::error::Error;

use argp::FromArgs;
use cmdarg_macro_derive::CmdArg;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::{V1Response, V1TokenOnly};
use log::*;
use serde::{Deserialize, Serialize};

use crate::{
    exit_codes::loggedin_only,
    functions::{get_url, post, v1_handle},
    CREDS,
};

#[derive(Serialize, Deserialize, Debug, CmdArg)]
enum GMServices {
    #[serde(rename = "tex")]
    Tex,
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "enable")]
/// Enable a GM service for you account.
pub struct Enable {
    #[argp(positional)]
    /// Service you want to enable.
    service: GMServices,
}

#[async_trait::async_trait]
impl CommandTrait for Enable {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }

        trace!("Logged in, proceeding with enabling service.");
        let body = V1TokenOnly {
            token: creds.token.clone(),
        };

        let url = get_url(match self.service {
            GMServices::Tex => "/api/generic/v1/create",
        })
        .await;

        let res: V1Response = post(&url, body).await?;
        v1_handle(&res)?;

        Ok(())
    }
}
