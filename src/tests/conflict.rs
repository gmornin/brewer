use crate::structs::DiffConflictAction::*;
use crate::structs::DiffConflictItem;
use crate::structs::TreeDiff;

#[test]
fn conflict1() {
    let this = TreeDiff {
        created: vec!["hello".into()],
        created_dirs: vec![],
        changed: vec![],
        deleted: vec![],
    };
    let remote = TreeDiff {
        created: vec![],
        created_dirs: vec![],
        changed: vec![],
        deleted: vec!["no".into()],
    };

    assert_eq!(this.conflict(&remote), vec![].into())
}

#[test]
fn conflict2() {
    let this = TreeDiff {
        created: vec!["hello".into()],
        created_dirs: vec![],
        changed: vec![],
        deleted: vec![],
    };
    let remote = TreeDiff {
        created: vec![],
        created_dirs: vec!["hello".into()],
        changed: vec![],
        deleted: vec![],
    };

    assert_eq!(
        this.conflict(&remote),
        vec![DiffConflictItem {
            path: "hello".into(),
            fs: Create,
            remote: CreateDir
        }]
        .into()
    )
}

#[test]
fn conflict3() {
    let this = TreeDiff {
        created: vec!["hello/file".into()],
        created_dirs: vec![],
        changed: vec![],
        deleted: vec![],
    };
    let remote = TreeDiff {
        created: vec![],
        created_dirs: vec![],
        changed: vec![],
        deleted: vec!["hello".into()],
    };

    assert_eq!(
        this.conflict(&remote),
        vec![DiffConflictItem {
            path: "hello/file".into(),
            fs: Create,
            remote: Delete
        }]
        .into()
    )
}

#[test]
fn conflict4() {
    let remote = TreeDiff {
        created: vec!["hello/file".into()],
        created_dirs: vec![],
        changed: vec![],
        deleted: vec![],
    };
    let this = TreeDiff {
        created: vec![],
        created_dirs: vec![],
        changed: vec![],
        deleted: vec!["hello".into()],
    };

    assert_eq!(
        this.conflict(&remote),
        vec![DiffConflictItem {
            path: "hello".into(),
            fs: Delete,
            remote: Create
        }]
        .into()
    )
}
