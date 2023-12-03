use std::{
    error::Error,
    io::{self, Write},
    path::PathBuf,
};

use argp::FromArgs;
use command_macro::CommandTrait;
use log::*;
use tokio::fs;

use crate::{
    exit_codes::{bad_head_json, file_not_found},
    functions::{get_string, url_domain},
    structs::{FsHead, GmIgnoreDefault, Repo},
    CREDS,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "bind")]
/// Bind local directory to a remote directory.
pub struct Bind {
    #[argp(positional)]
    /// Url of remote.
    pub url: String,
    #[argp(option, short = 'o', default = "PathBuf::from(\".\")")]
    /// Path to local directory
    pub output: PathBuf,
}

#[async_trait::async_trait]
impl CommandTrait for Bind {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let mut stdout = io::stdout();
        println!("Binding to remote");
        stdout.flush().unwrap();

        trace!("Checking if `{}` exists", self.output.to_string_lossy());
        if !fs::try_exists(&self.output).await? {
            file_not_found(&self.output);
        }

        let creds = unsafe { CREDS.get().unwrap() };
        let dom = url_domain(&self.url).to_string();
        let same_dom = dom == creds.instance;

        let (res, code) = get_string(&self.url, true, same_dom).await?;

        if !code.is_success() {
            trace!("Status code is not success, aborting.");
            bad_head_json()
        }

        let line = match res.lines().next() {
            Some(l) => l.trim(),
            None => {
                trace!("Response empty, first line not possible.");
                bad_head_json();
                unreachable!()
            }
        };

        if !(line.starts_with("<!--") && line.ends_with("-->")) {
            trace!("Expected first line is comment, but it is not.");
            bad_head_json()
        }

        let line = line[4..line.len() - 3].to_string();
        let head: FsHead = line.as_str().into();

        trace!("Creating gmrepo.json");
        let repo = Repo::new(dom.to_string(), head);
        repo.save(&self.output).await;

        if !fs::try_exists(self.output.join(".gmignore")).await? {
            GmIgnoreDefault::create(&self.output);
            println!("Created .gmignore file.")
        }

        println!("All done.");
        Ok(())
    }
}
