//! The macOS certificate panel, `.pfx` import through Keychain Access, and
//! URL opening.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use core_foundation::array::CFArray;
use core_foundation::base::TCFType;
use security_framework::certificate::SecCertificate;

use super::file_picker::open_panel;
use super::objc::{AutoreleasePool, Id, ObjcBool, class, on_main_thread, send0, send2};
use crate::platform::file_picker::{FileRequest, Picked};

#[link(name = "SecurityInterface", kind = "framework")]
unsafe extern "C" {}

const KEYCHAIN_ACCESS: &str = "com.apple.keychainaccess";

/// `SFCertificatePanel`, modal; needs the main thread.
pub fn view_certificate(der: &[u8], parent: Option<isize>) -> bool {
    let _ = parent;
    if !on_main_thread() {
        return false;
    }
    let Ok(certificate) = SecCertificate::from_der(der) else {
        return false;
    };
    let Some(panel_class) = class(c"SFCertificatePanel") else {
        log::info!("certificate viewer: SecurityInterface unavailable");
        return false;
    };
    let certificates = CFArray::from_CFTypes(&[certificate]);
    let _pool = AutoreleasePool::new();
    // SAFETY: on the main thread; `+sharedCertificatePanel` returns the
    // panel and `-runModalForCertificates:showGroup:` takes an `NSArray`
    // (toll-free bridged from `CFArray`) of `SecCertificateRef` and a BOOL.
    unsafe {
        let panel: Id = send0(panel_class, c"sharedCertificatePanel");
        if panel.is_null() {
            return false;
        }
        let array = certificates.as_concrete_TypeRef() as Id;
        let _: isize =
            send2::<Id, ObjcBool, isize>(panel, c"runModalForCertificates:showGroup:", array, 0);
    }
    true
}

/// Opens the file in Keychain Access, which asks for the password and
/// imports into the login keychain (`docs/ux.md` §8.5).
pub fn import_pfx(file: Option<&Path>, parent: Option<isize>) -> bool {
    let _ = parent;
    let Some(file) = file.map(Path::to_path_buf).or_else(choose_pfx) else {
        return false;
    };
    run_open(&["-b", KEYCHAIN_ACCESS], file.as_os_str())
}

pub fn open_url(url: &str) -> bool {
    run_open(&[], url.as_ref())
}

fn run_open(options: &[&str], target: &std::ffi::OsStr) -> bool {
    Command::new("/usr/bin/open")
        .args(options)
        .arg(target)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// `NSOpenPanel` for `.pfx`/`.p12` files, when the caller did not choose
/// one with a localized panel ([`crate::platform::file_picker`]).
fn choose_pfx() -> Option<PathBuf> {
    let request = FileRequest {
        title: String::new(),
        type_name: String::new(),
        extensions: vec!["pfx", "p12"],
    };
    match open_panel(&request) {
        Picked::Chosen(path) => Some(path),
        Picked::Cancelled | Picked::Unavailable => None,
    }
}
