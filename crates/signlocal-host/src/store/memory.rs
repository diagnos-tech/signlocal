//! The stores in memory: for tests, and for a host that has no data folder
//! (it then runs without persistence, `docs/architecture/overview.md`).

use super::documents::{
    ConnectionsDocument, ConsentDocument, ErrorsDocument, SettingsDocument, UsageDocument,
};
use super::{
    ConnectionRecord, ConnectionStore, ConsentRecord, ConsentStore, ErrorRecord, ErrorStore,
    Settings, SettingsStore, StoreError, Stores, UsageStore,
};

/// All five stores, forgotten when dropped.
#[derive(Debug, Default)]
pub struct MemoryStores {
    consent: ConsentDocument,
    connections: ConnectionsDocument,
    usage: UsageDocument,
    errors: ErrorsDocument,
    settings: SettingsDocument,
}

impl MemoryStores {
    pub fn new() -> MemoryStores {
        MemoryStores::default()
    }
}

impl Stores for MemoryStores {
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

impl ConsentStore for ConsentDocument {
    fn get(&mut self, key: &str) -> Result<Option<ConsentRecord>, StoreError> {
        Ok(ConsentDocument::get(self, key))
    }
    fn list(&mut self) -> Result<Vec<ConsentRecord>, StoreError> {
        Ok(self.records.clone())
    }
    fn remember(&mut self, key: &str, fingerprint: &str, now: i64) -> Result<(), StoreError> {
        ConsentDocument::remember(self, key, fingerprint, now);
        Ok(())
    }
    fn record_use(&mut self, key: &str, fingerprint: &str, now: i64) -> Result<(), StoreError> {
        ConsentDocument::record_use(self, key, fingerprint, now);
        Ok(())
    }
    fn revoke(&mut self, key: &str) -> Result<(), StoreError> {
        ConsentDocument::revoke(self, key);
        Ok(())
    }
}

impl ConnectionStore for ConnectionsDocument {
    fn record(&mut self, record: ConnectionRecord) -> Result<(), StoreError> {
        ConnectionsDocument::record(self, record);
        Ok(())
    }
    fn list(&mut self) -> Result<Vec<ConnectionRecord>, StoreError> {
        Ok(self.records.clone())
    }
}

impl UsageStore for UsageDocument {
    fn recent(&mut self) -> Result<Vec<String>, StoreError> {
        Ok(self.fingerprints.clone())
    }
    fn record(&mut self, fingerprint: &str, _now: i64) -> Result<(), StoreError> {
        UsageDocument::record(self, fingerprint);
        Ok(())
    }
}

impl ErrorStore for ErrorsDocument {
    fn record(&mut self, record: ErrorRecord) -> Result<(), StoreError> {
        ErrorsDocument::record(self, record);
        Ok(())
    }
    fn list(&mut self) -> Result<Vec<ErrorRecord>, StoreError> {
        Ok(self.records.clone())
    }
}

impl SettingsStore for SettingsDocument {
    fn get(&mut self) -> Result<Settings, StoreError> {
        Ok(self.settings.clone())
    }
    fn update(&mut self, change: &mut dyn FnMut(&mut Settings)) -> Result<(), StoreError> {
        change(&mut self.settings);
        Ok(())
    }
}
