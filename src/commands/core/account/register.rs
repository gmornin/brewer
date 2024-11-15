use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::{V1All3, V1Response};
use log::*;

use crate::{
    exit_codes::loggedin_not_allowed,
    functions::{get_url, post, v1_handle},
    CREDS, INSTANCE,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "register")]
/// Register a new GM account.
pub struct Register {
    #[argp(positional)]
    /// Invite code to server.
    pub invite: Option<String>,
    #[argp(
        option,
        default = "crate::functions::prompt_sync(\"Username\")",
        short = 'u'
    )]
    /// Identify yourself.
    pub username: String,
    #[argp(
        option,
        default = "crate::functions::prompt_sync(\"Email\")",
        short = 'e'
    )]
    /// Email for verification.
    pub email: String,
    #[argp(
        option,
        default = "crate::functions::prompt_sync(\"Instance\")",
        short = 'i'
    )]
    /// Instance domain or IP.
    pub instance: String,
    #[argp(option, default = "crate::functions::read_pw_confirm()", short = 'p')]
    /// You will be prompted to enter your password securely if you skip this option.
    pub password: String,
}

#[async_trait::async_trait]
impl CommandTrait for Register {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        if unsafe { CREDS.get().unwrap().is_loggedin() } {
            loggedin_not_allowed()
        }

        trace!("Not logged in, proceeding with registration.");
        unsafe { INSTANCE.set(self.instance.clone()).unwrap() };

        let body = V1All3 {
            username: self.username.clone(),
            email: self.email.clone(),
            password: self.password.clone(),
        };
        let mut url = get_url("/api/accounts/v1/create").await;
        if let Some(invite) = &self.invite {
            url.push_str(format!("?invite={invite}").as_str());
        }

        let res: V1Response = post(&url, body).await?;
        v1_handle(&res)?;

        Ok(())
    }
}
