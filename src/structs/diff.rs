use std::path::{Path, PathBuf};

use goodmorning_bindings::services::v1::{V1DirTreeItem, V1DirTreeNode};

#[derive(Debug, PartialEq, Eq)]
pub struct TreeDiff {
    pub deletes: Vec<PathBuf>,
}

pub struct TreeDiffItem {
    pub variant: TreeDiffItemVariant,
    pub path: PathBuf,
}

pub enum TreeDiffItemVariant {
    Created,
    Deleted,
    Changed,
    CreatedDir,
}

impl TreeDiff {
    pub fn cmp(old: &V1DirTreeNode, new: &V1DirTreeNode) -> Self {
        Self {
            deletes: cmp_on_recurse(old, new, PathBuf::new().as_path()),
        }
    }
}

// compare old to new
fn cmp_on_recurse(old: &V1DirTreeNode, new: &V1DirTreeNode, current: &Path) -> Vec<PathBuf> {
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
                            items.push(current.join(&entry.name));
                            continue;
                        }
                    };

                    if dir != is_dir(&find.content) {
                        items.push(current.join(&entry.name));
                        continue;
                    }

                    if dir {
                        items.append(&mut cmp_on_recurse(
                            entry,
                            find,
                            current.join(&entry.name).as_path(),
                        ))
                    }
                }
            }
            V1DirTreeItem::File { .. } => items.push(current.to_path_buf()),
        },
        V1DirTreeItem::File { .. } => unreachable!(),
    }

    items
}

fn is_dir(item: &V1DirTreeItem) -> bool {
    matches!(item, V1DirTreeItem::Dir { .. })
}
