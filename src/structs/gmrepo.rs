use std::{fs::OpenOptions, io::Write, path::Path};

use goodmorning_bindings::services::v1::V1DirTreeNode;
use log::*;
use serde::{Deserialize, Serialize};

use crate::{exit_codes::sync_failed, functions::ignore_tree};

use super::FsHead;

#[derive(Serialize, Deserialize)]
pub struct Repo {
    pub instance: String,
    pub user: i64,
    pub path: String,

    pub trees: RepoTree,
}

impl Repo {
    pub fn generate(path: &Path, remote: V1DirTreeNode, instance: String, head: FsHead) -> Self {
        Self {
            instance,
            user: head.id,
            path: head.path,

            trees: RepoTree::generate(path, remote),
        }
    }

    pub fn save(&self, path: &Path) {
        let json = serde_json::to_string(self).unwrap();
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path.join("gmrepo.json"))
            .map_err(|e| sync_failed(e.into()))
            .unwrap();
        file.write_all(json.as_bytes())
            .map_err(|e| sync_failed(e.into()))
            .ok();
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RepoTree {
    pub remote: V1DirTreeNode,
    pub fs: V1DirTreeNode,
}

impl RepoTree {
    pub fn generate(path: &Path, remote: V1DirTreeNode) -> Self {
        trace!("Generating fs repo tree.");
        Self {
            remote,
            fs: ignore_tree(path),
        }
    }
}
