use std::path::{Path, PathBuf};

use crate::{functions::*, BASE_PATH, FULL_PATH};
use goodmorning_bindings::services::v1::{V1DirTreeItem, V1DirTreeNode};

pub fn tree_show(tree: &V1DirTreeNode) {
    tree_recurse(tree, &[], &PathBuf::from(BASE_PATH.get().unwrap()))
}

fn tree_recurse(node: &V1DirTreeNode, wires: &[bool], path: &Path) {
    let dir = match &node.content {
        V1DirTreeItem::File { .. } => unreachable!(),
        V1DirTreeItem::Dir { content } => content,
    };

    if *FULL_PATH.get().unwrap() {
        println!(
            "{BLUE}/{}{}",
            path.to_string_lossy().trim_matches('/'),
            if path == Path::new("/") { "" } else { "/" }
        );
    } else if wires.is_empty() {
        println!("{BLUE}{}/", path.join(&node.name).to_string_lossy());
    } else {
        println!("{BLUE}{}/", node.name);
    }

    for (i, item) in dir.iter().enumerate() {
        for wire in wires.iter() {
            if *wire {
                print!("{GREY}│  ")
            } else {
                print!("   ")
            }
        }

        if i == dir.len() - 1 {
            print!("{GREY}└──")
        } else {
            print!("{GREY}├──")
        }

        match &item.content {
            V1DirTreeItem::File { .. } => {
                println!(
                    "{YELLOW}{}",
                    if *FULL_PATH.get().unwrap() {
                        PathBuf::from("/")
                            .join(path)
                            .join(&item.name)
                            .to_string_lossy()
                            .to_string()
                    } else {
                        item.name.clone()
                    }
                )
            }
            V1DirTreeItem::Dir { .. } => {
                let mut wires = wires.to_vec();
                wires.push(i != dir.len() - 1);
                tree_recurse(item, &wires, &path.join(&item.name));
            }
        }
    }
}
