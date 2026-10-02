//! macOS keychains, including CryptoTokenKit smart card tokens, signing
//! through `SecKeyCreateSignature`.
//!
//! Two sources, so each reports its own failures: `macos:keychain` for keys
//! in keychain files (an installed A1 `.p12`) and `macos:ctk` for keys on
//! tokens exposed by a CryptoTokenKit driver (A3 cards and USB tokens). Both
//! let the OS ask for the PIN or password.
//!
//! Calls are not serialized. Chromium puts every Security.framework call
//! behind one lock because the legacy keychain code is not thread-safe; the
//! app must do the same if it ever lists or signs from several threads.

mod algorithm;
mod errors;
mod identities;
mod sign;
mod token;

use std::collections::HashMap;

use probe_core::{Fingerprint, SourceKind};
use security_framework::identity::SecIdentity;

use identities::{Identity, Scope};

use super::{
    FoundKey, Keystore, KeystoreError, Opened, Options, PinPrompt, SignRequest, Signature,
};

/// Adds the keychain-file source and the CryptoTokenKit source to `opened`.
/// Opening cannot fail: the keychain is queried when listing.
pub fn open(_options: &Options, opened: &mut Opened) {
    for scope in [Scope::Keychains, Scope::Tokens] {
        opened.keystores.push(Box::new(MacKeystore::new(scope)));
    }
}

/// One scope of the macOS keychain.
struct MacKeystore {
    scope: Scope,
    /// Identities from the last listing, by [`FoundKey::locator`].
    identities: HashMap<String, SecIdentity>,
}

impl MacKeystore {
    fn new(scope: Scope) -> Self {
        Self {
            scope,
            identities: HashMap::new(),
        }
    }

    fn found_key(&self, item: &Identity) -> FoundKey {
        let (provider, hardware) = match (&item.token_id, self.scope) {
            (Some(id), _) => (token::driver(id).to_owned(), true),
            (None, Scope::Tokens) => ("cryptotokenkit".to_owned(), true),
            (None, Scope::Keychains) => ("keychain".to_owned(), false),
        };
        FoundKey {
            cert_der: item.cert_der.clone(),
            keystore: self.name(),
            kind: SourceKind::System,
            provider,
            hardware: Some(hardware),
            locator: locator(&item.cert_der),
            pin: PinPrompt::System,
        }
    }
}

impl Keystore for MacKeystore {
    fn name(&self) -> String {
        match self.scope {
            Scope::Keychains => "macos:keychain",
            Scope::Tokens => "macos:ctk",
        }
        .to_owned()
    }

    fn list(&mut self) -> Result<Vec<FoundKey>, KeystoreError> {
        let found = identities::find(self.scope)?;
        let mut unreadable = found.unreadable.into_iter();
        if let Some(first) = unreadable.next() {
            if found.identities.is_empty() {
                return Err(first);
            }
            // The Keystore trait has no warning channel. stderr reaches the
            // terminal; the app will need a diagnostics entry instead.
            eprintln!(
                "warning: {}: {} identities could not be read (first: {first})",
                self.name(),
                1 + unreadable.len()
            );
        }
        let keys: Vec<FoundKey> = found
            .identities
            .iter()
            .map(|item| self.found_key(item))
            .collect();
        self.identities = found
            .identities
            .into_iter()
            .map(|item| (locator(&item.cert_der), item.sec_identity))
            .collect();
        Ok(keys)
    }

    fn sign(
        &mut self,
        key: &FoundKey,
        request: &SignRequest<'_>,
    ) -> Result<Signature, KeystoreError> {
        if !self.identities.contains_key(&key.locator) {
            // A token inserted since the last listing, or a caller that never
            // listed: look again before giving up.
            self.list()?;
        }
        let identity = self
            .identities
            .get(&key.locator)
            .ok_or(KeystoreError::NotFound)?;
        sign::sign(identity, &key.cert_der, request)
    }
}

/// The certificate's SHA-256, which stays the same across runs, keychains and
/// token reinsertions, unlike keychain item references.
fn locator(cert_der: &[u8]) -> String {
    Fingerprint::of(cert_der).to_hex()
}
