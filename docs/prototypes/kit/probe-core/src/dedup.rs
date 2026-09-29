//! Merging the same certificate seen through more than one key source.
//!
//! A token with both an OS driver (minidriver, CryptoTokenKit) and a PKCS#11
//! module shows up twice. The user must see it once, and signing should go
//! through the OS, whose PIN dialog the user already knows.

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
    todo!()
}
