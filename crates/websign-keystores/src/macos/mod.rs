//! macOS keychains, including CryptoTokenKit smart card tokens, signing
//! through `SecKeyCreateSignature`.
//!
//! Two sources, so each reports its own failures: `macos:keychain` for keys
//! in keychain files (an installed A1 `.p12`) and `macos:ctk` for keys on
//! tokens exposed by a CryptoTokenKit driver (A3 cards and USB tokens). Both
//! let the OS ask for the PIN or password.
//!
//! Every Security.framework call runs under one process-wide lock
//! ([`serial`]), taken here and only here so it is never nested.

mod algorithm;
mod chain;
mod errors;
mod identities;
mod serial;
mod sign;
mod status;
mod token;
mod user_info;

use std::collections::HashMap;

use security_framework::identity::SecIdentity;
use websign_core::{Fingerprint, SourceKind};

use identities::{Identity, Scope};
use serial::serialized;

use super::{
    DeviceLink, FoundKey, Keystore, KeystoreError, Opened, Options, PinPrompt, SignRequest,
    Signature,
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
        let driver = item.token_id.as_deref().map(token::driver);
        let (provider, hardware) = match (driver, self.scope) {
            (Some(driver), _) => (driver.to_owned(), true),
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
            // Only the driver: the instance part of a token ID is often
            // the card's serial number.
            device: driver.map(|driver| DeviceLink::CryptoTokenKit {
                driver: driver.to_owned(),
            }),
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
        let found = serialized(|| identities::find(self.scope))?;
        let mut unreadable = found.unreadable.into_iter();
        if let Some(first) = unreadable.next() {
            if found.identities.is_empty() {
                return Err(first);
            }
            // TODO(gustavo): surface as a diagnostics entry (`Keystore::warnings`).
            log::warn!(
                "{}: {} identities could not be read (first: {first})",
                self.name(),
                1 + unreadable.len()
            );
        }
        let mut keys = Vec::with_capacity(found.identities.len());
        self.identities.clear();
        for item in found.identities {
            // The same certificate may sit in two keychains; list it once.
            let locator = locator(&item.cert_der);
            if !self.identities.contains_key(&locator) {
                keys.push(self.found_key(&item));
                self.identities.insert(locator, item.sec_identity);
            }
        }
        Ok(keys)
    }

    fn sign(
        &mut self,
        key: &FoundKey,
        request: &SignRequest<'_>,
    ) -> Result<Signature, KeystoreError> {
        // The locator is the certificate's fingerprint, so it is recomputed
        // from the certificate the caller holds instead of trusted: a key
        // listed for other bytes is not this certificate's key.
        let wanted = locator(&key.cert_der);
        if key.locator != wanted {
            return Err(KeystoreError::NotFound);
        }
        if !self.identities.contains_key(&wanted) {
            // A token inserted since the last listing, or a caller that never
            // listed: look again before giving up.
            self.list()?;
        }
        let identity = self
            .identities
            .get(&wanted)
            .ok_or(KeystoreError::NotFound)?;
        serialized(|| sign::sign(identity, &key.cert_der, request))
    }

    fn chain(&mut self, key: &FoundKey) -> Vec<Vec<u8>> {
        serialized(|| chain::issuers(&key.cert_der))
    }

    /// Drops the cached identities and with them every `SecKey` reference,
    /// so the next signature opens a fresh token session and the driver
    /// decides again whether to ask for the PIN.
    fn end_sessions(&mut self) {
        self.identities.clear();
    }
}

/// The certificate's SHA-256, which stays the same across runs, keychains and
/// token reinsertions, unlike keychain item references.
fn locator(cert_der: &[u8]) -> String {
    Fingerprint::of(cert_der).to_hex()
}
