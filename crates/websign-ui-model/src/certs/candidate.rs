//! What the host knows about one certificate before it becomes a row.

use websign_core::{CertInfo, Fingerprint, SignatureAlgorithm};

/// One de-duplicated certificate with everything the rules need.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertCandidate {
    pub fingerprint: Fingerprint,
    /// `Err` certificates are counted in diagnostics and never listed.
    pub info: Result<CertInfo, websign_core::CertError>,
    /// The path the signature will take (the OS when it has the key).
    pub source: KeySource,
    /// Other paths to the same key (`docs/ux.md` §5.11); path `n` of
    /// `UiEvent::Sign.via` is `alternates[n - 1]`.
    pub alternates: Vec<KeyPath>,
    pub device: Option<DeviceLabel>,
    pub pin: PinMode,
    /// What the key store can produce with this key.
    pub algorithms: Vec<SignatureAlgorithm>,
    /// `Some(true)` = key in hardware; `None` = unknown.
    pub hardware: Option<bool>,
    /// A source lists a private key for it.
    pub has_private_key: bool,
    /// Its token was removed while the window was open.
    pub removed: bool,
}

/// Another way to reach a candidate's key, with who asks for the PIN on
/// it: the OS store may show its own dialog while the token driver that
/// sees the same key needs our field (`docs/ux.md` §4.6, §5.11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyPath {
    pub source: KeySource,
    pub pin: PinMode,
}

/// Where a key is reached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeySource {
    /// Windows certificate store (CNG or CAPI).
    Windows,
    /// macOS keychain file (an imported A1).
    MacosKeychain,
    /// macOS CryptoTokenKit token.
    MacosToken,
    /// A PKCS#11 module, by path (user folder replaced by `~`/`%USERPROFILE%`).
    Driver { path: String },
}

/// The device a key lives on, as far as it can be named safely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceLabel {
    /// A `devices.json` token model: "Token SafeNet eToken 5110".
    Token { name: String },
    /// A card in a reader; the reader name goes to Details.
    CardInReader { reader: String },
    /// Hardware without a safe mapping: "Token or card".
    Unknown,
}

/// Who asks for the PIN (`docs/ux.md` §4.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinMode {
    /// The OS or the vendor middleware shows its own dialog.
    System,
    /// Our PIN field; `length` from the token when stated.
    App {
        length: Option<(u32, u32)>,
        count_low: bool,
        final_try: bool,
        locked: bool,
    },
    /// The reader's keypad (`CKF_PROTECTED_AUTHENTICATION_PATH`).
    PinPad,
    /// Already logged in this session; no PIN needed (`docs/plan.md` D5).
    Unlocked,
}
