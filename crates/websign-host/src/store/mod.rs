//! What the app remembers between runs, as small JSON files in the per-user
//! data folder. Several host processes may run at once (two browsers, a
//! desktop client), so every write is a locked read-modify-write with an
//! atomic rename, and readers re-read before each decision.
//!
//! Nothing here holds personal data beyond what the person chose to keep:
//! remembered origins/programs and certificate fingerprints stay on this
//! computer and never enter logs or diagnostics.

mod connections;
mod consent;
mod disk;
mod documents;
mod errors;
mod file;
mod memory;
mod paths;
mod private_file;
mod settings;
mod usage;

pub use connections::{ConnectionRecord, ConnectionStore};
pub use consent::{ConsentRecord, ConsentStore};
pub use disk::DiskStores;
pub use errors::{ErrorRecord, ErrorStore, MAX_RECENT_ERRORS};
pub use file::{JsonFile, StoreError};
pub use memory::MemoryStores;
pub use paths::data_dir;
pub use settings::{Settings, SettingsStore};
pub use usage::UsageStore;

/// Every store the engine uses, as one port.
pub trait Stores {
    fn consent(&mut self) -> &mut dyn ConsentStore;
    fn connections(&mut self) -> &mut dyn ConnectionStore;
    fn usage(&mut self) -> &mut dyn UsageStore;
    fn errors(&mut self) -> &mut dyn ErrorStore;
    fn settings(&mut self) -> &mut dyn SettingsStore;
}
