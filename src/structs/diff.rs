use std::{
    error::Error,
    fmt::Display,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU32, AtomicU64},
        Arc,
    },
};

use goodmorning_bindings::services::v1::{
    V1DirTreeItem, V1DirTreeNode, V1Error, V1PathOnly, V1Response,
};
use log::*;

use serde::{Deserialize, Serialize};

use crate::{
    exit_codes::{
        bad_clone_json, download_failed, fs_error, sync_failed, unexpected_response, FsAction,
        FsActionType,
    },
    functions::{download, get_url, post, upload, v1_handle, DEFAULT_VIS},
    CREDS, OUTPUT_DIR,
};

const DIR_SIZE: u64 = 0;

#[derive(Debug, PartialEq, Eq, Default)]
pub struct TreeDiff {
    pub created: Vec<TreeDiffItem>,
    pub created_dirs: Vec<TreeDiffItem>,
    pub changed: Vec<TreeDiffItem>,
    pub deleted: Vec<TreeDiffItem>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TreeDiffItem {
    pub size: u64,
    pub path: String,
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for TreeDiffItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(match self.path.len().partial_cmp(&other.path.len()) {
            Some(cmp) => cmp,
            None => self.path.cmp(&other.path),
        })
    }
}

impl Ord for TreeDiffItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DiffConflicts {
    pub conflicts: Vec<DiffConflictItem>,
}

impl Display for DiffConflicts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.conflicts.is_empty() {
            return f.write_str("");
        }

        let longest = self
            .conflicts
            .iter()
            .map(|item| item.path.len())
            .max()
            .unwrap();
        f.write_fmt(format_args!(
            "Conflicts with remote branch.\n   {:<longest$}  Local        Remote\n{}",
            "Path",
            self.conflicts
                .iter()
                .map(|item| format!(
                    " - {:<longest$}  {:<9?}       {:<9?}",
                    item.path, item.fs, item.remote
                ))
                .collect::<Vec<_>>()
                .join("\n")
        ))
    }
}

impl From<Vec<DiffConflictItem>> for DiffConflicts {
    fn from(value: Vec<DiffConflictItem>) -> Self {
        Self { conflicts: value }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DiffConflictItem {
    pub path: String,
    pub fs: DiffConflictAction,
    pub remote: DiffConflictAction,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DiffConflictAction {
    Create,
    CreateDir,
    Change,
    Delete,
}

impl TreeDiffItem {
    fn from(path: PathBuf, size: u64) -> Self {
        Self {
            size,
            path: path.to_string_lossy().to_string(),
        }
    }
}

// fn sort(vec: &mut [TreeDiffItem]) {
//     vec.sort_by_key(|item| item.path.to_string());
// }

impl TreeDiff {
    pub fn is_empty(&self) -> bool {
        self.created.is_empty()
            && self.created_dirs.is_empty()
            && self.changed.is_empty()
            && self.deleted.is_empty()
    }

    pub fn cmp(old: &V1DirTreeNode, new: &V1DirTreeNode) -> Self {
        let (created, created_dirs, changed) =
            cmp_updated_recurse(old, new, PathBuf::new().as_path());
        let deleted = cmp_del_recurse(old, new, PathBuf::new().as_path());

        let mut out = Self {
            created,
            created_dirs,
            changed,
            deleted,
        };
        out.sort();
        out
    }

    pub fn sort(&mut self) {
        self.created.sort();
        self.created_dirs.sort();
        self.changed.sort();
        self.deleted.sort();
    }

    pub fn conflict(&self, remote: &TreeDiff) -> DiffConflicts {
        let mut out = Vec::new();
        for created in self.created.iter() {
            if let Some(other) = remote.path_modified(&created.path) {
                out.push(DiffConflictItem {
                    path: created.path.to_string(),
                    fs: DiffConflictAction::Create,
                    remote: other,
                })
            }
        }
        for created_dir in self.created_dirs.iter() {
            if let Some(other) = remote.path_modified(&created_dir.path) {
                out.push(DiffConflictItem {
                    path: created_dir.path.to_string(),
                    fs: DiffConflictAction::CreateDir,
                    remote: other,
                })
            }
        }
        for changed in self.changed.iter() {
            if let Some(other) = remote.path_modified(&changed.path) {
                out.push(DiffConflictItem {
                    path: changed.path.to_string(),
                    fs: DiffConflictAction::Change,
                    remote: other,
                })
            }
        }
        for deleted in self.deleted.iter() {
            if let Some(other) = remote.folder_modified(&deleted.path) {
                out.push(DiffConflictItem {
                    path: deleted.path.to_string(),
                    fs: DiffConflictAction::Delete,
                    remote: other,
                })
            }
        }

        DiffConflicts { conflicts: out }
    }

    pub fn path_modified(&self, path: &str) -> Option<DiffConflictAction> {
        if self.deleted.iter().any(|deleted| {
            PathBuf::from(path)
                .strip_prefix(PathBuf::from(&deleted.path))
                .is_ok()
        }) {
            return Some(DiffConflictAction::Delete);
        }
        if self
            .created
            .binary_search_by_key(&path, |item| &item.path)
            .is_ok()
        {
            return Some(DiffConflictAction::Create);
        }
        if self
            .created_dirs
            .binary_search_by_key(&path, |item| &item.path)
            .is_ok()
        {
            return Some(DiffConflictAction::CreateDir);
        }
        if self
            .changed
            .binary_search_by_key(&path, |item| &item.path)
            .is_ok()
        {
            return Some(DiffConflictAction::Change);
        }

        None
    }

    pub fn folder_modified(&self, path: &str) -> Option<DiffConflictAction> {
        if self.created.iter().any(|created| {
            PathBuf::from(&created.path)
                .strip_prefix(PathBuf::from(&path))
                .is_ok()
        }) {
            return Some(DiffConflictAction::Create);
        }
        if self.created_dirs.iter().any(|created_dir| {
            PathBuf::from(&created_dir.path)
                .strip_prefix(PathBuf::from(&path))
                .is_ok()
        }) {
            return Some(DiffConflictAction::CreateDir);
        }
        if self.changed.iter().any(|changed| {
            PathBuf::from(&changed.path)
                .strip_prefix(PathBuf::from(&path))
                .is_ok()
        }) {
            return Some(DiffConflictAction::Change);
        }

        None
    }

    pub async fn pull(
        &self,
        head: &FsHead,
        instance: &str,
        owned: bool,
    ) -> Result<(), Box<dyn Error>> {
        let _stdout = io::stdout();
        let output = OUTPUT_DIR.get().unwrap();

        if !self.deleted.is_empty() {
            let counting = Arc::new(AtomicU32::new(0));
            let total = Arc::new(AtomicU32::new(self.deleted.len() as u32));

            async fn delete(
                path: PathBuf,
                display_path: &str,
                counting: Arc<AtomicU32>,
                total: Arc<AtomicU32>,
            ) {
                async fn task(
                    path: &Path,
                    display_path: &str,
                    counting: Arc<AtomicU32>,
                    total: Arc<AtomicU32>,
                ) -> Result<(), Box<dyn Error>> {
                    if !tokio::fs::try_exists(&path).await? {
                        trace!("{display_path} does not exist, skipping delete item.");
                        return Ok(());
                    }

                    let counting = counting.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let total = total.load(std::sync::atomic::Ordering::Relaxed);
                    print!(
                        "\rDeleting objects ({counting}/{total})... {}%",
                        counting * 100 / total
                    );
                    io::stdout().flush().unwrap();
                    if tokio::fs::metadata(&path).await?.is_dir() {
                        trace!("Deleting directory {display_path}.");
                        tokio::fs::remove_dir_all(&path).await?;
                    } else {
                        trace!("Deleting file {display_path}.");
                        tokio::fs::remove_file(&path).await?;
                    }

                    Ok(())
                }

                if let Err(e) = task(&path, display_path, counting.clone(), total.clone()).await {
                    fs_error(
                        &e.to_string(),
                        &FsAction::new(path, FsActionType::DeleteItem),
                    )
                }
            }

            let mut tasks = Vec::with_capacity(self.deleted.len());
            for item in self.deleted.iter() {
                let display_path = item.path.trim_matches('/');
                let path = output.join(display_path);

                tasks.push(delete(path, display_path, counting.clone(), total.clone()));
            }

            for task in tasks {
                task.await
            }

            let counting = counting.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let total = total.load(std::sync::atomic::Ordering::Relaxed);
            println!("\rDeleting objects ({counting}/{total}), done.",);
        }

        if !self.created_dirs.is_empty() {
            let counting = Arc::new(AtomicU32::new(0));
            let total = Arc::new(AtomicU32::new(self.created_dirs.len() as u32));

            async fn create_dir(
                path: PathBuf,
                display_path: &str,
                counting: Arc<AtomicU32>,
                total: Arc<AtomicU32>,
            ) {
                async fn task(
                    path: &Path,
                    display_path: &str,
                    counting: Arc<AtomicU32>,
                    total: Arc<AtomicU32>,
                ) -> Result<(), Box<dyn Error>> {
                    if tokio::fs::try_exists(&path).await? {
                        trace!("{display_path} already exist, skipping creating directory.");
                        return Ok(());
                    }

                    let counting = counting.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let total = total.load(std::sync::atomic::Ordering::Relaxed);
                    print!(
                        "\rCreating directories ({counting}/{total})... {}%",
                        counting * 100 / total
                    );
                    io::stdout().flush().unwrap();
                    trace!("Creating directory {display_path}.");
                    tokio::fs::create_dir_all(path).await?;

                    Ok(())
                }

                if let Err(e) = task(&path, display_path, counting.clone(), total.clone()).await {
                    fs_error(
                        &e.to_string(),
                        &FsAction::new(path, FsActionType::DeleteItem),
                    )
                }
            }

            let mut tasks = Vec::with_capacity(self.deleted.len());
            for item in self.created_dirs.iter() {
                let display_path = item.path.trim_matches('/');
                let path = output.join(display_path);
                tasks.push(create_dir(
                    path,
                    display_path,
                    counting.clone(),
                    total.clone(),
                ));
            }

            for task in tasks {
                task.await
            }

            let counting = counting.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let total = total.load(std::sync::atomic::Ordering::Relaxed);
            println!("\rCreating directories ({counting}/{total}), done.",);
        }

        if !(self.created.is_empty() && self.changed.is_empty()) {
            let creds = unsafe { CREDS.get().unwrap() };
            let counting = Arc::new(AtomicU64::new(0));
            let total = total(&self.created) + total(&self.changed);

            print!("\rDownloading objects (0/{total})... 0%");

            async fn download_one(
                path: PathBuf,
                display_path: String,
                url: String,
                size: u64,
                counting: Arc<AtomicU64>,
                total: u64,
            ) {
                async fn task(
                    path: &Path,
                    display_path: String,
                    url: String,
                    size: u64,
                    counting: Arc<AtomicU64>,
                    total: u64,
                ) -> Result<(), Box<dyn Error>> {
                    io::stdout().flush().unwrap();
                    trace!("Downloading item {display_path}.");
                    if download(&url, path).await.is_err() {
                        println!();
                        download_failed(&display_path);
                    }
                    let counting =
                        counting.fetch_add(size, std::sync::atomic::Ordering::Relaxed) + size;
                    print!(
                        "\rDownloading objects ({counting}/{total})... {}%",
                        counting * 100 / total
                    );

                    Ok(())
                }

                if let Err(e) = task(&path, display_path, url, size, counting.clone(), total).await
                {
                    fs_error(
                        &e.to_string(),
                        &FsAction::new(path, FsActionType::WriteFile),
                    )
                }
            }

            let mut tasks = Vec::with_capacity(self.changed.len() + self.created.len());
            for item in self.changed.iter().chain(self.created.iter()) {
                let remote_path = format!(
                    "{}/{}",
                    head.path.trim_matches('/'),
                    item.path.trim_matches('/')
                );
                let url = if owned {
                    format!(
                        "{instance}/api/storage/v1/file/{}/{remote_path}",
                        creds.token
                    )
                } else {
                    format!(
                        "{instance}/api/usercontent/v1/file/id/{}/{remote_path}",
                        head.id
                    )
                };

                let display_path = item.path.trim_matches('/').to_string();
                let path = output.join(&display_path);
                let size = item.size;
                let counting = counting.clone();
                tasks.push(tokio::task::spawn(download_one(
                    path,
                    display_path,
                    url,
                    size,
                    counting,
                    total,
                )))
            }

            for task in tasks {
                task.await?;
            }

            let counting = counting.load(std::sync::atomic::Ordering::Relaxed);
            println!("\rDownloading objects ({counting}/{total}), done. ");
        }

        Ok(())
    }

    pub fn push(&self, head: &FsHead) -> Result<(), Box<dyn Error>> {
        let mut stdout = io::stdout();
        let creds = unsafe { CREDS.get().unwrap() };

        if !self.deleted.is_empty() {
            let total = self.deleted.len();
            let url = get_url("/api/storage/v1/delete");
            for (i, item) in self.deleted.iter().enumerate() {
                trace!("Deleting {}.", item.path);
                print!("\rDeleting objects ({}/{total})...", i);
                stdout.flush().unwrap();

                let body = V1PathOnly {
                    token: creds.token.clone(),
                    path: format!("{}/{}", head.path, item.path.clone()),
                };

                let res: V1Response = match post(&url, body) {
                    Ok(res) => res,
                    Err(e) => {
                        sync_failed(e.into());
                        unreachable!()
                    }
                };

                match res {
                    V1Response::FileItemDeleted => trace!("Deleted {}", item.path),
                    V1Response::Error {
                        kind: V1Error::FileNotFound,
                    } => debug!("Delete {} returns file not found.", item.path),
                    res => {
                        v1_handle(&res).unwrap();
                        unexpected_response("FileItemDeleted", res)
                    }
                }
            }

            println!("\rDeleting objects ({total}/{total}), done.")
        }

        if !self.created_dirs.is_empty() {
            let total = self.created.len();
            let url = get_url("/api/storage/v1/mkdir");
            for (i, item) in self.created_dirs.iter().enumerate() {
                trace!("Creating directory {}.", item.path);
                print!("\rCreating directories ({}/{total})...", i);
                stdout.flush().unwrap();

                let body = V1PathOnly {
                    token: creds.token.clone(),
                    path: format!("{}/{}", head.path, item.path.clone()),
                };

                let res: V1Response = match post(&url, body) {
                    Ok(res) => res,
                    Err(e) => {
                        sync_failed(e.into());
                        unreachable!()
                    }
                };

                match res {
                    V1Response::FileItemCreated => trace!("Created directory {}", item.path),
                    res => {
                        v1_handle(&res).unwrap();
                        unexpected_response("FileItemCreated", res)
                    }
                }
            }

            println!("\rCreating directories ({total}/{total}), done.")
        }

        if !(self.changed.is_empty() && self.created.is_empty()) {
            let total = total(&self.changed) + total(&self.created);
            let mut count = 0;

            for item in self.changed.iter().chain(self.created.iter()) {
                trace!("Uploading item {}.", item.path);
                print!("\rUploading changes ({}/{total})...", count);
                let url = get_url(&format!(
                    "/api/storage/v1/upload-overwrite/{}/{}/{}",
                    creds.token, head.path, item.path
                ));

                let res: V1Response = match upload(&url, &PathBuf::from(&item.path)) {
                    Ok(res) => res,
                    Err(e) => {
                        sync_failed(e.into());
                        unreachable!()
                    }
                };

                match res {
                    V1Response::FileItemCreated => trace!("Uploaded file {}", item.path),
                    res => {
                        v1_handle(&res).unwrap();
                        unexpected_response("FileItemCreated", res)
                    }
                }

                count += item.size;
            }

            println!("\rUploading changes ({total}/{total})...");
        }

        Ok(())
    }
}

fn total(items: &[TreeDiffItem]) -> u64 {
    items.iter().map(|item| item.size).sum()
}

// fn get_path(path: &Path) -> PathBuf {
//     OUTPUT_DIR.get().unwrap().join(if path.starts_with("/") {
//         path.strip_prefix("/").unwrap()
//     } else {
//         &path
//     })
// }

impl From<&str> for TreeDiffItem {
    fn from(value: &str) -> Self {
        Self {
            size: 0,
            path: value.to_string(),
        }
    }
}

impl From<PathBuf> for TreeDiffItem {
    fn from(value: PathBuf) -> Self {
        Self {
            size: 0,
            path: value.to_string_lossy().to_string(),
        }
    }
}

// compare old to new
fn cmp_del_recurse(old: &V1DirTreeNode, new: &V1DirTreeNode, current: &Path) -> Vec<TreeDiffItem> {
    let mut items = Vec::new();

    match &old.content {
        V1DirTreeItem::Dir { content: oc } => match &new.content {
            V1DirTreeItem::Dir { content: nc } => {
                for entry in oc.iter() {
                    let dir = is_dir(&entry.content);
                    let find = nc.iter().find(|new_entry| new_entry.name == entry.name);

                    let find = match find {
                        Some(f) => f,
                        None => {
                            items.push(TreeDiffItem::from(
                                current.join(&entry.name),
                                if dir {
                                    DIR_SIZE
                                } else {
                                    file_meta(&entry.content).1
                                },
                            ));
                            continue;
                        }
                    };

                    if dir != is_dir(&find.content) {
                        items.push(TreeDiffItem::from(
                            current.join(&entry.name),
                            if dir {
                                DIR_SIZE
                            } else {
                                file_meta(&entry.content).1
                            },
                        ));
                        continue;
                    }

                    if dir {
                        items.append(&mut cmp_del_recurse(
                            entry,
                            find,
                            current.join(&entry.name).as_path(),
                        ))
                    }
                }
            }
            V1DirTreeItem::File { size, .. } => {
                items.push(TreeDiffItem::from(current.to_path_buf(), *size))
            }
        },
        V1DirTreeItem::File { .. } => unreachable!(),
    }

    items
}

// compare new to old
// (created, created_dirs , changed)
fn cmp_updated_recurse(
    old: &V1DirTreeNode,
    new: &V1DirTreeNode,
    current: &Path,
) -> (Vec<TreeDiffItem>, Vec<TreeDiffItem>, Vec<TreeDiffItem>) {
    let mut created = Vec::new();
    let mut created_dirs = Vec::new();
    let mut changed = Vec::new();

    match &new.content {
        V1DirTreeItem::Dir { content: nc } => match &old.content {
            V1DirTreeItem::Dir { content: oc } => {
                for entry in nc.iter() {
                    let dir = is_dir(&entry.content);
                    let find = oc.iter().find(|old_entry| old_entry.name == entry.name);

                    let find = match find {
                        Some(f) => f,
                        None if dir => {
                            created_dirs
                                .push(TreeDiffItem::from(current.join(&entry.name), DIR_SIZE));

                            let (mut sub_created, mut sub_created_dirs, mut sub_changed) =
                                cmp_updated_recurse(
                                    &V1DirTreeNode {
                                        visibility: DEFAULT_VIS,
                                        name: String::new(),
                                        content: V1DirTreeItem::Dir {
                                            content: Vec::new(),
                                        },
                                    },
                                    entry,
                                    current.join(&entry.name).as_path(),
                                );
                            created.append(&mut sub_created);
                            created_dirs.append(&mut sub_created_dirs);
                            changed.append(&mut sub_changed);
                            continue;
                        }
                        None => {
                            created.push(TreeDiffItem::from(
                                current.join(&entry.name),
                                file_meta(&entry.content).1,
                            ));
                            continue;
                        }
                    };

                    if dir != is_dir(&find.content) {
                        if dir {
                            created_dirs
                                .push(TreeDiffItem::from(current.join(&entry.name), DIR_SIZE));
                        } else {
                            created.push(TreeDiffItem::from(
                                current.join(&entry.name),
                                file_meta(&entry.content).1,
                            ));
                        }
                        continue;
                    }

                    if dir {
                        let (mut sub_created, mut sub_created_dirs, mut sub_changed) =
                            cmp_updated_recurse(find, entry, current.join(&entry.name).as_path());
                        created.append(&mut sub_created);
                        created_dirs.append(&mut sub_created_dirs);
                        changed.append(&mut sub_changed);
                        continue;
                    }

                    let (last_modified, size) = file_meta(&entry.content);
                    if last_modified > file_meta(&find.content).0 {
                        changed.push(TreeDiffItem::from(current.join(&entry.name), size))
                    }
                }
            }
            V1DirTreeItem::File { size, .. } => {
                created.push(TreeDiffItem::from(current.to_path_buf(), *size))
            }
        },
        V1DirTreeItem::File { .. } => unreachable!(),
    }

    (created, created_dirs, changed)
}

fn is_dir(item: &V1DirTreeItem) -> bool {
    matches!(item, V1DirTreeItem::Dir { .. })
}

fn file_meta(item: &V1DirTreeItem) -> (u64, u64) {
    match item {
        V1DirTreeItem::File {
            last_modified,
            size,
        } => (*last_modified, *size),
        V1DirTreeItem::Dir { .. } => unreachable!("expected file only"),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FsHead {
    pub path: String,
    pub id: i64,
}

impl From<&str> for FsHead {
    fn from(value: &str) -> Self {
        match serde_json::from_str::<Self>(value) {
            Ok(h) => Self {
                path: html_escape::decode_html_entities(&h.path).to_string(),
                ..h
            },
            Err(e) => {
                debug!("Error deserialising {e}");
                bad_clone_json();
                unreachable!()
            }
        }
    }
}
