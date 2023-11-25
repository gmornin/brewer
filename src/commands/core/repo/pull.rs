use std::{
    error::Error,
    io::{self, Write},
    path::PathBuf,
};

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::V1Response;
use log::*;
use tokio::fs;

use crate::{
    exit_codes::{
        missing_repo_json, repo_conflict, repo_not_found, sync_failed, unexpected_response,
    },
    functions::{get, get_url_instance, ignore_tree, v1_handle},
    structs::{FsHead, Repo, TreeDiff},
    BASE_PATH, CREDS, OUTPUT_DIR,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "pull")]
/// Pull updates from remote.
pub struct Pull {
    #[argp(switch, short = 'f')]
    /// Overwrite conflict files
    pub force: bool,
    #[argp(option, short = 'o', default = "PathBuf::from(\".\")")]
    /// Path to local repo
    pub output: PathBuf,
}

#[async_trait::async_trait]
impl CommandTrait for Pull {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        trace!("Checking if `{}` exists", self.output.to_string_lossy());
        if !fs::try_exists(&self.output).await? {
            repo_not_found(&self.output);
        }

        trace!("Start tracing parents for gmrepo.json");
        let output = match Repo::find(&self.output).await? {
            Some(path) => path,
            None => {
                missing_repo_json();
                unreachable!()
            }
        };

        let mut repo = Repo::load(&output).await;
        let creds = unsafe { CREDS.get().unwrap() };
        let own = repo.instance == creds.instance && repo.user == creds.id;

        let mut stdout = io::stdout();
        print!("Resolving objects");
        stdout.flush().unwrap();
        let url = get_url_instance(
            &if own {
                format!("/api/storage/v1/tree/{}/{}", creds.token, repo.path)
            } else {
                format!("/api/usercontent/v1/tree/id/{}/{}", repo.user, repo.path)
            },
            &repo.instance,
        );
        let res: V1Response = get(&url).await?;
        println!("\rResolving objects, done.");
        let remote_current = match res {
            V1Response::Tree { content } => content,
            _ => {
                v1_handle(&res).unwrap();
                unexpected_response("Tree", res);
                unreachable!()
            }
        };
        let fs_current = ignore_tree(&output).await;

        let remote_diff = TreeDiff::cmp(&repo.trees.remote, &remote_current);
        if remote_diff.is_empty() {
            println!("You are up to date.");
            return Ok(());
        }

        let fs_diff = TreeDiff::cmp(&repo.trees.fs, &fs_current);

        let conflicts = remote_diff.conflict(&fs_diff);

        if !conflicts.conflicts.is_empty() {
            println!("{}", conflicts);

            if !self.force {
                repo_conflict()
            }
        }

        println!("Pulling updates...");
        BASE_PATH.set(repo.path.to_string()).unwrap();
        OUTPUT_DIR.set(output.clone()).unwrap();

        let head = FsHead {
            path: repo.path.clone(),
            id: repo.user,
        };

        if let Err(e) = remote_diff.pull(&head, &repo.instance, own).await {
            sync_failed(e);
        }
        repo.trees.remote = remote_current;
        remote_diff.apply(&mut repo.trees.fs);
        repo.save(&output).await;

        println!("All done, you are now up to date.");
        Ok(())
    }
}
