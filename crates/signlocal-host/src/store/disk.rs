//! The stores as JSON files in the per-user data folder (`SPEC.md` §7).
//!
//! Every call re-reads the file: other host processes (a second browser, a
//! desktop client) and the diagnostics window change the same files, and a
//! revoked site must stop being remembered at once.

use std::path::Path;

use super::documents::{
    ConnectionsDocument, ConsentDocument, ErrorsDocument, SettingsDocument, UsageDocument,
};
use super::{
    ConnectionRecord, ConnectionStore, ConsentRecord, ConsentStore, ErrorRecord, ErrorStore,
    JsonFile, Settings, SettingsStore, StoreError, Stores, UsageStore,
};

/// All five stores under one folder.
#[derive(Debug)]
pub struct DiskStores {
    consent: DiskConsent,
    connections: DiskConnections,
    usage: DiskUsage,
    errors: DiskErrors,
    settings: DiskSettings,
}

impl DiskStores {
    /// Stores in `dir` (created on first write).
    pub fn new(dir: &Path) -> DiskStores {
        let file = |name: &str| JsonFile {
            path: dir.join(name),
        };
        DiskStores {
            consent: DiskConsent(file("consent.json")),
            connections: DiskConnections(file("connections.json")),
            usage: DiskUsage(file("usage.json")),
            errors: DiskErrors(file("errors.json")),
            settings: DiskSettings(file("settings.json")),
        }
    }
}

impl Stores for DiskStores {
    fn consent(&mut self) -> &mut dyn ConsentStore {
        &mut self.consent
    }
    fn connections(&mut self) -> &mut dyn ConnectionStore {
        &mut self.connections
    }
    fn usage(&mut self) -> &mut dyn UsageStore {
        &mut self.usage
    }
    fn errors(&mut self) -> &mut dyn ErrorStore {
        &mut self.errors
    }
    fn settings(&mut self) -> &mut dyn SettingsStore {
        &mut self.settings
    }
}

#[derive(Debug)]
struct DiskConsent(JsonFile);

impl ConsentStore for DiskConsent {
    fn get(&mut self, key: &str) -> Result<Option<ConsentRecord>, StoreError> {
        Ok(self.0.read::<ConsentDocument>()?.get(key))
    }
    fn list(&mut self) -> Result<Vec<ConsentRecord>, StoreError> {
        Ok(self.0.read::<ConsentDocument>()?.records)
    }
    fn remember(&mut self, key: &str, fingerprint: &str, now: i64) -> Result<(), StoreError> {
        self.0
            .update(|doc: &mut ConsentDocument| doc.remember(key, fingerprint, now))
    }
    fn record_use(&mut self, key: &str, fingerprint: &str, now: i64) -> Result<(), StoreError> {
        self.0
            .update(|doc: &mut ConsentDocument| doc.record_use(key, fingerprint, now))
    }
    fn revoke(&mut self, key: &str) -> Result<(), StoreError> {
        self.0.update(|doc: &mut ConsentDocument| doc.revoke(key))
    }
}

#[derive(Debug)]
struct DiskConnections(JsonFile);

impl ConnectionStore for DiskConnections {
    fn record(&mut self, record: ConnectionRecord) -> Result<(), StoreError> {
        self.0
            .update(|doc: &mut ConnectionsDocument| doc.record(record))
    }
    fn list(&mut self) -> Result<Vec<ConnectionRecord>, StoreError> {
        Ok(self.0.read::<ConnectionsDocument>()?.records)
    }
}

#[derive(Debug)]
struct DiskUsage(JsonFile);

impl UsageStore for DiskUsage {
    fn recent(&mut self) -> Result<Vec<String>, StoreError> {
        Ok(self.0.read::<UsageDocument>()?.fingerprints)
    }
    fn record(&mut self, fingerprint: &str, _now: i64) -> Result<(), StoreError> {
        self.0
            .update(|doc: &mut UsageDocument| doc.record(fingerprint))
    }
}

#[derive(Debug)]
struct DiskErrors(JsonFile);

impl ErrorStore for DiskErrors {
    fn record(&mut self, record: ErrorRecord) -> Result<(), StoreError> {
        self.0.update(|doc: &mut ErrorsDocument| doc.record(record))
    }
    fn list(&mut self) -> Result<Vec<ErrorRecord>, StoreError> {
        Ok(self.0.read::<ErrorsDocument>()?.records)
    }
}

#[derive(Debug)]
struct DiskSettings(JsonFile);

impl SettingsStore for DiskSettings {
    fn get(&mut self) -> Result<Settings, StoreError> {
        Ok(self.0.read::<SettingsDocument>()?.settings)
    }
    fn update(&mut self, change: &mut dyn FnMut(&mut Settings)) -> Result<(), StoreError> {
        self.0
            .update(|doc: &mut SettingsDocument| change(&mut doc.settings))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consent_survives_a_new_process() {
        let dir = tempfile::tempdir().unwrap();
        DiskStores::new(dir.path())
            .consent()
            .remember("https://a.example", "aa", 5)
            .unwrap();
        let mut other = DiskStores::new(dir.path());
        assert_eq!(other.consent().list().unwrap().len(), 1);
        other.consent().revoke("https://a.example").unwrap();
        assert!(
            DiskStores::new(dir.path())
                .consent()
                .list()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn files_carry_the_version() {
        let dir = tempfile::tempdir().unwrap();
        let mut stores = DiskStores::new(dir.path());
        stores.usage().record("aa", 1).unwrap();
        stores
            .settings()
            .update(&mut |settings| settings.test_signature_done = true)
            .unwrap();
        for name in ["usage.json", "settings.json"] {
            let text = std::fs::read_to_string(dir.path().join(name)).unwrap();
            assert!(text.contains("\"version\": 1"), "{name}: {text}");
        }
        assert!(stores.settings().get().unwrap().test_signature_done);
    }
}
