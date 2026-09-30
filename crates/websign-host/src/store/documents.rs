//! The five documents and the rules that change them, independent of where
//! they are kept: the JSON files use them for persistence and the in-memory
//! stores for tests and for running without a data folder.

use serde::{Deserialize, Serialize};

use super::connections::ConnectionRecord;
use super::consent::ConsentRecord;
use super::errors::{ErrorRecord, MAX_RECENT_ERRORS};
use super::settings::Settings;

/// Certificates remembered per caller.
const MAX_CERTIFICATES_PER_CALLER: usize = 20;
/// Fingerprints kept for "most recently used anywhere".
const MAX_USAGE: usize = 100;

/// Files are always written as version 1; a newer file is read as far as its
/// fields are understood.
const VERSION: u32 = 1;

fn version() -> u32 {
    VERSION
}

/// `consent.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct ConsentDocument {
    pub version: u32,
    pub records: Vec<ConsentRecord>,
}

impl Default for ConsentDocument {
    fn default() -> Self {
        ConsentDocument {
            version: version(),
            records: Vec::new(),
        }
    }
}

impl ConsentDocument {
    pub fn get(&self, key: &str) -> Option<ConsentRecord> {
        self.records
            .iter()
            .find(|record| record.key == key)
            .cloned()
    }

    /// Creates the record or refreshes it: the certificate moves to the
    /// front, the original `remembered_at` stays.
    pub fn remember(&mut self, key: &str, fingerprint: &str, now: i64) {
        match self.records.iter_mut().find(|record| record.key == key) {
            Some(record) => touch(record, fingerprint, now),
            None => {
                let mut record = ConsentRecord {
                    key: key.to_owned(),
                    remembered_at: now,
                    last_used_at: now,
                    certificates: Vec::new(),
                };
                touch(&mut record, fingerprint, now);
                self.records.push(record);
            }
        }
    }

    /// Only for callers that are already remembered: using a certificate
    /// never grants consent by itself.
    pub fn record_use(&mut self, key: &str, fingerprint: &str, now: i64) {
        if let Some(record) = self.records.iter_mut().find(|record| record.key == key) {
            touch(record, fingerprint, now);
        }
    }

    pub fn revoke(&mut self, key: &str) {
        self.records.retain(|record| record.key != key);
    }
}

fn touch(record: &mut ConsentRecord, fingerprint: &str, now: i64) {
    record.last_used_at = now;
    record.certificates.retain(|known| known != fingerprint);
    record.certificates.insert(0, fingerprint.to_owned());
    record.certificates.truncate(MAX_CERTIFICATES_PER_CALLER);
}

/// `usage.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct UsageDocument {
    pub version: u32,
    pub fingerprints: Vec<String>,
}

impl Default for UsageDocument {
    fn default() -> Self {
        UsageDocument {
            version: version(),
            fingerprints: Vec::new(),
        }
    }
}

impl UsageDocument {
    pub fn record(&mut self, fingerprint: &str) {
        self.fingerprints.retain(|known| known != fingerprint);
        self.fingerprints.insert(0, fingerprint.to_owned());
        self.fingerprints.truncate(MAX_USAGE);
    }
}

/// `connections.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct ConnectionsDocument {
    pub version: u32,
    pub records: Vec<ConnectionRecord>,
}

impl Default for ConnectionsDocument {
    fn default() -> Self {
        ConnectionsDocument {
            version: version(),
            records: Vec::new(),
        }
    }
}

impl ConnectionsDocument {
    /// One record per browser: the newest replaces the previous.
    pub fn record(&mut self, record: ConnectionRecord) {
        self.records.retain(|known| known.browser != record.browser);
        self.records.push(record);
    }
}

/// `errors.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct ErrorsDocument {
    pub version: u32,
    pub records: Vec<ErrorRecord>,
}

impl Default for ErrorsDocument {
    fn default() -> Self {
        ErrorsDocument {
            version: version(),
            records: Vec::new(),
        }
    }
}

impl ErrorsDocument {
    /// Newest last; the oldest fall off.
    pub fn record(&mut self, record: ErrorRecord) {
        self.records.push(record);
        let excess = self.records.len().saturating_sub(MAX_RECENT_ERRORS);
        self.records.drain(..excess);
    }
}

/// `settings.json`: the settings next to the version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct SettingsDocument {
    pub version: u32,
    #[serde(flatten)]
    pub settings: Settings,
}

impl Default for SettingsDocument {
    fn default() -> Self {
        SettingsDocument {
            version: version(),
            settings: Settings::default(),
        }
    }
}

#[cfg(test)]
mod tests;
