use std::path::{Path, PathBuf};

use goodmorning_bindings::services::v1::V1DirTreeNode;
use log::*;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
};

use crate::{
    exit_codes::{missing_repo_json, sync_failed},
    functions::ignore_tree,
};

use super::FsHead;

#[derive(Serialize, Deserialize)]
pub struct Repo {
    pub instance: String,
    pub user: i64,
    pub path: String,

    pub trees: RepoTree,
}

impl Repo {
    pub async fn generate(
        path: &Path,
        remote: V1DirTreeNode,
        instance: String,
        head: FsHead,
    ) -> Self {
        Self {
            instance,
            user: head.id,
            path: head.path,

            trees: RepoTree::generate(path, remote).await,
        }
    }

    pub async fn save(&self, path: &Path) {
        let json = serde_json::to_string(self).unwrap();
        trace!("Saving .gmrepo.json.");
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path.join(".gmrepo.json"))
            .await
            .map_err(|e| sync_failed(e.into()))
            .unwrap();
        file.write_all(json.as_bytes())
            .await
            .map_err(|e| sync_failed(e.into()))
            .ok();
    }

    pub async fn load() -> Self {
        trace!("Reading .gmrepo.json.");
        let path = PathBuf::from(".gmrepo.json");
        if !path.exists() {
            missing_repo_json()
        }
        let s = fs::read_to_string(path)
            .await
            .map_err(|e| sync_failed(e.into()))
            .unwrap();
        trace!("Deserializing .gmrepo.json.");
        serde_json::from_str(&s)
            .map_err(|e| sync_failed(e.into()))
            .unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RepoTree {
    pub remote: V1DirTreeNode,
    pub fs: V1DirTreeNode,
}

impl RepoTree {
    pub async fn generate(path: &Path, remote: V1DirTreeNode) -> Self {
        trace!("Generating fs repo tree.");
        Self {
            remote,
            fs: ignore_tree(path).await,
        }
    }
}
