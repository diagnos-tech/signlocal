//! Types.

use super::*;

#[test]
fn source_kind_is_copy_eq_and_hashable() {
    let kind = System;
    let copy = kind;
    assert_eq!(kind, copy);
    assert_ne!(System, Pkcs11);
    assert_eq!(
        [System, Pkcs11, System]
            .into_iter()
            .collect::<HashSet<_>>()
            .len(),
        2
    );
}

#[test]
fn deduped_is_cloneable_comparable_and_debuggable() {
    let group = Deduped {
        primary: 1,
        alternates: vec![2, 3],
    };
    assert_eq!(group.clone(), group);
    assert_ne!(
        group,
        Deduped {
            primary: 1,
            alternates: vec![3, 2]
        }
    );
    assert!(format!("{group:?}").contains("Deduped"));
}
