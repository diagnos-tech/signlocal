//! The Windows certificate viewer, `.pfx` import wizard and URL opening.

use std::path::Path;

use windows::Win32::Security::Cryptography::UI::{
    CRYPTUI_WIZ_IMPORT_SRC_INFO, CRYPTUI_WIZ_IMPORT_SRC_INFO_0, CRYPTUI_WIZ_IMPORT_SUBJECT_FILE,
    CRYPTUI_WIZ_IMPORT_TO_CURRENTUSER, CryptUIDlgViewContext, CryptUIWizImport,
};
use windows::Win32::Security::Cryptography::{
    CERT_CONTEXT, CertCreateCertificateContext, CertFreeCertificateContext, PKCS_7_ASN_ENCODING,
    X509_ASN_ENCODING,
};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{PCWSTR, w};

use super::wide::{wide, window};

/// `CERT_STORE_CERTIFICATE_CONTEXT`: the context type the viewer shows.
const CERTIFICATE_CONTEXT: u32 = 1;

/// Modal: returns when the person closes the viewer.
pub fn view_certificate(der: &[u8], parent: Option<isize>) -> bool {
    let Some(certificate) = Certificate::parse(der) else {
        return false;
    };
    // SAFETY: `certificate` is a valid context for the whole call.
    unsafe {
        CryptUIDlgViewContext(
            CERTIFICATE_CONTEXT,
            certificate.0.cast(),
            window(parent),
            PCWSTR::null(),
            0,
            std::ptr::null(),
        )
    }
    .as_bool()
}

/// The Windows import wizard, into the current user's store; it asks for
/// the file (when absent) and its password itself. `false` when cancelled.
pub fn import_pfx(file: Option<&Path>, parent: Option<isize>) -> bool {
    let path = file.map(wide);
    let source = path.as_ref().map(|path| CRYPTUI_WIZ_IMPORT_SRC_INFO {
        dwSize: size_of::<CRYPTUI_WIZ_IMPORT_SRC_INFO>() as u32,
        dwSubjectChoice: CRYPTUI_WIZ_IMPORT_SUBJECT_FILE,
        Anonymous: CRYPTUI_WIZ_IMPORT_SRC_INFO_0 {
            pwszFileName: PCWSTR(path.as_ptr()),
        },
        ..Default::default()
    });
    // SAFETY: `source` and the path it points to outlive the modal call.
    let imported = unsafe {
        CryptUIWizImport(
            CRYPTUI_WIZ_IMPORT_TO_CURRENTUSER,
            window(parent),
            PCWSTR::null(),
            source.as_ref().map(|source| source as *const _),
            None,
        )
    };
    if let Err(error) = &imported {
        log::info!("pfx import: not completed (0x{:08X})", error.code().0);
    }
    imported.is_ok()
}

pub fn open_url(url: &str) -> bool {
    let url = wide(url);
    // SAFETY: the strings are NUL-terminated and outlive the call.
    let result = unsafe {
        ShellExecuteW(
            None,
            w!("open"),
            PCWSTR(url.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    // Values above 32 mean success (a legacy `HINSTANCE` convention).
    result.0 as isize > 32
}

/// A certificate context owned by us, freed on drop.
struct Certificate(*const CERT_CONTEXT);

impl Certificate {
    fn parse(der: &[u8]) -> Option<Self> {
        // SAFETY: Windows copies `der`; null means it did not parse.
        let context =
            unsafe { CertCreateCertificateContext(X509_ASN_ENCODING | PKCS_7_ASN_ENCODING, der) };
        (!context.is_null()).then(|| Self(context))
    }
}

impl Drop for Certificate {
    fn drop(&mut self) {
        // SAFETY: the context came from `CertCreateCertificateContext` and
        // is freed exactly once.
        unsafe {
            let _ = CertFreeCertificateContext(Some(self.0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn garbage_is_not_a_certificate() {
        assert!(Certificate::parse(b"not a certificate").is_none());
        assert!(!view_certificate(b"not a certificate", None));
    }
}
