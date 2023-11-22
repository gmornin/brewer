use std::{
    error::Error,
    io::{self, Write},
    path::PathBuf,
};

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::V1Response;

use crate::{
    exit_codes::{permission_denied, repo_conflict, sync_failed, unexpected_response},
    functions::{get, get_url_instance, ignore_tree, v1_handle},
    structs::{FsHead, Repo, TreeDiff},
    BASE_PATH, CREDS, OUTPUT_DIR,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "push")]
/// Push updates to remote.
pub struct Push {
    #[argp(switch, short = 'f')]
    /// Overwrite conflict files
    pub force: bool,
}

#[async_trait::async_trait]
impl CommandTrait for Push {
    async fn run(&self) -> Result<(), Box<dyn Error>> {
        let mut repo = Repo::load().await;
        let creds = unsafe { CREDS.get().unwrap() };
        if repo.instance != creds.instance || repo.user != creds.id {
            println!("You must be the owner of this repository to push updates to it.");
            permission_denied()
        }

        let mut stdout = io::stdout();
        print!("Resolving objects");
        stdout.flush().unwrap();
        let url = get_url_instance(
            &format!("/api/storage/v1/tree/{}/{}", creds.token, repo.path),
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
        let fs_current = ignore_tree(&PathBuf::new()).await;

        let remote_diff = TreeDiff::cmp(&repo.trees.remote, &remote_current);
        let fs_diff = TreeDiff::cmp(&repo.trees.fs, &fs_current);

        if fs_diff.is_empty() {
            println!("Remote is up to date.");
            return Ok(());
        }

        let conflicts = remote_diff.conflict(&fs_diff);

        if !conflicts.conflicts.is_empty() {
            println!("{}", conflicts);

            if !self.force {
                repo_conflict()
            }
        }

        let output = PathBuf::new();
        println!("Pushing updates...");
        BASE_PATH.set(repo.path.to_string()).unwrap();
        OUTPUT_DIR.set(output.clone()).unwrap();

        let head = FsHead {
            path: repo.path.clone(),
            id: repo.user,
        };

        if let Err(e) = fs_diff.push(&head).await {
            sync_failed(e);
        }

        let res: V1Response = get(&url).await?;
        let remote_current = match res {
            V1Response::Tree { content } => content,
            _ => {
                return v1_handle(&res);
            }
        };
        repo.trees.remote = remote_current;
        repo.trees.fs = fs_current;
        repo.save(&output).await;

        println!("All done, updates are pushed to remote.");
        Ok(())
    }
}
