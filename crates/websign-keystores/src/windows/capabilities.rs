//! What a key in the Windows store can sign with, decided from where the key
//! lives (`CERT_KEY_PROV_INFO`) and how it will be acquired, never by
//! opening it.
//!
//! CNG keys sign PKCS#1 v1.5, PSS and ECDSA. A key in a legacy CSP reaches
//! CNG only through Windows' bridge, which covers Microsoft's own CSPs; any
//! other CSP, a CSP key acquired with `Allow`, and a key that fell back to
//! `Allow` after refusing the bridge (D9) sign through plain CAPI, which has
//! PKCS#1 v1.5 only.

use super::key_info::KeyLocation;
use crate::{KeyCapabilities, NcryptPreference};
use websign_core::SignatureAlgorithm;

/// Microsoft CSPs that `CryptAcquireCertificatePrivateKey` opens as CNG keys
/// under `PREFER_NCRYPT` (the kit proved it for the software CSPs; the smart
/// card CSP maps to the Smart Card KSP). An empty name is the system default
/// CSP, which is one of them.
const BRIDGED_CSPS: [&str; 5] = [
    "Microsoft Base Cryptographic Provider v1.0",
    "Microsoft Enhanced Cryptographic Provider v1.0",
    "Microsoft Strong Cryptographic Provider",
    "Microsoft Enhanced RSA and AES Cryptographic Provider",
    "Microsoft Base Smart Card Crypto Provider",
];

/// What the key at `location` can sign with when acquired with `preference`
/// (the configured one, or `Allow` once the key refused the bridge).
pub fn for_key(location: &KeyLocation, preference: NcryptPreference) -> KeyCapabilities {
    use SignatureAlgorithm::{RsaPkcs1v15, RsaPss};
    if location.is_cng() {
        return KeyCapabilities::ALL;
    }
    let bridged = location.provider.is_empty()
        || BRIDGED_CSPS
            .iter()
            .any(|name| name.eq_ignore_ascii_case(&location.provider));
    match (preference, bridged) {
        (NcryptPreference::Prefer | NcryptPreference::Only, true) => {
            KeyCapabilities::of(&[RsaPkcs1v15, RsaPss])
        }
        (NcryptPreference::Only, false) => KeyCapabilities::NONE,
        (NcryptPreference::Allow, _) | (NcryptPreference::Prefer, false) => {
            KeyCapabilities::of(&[RsaPkcs1v15])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use SignatureAlgorithm::{Ecdsa, RsaPkcs1v15, RsaPss};

    fn location(provider: &str, provider_type: u32) -> KeyLocation {
        KeyLocation {
            provider: provider.to_owned(),
            provider_type,
            container: String::new(),
            key_spec: 2,
            machine_keyset: false,
        }
    }

    #[test]
    fn cng_keys_can_do_everything_their_type_allows() {
        let ksp = location("Microsoft Software Key Storage Provider", 0);
        for preference in [
            NcryptPreference::Allow,
            NcryptPreference::Prefer,
            NcryptPreference::Only,
        ] {
            assert_eq!(
                for_key(&ksp, preference).algorithms(),
                [Ecdsa, RsaPkcs1v15, RsaPss]
            );
        }
    }

    #[test]
    fn microsoft_csps_gain_pss_through_the_bridge_only() {
        let csp = location("Microsoft Enhanced Cryptographic Provider v1.0", 1);
        assert_eq!(
            for_key(&csp, NcryptPreference::Prefer).algorithms(),
            [RsaPkcs1v15, RsaPss]
        );
        assert_eq!(
            for_key(&csp, NcryptPreference::Allow).algorithms(),
            [RsaPkcs1v15]
        );
        let default = location("", 24);
        assert!(for_key(&default, NcryptPreference::Prefer).supports(RsaPss));
    }

    #[test]
    fn third_party_csps_sign_pkcs1_only_and_nothing_under_only() {
        let vendor = location("SafeSign Standard Cryptographic Service Provider", 1);
        assert_eq!(
            for_key(&vendor, NcryptPreference::Prefer).algorithms(),
            [RsaPkcs1v15]
        );
        assert_eq!(
            for_key(&vendor, NcryptPreference::Only),
            KeyCapabilities::NONE
        );
    }
}
