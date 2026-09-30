//! The host's single entry point to every key source.

use websign_core::Fingerprint;

use crate::inventory::Inventory;
use crate::{KeystoreError, Opened, Options, PinState, SignRequest, Signature};

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
    opened: Option<Opened>,
    cache: Option<Inventory>,
}

impl std::fmt::Debug for KeystoreHub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeystoreHub")
            .field("options", &self.options)
            .field("opened", &self.opened.is_some())
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
        }
    }

    /// The current listing: cached, or listed now (opening sources first).
    pub fn inventory(&mut self) -> &Inventory {
        todo!("SPEC.md §6")
    }

    /// Drops the cached listing; the next [`Self::inventory`] lists again.
    /// Sources stay open (modules stay loaded).
    pub fn invalidate(&mut self) {
        self.cache = None;
    }

    /// Signs through `key`'s path. On `Native` errors from the primary path
    /// the caller may offer an alternate (`SPEC.md` §6.3); the hub never
    /// switches paths by itself.
    pub fn sign(
        &mut self,
        key: KeyRef,
        request: &SignRequest<'_>,
    ) -> Result<Signature, KeystoreError> {
        let _ = (key, request);
        todo!("SPEC.md §6")
    }

    /// Issuer certificates for `key` (best effort, possibly empty).
    pub fn chain(&mut self, key: KeyRef) -> Vec<Vec<u8>> {
        let _ = key;
        todo!("SPEC.md §6")
    }

    /// PIN state of `key`'s token (`None` for OS-owned PINs).
    pub fn pin_state(&mut self, key: KeyRef) -> Option<PinState> {
        let _ = key;
        todo!("SPEC.md §6")
    }

    /// Forgets cached authentication in every source.
    pub fn end_sessions(&mut self) {
        todo!("SPEC.md §6")
    }
}
