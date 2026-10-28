//! Which CryptoTokenKit token, if any, holds a private key.

use core_foundation::base::{CFType, TCFType};
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;
use security_framework::key::SecKey;
use security_framework_sys::item::kSecAttrTokenID;
use security_framework_sys::key::SecKeyCopyAttributes;

/// `kSecAttrTokenID` of `key`, e.g. `com.apple.pivtoken:3F2A…`; `None` for a
/// key stored in a keychain file.
pub fn token_id(key: &SecKey) -> Option<String> {
    // SAFETY: `key` keeps the SecKeyRef alive for the call. The result follows
    // the Create rule and may be NULL, which is checked before wrapping.
    let raw = unsafe { SecKeyCopyAttributes(key.as_concrete_TypeRef()) };
    if raw.is_null() {
        return None;
    }
    // SAFETY: `raw` is a non-NULL CFDictionary we own; the wrapper takes over
    // that single reference. Keychain attribute keys are always CFStrings.
    let attributes: CFDictionary<CFString, CFType> =
        unsafe { CFDictionary::wrap_under_create_rule(raw) };
    // SAFETY: an immutable CFString constant exported by Security.framework.
    let name = unsafe { kSecAttrTokenID };
    attributes
        .find(name)
        .and_then(|value| value.downcast::<CFString>())
        .map(|id| id.to_string())
}

/// The driver part of a token ID: `com.apple.pivtoken:3F2A…` →
/// `com.apple.pivtoken`.
///
/// It names the middleware that exposed the card, which is what diagnostics
/// need. The instance part after `:` is left out because drivers often build
/// it from the card's serial number.
pub fn driver(token_id: &str) -> &str {
    token_id
        .split_once(':')
        .map_or(token_id, |(driver, _instance)| driver)
}

#[cfg(test)]
mod tests {
    use super::driver;

    #[test]
    fn driver_drops_the_instance_part() {
        assert_eq!(driver("com.apple.pivtoken:3F2A00"), "com.apple.pivtoken");
        assert_eq!(
            driver("org.opensc-project.mac.opensctoken.OpenSCTokenApp.OpenSCToken:ab:cd"),
            "org.opensc-project.mac.opensctoken.OpenSCTokenApp.OpenSCToken"
        );
    }

    #[test]
    fn driver_keeps_ids_without_instance() {
        assert_eq!(driver("com.apple.setoken"), "com.apple.setoken");
    }
}
