//! Key sources: every place a signing certificate can come from.
//!
//! Each source implements [`Keystore`]: the Windows certificate store
//! (CNG and legacy CAPI), the macOS keychain (keychain files and
//! CryptoTokenKit tokens), and one keystore per loaded PKCS#11 module.
//! Sources never interpret certificates (that is `websign-core`'s job) and
//! never ask for a PIN themselves: OS keys let the OS prompt, PKCS#11 keys
//! receive the PIN in the [`SignRequest`].
//!
//! [`KeystoreHub`] is what the host uses: it opens every source once per
//! process, caches the de-duplicated listing until a device event invalidates
//! it, and routes a signature to the right source.
//!
//! The adapters, [`inventory`] and the model are promoted from the Phase-0
//! kit (`probe/src/keystores`), where CI proved them against software keys
//! on all three systems. `SPEC.md` holds the contract and the contract test
//! suite every adapter must pass.

#![cfg_attr(
    not(test),
    warn(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

mod capabilities;
pub mod contract;
#[cfg(test)]
mod fake;
mod hub;
pub mod inventory;
mod model;
pub mod pkcs11;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(windows)]
pub mod windows;

use log::trace;

pub use capabilities::KeyCapabilities;
pub use hub::{KeyRef, KeystoreHub};
pub use model::{
    DeviceLink, FoundKey, KeystoreError, NcryptPreference, Options, PinPrompt, PinState,
    SignRequest, Signature,
};

/// A source of signing keys.
///
/// Implementations are confined to one thread (the host's key store worker):
/// Security.framework's legacy keychain code is not thread-safe, and PKCS#11
/// sessions are simplest when one thread owns them. Nothing here is `Send`.
pub trait Keystore {
    /// Short, stable name for output, e.g. `"windows"` or `"pkcs11:libsofthsm2.so"`.
    fn name(&self) -> String;

    /// Certificates that have a usable private key. Must not prompt for a PIN,
    /// must not touch a card beyond reading public objects.
    fn list(&mut self) -> Result<Vec<FoundKey>, KeystoreError>;

    /// Signs `request.digest` with the key behind `key`, returning the
    /// signature in its final format (RSA block, or ECDSA raw `r || s`).
    fn sign(
        &mut self,
        key: &FoundKey,
        request: &SignRequest<'_>,
    ) -> Result<Signature, KeystoreError>;

    /// Issuer certificates for `key`, nearest first, leaf excluded: the OS
    /// chain engine, or the CA certificates stored on the token. Best effort;
    /// never prompts, never fails.
    fn chain(&mut self, key: &FoundKey) -> Vec<Vec<u8>> {
        let _ = key;
        Vec::new()
    }

    /// Which algorithms the store can produce with `key` (`SPEC.md` §1 rule
    /// 10). Never prompts, never logs in; the host intersects it with what
    /// the key type allows. The default suits sources that cannot tell.
    fn capabilities(&mut self, key: &FoundKey) -> KeyCapabilities {
        let _ = key;
        KeyCapabilities::ALL
    }

    /// The PIN state of `key`'s token, without logging in. `None` for keys
    /// whose PIN the OS owns.
    fn pin_state(&mut self, key: &FoundKey) -> Option<PinState> {
        let _ = key;
        None
    }

    /// Forgets every cached authentication (logs PKCS#11 sessions out, drops
    /// cached OS key handles). Called when the connection goes idle and when
    /// the token leaves (`docs/plan.md` D5).
    fn end_sessions(&mut self) {}
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

impl std::fmt::Debug for Opened {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names: Vec<String> = self.keystores.iter().map(|k| k.name()).collect();
        f.debug_struct("Opened")
            .field("keystores", &names)
            .field("failures", &self.failures)
            .finish()
    }
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
