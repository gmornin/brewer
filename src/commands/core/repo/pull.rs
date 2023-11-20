use std::{
    error::Error,
    io::{self, Write},
    path::PathBuf,
};

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::V1Response;

use crate::{
    exit_codes::{repo_conflict, sync_failed, unexpected_response},
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
}

impl CommandTrait for Pull {
    fn run(&self) -> Result<(), Box<dyn Error>> {
        let mut repo = Repo::load();
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
        let res: V1Response = get(&url)?;
        println!("\rResolving objects, done.");
        let remote_current = match res {
            V1Response::Tree { content } => content,
            _ => {
                v1_handle(&res).unwrap();
                unexpected_response("Tree", res);
                unreachable!()
            }
        };
        let fs_current = ignore_tree(&PathBuf::new());

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

        let output = PathBuf::new();
        println!("Pulling updates...");
        BASE_PATH.set(repo.path.to_string()).unwrap();
        OUTPUT_DIR.set(output.clone()).unwrap();

        let head = FsHead {
            path: repo.path.clone(),
            id: repo.user,
        };

        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                if let Err(e) = remote_diff.pull(&head, &repo.instance, own).await {
                    sync_failed(e);
                }
                repo.trees.remote = remote_current;
                repo.save(&output);
            });

        println!("All done, you are now up to date.");
        Ok(())
    }
}
