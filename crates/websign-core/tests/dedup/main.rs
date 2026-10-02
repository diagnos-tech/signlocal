//! SPEC §8: `dedup_by_fingerprint`, `SourceKind`, `Deduped`.

mod generic;
mod order;
mod primary;
mod trivial_inputs;
mod types;

use std::collections::HashSet;

use websign_core::{Deduped, Fingerprint, SourceKind, dedup_by_fingerprint};

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
