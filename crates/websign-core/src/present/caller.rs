//! How a desktop program that asks for a signature is shown and remembered.
//!
//! The app identifies the calling process itself (never trusting what the
//! program says): executable path, product name from the executable's
//! metadata, and the code-signing identity when the OS can verify one.

use std::path::PathBuf;

/// A calling program, as the platform layer identified it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopCaller {
    pub executable: PathBuf,
    /// `FileDescription`/`ProductName` (Windows), `CFBundleName` (macOS),
    /// `.desktop` `Name` (Linux) when found.
    pub product_name: Option<String>,
    /// Verified code signature, when there is one.
    pub signer: Option<CodeSigner>,
}

/// A verified code-signing identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeSigner {
    /// Windows Authenticode leaf subject CN (`"Microsoft Corporation"`).
    Authenticode { subject: String },
    /// macOS Developer ID / App Store: team ID and signing identifier.
    Apple { team_id: String, identifier: String },
}

/// What the confirmation window shows instead of a web origin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallerLabel {
    /// Emphasized: product name, else the executable's file name.
    pub name: String,
    /// Dimmed: "by <signer>" material, else the executable path.
    pub detail: String,
    /// False when no signer was verified: the window warns
    /// "Unverified program".
    pub verified: bool,
}

/// The label to show for `caller`.
pub fn caller_label(caller: &DesktopCaller) -> CallerLabel {
    let _ = caller;
    todo!("SPEC.md §12")
}

/// The key consent is remembered under: `"app:<signer>"` for signed
/// programs (survives updates that move the binary), `"path:<executable>"`
/// otherwise.
pub fn consent_key(caller: &DesktopCaller) -> String {
    let _ = caller;
    todo!("SPEC.md §12")
}
