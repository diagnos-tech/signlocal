//! One loaded PKCS#11 module as a [`Keystore`].

use cryptoki::context::Pkcs11;

use super::{listing, signing};
use crate::{FoundKey, Keystore, KeystoreError, SignRequest, Signature};

/// A module that is loaded and initialized; its tokens are read on demand.
pub struct Pkcs11Keystore {
    pkcs11: Pkcs11,
    module_file: String,
    path: std::path::PathBuf,
}

impl Pkcs11Keystore {
    pub fn new(pkcs11: Pkcs11, module_file: String, path: std::path::PathBuf) -> Self {
        Self {
            pkcs11,
            module_file,
            path,
        }
    }
}

impl Keystore for Pkcs11Keystore {
    fn name(&self) -> String {
        format!("pkcs11:{}", self.module_file)
    }

    fn list(&mut self) -> Result<Vec<FoundKey>, KeystoreError> {
        listing::list(&self.pkcs11, &self.name(), &self.module_file)
    }

    fn sign(
        &mut self,
        key: &FoundKey,
        request: &SignRequest<'_>,
    ) -> Result<Signature, KeystoreError> {
        signing::sign(&self.pkcs11, &self.path, key, request)
    }
}
