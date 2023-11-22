use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use goodmorning_bindings::services::v1::{
    ItemVisibility, V1DirTreeItem, V1DirTreeNode, V1Visibility,
};
use ignore::gitignore::GitignoreBuilder;
use log::*;
use tokio::fs;

use crate::exit_codes::ignore_add_failed;

pub const DEFAULT_VIS: V1Visibility = V1Visibility {
    inherited: true,
    visibility: ItemVisibility::Private,
};

pub async fn ignore_tree(path: &Path) -> V1DirTreeNode {
    trace!(
        "Started fs tree tracing in `{}`",
        path.to_string_lossy().to_string()
    );
    let mut builder = GitignoreBuilder::new(path);
    builder.add_line(None, ".gmrepo.json").unwrap();
    V1DirTreeNode {
        name: path
            .file_name()
            .unwrap_or(OsStr::new(""))
            .to_string_lossy()
            .to_string(),
        visibility: DEFAULT_VIS,
        content: ignore_tree_recurse(
            &if path == PathBuf::new().as_path() {
                PathBuf::from(".")
            } else {
                path.to_path_buf()
            },
            PathBuf::from("").as_path(),
            builder,
        )
        .await,
    }
}

#[async_recursion::async_recursion]
pub async fn ignore_tree_recurse(
    base: &Path,
    current: &Path,
    mut builder: GitignoreBuilder,
) -> V1DirTreeItem {
    trace!(
        "Fs tree tracing in `{}`",
        current.to_string_lossy().to_string()
    );
    let real_path = base.join(current);

    let gitignore = real_path.join(".gitignore");
    if fs::try_exists(&gitignore).await.unwrap() {
        if let Some(e) = tokio::task::block_in_place(|| builder.add(&gitignore)) {
            debug!("{e}");
            ignore_add_failed(&gitignore)
        }
    }

    let gmignore = real_path.join(".gmignore");
    if fs::try_exists(&gmignore).await.unwrap() {
        if let Some(e) = tokio::task::block_in_place(|| builder.add(&gmignore)) {
            debug!("{e}");
            ignore_add_failed(&gmignore)
        }
    }

    let ignores = builder.build().unwrap();

    let mut entries = Vec::new();

    let mut diritems = fs::read_dir(real_path).await.unwrap();

    while let Some(entry) = diritems.next_entry().await.unwrap() {
        let metadata = entry.metadata().await.unwrap();
        let display_path = match entry.path().strip_prefix(base) {
            Ok(p) => p.to_path_buf(),
            Err(_e) => entry.path(),
        };

        if ignores
            .matched(&display_path, metadata.is_dir())
            .is_ignore()
        {
            continue;
        }

        entries.push(V1DirTreeNode {
            visibility: DEFAULT_VIS,
            name: entry.file_name().to_string_lossy().to_string(),
            content: if metadata.is_file() {
                V1DirTreeItem::File {
                    last_modified: metadata
                        .modified()
                        .unwrap()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    size: metadata.len(),
                }
            } else {
                ignore_tree_recurse(base, &display_path, builder.clone()).await
            },
        });
    }
    V1DirTreeItem::Dir { content: entries }
}
