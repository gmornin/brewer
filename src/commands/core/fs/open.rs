use std::{error::Error, fs, path::PathBuf, time::UNIX_EPOCH};

use argp::FromArgs;
use chrono::Utc;
use command_macro::CommandTrait;

use log::*;

use crate::{
    exit_codes::missing_argument,
    functions::{download, get_url, get_url_instance},
    CREDS, MAX_AGE,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "open")]
/// View a remote file item.
pub struct Open {
    #[argp(positional)]
    /// Path to remote file system item.
    pub path: String,
    #[argp(option, short = 'u')]
    /// ID of user's directory to check
    pub id: Option<i64>,
    #[argp(option, short = 'i')]
    /// Instance the user is on
    pub instance: Option<String>,
    /// Refetch item even if it is still fresh
    #[argp(switch, short = 'f')]
    pub fetch: bool,
}

#[async_trait::async_trait]
impl CommandTrait for Open {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let creds = unsafe { CREDS.get_mut().unwrap() };
        if !creds.is_loggedin() {
            trace!("Not logged in, proceeding with opening remote file.");
            if self.instance.is_none() {
                missing_argument("instance")
            }
        } else {
            trace!("Logged in, proceeding with opening remote file.");
        }

        let path = self.path.trim_matches('/');

        let (url, user_path) = if !creds.is_loggedin()
            || self.id.is_some_and(|id| id != creds.id)
            || self.instance.as_ref().is_some_and(|i| i != &creds.instance)
        {
            let instance = self.instance.as_ref().unwrap_or(&creds.instance);
            let id = self.id.unwrap_or_else(|| {
                missing_argument("id");
                unreachable!()
            });
            (
                get_url_instance(
                    &format!("/api/usercontent/v1/file/id/{id}/{path}",),
                    self.instance.as_ref().unwrap_or(&creds.instance),
                ),
                PathBuf::from(instance).join(id.to_string()),
            )
        } else {
            (
                get_url(&format!("/api/storage/v1/file/{}/{}", creds.token, path)).await,
                PathBuf::from(&creds.instance).join(creds.id.to_string()),
            )
        };

        let output = dirs::cache_dir()
            .unwrap()
            .join(env!("CARGO_PKG_NAME"))
            .join("fs")
            .join(user_path)
            .join(path);

        let parent = output.parent().unwrap();
        if !parent.exists() {
            trace!(
                "Creating parent folder {} because it doesn't exist.",
                parent.to_string_lossy()
            );
            fs::create_dir_all(parent)?;
        }

        if self.fetch
            || !output.exists()
            || Utc::now().timestamp() as u64
                - output
                    .metadata()?
                    .modified()?
                    .duration_since(UNIX_EPOCH)?
                    .as_secs()
                > *MAX_AGE.get().unwrap()
        {
            trace!("Fetching file.");
            download(&url, &output).await?;
        } else {
            trace!("Not fetching file as it is still fresh.")
        }

        if let Err(e) = open::that_detached(&output) {
            error!(
                "Failed to open file `{}` with error {e}.",
                output.to_string_lossy()
            );
            return Err(e.into());
        }

        println!(
            "File has been opened with {}.",
            open::commands(output)[0].get_program().to_string_lossy()
        );

        Ok(())
    }
}
