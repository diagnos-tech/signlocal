//! The Windows store as a [`Keystore`]: listing, signing with the D9
//! fallback, chains, and the per-session key cache.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::time::Instant;

use windows::Win32::Foundation::HWND;

use super::acquire::AcquiredKey;
use super::cert_context::CertContext;
use super::fallback::{self, BridgeRefusals};
use super::key_info::KeyLocation;
use super::store::CertStore;
use super::thumbprint::Thumbprint;
use super::{NAME, capabilities, chain, listing, request, window};
use crate::{
    FoundKey, KeyCapabilities, Keystore, KeystoreError, NcryptPreference, Options, SignRequest,
    Signature,
};
use log::trace;

/// `CurrentUser\MY`, plus every key opened so far by locator.
///
/// Each key is opened once and kept until [`Keystore::end_sessions`]: smart
/// card middlewares tie their PIN cache to the open key, so a session asks
/// for the PIN once.
#[derive(Debug)]
pub struct WindowsKeystore {
    // Declared first so keys are released before the store is closed.
    keys: HashMap<String, AcquiredKey>,
    store: CertStore,
    preference: NcryptPreference,
    refusals: BridgeRefusals,
    /// Open and sign with `*_SILENT` flags: fail rather than show UI.
    silent: bool,
}

impl WindowsKeystore {
    pub fn new(store: CertStore, options: &Options) -> Self {
        Self {
            keys: HashMap::new(),
            store,
            preference: options.ncrypt,
            refusals: BridgeRefusals::default(),
            silent: options.silent,
        }
    }

    /// One attempt with `preference`, reusing the key if already open.
    fn sign_with(
        &mut self,
        locator: &str,
        preference: NcryptPreference,
        request: &SignRequest<'_>,
        owner: Option<HWND>,
    ) -> Result<(Vec<u8>, &'static str), KeystoreError> {
        let silent = self.silent;
        let key = match self.keys.entry(locator.to_owned()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let cert = find(&self.store, locator)?;
                entry.insert(AcquiredKey::open(cert, preference, owner, silent)?)
            }
        };
        key.sign(request, owner)
    }

    /// Whether the key behind `locator` lives in a legacy CSP.
    fn key_in_csp(&self, locator: &str) -> bool {
        find(&self.store, locator)
            .ok()
            .and_then(|cert| KeyLocation::of(&cert))
            .is_some_and(|location| !location.is_cng())
    }
}

impl Keystore for WindowsKeystore {
    fn name(&self) -> String {
        NAME.to_owned()
    }

    fn list(&mut self) -> Result<Vec<FoundKey>, KeystoreError> {
        Ok(listing::keys(&self.store))
    }

    fn sign(
        &mut self,
        key: &FoundKey,
        request: &SignRequest<'_>,
    ) -> Result<Signature, KeystoreError> {
        request::check(key, request)?;
        // Before any cached handle is reused: the certificate behind the
        // locator must still be the one the caller chose.
        find_same(&self.store, key)?;
        let started = Instant::now();
        // A silent run shows no dialog, so it needs no owner window.
        let owner = if self.silent {
            None
        } else {
            window::owner(request.parent_window)
        };
        let locator = key.locator.as_str();
        let preference = self.refusals.preference(locator, self.preference);
        let mut result = self.sign_with(locator, preference, request, owner);
        if let Err(error) = &result
            && fallback::should_retry_with_capi(preference, self.key_in_csp(locator), error)
        {
            trace!("the key's CSP refused the CNG bridge; retrying with plain CAPI");
            self.keys.remove(locator);
            self.refusals.remember(locator);
            result = self.sign_with(locator, NcryptPreference::Allow, request, owner);
        }
        if matches!(
            result,
            Err(KeystoreError::Native { .. } | KeystoreError::TokenRemoved)
        ) {
            // A removed card or a restarted middleware leaves the handle
            // dead; the next signature opens the key again.
            self.keys.remove(locator);
        }
        let (bytes, api) = result?;
        Ok(Signature {
            bytes,
            api,
            elapsed: started.elapsed(),
        })
    }

    /// From the key's provider and the preference its next signature will
    /// use, so a key that fell back to CAPI stops offering PSS.
    fn capabilities(&mut self, key: &FoundKey) -> KeyCapabilities {
        let Some(location) = find_same(&self.store, key)
            .ok()
            .and_then(|cert| KeyLocation::of(&cert))
        else {
            return KeyCapabilities::NONE;
        };
        let preference = self.refusals.preference(&key.locator, self.preference);
        capabilities::for_key(&location, preference)
    }

    fn chain(&mut self, key: &FoundKey) -> Vec<Vec<u8>> {
        find_same(&self.store, key)
            .map(|cert| chain::issuers(&cert))
            .unwrap_or_default()
    }

    fn end_sessions(&mut self) {
        // Closing the handles is what makes the middleware forget the PIN.
        self.keys.clear();
    }
}

/// The certificate behind `key`'s locator, only if its bytes are exactly
/// `key.cert_der`. The SHA-1 thumbprint only finds the certificate; it is
/// never trusted as its identity, so a replaced or altered certificate is
/// `NotFound` rather than signed for with another key.
fn find_same(store: &CertStore, key: &FoundKey) -> Result<CertContext, KeystoreError> {
    let cert = find(store, &key.locator)?;
    if cert.der() == key.cert_der.as_slice() {
        Ok(cert)
    } else {
        Err(KeystoreError::NotFound)
    }
}

/// The certificate behind a locator, if it is still installed.
fn find(store: &CertStore, locator: &str) -> Result<CertContext, KeystoreError> {
    let thumbprint = Thumbprint::from_hex(locator).ok_or(KeystoreError::NotFound)?;
    store.find(&thumbprint).ok_or(KeystoreError::NotFound)
}
