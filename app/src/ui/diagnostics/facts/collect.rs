//! One full scan: key stores, USB and PC/SC, browsers and the app's own
//! records. Slow (drivers load, readers answer), so it runs on a worker
//! thread; key stores are not `Send` and live and die on that thread.

use std::path::PathBuf;

use jiff::Timestamp;
use jiff::tz::TimeZone;
use websign_devices::Snapshot;
use websign_devices::hints::DeviceDatabase;
use websign_host::store::{DiskStores, Stores};
use websign_keystores::inventory::Inventory;
use websign_keystores::{FoundKey, Options};

use super::{Facts, browsers, certificates, devices, drivers, home, system};

/// What a scan needs from the window.
#[derive(Debug, Clone)]
pub struct ScanInput {
    /// The app's data folder (connection records, recent errors); `None`
    /// runs without them.
    pub data_dir: Option<PathBuf>,
    /// Drivers added with "Add driver…".
    pub user_modules: Vec<PathBuf>,
    pub now: Timestamp,
    pub zone: TimeZone,
}

impl ScanInput {
    /// The current user's input now: the data folder, and the drivers they
    /// added from the settings store.
    pub fn current() -> ScanInput {
        let data_dir = websign_host::store::data_dir();
        let user_modules = data_dir
            .as_ref()
            .and_then(|dir| DiskStores::new(dir).settings().get().ok())
            .map(|settings| settings.user_modules)
            .unwrap_or_default();
        ScanInput {
            data_dir,
            user_modules,
            now: Timestamp::now(),
            zone: TimeZone::system(),
        }
    }
}

/// Scans everything once.
pub fn collect(input: &ScanInput) -> Facts {
    let hints = DeviceDatabase::embedded()
        .inspect_err(|error| log::warn!("device hints unavailable: {error}"))
        .ok();
    let options = Options {
        extra_modules: input.user_modules.clone(),
        // Listing must never wait for a key provider's dialog.
        silent: true,
        ..Options::default()
    };
    let inventory = Inventory::collect(&options);
    let home = home::current();
    let keys: Vec<&FoundKey> = inventory.entries.iter().map(|entry| &entry.key).collect();
    let (connections, recent_errors) = records(input.data_dir.as_ref());
    Facts {
        browsers: browsers::collect(&connections),
        devices: devices::collect(&Snapshot::scan(), hints.as_ref(), &keys),
        drivers: drivers::collect(&inventory, &input.user_modules, home.as_deref()),
        certificates: certificates::collect(&inventory, hints.as_ref(), input.now, &input.zone),
        recent_errors,
        os: system::os_description(),
    }
}

/// Connection records and recent errors; empty when unreadable (a missing
/// file is the normal state of a fresh install).
fn records(
    data_dir: Option<&PathBuf>,
) -> (
    Vec<websign_host::store::ConnectionRecord>,
    Vec<websign_host::store::ErrorRecord>,
) {
    let Some(dir) = data_dir else {
        return (Vec::new(), Vec::new());
    };
    let mut stores = DiskStores::new(dir);
    let connections = stores.connections().list().unwrap_or_else(|error| {
        log::warn!("connection records unreadable: {error}");
        Vec::new()
    });
    let errors = stores.errors().list().unwrap_or_else(|error| {
        log::warn!("recent errors unreadable: {error}");
        Vec::new()
    });
    (connections, errors)
}
