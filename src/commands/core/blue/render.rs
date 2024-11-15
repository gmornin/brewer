use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::{V1Render, V1Response};
use log::*;

use crate::{
    exit_codes::loggedin_only,
    functions::{get_url, post, v1_handle},
    CREDS,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "render")]
/// Render a Minecraft world using BlueMap.
pub struct Render {
    #[argp(positional)]
    /// Original path of the zipped map.
    pub from: String,
    #[argp(positional)]
    /// Target path of the rendered map.
    pub to: String,
    #[argp(positional)]
    /// Rendering preset to use.
    pub preset: String,
}

#[async_trait::async_trait]
impl CommandTrait for Render {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }

        trace!("Logged in, proceeding with rendering task.");

        let from = self.from.trim_matches('/');
        let to = self.to.trim_matches('/');

        let body = V1Render {
            from: from.to_string(),
            to: to.to_string(),
            preset: self.preset.clone(),
            token: creds.token.clone(),
        };

        let url = get_url("/api/blue/v1/render").await;

        let res: V1Response = post(&url, body).await?;
        v1_handle(&res)?;

        Ok(())
    }
}
