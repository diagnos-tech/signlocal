//! Falling back from the CNG bridge to plain CAPI, per key.
//!
//! The store acquires keys with `PREFER_NCRYPT` so that keys in Microsoft's
//! CSPs sign through CNG, which adds RSASSA-PSS. Windows bridges only the
//! CSPs it knows: a third-party CSP may refuse to be opened or used through
//! CNG. Such a key is retried once with `ALLOW` (CAPI when the key is CAPI)
//! and remembered, so later signatures go straight to the path that works
//! instead of failing first every time.

use std::collections::HashSet;

use windows::Win32::Foundation::{NTE_BAD_PROVIDER, NTE_PROV_TYPE_NOT_DEF};
use windows::core::HRESULT;

use crate::{KeystoreError, NcryptPreference};

/// Locators whose key refused the CNG bridge.
#[derive(Debug, Default)]
pub struct BridgeRefusals(HashSet<String>);

impl BridgeRefusals {
    /// The preference to acquire `locator` with: the configured one, or
    /// `Allow` once the key has refused the bridge.
    pub fn preference(&self, locator: &str, configured: NcryptPreference) -> NcryptPreference {
        if configured == NcryptPreference::Prefer && self.0.contains(locator) {
            NcryptPreference::Allow
        } else {
            configured
        }
    }

    pub fn remember(&mut self, locator: &str) {
        self.0.insert(locator.to_owned());
    }
}

/// Whether a failure with `Prefer` on a key in a CSP (`dwProvType != 0`)
/// deserves one retry with `Allow`.
///
/// `NTE_BAD_PROVIDER` and `NTE_PROV_TYPE_NOT_DEF` stay native errors; the
/// "not supported" family (`NTE_NOT_SUPPORTED`,
/// `SCARD_E_UNSUPPORTED_FEATURE`) arrives already mapped to `Unsupported`.
/// Cancellation and PIN errors never retry: the user already answered.
pub fn should_retry_with_capi(
    preference: NcryptPreference,
    key_in_csp: bool,
    error: &KeystoreError,
) -> bool {
    const REFUSALS: [HRESULT; 2] = [NTE_BAD_PROVIDER, NTE_PROV_TYPE_NOT_DEF];
    if preference != NcryptPreference::Prefer || !key_in_csp {
        return false;
    }
    match error {
        KeystoreError::Unsupported(_) => true,
        KeystoreError::Native { code, .. } => REFUSALS
            .iter()
            .any(|refusal| i64::from(refusal.0 as u32) == *code),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use windows::Win32::Foundation::{NTE_BAD_KEYSET, NTE_BAD_PROVIDER, NTE_NOT_SUPPORTED};

    use super::{BridgeRefusals, should_retry_with_capi};
    use crate::windows::errors::from_code;
    use crate::{KeystoreError, NcryptPreference};

    const PREFER: NcryptPreference = NcryptPreference::Prefer;

    #[test]
    fn retries_bridge_refusals_of_csp_keys_with_prefer() {
        let refused = from_code("CryptAcquireCertificatePrivateKey", NTE_BAD_PROVIDER);
        assert!(should_retry_with_capi(PREFER, true, &refused));
        let unsupported = from_code("NCryptSignHash", NTE_NOT_SUPPORTED);
        assert!(should_retry_with_capi(PREFER, true, &unsupported));
    }

    #[test]
    fn never_retries_cng_keys_other_modes_or_user_answers() {
        let refused = from_code("x", NTE_BAD_PROVIDER);
        assert!(!should_retry_with_capi(PREFER, false, &refused));
        assert!(!should_retry_with_capi(
            NcryptPreference::Only,
            true,
            &refused
        ));
        assert!(!should_retry_with_capi(
            NcryptPreference::Allow,
            true,
            &refused
        ));
        for error in [KeystoreError::Cancelled, KeystoreError::WrongPin] {
            assert!(!should_retry_with_capi(PREFER, true, &error));
        }
        assert!(!should_retry_with_capi(
            PREFER,
            true,
            &from_code("x", NTE_BAD_KEYSET)
        ));
    }

    #[test]
    fn remembered_keys_use_allow_only_when_prefer_is_configured() {
        let mut refusals = BridgeRefusals::default();
        refusals.remember("aa");
        assert_eq!(refusals.preference("aa", PREFER), NcryptPreference::Allow);
        assert_eq!(refusals.preference("bb", PREFER), PREFER);
        assert_eq!(
            refusals.preference("aa", NcryptPreference::Only),
            NcryptPreference::Only
        );
    }
}
