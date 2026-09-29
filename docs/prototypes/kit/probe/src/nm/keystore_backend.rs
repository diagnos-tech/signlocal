//! The real [`Backend`]: the same key sources the CLI uses, opened once per
//! process on first use.
//!
//! Spike shortcuts, on purpose:
//! * the PKCS#11 PIN comes from `WEBSIGN_PROBE_PIN` (there is no PIN window
//!   yet, and the value is never logged);
//! * extra PKCS#11 modules come from `WEBSIGN_PROBE_MODULES`, a list of paths
//!   separated like `PATH` is, because a browser cannot pass `--module`;
//! * a panic inside a key source is caught and reported as an `internal`
//!   error, so an unfinished or crashing driver cannot take the host down
//!   mid-proof.

use std::ffi::OsString;
use std::panic::{AssertUnwindSafe, catch_unwind};

use secrecy::SecretString;

use super::backend::{Backend, CertificateList, CertificateSummary, SignJob, SignedDigest};
use super::protocol::{ErrorCode, ProtocolError};
use super::summary;
use crate::keystores::{KeystoreError, Options, PinPrompt, SignRequest};

mod sources;

use sources::{Key, State, ensure_loaded, panic_message};

/// Environment variable with the PKCS#11 PIN, for tests only.
pub const PIN_ENV: &str = "WEBSIGN_PROBE_PIN";
/// Environment variable with extra PKCS#11 module paths, for tests only.
pub const MODULES_ENV: &str = "WEBSIGN_PROBE_MODULES";

pub struct KeystoreBackend {
    state: State,
    /// Read once at startup: the spike has no PIN window yet.
    pin: Option<SecretString>,
    options: Options,
}

impl KeystoreBackend {
    /// A backend that opens the machine's key sources on first use, taking
    /// the PKCS#11 PIN from [`PIN_ENV`] and extra modules from
    /// [`MODULES_ENV`], if set.
    pub fn new() -> Self {
        Self {
            state: State::Unloaded,
            pin: std::env::var(PIN_ENV).ok().map(SecretString::from),
            options: options_with_modules(std::env::var_os(MODULES_ENV)),
        }
    }
}

impl Backend for KeystoreBackend {
    fn certificates(&mut self) -> Result<CertificateList, ProtocolError> {
        let loaded = ensure_loaded(&mut self.state, &self.options)?;
        let certificates = loaded
            .groups
            .iter()
            .map(|group| {
                summary::summarize(&group.info, &loaded.keys[group.primary].found, group.paths)
            })
            .collect::<Vec<CertificateSummary>>();
        Ok(CertificateList {
            certificates,
            warnings: loaded.warnings.clone(),
        })
    }

    fn sign(&mut self, job: &SignJob) -> Result<SignedDigest, ProtocolError> {
        let Self {
            state,
            pin,
            options,
        } = self;
        let loaded = ensure_loaded(state, options)?;
        let group = loaded
            .groups
            .iter()
            .find(|group| group.info.fingerprint == job.fingerprint)
            .ok_or_else(|| {
                ProtocolError::new(ErrorCode::NotFound, "no certificate has that fingerprint")
            })?;
        if !group.info.can_sign() || !group.info.key.supports(job.algorithm) {
            return Err(ProtocolError::new(
                ErrorCode::Unsupported,
                format!("this certificate cannot sign with {}", job.algorithm),
            ));
        }

        let Key { store, found } = &loaded.keys[group.primary];
        let pin = match found.pin {
            PinPrompt::App {
                protected_path: false,
            } => Some(pin.as_ref().ok_or_else(missing_pin)?),
            PinPrompt::System | PinPrompt::App { .. } => None,
        };
        let request = SignRequest {
            hash: job.hash,
            algorithm: job.algorithm,
            digest: &job.digest,
            pin,
            parent_window: job.parent_window,
        };
        let keystore = &mut loaded.keystores[*store];
        let signature = catch_unwind(AssertUnwindSafe(|| keystore.sign(found, &request)))
            .map_err(|payload| {
                ProtocolError::new(
                    ErrorCode::Internal,
                    format!("key source crashed: {}", panic_message(&*payload)),
                )
            })?
            .map_err(map_error)?;

        let verified = probe_core::verify(
            &found.cert_der,
            job.hash,
            job.algorithm,
            &job.digest,
            &signature.bytes,
        )
        .is_ok();
        Ok(SignedDigest {
            signature: signature.bytes,
            api: signature.api,
            elapsed_ms: u64::try_from(signature.elapsed.as_millis()).unwrap_or(u64::MAX),
            verified,
        })
    }
}

/// The default options plus the modules named in `list`, a `PATH`-style list.
fn options_with_modules(list: Option<OsString>) -> Options {
    Options {
        extra_modules: list
            .map(|list| std::env::split_paths(&list).collect())
            .unwrap_or_default(),
        ..Options::default()
    }
}

fn missing_pin() -> ProtocolError {
    ProtocolError::new(
        ErrorCode::PinRequired,
        format!("this key needs a PIN; the spike reads it from {PIN_ENV}"),
    )
}

fn map_error(error: KeystoreError) -> ProtocolError {
    let code = match error {
        KeystoreError::WrongPin => ErrorCode::WrongPin,
        KeystoreError::PinLocked => ErrorCode::PinLocked,
        KeystoreError::PinRequired => ErrorCode::PinRequired,
        KeystoreError::Cancelled => ErrorCode::Cancelled,
        KeystoreError::NotFound => ErrorCode::NotFound,
        KeystoreError::Unsupported(_) => ErrorCode::Unsupported,
        KeystoreError::Native { .. } | KeystoreError::Other(_) => ErrorCode::Internal,
    };
    ProtocolError::new(code, error.to_string())
}

#[cfg(test)]
mod fixture;
#[cfg(test)]
mod tests;
