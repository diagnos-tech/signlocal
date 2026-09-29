//! SPEC §8: `dedup_by_fingerprint`, `SourceKind`, `Deduped`.

use std::collections::HashSet;

use probe_core::{Deduped, Fingerprint, SourceKind, dedup_by_fingerprint};

use SourceKind::{Pkcs11, System};

/// An item as a key store would report it: a label to tell items apart, the
/// certificate's fingerprint and where it was found.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Item {
    label: &'static str,
    fp: Fingerprint,
    kind: SourceKind,
}

fn fp(n: u8) -> Fingerprint {
    Fingerprint::from_bytes([n; 32])
}

fn item(label: &'static str, n: u8, kind: SourceKind) -> Item {
    Item {
        label,
        fp: fp(n),
        kind,
    }
}

fn dedup(items: Vec<Item>) -> Vec<Deduped<Item>> {
    dedup_by_fingerprint(items, |i| (i.fp, i.kind))
}

/// `(primary, alternates)` as labels, for compact expectations.
fn labels(groups: &[Deduped<Item>]) -> Vec<(&'static str, Vec<&'static str>)> {
    groups
        .iter()
        .map(|g| {
            (
                g.primary.label,
                g.alternates.iter().map(|a| a.label).collect(),
            )
        })
        .collect()
}

// --- trivial inputs --------------------------------------------------------------------------

#[test]
fn empty_input_gives_empty_output() {
    assert!(dedup(vec![]).is_empty());
}

#[test]
fn a_single_item_is_its_own_primary() {
    for kind in [System, Pkcs11] {
        let out = dedup(vec![item("only", 1, kind)]);
        assert_eq!(labels(&out), [("only", vec![])]);
    }
}

#[test]
fn distinct_fingerprints_stay_separate_and_keep_input_order() {
    let out = dedup(vec![
        item("c", 3, Pkcs11),
        item("a", 1, System),
        item("b", 2, Pkcs11),
    ]);
    assert_eq!(labels(&out), [("c", vec![]), ("a", vec![]), ("b", vec![])]);
}

// --- choosing the primary --------------------------------------------------------------------------

#[test]
fn prefers_the_system_item_over_a_pkcs11_item_seen_first() {
    let out = dedup(vec![item("token", 1, Pkcs11), item("os", 1, System)]);
    assert_eq!(labels(&out), [("os", vec!["token"])]);
}

#[test]
fn keeps_a_system_item_seen_first_as_primary() {
    let out = dedup(vec![item("os", 1, System), item("token", 1, Pkcs11)]);
    assert_eq!(labels(&out), [("os", vec!["token"])]);
}

#[test]
fn takes_the_first_pkcs11_item_when_no_system_item_exists() {
    let out = dedup(vec![
        item("first", 1, Pkcs11),
        item("second", 1, Pkcs11),
        item("third", 1, Pkcs11),
    ]);
    assert_eq!(labels(&out), [("first", vec!["second", "third"])]);
}

#[test]
fn takes_the_first_system_item_when_there_are_several() {
    let out = dedup(vec![item("os-1", 1, System), item("os-2", 1, System)]);
    assert_eq!(labels(&out), [("os-1", vec!["os-2"])]);
}

#[test]
fn the_first_system_item_wins_among_a_mix_and_alternates_keep_input_order() {
    let out = dedup(vec![
        item("p1", 1, Pkcs11),
        item("s1", 1, System),
        item("s2", 1, System),
        item("p2", 1, Pkcs11),
    ]);
    assert_eq!(labels(&out), [("s1", vec!["p1", "s2", "p2"])]);
}

#[test]
fn a_late_system_item_still_beats_many_earlier_pkcs11_items() {
    let mut items: Vec<Item> = (0..5).map(|_| item("p", 1, Pkcs11)).collect();
    items.push(item("os", 1, System));
    let out = dedup(items);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].primary.label, "os");
    assert_eq!(out[0].alternates.len(), 5);
    assert!(out[0].alternates.iter().all(|a| a.kind == Pkcs11));
}

// --- order and grouping ---------------------------------------------------------------------------------

#[test]
fn orders_groups_by_the_first_appearance_of_their_fingerprint() {
    // Fingerprint 1 appears first (as PKCS#11), 2 second, then 1 again as System.
    let out = dedup(vec![
        item("a-token", 1, Pkcs11),
        item("b", 2, System),
        item("a-os", 1, System),
    ]);
    assert_eq!(labels(&out), [("a-os", vec!["a-token"]), ("b", vec![])]);
}

#[test]
fn interleaved_fingerprints_form_separate_groups() {
    let out = dedup(vec![
        item("x1", 10, Pkcs11),
        item("y1", 20, Pkcs11),
        item("z1", 30, System),
        item("x2", 10, System),
        item("y2", 20, Pkcs11),
        item("z2", 30, Pkcs11),
        item("x3", 10, Pkcs11),
        item("w1", 40, Pkcs11),
    ]);
    assert_eq!(
        labels(&out),
        [
            ("x2", vec!["x1", "x3"]),
            ("y1", vec!["y2"]),
            ("z1", vec!["z2"]),
            ("w1", vec![]),
        ]
    );
}

#[test]
fn does_not_order_by_fingerprint_value() {
    let out = dedup(vec![item("high", 0xFF, System), item("low", 0x01, System)]);
    assert_eq!(labels(&out), [("high", vec![]), ("low", vec![])]);
}

#[test]
fn fingerprints_that_differ_in_one_byte_are_different_certificates() {
    let mut bytes = [7u8; 32];
    let a = Item {
        label: "a",
        fp: Fingerprint::from_bytes(bytes),
        kind: System,
    };
    bytes[31] = 8;
    let b = Item {
        label: "b",
        fp: Fingerprint::from_bytes(bytes),
        kind: System,
    };
    assert_eq!(dedup(vec![a, b]).len(), 2);
}

#[test]
fn identical_items_are_not_discarded() {
    let twin = item("twin", 1, System);
    let out = dedup(vec![twin.clone(), twin.clone()]);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].primary, twin);
    assert_eq!(out[0].alternates, [twin]);
}

#[test]
fn keeps_every_input_item_exactly_once_in_the_right_group() {
    struct Numbered {
        id: u8,
        kind: SourceKind,
    }
    // Ids 0..60 fall into 7 groups by `id % 7`; ids divisible by 3 are System.
    let items: Vec<Numbered> = (0..60u8)
        .map(|id| Numbered {
            id,
            kind: if id % 3 == 0 { System } else { Pkcs11 },
        })
        .collect();
    let out = dedup_by_fingerprint(items, |n| (fp(n.id % 7), n.kind));
    assert_eq!(out.len(), 7);

    let mut seen = HashSet::new();
    for (k, group) in out.iter().enumerate() {
        let members: Vec<u8> = (0..60).filter(|id| usize::from(id % 7) == k).collect();
        let primary = members
            .iter()
            .copied()
            .find(|id| id % 3 == 0)
            .unwrap_or(members[0]);
        let alternates: Vec<u8> = members
            .iter()
            .copied()
            .filter(|id| *id != primary)
            .collect();
        assert_eq!(group.primary.id, primary, "group {k} primary");
        assert_eq!(
            group.alternates.iter().map(|n| n.id).collect::<Vec<_>>(),
            alternates,
            "group {k} alternates"
        );
        for id in std::iter::once(group.primary.id).chain(group.alternates.iter().map(|n| n.id)) {
            assert!(seen.insert(id), "{id} appears twice");
        }
    }
    assert_eq!(seen.len(), 60);
}

// --- generic over the item type -------------------------------------------------------------------------------

#[test]
fn works_with_items_that_are_not_clone_or_debug() {
    struct Opaque(u32);
    let items = vec![Opaque(1), Opaque(2), Opaque(3), Opaque(4)];
    let out = dedup_by_fingerprint(items, |o| {
        (fp((o.0 % 2) as u8), if o.0 == 4 { System } else { Pkcs11 })
    });
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].primary.0, 1);
    assert_eq!(
        out[0].alternates.iter().map(|o| o.0).collect::<Vec<_>>(),
        [3]
    );
    assert_eq!(
        out[1].primary.0, 4,
        "the System item beats the earlier PKCS#11 one"
    );
    assert_eq!(
        out[1].alternates.iter().map(|o| o.0).collect::<Vec<_>>(),
        [2]
    );
}

#[test]
fn works_with_owned_strings_and_a_key_closure_that_borrows_them() {
    let items = vec!["a:1:p".to_owned(), "b:2:s".to_owned(), "c:1:s".to_owned()];
    let out = dedup_by_fingerprint(items, |s| {
        let mut parts = s.split(':');
        let _ = parts.next();
        let n: u8 = parts.next().unwrap().parse().unwrap();
        (
            fp(n),
            if parts.next() == Some("s") {
                System
            } else {
                Pkcs11
            },
        )
    });
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].primary, "c:1:s");
    assert_eq!(out[0].alternates, ["a:1:p"]);
    assert_eq!(out[1].primary, "b:2:s");
}

// --- types -------------------------------------------------------------------------------------------------------

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
