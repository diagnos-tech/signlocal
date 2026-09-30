//! How a desktop program that asks for a signature is shown and remembered.
//!
//! The app identifies the calling process itself (never trusting what the
//! program says): executable path, product name from the executable's
//! metadata, and the code-signing identity when the OS can verify one.

use std::path::PathBuf;

use super::visible::visible;

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

/// Longest product name shown, ellipsis included: the window has room for
/// one line of it, and a caller must not push the signer out of view.
const MAX_NAME_CHARS: usize = 64;

/// The label to show for `caller`.
///
/// Every text comes from the calling program's own metadata, so control and
/// bidirectional formatting characters are removed before it is shown.
pub fn caller_label(caller: &DesktopCaller) -> CallerLabel {
    let label = raw_label(caller);
    CallerLabel {
        name: visible(&label.name),
        detail: visible(&label.detail),
        verified: label.verified,
    }
}

fn raw_label(caller: &DesktopCaller) -> CallerLabel {
    let name = caller
        .product_name
        .as_deref()
        .map(|name| visible(name).trim().to_owned())
        .filter(|name| !name.is_empty())
        .map_or_else(|| file_name(caller), |name| shorten(&name));
    match &caller.signer {
        Some(CodeSigner::Authenticode { subject }) => CallerLabel {
            name,
            detail: subject.clone(),
            verified: true,
        },
        Some(CodeSigner::Apple {
            team_id,
            identifier,
        }) => CallerLabel {
            name,
            detail: format!("{identifier} ({team_id})"),
            verified: true,
        },
        None => CallerLabel {
            name,
            detail: caller.executable.display().to_string(),
            verified: false,
        },
    }
}

/// The key consent is remembered under: `"app:<signer>"` for signed
/// programs (survives updates that move the binary), `"path:<executable>"`
/// otherwise.
///
/// The three prefixes keep the families apart: an unsigned program at a path
/// that spells a signed key still gets a `path:` key. A path that is not
/// valid Unicode is keyed by its lossy form; the platform layer reports
/// absolute, real paths, and two such paths differing only in invalid bytes
/// are not a practical way to borrow another program's consent.
pub fn consent_key(caller: &DesktopCaller) -> String {
    match &caller.signer {
        Some(CodeSigner::Authenticode { subject }) => {
            format!(
                "app:authenticode:{subject}:{}",
                file_name(caller).to_lowercase()
            )
        }
        Some(CodeSigner::Apple {
            team_id,
            identifier,
        }) => format!("app:apple:{team_id}:{identifier}"),
        None => format!("path:{}", caller.executable.display()),
    }
}

/// The last path component, split the way the host OS splits paths; the
/// whole path when there is none (`/`, `..`, empty).
fn file_name(caller: &DesktopCaller) -> String {
    caller
        .executable
        .file_name()
        .unwrap_or(caller.executable.as_os_str())
        .to_string_lossy()
        .into_owned()
}

/// `name` when it fits [`MAX_NAME_CHARS`]; else its first 63 characters,
/// trailing whitespace removed, and `…`.
fn shorten(name: &str) -> String {
    if name.chars().count() <= MAX_NAME_CHARS {
        return name.to_owned();
    }
    let kept: String = name.chars().take(MAX_NAME_CHARS - 1).collect();
    format!("{}…", kept.trim_end())
}
