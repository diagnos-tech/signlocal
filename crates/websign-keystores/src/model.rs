//! Data exchanged between the commands and the key sources.

use std::path::PathBuf;
use std::time::Duration;

use secrecy::SecretString;
use websign_core::{HashAlgorithm, SignatureAlgorithm, SourceKind};

/// A certificate whose private key a source can sign with.
///
/// Promoted from the Phase-0 kit. `device` is new: filled by each source when
/// it can tie the key to hardware (`docs/ux.md` R8), `None` otherwise.
#[derive(Debug, Clone)]
pub struct FoundKey {
    pub cert_der: Vec<u8>,
    /// [`super::Keystore::name`] of the source that found it.
    pub keystore: String,
    pub kind: SourceKind,
    /// Where the key lives, for people: KSP/CSP name, token model and module,
    /// or CryptoTokenKit driver. It ends up in published reports, so never a
    /// token label, key container name or serial number.
    pub provider: String,
    /// `Some(true)` when the source says the key is in hardware.
    pub hardware: Option<bool>,
    /// Opaque, source-defined handle to find the key again when signing.
    pub locator: String,
    pub pin: PinPrompt,
    /// The hardware this key lives on, when the source can tell safely.
    pub device: Option<DeviceLink>,
}

/// What ties a key to a physical device, for "Token SafeNet eToken 5110" /
/// "Card in reader" and for dropping it from "possible certificates".
///
/// Never a serial number or a token label (labels often carry the holder's
/// name).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceLink {
    /// A PC/SC reader, by its name without the serial number
    /// (`NCRYPT_READER_PROPERTY` on Windows, slot description on PKCS#11).
    Reader { name: String },
    /// A CryptoTokenKit token, by driver (`com.apple.pivtoken`).
    CryptoTokenKit { driver: String },
    /// A PKCS#11 token, by model and manufacturer from `CK_TOKEN_INFO`.
    Pkcs11Token { model: String, manufacturer: String },
}

/// The PIN situation of a key right now, read without a login
/// (`CK_TOKEN_INFO` flags and limits). OS keys report `None`: the OS owns
/// their PIN.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PinState {
    /// `ulMinPinLen`/`ulMaxPinLen` when the token states them.
    pub length: Option<(u32, u32)>,
    /// `CKF_USER_PIN_COUNT_LOW`.
    pub count_low: bool,
    /// `CKF_USER_PIN_FINAL_TRY`.
    pub final_try: bool,
    /// `CKF_USER_PIN_LOCKED`.
    pub locked: bool,
    /// The session is logged in (`docs/plan.md` D5): signing needs no PIN.
    pub unlocked: bool,
    /// `CKA_ALWAYS_AUTHENTICATE`: every signature asks for the PIN again.
    pub always_authenticate: bool,
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
    /// The token or card left during the operation.
    #[error("the token was removed")]
    TokenRemoved,
    #[error("{0}")]
    Other(String),
}

/// How the Windows store acquires private keys
/// (`CRYPT_ACQUIRE_*_NCRYPT_KEY_FLAG`).
///
/// The app uses [`NcryptPreference::Prefer`] (`docs/plan.md` D9): the kit
/// proved that Microsoft CSP keys then sign through CNG, which adds RSASSA-PSS.
/// A third-party CSP that refuses the bridge is retried with `Allow` (plain
/// CAPI) for that key (`SPEC.md` §4.2).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NcryptPreference {
    /// CNG when the key is CNG, legacy CAPI otherwise.
    Allow,
    /// CNG even for CAPI keys, through the CNG-to-CAPI bridge.
    #[default]
    Prefer,
    /// CNG only; CAPI-only keys fail.
    Only,
}

/// How key sources are opened.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// PKCS#11 modules the person added in Diagnostics › Devices, plus
    /// `--module` in tests.
    pub extra_modules: Vec<PathBuf>,
    /// Do not probe the built-in list of known PKCS#11 module paths.
    pub no_known_modules: bool,
    /// Do not load modules registered with p11-kit.
    pub no_p11_kit: bool,
    /// Windows only: how to acquire private keys.
    pub ncrypt: NcryptPreference,
    /// Windows only: never let a key provider show UI (PIN, consent, "insert
    /// card"). A key that needs it fails with NTE_SILENT_CONTEXT instead of
    /// waiting for a dialog; for unattended runs such as CI.
    pub silent: bool,
}
