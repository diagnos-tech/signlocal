//! One loaded PKCS#11 module as a [`Keystore`].

use std::path::PathBuf;

use cryptoki::context::Pkcs11;

use super::capabilities::SlotCapabilities;
use super::locator::Locator;
use super::sessions::Sessions;
use super::signing::{self, Module};
use super::{chain, finder, listing, objects, pin_state};
use crate::{FoundKey, KeyCapabilities, Keystore, KeystoreError, PinState, SignRequest, Signature};
use log::trace;

/// A module that is loaded and initialized; its tokens are read on demand.
/// Tokens unlocked by a signature stay unlocked in `sessions` until
/// [`Keystore::end_sessions`] or until this keystore is dropped.
pub struct Pkcs11Keystore {
    // First, so parked sessions log out before anything else is dropped.
    sessions: Sessions,
    capabilities: SlotCapabilities,
    pkcs11: Pkcs11,
    module_file: String,
    path: PathBuf,
}

impl std::fmt::Debug for Pkcs11Keystore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pkcs11Keystore")
            .field("module_file", &self.module_file)
            .finish_non_exhaustive()
    }
}

impl Pkcs11Keystore {
    pub fn new(pkcs11: Pkcs11, module_file: String, path: PathBuf) -> Self {
        Self {
            sessions: Sessions::default(),
            capabilities: SlotCapabilities::default(),
            pkcs11,
            module_file,
            path,
        }
    }

    fn stored_certificates(&self, key: &FoundKey) -> Result<Vec<Vec<u8>>, KeystoreError> {
        let locator = Locator::parse(&key.locator).ok_or(KeystoreError::NotFound)?;
        let located = finder::find_certificate(&self.pkcs11, &locator, &key.cert_der)?;
        let stored = objects::certificates(&located.session)
            .map_err(|error| KeystoreError::Other(error.to_string()))?;
        Ok(stored
            .into_iter()
            .map(|certificate| certificate.der)
            .collect())
    }
}

impl Keystore for Pkcs11Keystore {
    fn name(&self) -> String {
        format!("pkcs11:{}", self.module_file)
    }

    fn list(&mut self) -> Result<Vec<FoundKey>, KeystoreError> {
        self.capabilities.clear();
        listing::list(&self.pkcs11, &self.name(), &self.module_file)
    }

    fn sign(
        &mut self,
        key: &FoundKey,
        request: &SignRequest<'_>,
    ) -> Result<Signature, KeystoreError> {
        let module = Module {
            pkcs11: &self.pkcs11,
            path: &self.path,
        };
        signing::sign(module, &mut self.sessions, key, request)
    }

    fn chain(&mut self, key: &FoundKey) -> Vec<Vec<u8>> {
        match self.stored_certificates(key) {
            Ok(stored) => chain::issuers(&key.cert_der, &stored),
            Err(error) => {
                trace!("{}: no chain from the token: {error}", self.module_file);
                Vec::new()
            }
        }
    }

    fn capabilities(&mut self, key: &FoundKey) -> KeyCapabilities {
        self.capabilities.of(&self.pkcs11, key)
    }

    fn pin_state(&mut self, key: &FoundKey) -> Option<PinState> {
        pin_state::read(&self.pkcs11, &mut self.sessions, key)
            .inspect_err(|error| trace!("{}: no PIN state: {error}", self.module_file))
            .ok()
    }

    fn end_sessions(&mut self) {
        self.sessions.end_all();
    }
}
