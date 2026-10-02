//! Generic over the item type.

use super::*;

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
