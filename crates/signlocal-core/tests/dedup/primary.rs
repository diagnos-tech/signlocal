//! Choosing the primary.

use super::*;

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
