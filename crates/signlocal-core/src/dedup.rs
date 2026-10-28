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
