//! Order and grouping.

use super::*;

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
