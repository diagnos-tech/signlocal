//! Trivial inputs.

use super::*;

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
