//! Merging the same certificate seen through more than one key source.
//!
//! A token with both an OS driver (minidriver, CryptoTokenKit) and a PKCS#11
//! module shows up twice. The user must see it once, and signing should go
//! through the OS, whose PIN dialog the user already knows.

use std::collections::HashMap;

use crate::fingerprint::Fingerprint;

/// Where a key was found, for de-duplication priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceKind {
    /// The OS key store (CNG/CAPI, Keychain/CryptoTokenKit). Preferred.
    System,
    /// A PKCS#11 module loaded by the app.
    Pkcs11,
}

/// One certificate after merging, with the other ways to reach it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deduped<T> {
    pub primary: T,
    pub alternates: Vec<T>,
}

/// Groups `items` by fingerprint, keeping first-appearance order.
///
/// The primary of each group is its first [`SourceKind::System`] item, or its
/// first item when none is; every other item becomes an alternate, in input
/// order.
pub fn dedup_by_fingerprint<T>(
    items: Vec<T>,
    key: impl Fn(&T) -> (Fingerprint, SourceKind),
) -> Vec<Deduped<T>> {
    let mut order = Vec::new();
    let mut groups: HashMap<Fingerprint, Group<T>> = HashMap::new();

    for item in items {
        let (fingerprint, kind) = key(&item);
        match groups.get_mut(&fingerprint) {
            Some(group) => group.add(item, kind),
            None => {
                order.push(fingerprint);
                groups.insert(fingerprint, Group::new(item, kind));
            }
        }
    }

    order
        .into_iter()
        .filter_map(|fingerprint| groups.remove(&fingerprint))
        .map(Group::finish)
        .collect()
}

/// A group under construction. Items are merged as they arrive so the input
/// is walked once and never indexed.
struct Group<T> {
    primary: T,
    primary_kind: SourceKind,
    alternates: Vec<T>,
}

impl<T> Group<T> {
    fn new(primary: T, primary_kind: SourceKind) -> Self {
        Self {
            primary,
            primary_kind,
            alternates: Vec::new(),
        }
    }

    fn add(&mut self, item: T, kind: SourceKind) {
        let displaces_primary =
            self.primary_kind != SourceKind::System && kind == SourceKind::System;
        if displaces_primary {
            // A non-System primary is always the group's first item, so it
            // goes to the front of the alternates to keep input order.
            let demoted = std::mem::replace(&mut self.primary, item);
            self.alternates.insert(0, demoted);
            self.primary_kind = kind;
        } else {
            self.alternates.push(item);
        }
    }

    fn finish(self) -> Deduped<T> {
        Deduped {
            primary: self.primary,
            alternates: self.alternates,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fp(n: u8) -> Fingerprint {
        Fingerprint::from_bytes([n; 32])
    }

    type Item = (&'static str, u8, SourceKind);

    fn run(items: Vec<Item>) -> Vec<Deduped<&'static str>> {
        dedup_by_fingerprint(items, |(_, n, kind)| (fp(*n), *kind))
            .into_iter()
            .map(|d| Deduped {
                primary: d.primary.0,
                alternates: d.alternates.into_iter().map(|a| a.0).collect(),
            })
            .collect()
    }

    fn group(primary: &'static str, alternates: &[&'static str]) -> Deduped<&'static str> {
        Deduped {
            primary,
            alternates: alternates.to_vec(),
        }
    }

    #[test]
    fn empty_input_gives_empty_output() {
        assert!(run(vec![]).is_empty());
    }

    #[test]
    fn system_beats_pkcs11_even_when_it_comes_later() {
        use SourceKind::{Pkcs11, System};
        let out = run(vec![
            ("p1", 1, Pkcs11),
            ("s1", 1, System),
            ("p2", 1, Pkcs11),
        ]);
        assert_eq!(out, vec![group("s1", &["p1", "p2"])]);
    }

    #[test]
    fn first_system_item_wins_among_several() {
        use SourceKind::{Pkcs11, System};
        let out = run(vec![
            ("s1", 1, System),
            ("p1", 1, Pkcs11),
            ("s2", 1, System),
        ]);
        assert_eq!(out, vec![group("s1", &["p1", "s2"])]);
    }

    #[test]
    fn first_item_is_primary_when_no_system_item_exists() {
        use SourceKind::Pkcs11;
        let out = run(vec![("a", 1, Pkcs11), ("b", 1, Pkcs11)]);
        assert_eq!(out, vec![group("a", &["b"])]);
    }

    #[test]
    fn output_follows_first_appearance_of_each_fingerprint() {
        use SourceKind::{Pkcs11, System};
        let out = run(vec![
            ("b1", 2, Pkcs11),
            ("a1", 1, System),
            ("b2", 2, System),
            ("c1", 3, Pkcs11),
        ]);
        assert_eq!(
            out,
            vec![group("b2", &["b1"]), group("a1", &[]), group("c1", &[])]
        );
    }

    #[test]
    fn identical_items_are_kept_as_alternates() {
        use SourceKind::System;
        let out = run(vec![("x", 1, System), ("x", 1, System)]);
        assert_eq!(out, vec![group("x", &["x"])]);
    }
}
