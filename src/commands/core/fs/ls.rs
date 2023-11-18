use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::V1Response;
use log::*;

use crate::{
    exit_codes::{loggedin_only, missing_argument},
    functions::{get, get_url, get_url_instance, v1_handle},
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
    #[argp(option, short = 'u')]
    /// ID of user's directory to list
    pub id: Option<i64>,
    #[argp(option, short = 'i')]
    /// Instance the user is on
    pub instance: Option<String>,
}

impl CommandTrait for Ls {
    fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }

        trace!("Logged in, proceeding with listing directory items.");

        let path = self.path.trim_matches('/');

        let url = if self.id.is_some_and(|id| id != creds.id)
            || self.instance.as_ref().is_some_and(|i| i != &creds.instance)
        {
            get_url_instance(
                &format!(
                    "/api/usercontent/v1/diritems/id/{}/{}",
                    self.id.unwrap_or_else(|| {
                        missing_argument("id");
                        unreachable!()
                    }),
                    path
                ),
                self.instance.as_ref().unwrap(),
            )
        } else {
            get_url(&format!(
                "/api/storage/v1/diritems/{}/{}",
                creds.token, path
            ))
        };

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
