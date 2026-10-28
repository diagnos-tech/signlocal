//! Asking a smart card provider for the reader of a key container, silently
//! and without a PIN. The key or context is opened only for the question
//! and released right after; signing opens its own.

use windows::Win32::Security::Cryptography::{
    CERT_KEY_SPEC, CRYPT_SILENT, CryptAcquireContextW, CryptGetProvParam, NCRYPT_KEY_HANDLE,
    NCRYPT_PROV_HANDLE, NCRYPT_READER_PROPERTY, NCRYPT_SILENT_FLAG, NCryptGetProperty,
    NCryptOpenKey, NCryptOpenStorageProvider, PP_SMARTCARD_READER,
};
use windows::Win32::Security::OBJECT_SECURITY_INFORMATION;
use windows::core::{HSTRING, Owned};

use super::handles::{CryptProv, NcryptKey};
use super::key_info::KeyLocation;
use log::trace;

/// `NCRYPT_READER_PROPERTY` of a CNG key (UTF-16).
pub fn ksp_reader(location: &KeyLocation) -> Option<String> {
    trace!("NCryptOpenKey(silent) + NCryptGetProperty(SmartCardReader) on a smart card KSP key");
    let provider_name = HSTRING::from(location.provider.as_str());
    let mut provider = NCRYPT_PROV_HANDLE::default();
    // SAFETY: `provider_name` is NUL-terminated and outlives the call.
    unsafe { NCryptOpenStorageProvider(&mut provider, &provider_name, 0) }.ok()?;
    // SAFETY: just opened; freed only by `Owned`.
    let provider = unsafe { Owned::new(provider) };
    let container = HSTRING::from(location.container.as_str());
    let mut handle = NCRYPT_KEY_HANDLE::default();
    // SAFETY: valid provider handle; `container` outlives the call. The
    // silent flag makes the provider fail instead of asking for the card.
    unsafe {
        NCryptOpenKey(
            *provider,
            &mut handle,
            &container,
            CERT_KEY_SPEC(location.key_spec),
            NCRYPT_SILENT_FLAG,
        )
    }
    .ok()?;
    // SAFETY: just opened by us; nothing else frees it.
    let key = unsafe { NcryptKey::new(handle, true) };
    let silent = OBJECT_SECURITY_INFORMATION(NCRYPT_SILENT_FLAG.0);
    let mut len = 0u32;
    // SAFETY: size query on a valid key handle; no output buffer.
    unsafe {
        NCryptGetProperty(
            key.raw().into(),
            NCRYPT_READER_PROPERTY,
            None,
            &mut len,
            silent,
        )
    }
    .ok()?;
    let mut bytes = vec![0u8; len as usize];
    // SAFETY: the wrapper passes `bytes` with its real length.
    unsafe {
        NCryptGetProperty(
            key.raw().into(),
            NCRYPT_READER_PROPERTY,
            Some(&mut bytes),
            &mut len,
            silent,
        )
    }
    .ok()?;
    bytes.truncate(len as usize);
    let (pairs, _) = bytes.as_chunks::<2>();
    let wide: Vec<u16> = pairs
        .iter()
        .map(|&pair| u16::from_ne_bytes(pair))
        .take_while(|&unit| unit != 0)
        .collect();
    Some(String::from_utf16_lossy(&wide))
}

/// `PP_SMARTCARD_READER` of a CAPI key container (ANSI).
pub fn csp_reader(location: &KeyLocation) -> Option<String> {
    trace!("CryptAcquireContext(silent) + PP_SMARTCARD_READER on a smart card CSP container");
    let container = HSTRING::from(location.container.as_str());
    let provider_name = HSTRING::from(location.provider.as_str());
    let mut handle = 0usize;
    // SAFETY: both names are NUL-terminated and outlive the call;
    // CRYPT_SILENT makes the CSP fail instead of asking for the card.
    unsafe {
        CryptAcquireContextW(
            &mut handle,
            &container,
            &provider_name,
            location.provider_type,
            CRYPT_SILENT,
        )
    }
    .ok()?;
    // SAFETY: just acquired; released only by `context`.
    let context = unsafe { CryptProv::new(handle, true) };
    let mut len = 0u32;
    // SAFETY: size query on a valid context; no output buffer.
    unsafe { CryptGetProvParam(context.raw(), PP_SMARTCARD_READER.0, None, &mut len, 0) }.ok()?;
    let mut bytes = vec![0u8; len as usize];
    // SAFETY: `bytes` has `len` writable bytes.
    unsafe {
        CryptGetProvParam(
            context.raw(),
            PP_SMARTCARD_READER.0,
            Some(bytes.as_mut_ptr()),
            &mut len,
            0,
        )
    }
    .ok()?;
    bytes.truncate(len as usize);
    let text = bytes.split(|&byte| byte == 0).next().unwrap_or_default();
    Some(String::from_utf8_lossy(text).into_owned())
}
