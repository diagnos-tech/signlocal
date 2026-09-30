//! The host's single entry point to every key source.

use websign_core::Fingerprint;

use crate::inventory::Inventory;
use crate::{KeystoreError, Opened, Options, PinState, SignRequest, Signature, open_all};

/// One way to reach a certificate's key: its primary path (the OS when it
/// has the key) or one of the alternates (a PKCS#11 module that sees the same
/// certificate, `docs/ux.md` §5.11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyRef {
    pub fingerprint: Fingerprint,
    /// 0 = primary; n = the n-th alternate of the de-duplicated group.
    pub path: usize,
}

/// Every source, opened once per process, with a cached listing.
///
/// Opening loads PKCS#11 modules (`C_Initialize` can take a second), so it
/// happens on first use and never again; modules are never finalized. The
/// listing is cached until [`KeystoreHub::invalidate`], which the host calls
/// on PC/SC reader and card events and on "Scan again".
pub struct KeystoreHub {
    options: Options,
    /// The sources while no listing is cached; the listing owns them otherwise.
    opened: Option<Opened>,
    cache: Option<Inventory>,
    /// How many of `failures` come from opening, so relisting can drop the
    /// listing failures of the previous round and keep these.
    open_failures: usize,
}

impl std::fmt::Debug for KeystoreHub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeystoreHub")
            .field("options", &self.options)
            .field("opened", &(self.opened.is_some() || self.cache.is_some()))
            .field("cached", &self.cache.is_some())
            .finish()
    }
}

impl KeystoreHub {
    /// A hub that opens nothing until first used.
    pub fn new(options: Options) -> KeystoreHub {
        KeystoreHub {
            options,
            opened: None,
            cache: None,
            open_failures: 0,
        }
    }

    /// A hub over sources the caller opened itself: fakes in tests, or a
    /// chosen subset of sources.
    pub fn with_sources(opened: Opened) -> KeystoreHub {
        KeystoreHub {
            open_failures: opened.failures.len(),
            opened: Some(opened),
            ..KeystoreHub::new(Options::default())
        }
    }

    /// The current listing: cached, or listed now (opening sources first).
    pub fn inventory(&mut self) -> &Inventory {
        self.listed()
    }

    /// Drops the cached listing; the next [`Self::inventory`] lists again.
    /// Sources stay open (modules stay loaded, PKCS#11 tokens stay unlocked).
    pub fn invalidate(&mut self) {
        if let Some(inventory) = self.cache.take() {
            let mut opened = inventory.opened;
            opened.failures.truncate(self.open_failures);
            self.opened = Some(opened);
        }
    }

    /// Signs through `key`'s path. On `Native` errors from the primary path
    /// the caller may offer an alternate (`SPEC.md` §6); the hub never
    /// switches paths by itself.
    pub fn sign(
        &mut self,
        key: KeyRef,
        request: &SignRequest<'_>,
    ) -> Result<Signature, KeystoreError> {
        let Inventory { opened, entries } = self.listed();
        let entry = resolve(entries, key).ok_or(KeystoreError::NotFound)?;
        let keystore = opened
            .keystores
            .get_mut(entry.store)
            .ok_or(KeystoreError::NotFound)?;
        keystore.sign(&entry.key, request)
    }

    /// Issuer certificates for `key` (best effort, possibly empty).
    pub fn chain(&mut self, key: KeyRef) -> Vec<Vec<u8>> {
        let Inventory { opened, entries } = self.listed();
        let Some(entry) = resolve(entries, key) else {
            return Vec::new();
        };
        opened
            .keystores
            .get_mut(entry.store)
            .map(|keystore| keystore.chain(&entry.key))
            .unwrap_or_default()
    }

    /// PIN state of `key`'s token (`None` for OS-owned PINs).
    pub fn pin_state(&mut self, key: KeyRef) -> Option<PinState> {
        let Inventory { opened, entries } = self.listed();
        let entry = resolve(entries, key)?;
        opened.keystores.get_mut(entry.store)?.pin_state(&entry.key)
    }

    /// Forgets cached authentication in every source.
    pub fn end_sessions(&mut self) {
        let opened = match (&mut self.cache, &mut self.opened) {
            (Some(inventory), _) => &mut inventory.opened,
            (None, Some(opened)) => opened,
            (None, None) => return,
        };
        for keystore in &mut opened.keystores {
            keystore.end_sessions();
        }
    }

    fn listed(&mut self) -> &mut Inventory {
        let (options, opened, open_failures) =
            (&self.options, &mut self.opened, &mut self.open_failures);
        self.cache.get_or_insert_with(|| {
            let opened = opened.take().unwrap_or_else(|| {
                let opened = open_all(options);
                *open_failures = opened.failures.len();
                opened
            });
            Inventory::list(opened)
        })
    }
}

/// The entry `key` names: its group's primary for path 0, the n-th
/// alternate for path n.
fn resolve(entries: &[crate::inventory::Entry], key: KeyRef) -> Option<&crate::inventory::Entry> {
    let groups = crate::inventory::group_entries(entries);
    let group = groups.into_iter().find(|group| {
        entries
            .get(group.primary)
            .is_some_and(|entry| Fingerprint::of(&entry.key.cert_der) == key.fingerprint)
    })?;
    let index = match key.path {
        0 => group.primary,
        n => *group.alternates.get(n - 1)?,
    };
    entries.get(index)
}

#[cfg(test)]
mod tests;
