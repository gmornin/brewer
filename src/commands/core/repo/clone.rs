use std::{
    error::Error,
    fs,
    io::{self, Write},
    path::PathBuf,
};

use argp::FromArgs;
use command_macro::CommandTrait;
use goodmorning_bindings::services::v1::{V1DirTreeNode, V1Response};
use log::*;

use crate::{
    exit_codes::{bad_clone_json, output_path_occupied, sync_failed, unexpected_response},
    functions::{get, get_string, get_url_instance, url_domain, v1_handle, DEFAULT_VIS},
    structs::{FsHead, GmIgnoreDefault, Repo, TreeDiff},
    BASE_PATH, CREDS, OUTPUT_DIR,
};

#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(FromArgs)]
#[argp(subcommand, name = "clone")]
/// Clones a remote directory.
pub struct Clone {
    #[argp(positional)]
    /// Url of remote.
    pub url: String,
    #[argp(option, short = 'o', default = "String::new()")]
    /// Target directory
    pub output: String,
}

impl CommandTrait for Clone {
    fn run(&self) -> Result<(), Box<dyn Error>> {
        let mut stdout = io::stdout();
        print!("Resolving objects");
        stdout.flush().unwrap();

        let creds = unsafe { CREDS.get().unwrap() };
        let dom = url_domain(&self.url);
        let same_dom = dom == creds.instance;

        let (res, code) = get_string(&self.url, true, same_dom)?;

        if !code.is_success() {
            trace!("Status code is not success, aborting.");
            bad_clone_json()
        }

        let line = match res.lines().next() {
            Some(l) => l.trim(),
            None => {
                trace!("Response empty, first line not possible.");
                bad_clone_json();
                unreachable!()
            }
        };

        if !(line.starts_with("<!--") && line.ends_with("-->")) {
            trace!("Expected first line is comment, but it is not.");
            bad_clone_json()
        }

        let line = line[4..line.len() - 3].to_string();
        let head: FsHead = line.as_str().into();
        let own = head.id == creds.id && same_dom;

        let url = get_url_instance(
            &if own {
                format!(
                    "/api/storage/v1/tree/{}/{}",
                    creds.token,
                    head.path.trim_matches('/')
                )
            } else {
                format!(
                    "/api/usercontent/v1/tree/id/{}/{}",
                    head.id,
                    head.path.trim_matches('/')
                )
            },
            dom,
        );
        let res: V1Response = get(&url)?;

        let tree = match res {
            V1Response::Tree { content } => content,
            res => {
                v1_handle(&res).unwrap();
                unexpected_response("Tree", res);
                unreachable!()
            }
        };

        let blank = V1DirTreeNode {
            visibility: DEFAULT_VIS,
            name: String::new(),
            content: goodmorning_bindings::services::v1::V1DirTreeItem::Dir {
                content: Vec::new(),
            },
        };

        let diff = TreeDiff::cmp(&blank, &tree);
        let name = if self.output.is_empty() {
            tree.name.trim_matches('/')
        } else {
            &self.output
        };
        let output = PathBuf::from(name);

        if output.exists() {
            output_path_occupied(&output);
        } else {
            fs::create_dir_all(&output)?;
        }

        println!("\rResolving objects, done.");
        println!("Cloning into '{}'...", output.to_string_lossy());
        BASE_PATH.set(head.path.to_string()).unwrap();
        OUTPUT_DIR.set(output.clone()).unwrap();

        if let Err(e) = diff.pull(&head, url.split('/').next().unwrap(), own) {
            sync_failed(e);
        }

        if !output.join(".gmignore").exists() {
            GmIgnoreDefault::create(&output);
            println!("Created .gmignore file.")
        }

        trace!("Creating gmrepo.json");
        let repo = Repo::generate(&output, tree, dom.to_string(), head);
        repo.save(&output);

        println!("All done, you are now up to date.");
        Ok(())
    }
}
