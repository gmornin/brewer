use crate::{functions::DEFAULT_VIS, structs::TreeDiff};
use goodmorning_bindings::services::v1::{V1DirTreeItem, V1DirTreeNode};
use std::path::PathBuf;

#[test]
fn delete_1() {
    let tree1 = V1DirTreeNode {
        name: "hello".to_string(),
        visibility: DEFAULT_VIS,
        content: V1DirTreeItem::Dir {
            content: vec![V1DirTreeNode {
                name: "hi".to_string(),
                visibility: DEFAULT_VIS,
                content: V1DirTreeItem::File {
                    last_modified: 0,
                    size: 0,
                },
            }],
        },
    };
    let tree2 = V1DirTreeNode {
        name: "hello".to_string(),
        visibility: DEFAULT_VIS,
        content: V1DirTreeItem::Dir { content: vec![] },
    };

    let diff = TreeDiff::cmp(&tree1, &tree2).deletes;

    assert_eq!(diff, vec![PathBuf::from("hi")])
}

#[test]
fn delete_2() {
    let tree1 = V1DirTreeNode {
        name: "hello".to_string(),
        visibility: DEFAULT_VIS,
        content: V1DirTreeItem::Dir {
            content: vec![
                V1DirTreeNode {
                    name: "test1".to_string(),
                    visibility: DEFAULT_VIS,
                    content: V1DirTreeItem::File {
                        last_modified: 0,
                        size: 0,
                    },
                },
                V1DirTreeNode {
                    name: "test2".to_string(),
                    visibility: DEFAULT_VIS,
                    content: V1DirTreeItem::File {
                        last_modified: 0,
                        size: 0,
                    },
                },
                V1DirTreeNode {
                    name: "test3".to_string(),
                    visibility: DEFAULT_VIS,
                    content: V1DirTreeItem::File {
                        last_modified: 0,
                        size: 0,
                    },
                },
            ],
        },
    };
    let tree2 = V1DirTreeNode {
        name: "hello".to_string(),
        visibility: DEFAULT_VIS,
        content: V1DirTreeItem::Dir {
            content: vec![V1DirTreeNode {
                name: "test2".to_string(),
                visibility: DEFAULT_VIS,
                content: V1DirTreeItem::File {
                    last_modified: 0,
                    size: 0,
                },
            }],
        },
    };

    let diff = TreeDiff::cmp(&tree1, &tree2).deletes;

    assert_eq!(diff, vec![PathBuf::from("test1"), PathBuf::from("test3")])
}

#[test]
fn delete_3() {
    let tree1 = V1DirTreeNode {
        name: "hello".to_string(),
        visibility: DEFAULT_VIS,
        content: V1DirTreeItem::Dir {
            content: vec![
                V1DirTreeNode {
                    name: "test1".to_string(),
                    visibility: DEFAULT_VIS,
                    content: V1DirTreeItem::File {
                        last_modified: 0,
                        size: 0,
                    },
                },
                V1DirTreeNode {
                    name: "test2".to_string(),
                    visibility: DEFAULT_VIS,
                    content: V1DirTreeItem::Dir {
                        content: vec![V1DirTreeNode {
                            name: "test3".to_string(),
                            visibility: DEFAULT_VIS,
                            content: V1DirTreeItem::File {
                                last_modified: 0,
                                size: 0,
                            },
                        }],
                    },
                },
                V1DirTreeNode {
                    name: "test4".to_string(),
                    visibility: DEFAULT_VIS,
                    content: V1DirTreeItem::Dir {
                        content: vec![V1DirTreeNode {
                            name: "test5".to_string(),
                            visibility: DEFAULT_VIS,
                            content: V1DirTreeItem::Dir {
                                content: vec![V1DirTreeNode {
                                    name: "test6".to_string(),
                                    visibility: DEFAULT_VIS,
                                    content: V1DirTreeItem::File {
                                        last_modified: 0,
                                        size: 0,
                                    },
                                }],
                            },
                        }],
                    },
                },
            ],
        },
    };
    let tree2 = V1DirTreeNode {
        name: "hello".to_string(),
        visibility: DEFAULT_VIS,
        content: V1DirTreeItem::Dir {
            content: vec![
                V1DirTreeNode {
                    name: "test2".to_string(),
                    visibility: DEFAULT_VIS,
                    content: V1DirTreeItem::Dir { content: vec![] },
                },
                V1DirTreeNode {
                    name: "test4".to_string(),
                    visibility: DEFAULT_VIS,
                    content: V1DirTreeItem::Dir {
                        content: vec![V1DirTreeNode {
                            name: "test5".to_string(),
                            visibility: DEFAULT_VIS,
                            content: V1DirTreeItem::Dir { content: vec![] },
                        }],
                    },
                },
            ],
        },
    };

    let diff = TreeDiff::cmp(&tree1, &tree2).deletes;

    assert_eq!(
        diff,
        vec![
            PathBuf::from("test1"),
            PathBuf::from("test2/test3"),
            PathBuf::from("test4/test5/test6")
        ]
    )
}
