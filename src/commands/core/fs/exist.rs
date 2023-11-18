use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::{V1PathOnly, V1Response};
use log::*;

use crate::{
    exit_codes::{loggedin_only, missing_argument},
    functions::{get, get_url, get_url_instance, post, v1_handle},
    CREDS,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "exist")]
/// Checks if file item exists.
pub struct Exist {
    #[argp(positional)]
    /// Path to file system item.
    pub path: String,
    #[argp(option, short = 'u')]
    /// ID of user's directory to check
    pub id: Option<i64>,
    #[argp(option, short = 'i')]
    /// Instance the user is on
    pub instance: Option<String>,
}

impl CommandTrait for Exist {
    fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }

        trace!("Logged in, proceeding with checking item existence.");

        let path = self.path.trim_matches('/');

        let res: V1Response = if self.id.is_some_and(|id| id != creds.id)
            || self.instance.as_ref().is_some_and(|i| i != &creds.instance)
        {
            let url = get_url_instance(
                &format!(
                    "/api/usercontent/v1/exists/id/{}/{}",
                    self.id.unwrap_or_else(|| {
                        missing_argument("id");
                        unreachable!()
                    }),
                    path
                ),
                self.instance.as_ref().unwrap(),
            );
            get(&url)
        } else {
            let url = get_url("/api/storage/v1/exists");
            let body = V1PathOnly {
                token: creds.token.clone(),
                path: path.to_string(),
            };
            post(&url, body)
        }?;

        v1_handle(&res)?;

        Ok(())
    }
}
