//! An in-memory registry for tests of the Windows logic on any OS.

use std::collections::BTreeMap;
use std::io;
use std::sync::{Mutex, MutexGuard, PoisonError};

use super::{Hive, Registry};

/// Values by `(hive, lowercase subkey)`, then by lowercase value name. Keys
/// keep their first spelling for [`Registry::subkey_names`].
type Keys = BTreeMap<(Hive, String), Entry>;

#[derive(Debug, Default)]
struct Entry {
    spelling: String,
    values: BTreeMap<String, String>,
}

/// A registry held in memory. [`MemoryRegistry::seed`] fills `HKLM` too, for
/// tests that read what an installer would have written there.
#[derive(Debug, Default)]
pub struct MemoryRegistry {
    keys: Mutex<Keys>,
}

impl MemoryRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a string value in any hive, creating the key and its parents.
    pub fn seed(&self, hive: Hive, subkey: &str, name: &str, value: &str) {
        let mut keys = self.lock();
        create(&mut keys, hive, subkey)
            .values
            .insert(name.to_lowercase(), value.to_owned());
    }

    /// Every `HKCU` key, lowercase, for assertions.
    pub fn current_user_keys(&self) -> Vec<String> {
        self.lock()
            .keys()
            .filter(|(hive, _)| *hive == Hive::CurrentUser)
            .map(|(_, key)| key.clone())
            .collect()
    }

    fn lock(&self) -> MutexGuard<'_, Keys> {
        self.keys.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

fn normalize(subkey: &str) -> String {
    subkey.trim_matches('\\').to_lowercase()
}

fn create<'a>(keys: &'a mut Keys, hive: Hive, subkey: &str) -> &'a mut Entry {
    let trimmed = subkey.trim_matches('\\');
    let mut end = 0;
    for part in trimmed.split('\\') {
        end += part.len();
        let spelling = &trimmed[..end];
        keys.entry((hive, spelling.to_lowercase()))
            .or_insert_with(|| Entry {
                spelling: spelling.to_owned(),
                values: BTreeMap::new(),
            });
        end += 1;
    }
    keys.entry((hive, normalize(subkey))).or_default()
}

impl Registry for MemoryRegistry {
    fn get_string(&self, hive: Hive, subkey: &str, name: &str) -> io::Result<Option<String>> {
        Ok(self
            .lock()
            .get(&(hive, normalize(subkey)))
            .and_then(|entry| entry.values.get(&name.to_lowercase()).cloned()))
    }

    fn subkey_names(&self, hive: Hive, subkey: &str) -> io::Result<Vec<String>> {
        let prefix = format!("{}\\", normalize(subkey));
        Ok(self
            .lock()
            .iter()
            .filter(|((h, key), _)| *h == hive && key.starts_with(&prefix))
            .filter(|((_, key), _)| !key[prefix.len()..].contains('\\'))
            .filter_map(|(_, entry)| entry.spelling.rsplit('\\').next().map(str::to_owned))
            .collect())
    }

    fn key_exists(&self, hive: Hive, subkey: &str) -> bool {
        self.lock().contains_key(&(hive, normalize(subkey)))
    }

    fn set_string(&self, subkey: &str, name: &str, value: &str) -> io::Result<()> {
        self.seed(Hive::CurrentUser, subkey, name, value);
        Ok(())
    }

    fn remove_tree(&self, subkey: &str) -> io::Result<()> {
        let key = normalize(subkey);
        let prefix = format!("{key}\\");
        self.lock().retain(|(hive, existing), _| {
            *hive != Hive::CurrentUser || (*existing != key && !existing.starts_with(&prefix))
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_case_insensitive_and_create_their_parents() {
        let registry = MemoryRegistry::new();
        registry
            .set_string(r"Software\Vendor\App", "", "x")
            .unwrap();
        assert_eq!(
            registry
                .get_string(Hive::CurrentUser, r"software\VENDOR\app", "")
                .unwrap(),
            Some("x".into())
        );
        assert!(registry.key_exists(Hive::CurrentUser, r"Software\Vendor"));
        assert!(!registry.key_exists(Hive::LocalMachine, r"Software\Vendor"));
        assert_eq!(
            registry
                .subkey_names(Hive::CurrentUser, "Software")
                .unwrap(),
            ["Vendor"]
        );
    }

    #[test]
    fn remove_tree_takes_children_and_spares_siblings_and_other_hives() {
        let registry = MemoryRegistry::new();
        registry.set_string(r"A\B\C", "v", "1").unwrap();
        registry.set_string(r"A\BB", "v", "2").unwrap();
        registry.seed(Hive::LocalMachine, r"A\B", "v", "3");
        registry.remove_tree(r"a\b").unwrap();
        assert!(!registry.key_exists(Hive::CurrentUser, r"A\B\C"));
        assert!(registry.key_exists(Hive::CurrentUser, r"A\BB"));
        assert!(registry.key_exists(Hive::LocalMachine, r"A\B"));
        registry.remove_tree(r"A\missing").unwrap();
    }
}
