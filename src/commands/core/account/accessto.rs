use std::error::Error;

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::{V1Response, V1TokenAccessTypeOptionIdentifier};
use log::*;

use crate::{
    exit_codes::loggedin_only,
    functions::{get_url, post, v1_handle},
    CREDS,
};

use super::AccessType;

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "accessto")]
/// View accounts you have access to.
pub struct AccessTo {
    #[argp(positional)]
    /// Access type to show.
    access: AccessType,
    #[argp(option, short = 'u')]
    /// User you want to view access of
    user: Option<String>,
}

#[async_trait::async_trait]
impl CommandTrait for AccessTo {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }

        trace!("Logged in, proceeding with viewing access.");
        let body = V1TokenAccessTypeOptionIdentifier {
            token: creds.token.clone(),
            access_type: self.access.into(),
            identifier_type: Some(goodmorning_bindings::services::v1::V1IdentifierType::Username),
            identifier: self.user.clone(),
        };

        let url = get_url("/api/accounts/v1/accessto").await;

        let res: V1Response = post(&url, body).await?;
        v1_handle(&res)?;

        Ok(())
    }
}
