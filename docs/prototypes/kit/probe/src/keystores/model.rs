//! Data exchanged between the commands and the key sources.

use std::path::PathBuf;
use std::time::Duration;

use probe_core::{HashAlgorithm, SignatureAlgorithm, SourceKind};
use secrecy::SecretString;

/// A certificate whose private key a source can sign with.
#[derive(Debug, Clone)]
pub struct FoundKey {
    pub cert_der: Vec<u8>,
    /// [`super::Keystore::name`] of the source that found it.
    pub keystore: String,
    pub kind: SourceKind,
    /// Where the key lives, for people: KSP/CSP name, token label and model.
    pub provider: String,
    /// `Some(true)` when the source says the key is in hardware.
    pub hardware: Option<bool>,
    /// Opaque, source-defined handle to find the key again when signing.
    pub locator: String,
    pub pin: PinPrompt,
}

/// Who collects the PIN for a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinPrompt {
    /// The OS or the vendor middleware shows its own dialog.
    // Linux has no OS key store; only PKCS#11 keys exist there.
    #[cfg_attr(not(any(windows, target_os = "macos")), allow(dead_code))]
    System,
    /// The app must collect it. `protected_path` means a PIN pad on the reader.
    App { protected_path: bool },
}

/// One signing operation.
#[derive(Debug)]
pub struct SignRequest<'a> {
    pub hash: HashAlgorithm,
    pub algorithm: SignatureAlgorithm,
    /// Exactly `hash.digest_len()` bytes; callers validate before signing.
    pub digest: &'a [u8],
    /// Only for keys with [`PinPrompt::App`].
    pub pin: Option<&'a SecretString>,
    /// Native window handle that should own OS PIN dialogs (HWND on Windows),
    /// so they open in front of the browser instead of behind it.
    #[cfg_attr(not(windows), allow(dead_code))]
    pub parent_window: Option<isize>,
}

/// A signature in its final format, plus how it was produced.
#[derive(Debug)]
pub struct Signature {
    pub bytes: Vec<u8>,
    /// The native call that produced it, e.g. `"NCryptSignHash"` or `"C_Sign"`.
    pub api: &'static str,
    pub elapsed: Duration,
}

/// Why a source could not list or sign.
#[derive(Debug, thiserror::Error)]
pub enum KeystoreError {
    /// A native call failed; `code` is the raw status (HRESULT, CKR_*, OSStatus).
    #[error("{api} failed with {code:#x}: {message}")]
    Native {
        api: &'static str,
        code: i64,
        message: String,
    },
    #[error("a PIN is required")]
    PinRequired,
    #[error("wrong PIN")]
    WrongPin,
    #[error("PIN locked")]
    PinLocked,
    #[error("cancelled by the user")]
    Cancelled,
    #[error("not supported by this key: {0}")]
    Unsupported(String),
    #[error("key not found")]
    NotFound,
    #[error("{0}")]
    Other(String),
}

/// How the Windows store acquires private keys
/// (`CRYPT_ACQUIRE_*_NCRYPT_KEY_FLAG`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum NcryptPreference {
    /// CNG when the key is CNG, legacy CAPI otherwise (what the app ships).
    #[default]
    Allow,
    /// CNG even for CAPI keys, through the CNG-to-CAPI bridge.
    Prefer,
    /// CNG only; CAPI-only keys fail.
    Only,
}

/// Knobs shared by every command that opens key sources.
#[derive(Debug, Clone, Default, clap::Args)]
pub struct Options {
    /// Extra PKCS#11 module to load (repeatable).
    #[arg(long = "module", value_name = "PATH")]
    pub extra_modules: Vec<PathBuf>,
    /// Do not probe the built-in list of known PKCS#11 module paths.
    #[arg(long)]
    pub no_known_modules: bool,
    /// Do not load modules registered with p11-kit.
    #[arg(long)]
    pub no_p11_kit: bool,
    /// Windows only: how to acquire private keys.
    #[arg(long, value_enum, default_value_t)]
    pub ncrypt: NcryptPreference,
}
