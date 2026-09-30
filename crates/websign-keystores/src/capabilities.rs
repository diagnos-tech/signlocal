//! What a key store can produce with a key, independent of the key's type.
//!
//! The certificate says which algorithms a key could do (RSA → PKCS#1 v1.5
//! and PSS, EC → ECDSA); the store decides which of them it can actually
//! run: legacy CAPI has no PSS, a PKCS#11 token may lack `CKM_RSA_PKCS_PSS`,
//! the Keychain refuses brainpool keys. The host offers the intersection, so
//! a caller never picks an algorithm that fails after the PIN was typed.

use websign_core::SignatureAlgorithm;

/// The signature algorithms a store can produce with one key, read without
/// prompting (mechanism lists, provider types, `SecKeyIsAlgorithmSupported`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyCapabilities {
    ecdsa: bool,
    rsa_pkcs1v15: bool,
    rsa_pss: bool,
}

impl KeyCapabilities {
    /// Everything; what a source that cannot tell reports, leaving the
    /// decision to the key type.
    pub const ALL: KeyCapabilities = KeyCapabilities {
        ecdsa: true,
        rsa_pkcs1v15: true,
        rsa_pss: true,
    };

    /// Nothing: the store cannot sign with this key at all.
    pub const NONE: KeyCapabilities = KeyCapabilities {
        ecdsa: false,
        rsa_pkcs1v15: false,
        rsa_pss: false,
    };

    /// Exactly the algorithms for which `can` answers `true`.
    pub fn from_fn(mut can: impl FnMut(SignatureAlgorithm) -> bool) -> KeyCapabilities {
        KeyCapabilities {
            ecdsa: can(SignatureAlgorithm::Ecdsa),
            rsa_pkcs1v15: can(SignatureAlgorithm::RsaPkcs1v15),
            rsa_pss: can(SignatureAlgorithm::RsaPss),
        }
    }

    /// Exactly `algorithms`.
    pub fn of(algorithms: &[SignatureAlgorithm]) -> KeyCapabilities {
        KeyCapabilities::from_fn(|algorithm| algorithms.contains(&algorithm))
    }

    pub fn supports(self, algorithm: SignatureAlgorithm) -> bool {
        match algorithm {
            SignatureAlgorithm::Ecdsa => self.ecdsa,
            SignatureAlgorithm::RsaPkcs1v15 => self.rsa_pkcs1v15,
            SignatureAlgorithm::RsaPss => self.rsa_pss,
        }
    }

    /// The supported algorithms, in [`SignatureAlgorithm::ALL`] order.
    pub fn algorithms(self) -> Vec<SignatureAlgorithm> {
        SignatureAlgorithm::ALL
            .into_iter()
            .filter(|&algorithm| self.supports(algorithm))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use SignatureAlgorithm::{Ecdsa, RsaPkcs1v15, RsaPss};

    #[test]
    fn lists_what_it_was_built_from_in_canonical_order() {
        let capabilities = KeyCapabilities::of(&[RsaPss, RsaPkcs1v15]);
        assert_eq!(capabilities.algorithms(), [RsaPkcs1v15, RsaPss]);
        assert!(!capabilities.supports(Ecdsa));
    }

    #[test]
    fn all_and_none_are_the_extremes() {
        assert_eq!(KeyCapabilities::ALL.algorithms(), SignatureAlgorithm::ALL);
        assert!(KeyCapabilities::NONE.algorithms().is_empty());
        assert_eq!(KeyCapabilities::from_fn(|_| true), KeyCapabilities::ALL);
    }
}
