//! Reopening A1 keys from Microsoft's SHA-1-only CSPs in the Enhanced RSA
//! and AES CSP, which reads the same key containers and knows SHA-2 (what
//! .NET does). Only needed when a key is signed through plain CAPI.

use windows::Win32::Security::Cryptography::{
    CRYPT_MACHINE_KEYSET, CRYPT_SILENT, CryptAcquireContextW, MS_ENH_RSA_AES_PROV_W, PROV_RSA_AES,
    PROV_RSA_FULL,
};
use windows::core::HSTRING;

use super::errors;
use super::handles::CryptProv;
use super::key_info::KeyLocation;
use crate::KeystoreError;
use log::trace;

/// Microsoft's `PROV_RSA_FULL` software CSPs, which predate SHA-2. The
/// certificate import wizard puts PFX keys (A1 certificates) in them.
const SHA1_ONLY_CSPS: [&str; 3] = [
    "Microsoft Base Cryptographic Provider v1.0",
    "Microsoft Enhanced Cryptographic Provider v1.0",
    "Microsoft Strong Cryptographic Provider",
];

/// Whether the key sits in one of [`SHA1_ONLY_CSPS`].
pub fn needs_aes_provider(location: &KeyLocation) -> bool {
    location.provider_type == PROV_RSA_FULL
        && SHA1_ONLY_CSPS
            .iter()
            .any(|name| name.eq_ignore_ascii_case(&location.provider))
}

/// The same container, opened in the AES CSP (`CRYPT_SILENT` when `silent`).
pub fn open_in_aes_provider(
    location: &KeyLocation,
    silent: bool,
) -> Result<CryptProv, KeystoreError> {
    let container = HSTRING::from(location.container.as_str());
    let mut flags = if location.machine_keyset {
        CRYPT_MACHINE_KEYSET.0
    } else {
        0
    };
    if silent {
        flags |= CRYPT_SILENT;
    }
    trace!("CryptAcquireContext(same container, Enhanced RSA and AES CSP, silent: {silent})");
    let mut handle = 0usize;
    // SAFETY: container and provider names are NUL-terminated and outlive
    // the call; `handle` is a live out-pointer.
    unsafe {
        CryptAcquireContextW(
            &mut handle,
            &container,
            MS_ENH_RSA_AES_PROV_W,
            PROV_RSA_AES,
            flags,
        )
    }
    .map_err(|error| errors::native("CryptAcquireContext", &error))?;
    // SAFETY: just acquired; released only by the returned value.
    Ok(unsafe { CryptProv::new(handle, true) })
}

#[cfg(test)]
mod tests {
    use super::needs_aes_provider;
    use crate::windows::key_info::KeyLocation;

    fn location(provider: &str, provider_type: u32) -> KeyLocation {
        KeyLocation {
            provider: provider.to_owned(),
            provider_type,
            container: String::new(),
            key_spec: 1,
            machine_keyset: false,
        }
    }

    #[test]
    fn only_microsoft_sha1_software_csps_are_reopened() {
        assert!(needs_aes_provider(&location(
            "Microsoft Enhanced Cryptographic Provider v1.0",
            1
        )));
        assert!(needs_aes_provider(&location(
            "microsoft base cryptographic provider v1.0",
            1
        )));
        assert!(!needs_aes_provider(&location(
            "Microsoft Enhanced RSA and AES Cryptographic Provider",
            24
        )));
        assert!(!needs_aes_provider(&location(
            "Microsoft Base Smart Card Crypto Provider",
            1
        )));
        assert!(!needs_aes_provider(&location(
            "eToken Base Cryptographic Provider",
            1
        )));
    }
}
