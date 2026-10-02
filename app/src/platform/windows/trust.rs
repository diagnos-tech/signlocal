//! `WinVerifyTrust` with the default Authenticode policy, and the name of
//! the signer it verified.
//!
//! The policy: chain to a trusted root, no revocation check over the
//! network (it would stall the window for a minute offline), no UI. The
//! same call verifies an embedded signature or a catalog entry; only the
//! subject differs.

use windows::Win32::Foundation::{HANDLE, HWND, INVALID_HANDLE_VALUE};
use windows::Win32::Security::Cryptography::{CERT_NAME_SIMPLE_DISPLAY_TYPE, CertGetNameStringW};
use windows::Win32::Security::WinTrust::{
    WINTRUST_ACTION_GENERIC_VERIFY_V2, WINTRUST_CATALOG_INFO, WINTRUST_DATA, WINTRUST_DATA_0,
    WINTRUST_FILE_INFO, WTD_CACHE_ONLY_URL_RETRIEVAL, WTD_CHOICE_CATALOG, WTD_CHOICE_FILE,
    WTD_REVOKE_NONE, WTD_STATEACTION_CLOSE, WTD_STATEACTION_VERIFY, WTD_UI_NONE,
    WTHelperGetProvSignerFromChain, WTHelperProvDataFromStateData, WinVerifyTrust,
};

use super::wide::from_wide;

/// What to verify.
pub enum Subject<'a> {
    /// A file's embedded signature.
    File(&'a mut WINTRUST_FILE_INFO),
    /// A file's entry in a signed catalog.
    Catalog(&'a mut WINTRUST_CATALOG_INFO),
}

/// The verified signer's display name (the leaf certificate's CN), or
/// `None` when the subject is unsigned or not trusted.
pub fn verified_signer(subject: Subject<'_>) -> Option<String> {
    let (choice, union, kind) = match subject {
        Subject::File(file) => (WTD_CHOICE_FILE, WINTRUST_DATA_0 { pFile: file }, "embedded"),
        Subject::Catalog(catalog) => (
            WTD_CHOICE_CATALOG,
            WINTRUST_DATA_0 { pCatalog: catalog },
            "catalog",
        ),
    };
    let mut data = WINTRUST_DATA {
        cbStruct: size_of::<WINTRUST_DATA>() as u32,
        dwUIChoice: WTD_UI_NONE,
        fdwRevocationChecks: WTD_REVOKE_NONE,
        dwUnionChoice: choice,
        Anonymous: union,
        dwStateAction: WTD_STATEACTION_VERIFY,
        dwProvFlags: WTD_CACHE_ONLY_URL_RETRIEVAL,
        ..Default::default()
    };
    let mut action = WINTRUST_ACTION_GENERIC_VERIFY_V2;
    // SAFETY: `data` and the subject structure (with the strings and handle
    // it points to, which the caller keeps alive) outlive the guard below,
    // which closes the verification; the window handle means "no UI".
    let status = unsafe { WinVerifyTrust(no_ui(), &mut action, (&raw mut data).cast()) };
    let verification = Verification { data: &mut data };
    if status != 0 {
        log::info!("Authenticode ({kind}): not verified (0x{status:08X})");
        return None;
    }
    leaf_subject(verification.data.hWVTStateData)
}

fn leaf_subject(state: HANDLE) -> Option<String> {
    // SAFETY: `state` comes from a successful verify that is still open.
    let provider = unsafe { WTHelperProvDataFromStateData(state) };
    if provider.is_null() {
        return None;
    }
    // SAFETY: `provider` is the non-null provider data of that state.
    let signer = unsafe { WTHelperGetProvSignerFromChain(provider, 0, false, 0) };
    // SAFETY: non-null signers point into the open state's data.
    let signer = unsafe { signer.as_ref() }?;
    if signer.csCertChain == 0 {
        return None;
    }
    // SAFETY: `pasCertChain` holds `csCertChain` (≥ 1) entries; the first is the leaf.
    let leaf = unsafe { signer.pasCertChain.as_ref() }?.pCert;
    if leaf.is_null() {
        return None;
    }
    // SAFETY: `leaf` is a valid certificate context; a size query first.
    let len = unsafe { CertGetNameStringW(leaf, CERT_NAME_SIMPLE_DISPLAY_TYPE, 0, None, None) };
    let mut name = vec![0u16; len as usize];
    // SAFETY: `name` holds the `len` units the query asked for.
    unsafe {
        CertGetNameStringW(
            leaf,
            CERT_NAME_SIMPLE_DISPLAY_TYPE,
            0,
            None,
            Some(&mut name),
        )
    };
    Some(from_wide(&name)).filter(|name| !name.is_empty())
}

fn no_ui() -> HWND {
    HWND(INVALID_HANDLE_VALUE.0)
}

/// Closes an open `WinVerifyTrust` state (required after every verify,
/// whatever its result).
struct Verification<'a> {
    data: &'a mut WINTRUST_DATA,
}

impl Drop for Verification<'_> {
    fn drop(&mut self) {
        self.data.dwStateAction = WTD_STATEACTION_CLOSE;
        let mut action = WINTRUST_ACTION_GENERIC_VERIFY_V2;
        // SAFETY: the same `WINTRUST_DATA` the verify used, still valid.
        unsafe { WinVerifyTrust(no_ui(), &mut action, (&raw mut *self.data).cast()) };
    }
}
