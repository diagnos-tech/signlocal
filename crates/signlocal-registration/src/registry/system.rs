//! The real Windows registry, through `windows-registry`.

use std::io;

use windows_registry::{CURRENT_USER, Key, LOCAL_MACHINE};

use super::{Hive, Registry};

#[derive(Debug)]
pub struct WindowsRegistry;

fn root(hive: Hive) -> &'static Key {
    match hive {
        Hive::CurrentUser => CURRENT_USER,
        Hive::LocalMachine => LOCAL_MACHINE,
    }
}

impl Registry for WindowsRegistry {
    fn get_string(&self, hive: Hive, subkey: &str, name: &str) -> io::Result<Option<String>> {
        let Ok(key) = root(hive).open(subkey) else {
            return Ok(None);
        };
        // A missing value and a non-string one read the same to callers:
        // neither names a manifest or an executable.
        Ok(key.get_string(name).ok())
    }

    fn subkey_names(&self, hive: Hive, subkey: &str) -> io::Result<Vec<String>> {
        let Ok(key) = root(hive).open(subkey) else {
            return Ok(Vec::new());
        };
        Ok(key.keys()?.collect())
    }

    fn key_exists(&self, hive: Hive, subkey: &str) -> bool {
        root(hive).open(subkey).is_ok()
    }

    fn set_string(&self, subkey: &str, name: &str, value: &str) -> io::Result<()> {
        CURRENT_USER.create(subkey)?.set_string(name, value)?;
        Ok(())
    }

    fn remove_tree(&self, subkey: &str) -> io::Result<()> {
        if CURRENT_USER.open(subkey).is_err() {
            return Ok(());
        }
        CURRENT_USER.remove_tree(subkey)?;
        Ok(())
    }
}
