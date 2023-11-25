use std::{
    error::Error,
    path::{Path, PathBuf},
};

use goodmorning_bindings::services::v1::V1DirTreeNode;
use log::*;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
};

use crate::{
    exit_codes::{missing_repo_json, sync_failed},
    functions::{ignore_tree, DEFAULT_VIS},
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
    pub fn new(instance: String, head: FsHead) -> Self {
        let blank = V1DirTreeNode {
            visibility: DEFAULT_VIS,
            name: String::new(),
            content: goodmorning_bindings::services::v1::V1DirTreeItem::Dir {
                content: Vec::new(),
            },
        };

        Self {
            instance,
            user: head.id,
            path: head.path,

            trees: RepoTree {
                remote: blank.clone(),
                fs: blank,
            },
        }
    }

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

    pub async fn load(path: &Path) -> Self {
        trace!("Reading .gmrepo.json.");
        let path = path.join(".gmrepo.json");
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

    pub async fn find(path: &Path) -> Result<Option<PathBuf>, Box<dyn Error>> {
        trace!("Canonicalising file path.");
        let mut path = path.canonicalize()?;

        trace!("Checking gmrepo at `{}`.", path.to_string_lossy());
        if fs::try_exists(path.join(".gmrepo.json")).await? {
            trace!("Found gmrepo");
            return Ok(Some(path));
        }

        while let Some(parent) = path.parent() {
            trace!("Checking gmrepo at `{}`.", parent.to_string_lossy());
            if fs::try_exists(parent.join(".gmrepo.json")).await? {
                trace!("Found gmrepo");
                return Ok(Some(parent.to_path_buf()));
            }

            path = parent.to_path_buf()
        }

        Ok(None)
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
