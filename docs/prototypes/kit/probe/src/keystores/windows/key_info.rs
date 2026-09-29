//! Where a certificate's private key lives, read from the certificate's
//! `CERT_KEY_PROV_INFO` property and from provider metadata only, so that
//! listing never opens a key, never touches a card and never prompts.

use std::collections::HashMap;

use windows::Win32::Security::Cryptography::{
    AT_KEYEXCHANGE, AT_SIGNATURE, CERT_KEY_PROV_INFO_PROP_ID, CRYPT_IMPL_HARDWARE,
    CRYPT_IMPL_REMOVABLE, CRYPT_KEY_PROV_INFO, CRYPT_MACHINE_KEYSET, CRYPT_SILENT,
    CRYPT_VERIFYCONTEXT, CryptAcquireContextW, CryptGetProvParam, NCRYPT_IMPL_TYPE_PROPERTY,
    NCRYPT_PROV_HANDLE, NCRYPT_SILENT_FLAG, NCryptGetProperty, NCryptOpenStorageProvider,
    PP_IMPTYPE,
};
use windows::Win32::Security::OBJECT_SECURITY_INFORMATION;
use windows::core::{HSTRING, Owned, PCWSTR, PWSTR};

use super::handles::CryptProv;
use super::store::CertContext;

/// The key container a certificate points to.
///
/// Certificates with only an in-memory CNG handle (`CERT_NCRYPT_KEY_HANDLE_PROP_ID`)
/// are not covered: that property is never persisted, so it cannot appear
/// in a store this process has just opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyLocation {
    /// KSP (CNG) or CSP (CAPI) name; empty means the system default.
    pub provider: String,
    /// CAPI provider type (`PROV_RSA_FULL`, `PROV_RSA_AES`, ...); 0 for CNG.
    pub provider_type: u32,
    /// Container name. Never shown: some middlewares name it after the holder.
    pub container: String,
    /// `AT_KEYEXCHANGE` or `AT_SIGNATURE` for CAPI keys.
    pub key_spec: u32,
    pub machine_keyset: bool,
}

impl KeyLocation {
    /// `None` when the certificate has no associated private key.
    pub fn of(cert: &CertContext) -> Option<Self> {
        let property = cert.property(CERT_KEY_PROV_INFO_PROP_ID)?;
        if property.bytes().len() < size_of::<CRYPT_KEY_PROV_INFO>() {
            return None;
        }
        // SAFETY: the buffer is 8-byte aligned and starts with the structure;
        // its string pointers point into the same buffer, alive until return.
        let info = unsafe { &*property.as_ptr().cast::<CRYPT_KEY_PROV_INFO>() };
        Some(Self {
            // SAFETY: as above.
            provider: unsafe { wide(info.pwszProvName) },
            provider_type: info.dwProvType,
            // SAFETY: as above.
            container: unsafe { wide(info.pwszContainerName) },
            key_spec: info.dwKeySpec,
            machine_keyset: info.dwFlags.0 & CRYPT_MACHINE_KEYSET.0 != 0,
        })
    }

    pub fn is_cng(&self) -> bool {
        self.provider_type == 0
    }

    /// For people: provider name, API, CAPI type and key spec, and the
    /// reader when the container name is fully qualified (`\\.\reader\...`).
    pub fn describe(&self) -> String {
        let name = if self.provider.is_empty() {
            "default provider"
        } else {
            &self.provider
        };
        let mut text = if self.is_cng() {
            format!("{name} [CNG]")
        } else {
            let spec = match self.key_spec {
                spec if spec == AT_SIGNATURE.0 => "AT_SIGNATURE".to_owned(),
                spec if spec == AT_KEYEXCHANGE.0 => "AT_KEYEXCHANGE".to_owned(),
                spec => format!("key spec {spec}"),
            };
            format!("{name} [CAPI type {}, {spec}]", self.provider_type)
        };
        if let Some(reader) = reader(&self.container) {
            text.push_str(&format!(", reader {reader}"));
        }
        text
    }
}

/// Reader name from a fully qualified smart card container name.
fn reader(container: &str) -> Option<&str> {
    let rest = container.strip_prefix(r"\\.\")?;
    let reader = rest.split('\\').next()?;
    (!reader.is_empty()).then_some(reader)
}

/// Copies a NUL-terminated UTF-16 string; empty for null.
///
/// # Safety
/// `text` must be null or point to a live NUL-terminated string.
unsafe fn wide(text: PWSTR) -> String {
    if text.is_null() {
        return String::new();
    }
    // SAFETY: guaranteed by the caller.
    String::from_utf16_lossy(unsafe { text.as_wide() })
}

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
    use super::{KeyLocation, hardware_by_name, reader};

    fn location(provider: &str, provider_type: u32, key_spec: u32, container: &str) -> KeyLocation {
        KeyLocation {
            provider: provider.to_owned(),
            provider_type,
            container: container.to_owned(),
            key_spec,
            machine_keyset: false,
        }
    }

    #[test]
    fn describes_cng_and_capi_keys_without_the_container() {
        let cng = location(
            "Microsoft Software Key Storage Provider",
            0,
            0,
            "holder name",
        );
        assert_eq!(
            cng.describe(),
            "Microsoft Software Key Storage Provider [CNG]"
        );
        let capi = location("Microsoft Enhanced Cryptographic Provider v1.0", 1, 1, "x");
        assert_eq!(
            capi.describe(),
            "Microsoft Enhanced Cryptographic Provider v1.0 [CAPI type 1, AT_KEYEXCHANGE]"
        );
        let card = location("Vendor CSP", 1, 2, r"\\.\ACS ACR38U 0\c1");
        assert_eq!(
            card.describe(),
            "Vendor CSP [CAPI type 1, AT_SIGNATURE], reader ACS ACR38U 0"
        );
    }

    #[test]
    fn reader_needs_a_fully_qualified_container() {
        assert_eq!(reader(r"\\.\Reader 0\"), Some("Reader 0"));
        assert_eq!(reader(r"\\.\\x"), None);
        assert_eq!(reader("le-1234"), None);
    }

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
