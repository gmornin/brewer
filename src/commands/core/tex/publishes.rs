use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::V1Response;
use log::*;

use crate::{
    exit_codes::missing_argument,
    functions::{get, get_url_instance, v1_handle},
    CREDS, INSTANCE, USER_ID,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "publishes")]
/// List published items.
pub struct Publishes {
    #[argp(option, short = 'u')]
    /// ID of user's publishes to list
    pub id: Option<i64>,
    #[argp(option, short = 'i')]
    /// Instance the user is on
    pub instance: Option<String>,
    #[argp(option, short = 'p', default = "1")]
    /// Page to view
    pub page: u64,
}

#[async_trait::async_trait]
impl CommandTrait for Publishes {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            trace!("Not logged in, proceeding with listing published items.");
            if self.instance.is_none() {
                missing_argument("instance")
            }
            if self.id.is_none() {
                missing_argument("id")
            }
        } else {
            trace!("Logged in, proceeding with listing published items.");
        }

        let instance = self.instance.as_ref().unwrap_or(&creds.instance);
        unsafe { *INSTANCE.get_mut().unwrap() = instance.clone() };
        let id = self.id.unwrap_or(creds.id);
        unsafe { USER_ID.set(id).unwrap() };

        let url = get_url_instance(&format!("/api/publish/v1/publishes/id/{id}"), instance);

        let res: V1Response = get(&url).await?;
        v1_handle(&res)?;

        Ok(())
    }
}
