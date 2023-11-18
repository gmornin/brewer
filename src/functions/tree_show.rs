use crate::functions::*;
use goodmorning_bindings::services::v1::{V1DirTreeItem, V1DirTreeNode};

pub fn tree_show(tree: &V1DirTreeNode) {
    tree_recurse(tree, &[])
}

fn tree_recurse(node: &V1DirTreeNode, wires: &[bool]) {
    let dir = match &node.content {
        V1DirTreeItem::File { .. } => unreachable!(),
        V1DirTreeItem::Dir { content } => content,
    };

    println!("{BLUE}{}/", node.name);

    for (i, item) in dir.iter().enumerate() {
        for wire in wires.iter() {
            if *wire {
                print!("{GRAY}│  ")
            } else {
                print!("   ")
            }
        }

        if i == dir.len() - 1 {
            print!("{GRAY}└──")
        } else {
            print!("{GRAY}├──")
        }

        match &item.content {
            V1DirTreeItem::File { .. } => println!("{YELLOW}{}", item.name),
            V1DirTreeItem::Dir { .. } => {
                let mut wires = wires.to_vec();
                wires.push(i != dir.len() - 1);
                tree_recurse(item, &wires);
            }
        }
    }
}
