//! Windows certificate store (`CurrentUser\MY`), signing through CNG (NCrypt)
//! or legacy CAPI, whichever the key's provider speaks.
//!
//! Listing reads certificate properties and provider metadata only, so it
//! never touches a card and never prompts. Signing opens each key once and
//! keeps it for the life of the keystore: smart card middlewares tie their
//! PIN cache to the open key, so a session asks for the PIN once.

mod acquire;
mod capi;
mod errors;
mod handles;
mod hardware;
mod key_info;
mod ncrypt;
mod store;
mod thumbprint;
mod window;

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::time::Instant;

use probe_core::SourceKind;
use windows::Win32::Foundation::HWND;

use super::{
    FoundKey, Keystore, KeystoreError, NcryptPreference, Opened, Options, PinPrompt, SignRequest,
    Signature, SourceFailure,
};
use acquire::AcquiredKey;
use hardware::HardwareProbe;
use key_info::KeyLocation;
use store::CertStore;
use thumbprint::Thumbprint;

const NAME: &str = "windows";

/// Room for a 16384-bit RSA signature, the largest key CNG and CAPI create.
const MAX_SIGNATURE_LEN: usize = 2048;

/// Adds the Windows store to `opened`.
pub fn open(options: &Options, opened: &mut Opened) {
    match CertStore::open_current_user_my() {
        Ok(store) => opened.keystores.push(Box::new(WindowsKeystore {
            keys: HashMap::new(),
            store,
            preference: options.ncrypt,
        })),
        Err(error) => opened.failures.push(SourceFailure {
            source: NAME.to_owned(),
            error: error.to_string(),
        }),
    }
}

/// `CurrentUser\MY`, plus every key opened so far by locator.
struct WindowsKeystore {
    // Declared first so keys are released before the store is closed.
    keys: HashMap<String, AcquiredKey>,
    store: CertStore,
    preference: NcryptPreference,
}

impl Keystore for WindowsKeystore {
    fn name(&self) -> String {
        NAME.to_owned()
    }

    fn list(&mut self) -> Result<Vec<FoundKey>, KeystoreError> {
        let mut hardware = HardwareProbe::default();
        let mut found = Vec::new();
        for cert in self.store.certificates() {
            let (Some(location), Some(thumbprint)) = (KeyLocation::of(&cert), cert.thumbprint())
            else {
                continue;
            };
            found.push(FoundKey {
                cert_der: cert.der().to_vec(),
                keystore: NAME.to_owned(),
                kind: SourceKind::System,
                provider: location.describe(),
                hardware: hardware.is_hardware(&location),
                locator: thumbprint.to_hex(),
                pin: PinPrompt::System,
            });
        }
        Ok(found)
    }

    fn sign(
        &mut self,
        key: &FoundKey,
        request: &SignRequest<'_>,
    ) -> Result<Signature, KeystoreError> {
        let started = Instant::now();
        let owner = window::owner(request.parent_window);
        let result = self
            .opened_key(&key.locator, owner)
            .and_then(|opened| opened.sign(request, owner));
        if matches!(result, Err(KeystoreError::Native { .. })) {
            // A removed card or a restarted middleware leaves the handle
            // dead; the next signature opens the key again.
            self.keys.remove(&key.locator);
        }
        let (bytes, api) = result?;
        Ok(Signature {
            bytes,
            api,
            elapsed: started.elapsed(),
        })
    }
}

impl WindowsKeystore {
    fn opened_key(
        &mut self,
        locator: &str,
        owner: Option<HWND>,
    ) -> Result<&AcquiredKey, KeystoreError> {
        match self.keys.entry(locator.to_owned()) {
            Entry::Occupied(entry) => Ok(entry.into_mut()),
            Entry::Vacant(entry) => {
                let thumbprint = Thumbprint::from_hex(locator).ok_or(KeystoreError::NotFound)?;
                let cert = self
                    .store
                    .find(&thumbprint)
                    .ok_or(KeystoreError::NotFound)?;
                let key = AcquiredKey::open(cert, self.preference, owner)?;
                Ok(entry.insert(key))
            }
        }
    }
}
