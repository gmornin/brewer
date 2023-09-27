use crate::{functions::DEFAULT_VIS, structs::TreeDiff};
use goodmorning_bindings::services::v1::{V1DirTreeItem, V1DirTreeNode};

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

    let diff = TreeDiff::cmp(&tree1, &tree2).deleted;

    assert_eq!(diff, vec!["hi".into()])
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

    let diff = TreeDiff::cmp(&tree1, &tree2).deleted;

    assert_eq!(diff, vec!["test1".into(), "test3".into()])
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

    let diff = TreeDiff::cmp(&tree1, &tree2).deleted;

    assert_eq!(
        diff,
        vec![
            "test1".into(),
            "test2/test3".into(),
            "test4/test5/test6".into()
        ]
    )
}

#[test]
fn all_1() {
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
                V1DirTreeNode {
                    name: "test1".to_string(),
                    visibility: DEFAULT_VIS,
                    content: V1DirTreeItem::File {
                        last_modified: 1000,
                        size: 0,
                    },
                },
            ],
        },
    };

    let diff = TreeDiff::cmp(&tree1, &tree2);

    assert_eq!(
        diff,
        TreeDiff {
            deleted: vec!["test2/test3".into(), "test4/test5/test6".into()],
            changed: vec!["test1".into(),],
            ..Default::default()
        }
    )
}

#[test]
fn all_2() {
    let tree1 = V1DirTreeNode {
        name: "hello".to_string(),
        visibility: DEFAULT_VIS,
        content: V1DirTreeItem::Dir {
            content: vec![
                V1DirTreeNode {
                    name: "test1".to_string(),
                    visibility: DEFAULT_VIS,
                    content: V1DirTreeItem::Dir { content: vec![] },
                },
                V1DirTreeNode {
                    name: "test4".to_string(),
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
                    content: V1DirTreeItem::File {
                        last_modified: 1,
                        size: 0,
                    },
                },
            ],
        },
    };

    let diff = TreeDiff::cmp(&tree1, &tree2);

    assert_eq!(
        diff,
        TreeDiff {
            created: vec!["test1".into(), "test2/test3".into()],
            created_dirs: vec!["test2".into()],
            deleted: vec!["test1".into()],
            changed: vec!["test4".into()],
        }
    )
}
