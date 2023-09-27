use std::path::{Path, PathBuf};

use goodmorning_bindings::services::v1::{V1DirTreeItem, V1DirTreeNode};

use crate::functions::DEFAULT_VIS;

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
    pub path: PathBuf,
}

impl TreeDiffItem {
    fn from(path: PathBuf, size: u64) -> Self {
        Self { size, path }
    }
}

impl TreeDiff {
    pub fn cmp(old: &V1DirTreeNode, new: &V1DirTreeNode) -> Self {
        let (created, created_dirs, changed) =
            cmp_updated_recurse(old, new, PathBuf::new().as_path());
        Self {
            created,
            created_dirs,
            changed,
            deleted: cmp_del_recurse(old, new, PathBuf::new().as_path()),
        }
    }
}

impl From<&str> for TreeDiffItem {
    fn from(value: &str) -> Self {
        PathBuf::from(value).into()
    }
}

impl From<PathBuf> for TreeDiffItem {
    fn from(value: PathBuf) -> Self {
        Self::from(value, 0)
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
