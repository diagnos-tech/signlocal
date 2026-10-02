//! OS windows the app hands work to: the certificate viewer and the .pfx
//! import (`docs/ux.md` §5.12, §8.5), and opening URLs.
//!
//! The app never asks for a `.pfx` password itself: the OS import (the
//! Windows wizard, Keychain Access) asks for it and stores the key, so the
//! password never passes through this process.

use std::path::Path;

/// Shows the OS certificate viewer for `der` (`CryptUIDlgViewContext`,
/// `SFCertificatePanel`, `gcr-viewer`); `false` when there is none.
///
/// `parent` is the owner window's native handle (`HWND` on Windows; unused
/// elsewhere). On Windows and macOS the viewer is modal and this returns
/// when it closes; call it on the UI thread. On Linux the viewer is a
/// separate program and this returns once it started.
pub fn view_certificate(der: &[u8], parent: Option<isize>) -> bool {
    if der.is_empty() {
        return false;
    }
    super::os::system_ui::view_certificate(der, parent)
}

/// Starts the OS import of a `.pfx` (`CryptUIWizImport`; Keychain Access on
/// macOS). Linux has none: there is no user certificate store every program
/// reads, so the Certificates tab hides the button and points to tokens
/// (`docs/ux.md` §8.5); software certificates there live in each program's
/// NSS database or a p11-kit module, which the person manages with that
/// program.
///
/// With `file` absent, the OS picker (the wizard's own page on Windows, an
/// open panel on macOS) chooses it. `true` when the import finished on
/// Windows, or when Keychain Access received the file on macOS (the import
/// finishes there); the Certificates tab re-reads the list when its window
/// regains focus.
pub fn import_pfx(file: Option<&Path>, parent: Option<isize>) -> bool {
    super::os::system_ui::import_pfx(file, parent)
}

/// Opens `url` in the default browser. Only `https:` URLs are opened: every
/// link the app shows is one, and any other scheme could start a local
/// program.
pub fn open_url(url: &str) -> bool {
    if !is_web_link(url) {
        log::warn!("open_url: refused a non-https URL");
        return false;
    }
    super::os::system_ui::open_url(url)
}

fn is_web_link(url: &str) -> bool {
    url.get(..8)
        .is_some_and(|scheme| scheme.eq_ignore_ascii_case("https://"))
        && url.len() > 8
        && !url.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_https_links_open() {
        assert!(is_web_link("https://diagnos-tech.github.io/web-esign/"));
        assert!(is_web_link("HTTPS://example.com"));
        assert!(!is_web_link("https://"));
        assert!(!is_web_link("http://example.com"));
        assert!(!is_web_link("file:///etc/passwd"));
        assert!(!is_web_link("-https://x"));
        assert!(!is_web_link("https://example.com/\nrm"));
        assert!(!is_web_link("ht"));
    }

    #[test]
    fn empty_certificates_are_not_shown() {
        assert!(!view_certificate(&[], None));
    }
}
