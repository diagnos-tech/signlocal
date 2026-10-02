//! Where a certificate's private key lives, read from the certificate's
//! `CERT_KEY_PROV_INFO` property only, so that listing never opens a key,
//! never touches a card and never prompts.

use windows::Win32::Security::Cryptography::{
    AT_KEYEXCHANGE, AT_SIGNATURE, CERT_KEY_PROV_INFO_PROP_ID, CRYPT_KEY_PROV_INFO,
    CRYPT_MACHINE_KEYSET,
};
use windows::core::PWSTR;

use super::cert_context::CertContext;
use websign_devices::anonymous_reader_name;

/// The key container a certificate points to.
///
/// Certificates with only an in-memory CNG handle (`CERT_NCRYPT_KEY_HANDLE_PROP_ID`)
/// are not covered: that property is never persisted, so it cannot appear
/// in a store this process has just opened.
#[derive(Clone, PartialEq, Eq)]
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

/// Leaves the container name out: it may carry the holder's name, and
/// `Debug` output ends up in logs.
impl std::fmt::Debug for KeyLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyLocation")
            .field("provider", &self.provider)
            .field("provider_type", &self.provider_type)
            .field("key_spec", &self.key_spec)
            .field("machine_keyset", &self.machine_keyset)
            .finish_non_exhaustive()
    }
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
        if let Some(reader) = self.qualified_reader() {
            text.push_str(&format!(", reader {}", anonymous_reader_name(reader)));
        }
        text
    }

    /// The reader named by a fully qualified smart card container name
    /// (`\\.\<reader>\<container>`), the only form that says where the
    /// card is without asking the card.
    pub fn qualified_reader(&self) -> Option<&str> {
        reader(&self.container)
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

#[cfg(test)]
mod tests {
    use super::{KeyLocation, reader};

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
}
