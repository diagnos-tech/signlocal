//! OS windows the app hands work to: the certificate viewer and the .pfx
//! import (`docs/ux.md` §5.12, §8.5), and opening URLs.

use std::path::Path;

/// Shows the OS certificate viewer for `der` (`CryptUIDlgViewContext`,
/// `SFCertificatePanel`, `gcr-viewer`); `false` when there is none.
pub fn view_certificate(der: &[u8], parent: Option<isize>) -> bool {
    let _ = (der, parent);
    todo!("ux.md §5.12")
}

/// Starts the OS import of a `.pfx` (`CryptUIWizImport`; Keychain Access on
/// macOS). Linux has none.
pub fn import_pfx(file: Option<&Path>, parent: Option<isize>) -> bool {
    let _ = (file, parent);
    todo!("ux.md §8.5")
}

/// Opens `url` in the default browser.
pub fn open_url(url: &str) -> bool {
    let _ = url;
    todo!("ux.md §6.2")
}
