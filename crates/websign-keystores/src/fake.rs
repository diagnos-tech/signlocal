//! A scripted [`Keystore`] for unit tests of the code above the adapters.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use websign_core::SourceKind;

use crate::{FoundKey, Keystore, KeystoreError, PinPrompt, PinState, SignRequest, Signature};

/// Lists `keys`, signs by echoing the key's locator, and counts calls.
#[derive(Debug, Default)]
pub struct FakeKeystore {
    pub name: String,
    pub keys: Vec<FoundKey>,
    pub fail_list: bool,
    pub lists: Rc<Cell<usize>>,
    pub ended: Rc<Cell<usize>>,
}

impl FakeKeystore {
    pub fn new(name: &str, keys: Vec<FoundKey>) -> FakeKeystore {
        FakeKeystore {
            name: name.to_owned(),
            keys,
            ..FakeKeystore::default()
        }
    }
}

/// A key whose "certificate" is `cert` (never parsed by the code under test).
pub fn key(cert: &[u8], kind: SourceKind, locator: &str) -> FoundKey {
    FoundKey {
        cert_der: cert.to_vec(),
        keystore: String::new(),
        kind,
        provider: "fake provider".to_owned(),
        hardware: None,
        locator: locator.to_owned(),
        pin: PinPrompt::App {
            protected_path: false,
        },
        device: None,
    }
}

impl Keystore for FakeKeystore {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn list(&mut self) -> Result<Vec<FoundKey>, KeystoreError> {
        self.lists.set(self.lists.get() + 1);
        if self.fail_list {
            return Err(KeystoreError::Other("listing failed".to_owned()));
        }
        Ok(self.keys.clone())
    }

    fn sign(
        &mut self,
        key: &FoundKey,
        _request: &SignRequest<'_>,
    ) -> Result<Signature, KeystoreError> {
        Ok(Signature {
            bytes: key.locator.clone().into_bytes(),
            api: "fake",
            elapsed: Duration::ZERO,
        })
    }

    fn chain(&mut self, key: &FoundKey) -> Vec<Vec<u8>> {
        vec![key.locator.clone().into_bytes()]
    }

    fn pin_state(&mut self, _key: &FoundKey) -> Option<PinState> {
        Some(PinState {
            unlocked: self.ended.get() == 0,
            ..PinState::default()
        })
    }

    fn end_sessions(&mut self) {
        self.ended.set(self.ended.get() + 1);
    }
}
