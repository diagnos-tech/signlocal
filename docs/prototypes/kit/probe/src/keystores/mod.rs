//! Key sources: every place a signing certificate can come from.
//!
//! Each source implements [`Keystore`]. Sources never interpret certificates
//! (that is `probe-core`'s job) and never ask for a PIN themselves: OS keys
//! let the OS prompt, PKCS#11 keys receive the PIN in the [`SignRequest`].

pub mod inventory;
mod model;
pub mod pkcs11;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(windows)]
pub mod windows;

use crate::trace::trace;
#[cfg(windows)]
pub use model::NcryptPreference;

pub use model::{FoundKey, KeystoreError, Options, PinPrompt, SignRequest, Signature};

/// A source of signing keys.
pub trait Keystore {
    /// Short, stable name for output, e.g. `"windows"` or `"pkcs11:libsofthsm2.so"`.
    fn name(&self) -> String;

    /// Certificates that have a usable private key. Must not prompt for a PIN.
    fn list(&mut self) -> Result<Vec<FoundKey>, KeystoreError>;

    /// Signs `request.digest` with the key behind `key`, returning the
    /// signature in its final format (RSA block, or ECDSA raw `r || s`).
    fn sign(
        &mut self,
        key: &FoundKey,
        request: &SignRequest<'_>,
    ) -> Result<Signature, KeystoreError>;
}

/// A source that could not be opened, reported instead of aborting the run.
#[derive(Debug)]
pub struct SourceFailure {
    pub source: String,
    pub error: String,
}

/// Every key source available on this machine.
#[derive(Default)]
pub struct Opened {
    pub keystores: Vec<Box<dyn Keystore>>,
    pub failures: Vec<SourceFailure>,
}

/// Opens the OS key store first, then every PKCS#11 module, so that
/// de-duplication sees OS entries first.
pub fn open_all(options: &Options) -> Opened {
    let mut opened = Opened::default();
    #[cfg(windows)]
    {
        trace!("opening source: Windows certificate store");
        windows::open(options, &mut opened);
    }
    #[cfg(target_os = "macos")]
    {
        trace!("opening source: macOS keychain");
        macos::open(options, &mut opened);
    }
    trace!("opening source: PKCS#11 modules");
    pkcs11::open(options, &mut opened);
    trace!(
        "sources opened: {} ({} failed)",
        opened.keystores.len(),
        opened.failures.len()
    );
    opened
}
