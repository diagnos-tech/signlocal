//! Every extension that connected (`hello` with `browser`), for the
//! diagnostics Browsers tab ("Extension 1.4.2 connected · 3 min ago").

use serde::{Deserialize, Serialize};
use websign_protocol::types::BrowserName;

use super::StoreError;

/// The last connection per browser.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionRecord {
    pub browser: BrowserName,
    pub browser_version: String,
    pub extension_version: String,
    /// Unix seconds.
    pub last_seen: i64,
}

/// Connection records, one per browser.
pub trait ConnectionStore {
    fn record(&mut self, record: ConnectionRecord) -> Result<(), StoreError>;
    fn list(&mut self) -> Result<Vec<ConnectionRecord>, StoreError>;
}
