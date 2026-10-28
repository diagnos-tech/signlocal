//! Whether a provider keeps its keys in hardware, asked of the provider
//! itself (never of a key), so that listing never touches a card.

use std::collections::HashMap;

use windows::Win32::Security::Cryptography::{
    CRYPT_IMPL_HARDWARE, CRYPT_IMPL_REMOVABLE, CRYPT_SILENT, CRYPT_VERIFYCONTEXT,
    CryptAcquireContextW, CryptGetProvParam, NCRYPT_IMPL_TYPE_PROPERTY, NCRYPT_PROV_HANDLE,
    NCRYPT_SILENT_FLAG, NCryptGetProperty, NCryptOpenStorageProvider, PP_IMPTYPE,
};
use windows::Win32::Security::OBJECT_SECURITY_INFORMATION;
use windows::core::{HSTRING, Owned, PCWSTR};

use super::handles::CryptProv;
use super::key_info::KeyLocation;
use log::trace;

/// Whether keys are in hardware, asked once per provider.
#[derive(Debug, Default)]
pub struct HardwareProbe(HashMap<(String, u32), Option<bool>>);

impl HardwareProbe {
    /// Asks the provider for its implementation type (`NCRYPT_IMPL_TYPE_PROPERTY`
    /// or `PP_IMPTYPE`) without opening any key; falls back to the provider name.
    pub fn is_hardware(&mut self, key: &KeyLocation) -> Option<bool> {
        *self
            .0
            .entry((key.provider.clone(), key.provider_type))
            .or_insert_with(|| {
                let flags = if key.provider.is_empty() {
                    None
                } else if key.is_cng() {
                    ksp_impl_type(&key.provider)
                } else {
                    csp_impl_type(&key.provider, key.provider_type)
                };
                // The CNG and CAPI flags share their values.
                flags
                    .map(|flags| flags & (CRYPT_IMPL_HARDWARE | CRYPT_IMPL_REMOVABLE) != 0)
                    .or_else(|| hardware_by_name(&key.provider))
            })
    }
}

fn ksp_impl_type(provider: &str) -> Option<u32> {
    trace!("NCryptOpenStorageProvider(\"{provider}\") + NCryptGetProperty(Impl Type, silent)");
    let name = HSTRING::from(provider);
    let mut handle = NCRYPT_PROV_HANDLE::default();
    // SAFETY: `name` is NUL-terminated and outlives the call. Opening a
    // provider loads it without opening any key.
    unsafe { NCryptOpenStorageProvider(&mut handle, &name, 0) }.ok()?;
    // SAFETY: the handle was just opened and is freed only by `Owned`.
    let handle = unsafe { Owned::new(handle) };
    let mut value = [0u8; 4];
    let mut len = 0u32;
    // SAFETY: valid provider handle; the wrapper passes `value` with its size.
    unsafe {
        NCryptGetProperty(
            (*handle).into(),
            NCRYPT_IMPL_TYPE_PROPERTY,
            Some(&mut value),
            &mut len,
            OBJECT_SECURITY_INFORMATION(NCRYPT_SILENT_FLAG.0),
        )
    }
    .ok()?;
    (len == 4).then(|| u32::from_ne_bytes(value))
}

fn csp_impl_type(provider: &str, provider_type: u32) -> Option<u32> {
    trace!(
        "CryptAcquireContext(\"{provider}\", type {provider_type}, VERIFYCONTEXT | SILENT) + PP_IMPTYPE"
    );
    let name = HSTRING::from(provider);
    let mut handle = 0usize;
    // SAFETY: `name` outlives the call. With no container and
    // CRYPT_VERIFYCONTEXT | CRYPT_SILENT the CSP opens no key and shows no UI.
    unsafe {
        CryptAcquireContextW(
            &mut handle,
            PCWSTR::null(),
            &name,
            provider_type,
            CRYPT_VERIFYCONTEXT | CRYPT_SILENT,
        )
    }
    .ok()?;
    // SAFETY: the context was just acquired and is released only here.
    let context = unsafe { CryptProv::new(handle, true) };
    let mut value = 0u32;
    let mut len = size_of::<u32>() as u32;
    // SAFETY: `value` is a live u32 and `len` says so.
    unsafe {
        CryptGetProvParam(
            context.raw(),
            PP_IMPTYPE,
            Some((&raw mut value).cast()),
            &mut len,
            0,
        )
    }
    .ok()?;
    Some(value)
}

/// Last resort when the provider cannot be asked. Microsoft's smart card
/// and TPM providers are hardware; its other providers are software. Third
/// party providers stay unknown.
fn hardware_by_name(provider: &str) -> Option<bool> {
    let name = provider.to_ascii_lowercase();
    if name.contains("smart card") || name.contains("platform crypto provider") {
        Some(true)
    } else if name.starts_with("microsoft ") {
        Some(false)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::hardware_by_name;

    #[test]
    fn hardware_heuristic() {
        assert_eq!(
            hardware_by_name("Microsoft Smart Card Key Storage Provider"),
            Some(true)
        );
        assert_eq!(
            hardware_by_name("Microsoft Base Smart Card Crypto Provider"),
            Some(true)
        );
        assert_eq!(
            hardware_by_name("Microsoft Platform Crypto Provider"),
            Some(true)
        );
        assert_eq!(
            hardware_by_name("Microsoft Software Key Storage Provider"),
            Some(false)
        );
        assert_eq!(
            hardware_by_name("Microsoft Enhanced RSA and AES Cryptographic Provider"),
            Some(false)
        );
        assert_eq!(
            hardware_by_name("SafeSign Standard Cryptographic Service Provider"),
            None
        );
    }
}
