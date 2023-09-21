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
/// Create new file.
pub struct Vis {
    #[argp(positional)]
    /// The path of new file.
    pub path: String,
    #[argp(positional)]
    /// New visibility.
    pub visibility: Visibility,
}

impl CommandTrait for Vis {
    fn run(&self) -> Result<(), Box<dyn Error>> {
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

            let url = get_url("/api/storage/v1/remove-visibility");
            post(&url, body)?
        } else {
            let body = V1PathVisibility {
                token: creds.token.clone(),
                path: path.to_string(),
                visibility: self.visibility.into(),
            };

            let url = get_url("/api/storage/v1/set-visibility");
            post(&url, body)?
        };

        v1_handle(&res)?;

        Ok(())
    }
}
