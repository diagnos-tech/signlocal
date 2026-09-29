//! Everything every key source can see, parsed and grouped by certificate.
//! The CLI commands and the native host read the machine through this.

use probe_core::{CertError, CertInfo, Deduped, Fingerprint, dedup_by_fingerprint};

use super::{FoundKey, Opened, Options, SourceFailure, open_all};

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
    /// Opens every source on this machine and lists it.
    pub fn collect(options: &Options) -> Inventory {
        Inventory::list(open_all(options))
    }

    /// Lists every opened source. Sources that fail to list join
    /// `opened.failures`; they never abort the run.
    pub fn list(mut opened: Opened) -> Inventory {
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

    /// Indices into [`Inventory::entries`], one group per certificate, with
    /// OS sources as primary.
    pub fn groups(&self) -> Vec<Deduped<usize>> {
        dedup_by_fingerprint((0..self.entries.len()).collect(), |&index| {
            let key = &self.entries[index].key;
            (Fingerprint::of(&key.cert_der), key.kind)
        })
    }

    /// `"<source>: <error>"` for every source that could not be opened or listed.
    pub fn warnings(&self) -> Vec<String> {
        self.opened
            .failures
            .iter()
            .map(|failure| format!("{}: {}", failure.source, failure.error))
            .collect()
    }
}
