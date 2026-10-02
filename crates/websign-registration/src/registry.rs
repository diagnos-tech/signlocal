//! The Windows registry behind a trait, so the Windows registration logic
//! (native messaging keys, the URL scheme, extension pre-registration,
//! browser detection) runs and is tested on every OS against
//! [`MemoryRegistry`].
//!
//! Writes can only target `HKCU`: the app never needs elevation, and a type
//! that cannot express `HKLM` writes keeps it that way.

use std::io;

mod memory;
#[cfg(windows)]
mod system;

pub use memory::MemoryRegistry;

/// A root key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Hive {
    CurrentUser,
    LocalMachine,
}

impl Hive {
    /// The prefix Windows tools print (`HKCU`, `HKLM`).
    pub const fn short_name(self) -> &'static str {
        match self {
            Self::CurrentUser => "HKCU",
            Self::LocalMachine => "HKLM",
        }
    }
}

/// The operations registration needs. Subkeys use `\` separators and are
/// case-insensitive, as in Windows; the value name `""` is the key's default
/// value.
pub trait Registry {
    /// A string value; `Ok(None)` when the key or the value does not exist.
    fn get_string(&self, hive: Hive, subkey: &str, name: &str) -> io::Result<Option<String>>;

    /// Names of the direct children of `subkey`; empty when it does not exist.
    fn subkey_names(&self, hive: Hive, subkey: &str) -> io::Result<Vec<String>>;

    fn key_exists(&self, hive: Hive, subkey: &str) -> bool;

    /// Creates `HKCU\<subkey>` (and its parents) and sets a string value.
    fn set_string(&self, subkey: &str, name: &str, value: &str) -> io::Result<()>;

    /// Removes `HKCU\<subkey>` and everything under it; absent is not an error.
    fn remove_tree(&self, subkey: &str) -> io::Result<()>;
}

/// This machine's registry: the real one on Windows; elsewhere one that is
/// always empty and refuses writes.
pub fn system() -> Box<dyn Registry> {
    #[cfg(windows)]
    return Box::new(system::WindowsRegistry);
    #[cfg(not(windows))]
    Box::new(Absent)
}

/// Stands in for the registry where there is none.
#[cfg(not(windows))]
#[derive(Debug)]
struct Absent;

#[cfg(not(windows))]
impl Registry for Absent {
    fn get_string(&self, _: Hive, _: &str, _: &str) -> io::Result<Option<String>> {
        Ok(None)
    }

    fn subkey_names(&self, _: Hive, _: &str) -> io::Result<Vec<String>> {
        Ok(Vec::new())
    }

    fn key_exists(&self, _: Hive, _: &str) -> bool {
        false
    }

    fn set_string(&self, _: &str, _: &str, _: &str) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "the registry only exists on Windows",
        ))
    }

    fn remove_tree(&self, _: &str) -> io::Result<()> {
        Ok(())
    }
}
