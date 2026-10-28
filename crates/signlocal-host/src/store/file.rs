//! A versioned JSON document with locked, atomic updates.

use std::fs::{self, File, TryLockError};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::Serialize;
use serde::de::DeserializeOwned;

use super::private_file;

/// How long an update waits for another process's lock before giving up:
/// a store must never make a signature wait for long.
const LOCK_WAIT: Duration = Duration::from_secs(2);
const LOCK_POLL: Duration = Duration::from_millis(10);

/// Why a store could not be read or written. Callers degrade (no
/// persistence) and log; a store failure never blocks signing.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("I/O error on {path}: {detail}")]
    Io { path: String, detail: String },
    #[error("{path} is not a valid store file: {detail}")]
    Corrupt { path: String, detail: String },
}

impl StoreError {
    /// The kind alone, safe for logs: the path names the user's profile
    /// folder and the detail can echo file contents.
    pub fn kind(&self) -> &'static str {
        match self {
            StoreError::Io { .. } => "I/O error",
            StoreError::Corrupt { .. } => "corrupt file",
        }
    }
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
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(T::default()),
            Err(error) => return Err(self.io_error(&error)),
        };
        match serde_json::from_slice(&bytes) {
            Ok(value) => Ok(value),
            Err(_) => {
                self.quarantine();
                Ok(T::default())
            }
        }
    }

    /// Locks (`File::lock` on `<name>.lock`), re-reads, applies `change`,
    /// writes `<name>.tmp` and renames it over the file.
    pub fn update<T, R>(&self, change: impl FnOnce(&mut T) -> R) -> Result<R, StoreError>
    where
        T: DeserializeOwned + Serialize + Default,
    {
        if let Some(parent) = self.path.parent() {
            private_file::create_dir(parent).map_err(|error| self.io_error(&error))?;
        }
        let _lock = self.lock()?;
        let mut value: T = self.read()?;
        let result = change(&mut value);
        self.write(&value)?;
        Ok(result)
    }

    /// Holds the exclusive lock until dropped. Another host process may hold
    /// it (two browsers), so waiting is bounded.
    fn lock(&self) -> Result<File, StoreError> {
        let file = private_file::open(&self.sibling("lock"), false)
            .map_err(|error| self.io_error(&error))?;
        let started = Instant::now();
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(file),
                Err(TryLockError::WouldBlock) if started.elapsed() < LOCK_WAIT => {
                    std::thread::sleep(LOCK_POLL);
                }
                Err(TryLockError::WouldBlock) => {
                    return Err(StoreError::Io {
                        path: self.display(),
                        detail: "timed out waiting for the file lock".to_owned(),
                    });
                }
                Err(TryLockError::Error(error)) => return Err(self.io_error(&error)),
            }
        }
    }

    /// Write-then-rename, so a reader never sees half a document and a crash
    /// leaves the previous one. The new file is owner-only.
    fn write<T: Serialize>(&self, value: &T) -> Result<(), StoreError> {
        let temporary = self.sibling("tmp");
        let bytes = serde_json::to_vec_pretty(value).map_err(|error| StoreError::Corrupt {
            path: self.display(),
            detail: error.to_string(),
        })?;
        let written = private_file::open(&temporary, true).and_then(|mut file| {
            file.write_all(&bytes)?;
            file.sync_all()
        });
        written
            .and_then(|()| fs::rename(&temporary, &self.path))
            .map_err(|error| self.io_error(&error))
    }

    /// Moves an unreadable file out of the way, keeping it for support.
    fn quarantine(&self) {
        log::warn!("a store file is not valid JSON; moving it aside");
        if let Err(error) = fs::rename(&self.path, self.sibling("corrupt")) {
            log::warn!(
                "could not move the corrupt store file aside: {}",
                error.kind()
            );
        }
    }

    /// `<file name>.<extension>` next to the file.
    fn sibling(&self, extension: &str) -> PathBuf {
        let mut name = self.path.file_name().unwrap_or_default().to_owned();
        name.push(".");
        name.push(extension);
        self.path.with_file_name(name)
    }

    fn display(&self) -> String {
        self.path.display().to_string()
    }

    fn io_error(&self, error: &io::Error) -> StoreError {
        StoreError::Io {
            path: self.display(),
            detail: error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests;
