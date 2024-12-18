use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::V1Response;
use log::trace;

use crate::{
    exit_codes::missing_argument,
    functions::{get, get_url, get_url_instance, v1_handle},
    BASE_PATH, CREDS, FULLPATH, FULL_PATH,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "tree")]
/// Recursively tree directory items.
pub struct Tree {
    #[argp(positional)]
    /// The directory path to tree.
    pub path: String,
    #[argp(option, short = 'u')]
    /// ID of user's directory to tree
    pub id: Option<i64>,
    #[argp(option, short = 'i')]
    /// Instance the user is on
    pub instance: Option<String>,
    #[argp(switch, short = 'f')]
    /// Display full item path
    pub full: bool,
}

#[async_trait::async_trait]
impl CommandTrait for Tree {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        unsafe { *FULLPATH.get_mut().unwrap() = self.full };

        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            trace!("Not logged in, proceeding with treeing directory items.");
            if self.instance.is_none() {
                missing_argument("instance")
            }
        } else {
            trace!("Logged in, proceeding with treeing directory items.");
        }

        let path = self.path.trim_matches('/');

        let url = if !creds.is_loggedin()
            || self.id.is_some_and(|id| id != creds.id)
            || self.instance.as_ref().is_some_and(|i| i != &creds.instance)
        {
            get_url_instance(
                &format!(
                    "/api/usercontent/v1/tree/id/{}/{}",
                    self.id.unwrap_or_else(|| {
                        missing_argument("id");
                        unreachable!()
                    }),
                    path
                ),
                self.instance.as_ref().unwrap_or(&creds.instance),
            )
        } else {
            get_url(&format!("/api/storage/v1/tree/{}/{}", creds.token, path)).await
        };

        if self.full {
            FULL_PATH.set(true).unwrap();
            BASE_PATH.set(self.path.clone()).unwrap();
        } else {
            FULL_PATH.set(false).unwrap();
            BASE_PATH.set(String::new()).unwrap();
        }

        let res: V1Response = get(&url).await?;

        v1_handle(&res)?;

        Ok(())
    }
}
