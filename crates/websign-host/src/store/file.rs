//! A versioned JSON document with locked, atomic updates.

use std::path::PathBuf;

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Why a store could not be read or written. Callers degrade (no
/// persistence) and log; a store failure never blocks signing.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("I/O error on {path}: {detail}")]
    Io { path: String, detail: String },
    #[error("{path} is not a valid store file: {detail}")]
    Corrupt { path: String, detail: String },
}

/// One file: `{"version": 1, ...}`. A missing file reads as the default
/// value; a corrupt file is renamed to `<name>.corrupt` and reads as default.
#[derive(Debug, Clone)]
pub struct JsonFile {
    pub path: PathBuf,
}

impl JsonFile {
    /// Reads the current value.
    pub fn read<T: DeserializeOwned + Default>(&self) -> Result<T, StoreError> {
        todo!("SPEC.md §7.1")
    }

    /// Locks (`File::lock` on `<name>.lock`), re-reads, applies `change`,
    /// writes `<name>.tmp` and renames it over the file.
    pub fn update<T, R>(&self, change: impl FnOnce(&mut T) -> R) -> Result<R, StoreError>
    where
        T: DeserializeOwned + Serialize + Default,
    {
        let _ = change;
        todo!("SPEC.md §7.1")
    }
}
