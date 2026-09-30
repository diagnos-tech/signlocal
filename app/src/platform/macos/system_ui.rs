//! The macOS certificate panel, `.pfx` import through Keychain Access, and
//! URL opening.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use core_foundation::array::CFArray;
use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use security_framework::certificate::SecCertificate;

use super::objc::{AutoreleasePool, Id, ObjcBool, class, on_main_thread, send0, send1, send2};

#[link(name = "SecurityInterface", kind = "framework")]
unsafe extern "C" {}

/// `NSModalResponseOK`.
const MODAL_OK: isize = 1;
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

/// `NSOpenPanel` for `.pfx`/`.p12` files; needs the main thread.
fn choose_pfx() -> Option<PathBuf> {
    if !on_main_thread() {
        return None;
    }
    let panel_class = class(c"NSOpenPanel")?;
    let types = CFArray::from_CFTypes(&[CFString::new("pfx"), CFString::new("p12")]);
    let _pool = AutoreleasePool::new();
    // SAFETY: on the main thread; each method exists with the signature
    // used; `setAllowedFileTypes:` takes an `NSArray` of `NSString`
    // (toll-free bridged from `CFArray` of `CFString`); the returned
    // `NSString` path is bridged back as a `CFString` under the get rule.
    unsafe {
        let panel: Id = send0(panel_class, c"openPanel");
        if panel.is_null() {
            return None;
        }
        send1::<Id, ()>(
            panel,
            c"setAllowedFileTypes:",
            types.as_concrete_TypeRef() as Id,
        );
        if send0::<isize>(panel, c"runModal") != MODAL_OK {
            return None;
        }
        let url: Id = send0(panel, c"URL");
        if url.is_null() {
            return None;
        }
        let path: Id = send0(url, c"path");
        if path.is_null() {
            return None;
        }
        Some(PathBuf::from(
            CFString::wrap_under_get_rule(path as CFStringRef).to_string(),
        ))
    }
}
