use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::V1Response;
use log::*;

use crate::{
    exit_codes::loggedin_only,
    functions::{get, get_url, v1_handle},
    BASE_PATH, CREDS,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "ls")]
/// List directory items.
pub struct Ls {
    #[argp(positional)]
    /// The directory path to list.
    pub path: String,
}

impl CommandTrait for Ls {
    fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }

        trace!("Logged in, proceeding with listing directory items.");

        let path = self.path.trim_matches('/');
        let url = get_url(&format!(
            "/api/storage/v1/diritems/{}/{}",
            creds.token, path
        ));

        BASE_PATH
            .set(if path.is_empty() {
                "/".to_string()
            } else {
                format!("/{path}/")
            })
            .unwrap();

        let res: V1Response = get(&url)?;
        v1_handle(&res)?;

        Ok(())
    }
}
