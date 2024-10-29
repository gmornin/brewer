use std::error::Error;

use argp::FromArgs;
use cmdarg_macro_derive::CmdArg;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::{
    AccessType as BindingAccess, V1Response, V1TokenAccessType,
};
use log::*;
use serde::{Deserialize, Serialize};

use crate::{
    exit_codes::loggedin_only,
    functions::{get_url, post, v1_handle},
    CREDS,
};

#[derive(Serialize, Deserialize, Debug, CmdArg, Clone, Copy)]
enum AccessType {
    #[serde(rename = "file")]
    File,
}

impl From<AccessType> for BindingAccess {
    fn from(val: AccessType) -> Self {
        match val {
            AccessType::File => BindingAccess::File,
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "access")]
/// View accounts with the specified access.
pub struct Access {
    #[argp(positional)]
    /// Access type to show.
    access: AccessType,
}

#[async_trait::async_trait]
impl CommandTrait for Access {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            loggedin_only()
        }

        trace!("Logged in, proceeding with viewing access.");
        let body = V1TokenAccessType {
            token: creds.token.clone(),
            access_type: self.access.into(),
        };

        let url = get_url("/api/accounts/v1/access").await;

        let res: V1Response = post(&url, body).await?;
        v1_handle(&res)?;

        Ok(())
    }
}
