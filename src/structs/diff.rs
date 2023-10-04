use std::{
    error::Error,
    fmt::Display,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use goodmorning_bindings::services::v1::{V1DirTreeItem, V1DirTreeNode};
use log::*;
use serde::{Deserialize, Serialize};

use crate::{
    exit_codes::{bad_clone_json, download_failed},
    functions::{download, DEFAULT_VIS},
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

fn sort(vec: &mut [TreeDiffItem]) {
    vec.sort_by_key(|item| item.path.to_string());
}

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
        sort(&mut self.created);
        sort(&mut self.created_dirs);
        sort(&mut self.changed);
        sort(&mut self.deleted);
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

    pub fn pull(&self, head: &FsHead, instance: &str, owned: bool) -> Result<(), Box<dyn Error>> {
        let mut stdout = io::stdout();
        let output = OUTPUT_DIR.get().unwrap();

        if !self.deleted.is_empty() {
            print!("\rDeleting objects...%",);
            stdout.flush().unwrap();
            for item in self.deleted.iter() {
                let display_path = item.path.trim_matches('/');
                let path = output.join(display_path);

                if !path.exists() {
                    continue;
                }

                trace!("Deleting directory {display_path}.");

                if fs::metadata(&path)?.is_dir() {
                    trace!("Deleting directory {display_path}.");
                    fs::remove_dir_all(path)?;
                } else {
                    trace!("Deleting file {display_path}.");
                    fs::remove_file(path)?;
                }
            }
            println!("\rDeleting objects, done. ");
        }

        if !self.created_dirs.is_empty() {
            for item in self.created_dirs.iter() {
                let display_path = item.path.trim_matches('/');
                let path = output.join(display_path);
                trace!("Creating directory {display_path}.");
                fs::create_dir(path)?;
            }
        }

        if !(self.created.is_empty() && self.changed.is_empty()) {
            let creds = unsafe { CREDS.get().unwrap() };
            let total = total(&self.created) + total(&self.changed);
            let mut counting = 0;

            for item in self.changed.iter().chain(self.created.iter()) {
                print!(
                    "\rDownloading objects ({counting}/{total})... {}%",
                    counting * 100 / total
                );
                stdout.flush().unwrap();

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

                let display_path = item.path.trim_matches('/');
                let path = output.join(display_path);

                if download(&url, &path).is_err() {
                    println!();
                    download_failed(display_path);
                }

                counting += item.size;
            }

            println!("\rDownloading objects ({counting}/{total}), done. ");
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

#[derive(Serialize, Deserialize, Debug)]
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
