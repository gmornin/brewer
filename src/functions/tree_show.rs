use std::path::{Path, PathBuf};

use crate::{functions::*, BASE_PATH};
use goodmorning_bindings::services::v1::{V1DirTreeItem, V1DirTreeNode};

pub fn tree_show(tree: &V1DirTreeNode) {
    tree_recurse(tree, &[], &PathBuf::from(BASE_PATH.get().unwrap()))
}

fn tree_recurse(node: &V1DirTreeNode, wires: &[bool], path: &Path) {
    let dir = match &node.content {
        V1DirTreeItem::File { .. } => unreachable!(),
        V1DirTreeItem::Dir { content } => content,
    };

    println!("{BLUE}{}/", path.join(&node.name).to_string_lossy());

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
                println!("{YELLOW}{}", path.join(&item.name).to_string_lossy())
            }
            V1DirTreeItem::Dir { .. } => {
                let mut wires = wires.to_vec();
                wires.push(i != dir.len() - 1);
                tree_recurse(item, &wires, &path.join(&item.name));
            }
        }
    }
}
