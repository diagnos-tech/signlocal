//! Everything every key source can see, parsed and de-duplicated.

use probe_core::{CertError, CertInfo, Deduped, Fingerprint, dedup_by_fingerprint};

use crate::keystores::{self, FoundKey, Opened, Options, SourceFailure};

/// One key as found by one source.
#[derive(Debug)]
pub struct Entry {
    /// Index into [`Inventory::opened`]`.keystores`.
    pub store: usize,
    pub key: FoundKey,
    pub info: Result<CertInfo, CertError>,
}

/// All sources, and every key they listed.
pub struct Inventory {
    pub opened: Opened,
    pub entries: Vec<Entry>,
}

impl Inventory {
    /// Opens every source and lists it. Sources that fail to open or list
    /// end up in `opened.failures`; they never abort the run.
    pub fn collect(options: &Options) -> Inventory {
        let mut opened = keystores::open_all(options);
        let mut entries = Vec::new();
        let mut failures = Vec::new();
        for (store, keystore) in opened.keystores.iter_mut().enumerate() {
            match keystore.list() {
                Ok(keys) => entries.extend(keys.into_iter().map(|key| Entry {
                    store,
                    info: CertInfo::from_der(&key.cert_der),
                    key,
                })),
                Err(error) => failures.push(SourceFailure {
                    source: keystore.name(),
                    error: error.to_string(),
                }),
            }
        }
        opened.failures.extend(failures);
        Inventory { opened, entries }
    }

    /// Entries merged by certificate, OS sources first.
    pub fn deduped(&self) -> Vec<Deduped<&Entry>> {
        dedup_by_fingerprint(self.entries.iter().collect(), |entry| {
            (Fingerprint::of(&entry.key.cert_der), entry.key.kind)
        })
    }
}
