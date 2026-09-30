//! What a keychain or CryptoTokenKit key can sign with, asked of the key
//! itself with `SecKeyIsAlgorithmSupported` (for token keys, of the token
//! driver), which never prompts.
//!
//! The Security framework has no brainpool curves, so a brainpool key in the
//! Keychain answers "no" to every ECDSA algorithm and is not offered; its
//! token's PKCS#11 module, when installed, remains an alternate path.

use security_framework::identity::SecIdentity;
use websign_core::{HashAlgorithm, SignatureAlgorithm};

use super::{algorithm, sign};
use crate::KeyCapabilities;

/// What `identity`'s private key can sign; nothing when the key cannot be
/// read.
pub fn of_identity(identity: &SecIdentity) -> KeyCapabilities {
    let Ok(key) = identity.private_key() else {
        return KeyCapabilities::NONE;
    };
    from_answers(|hash, algorithm| sign::supports(&key, algorithm::for_digest(hash, algorithm)))
}

/// An algorithm counts only when the key signs it with every hash: the
/// algorithm is offered before the caller's hash is known.
fn from_answers(
    mut supported: impl FnMut(HashAlgorithm, SignatureAlgorithm) -> bool,
) -> KeyCapabilities {
    KeyCapabilities::from_fn(|algorithm| {
        HashAlgorithm::ALL
            .into_iter()
            .all(|hash| supported(hash, algorithm))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use SignatureAlgorithm::{Ecdsa, RsaPkcs1v15, RsaPss};

    #[test]
    fn a_key_that_refuses_every_ecdsa_digest_offers_no_ecdsa() {
        let capabilities = from_answers(|_, algorithm| algorithm != Ecdsa);
        assert_eq!(capabilities.algorithms(), [RsaPkcs1v15, RsaPss]);
    }

    #[test]
    fn an_algorithm_missing_one_hash_is_not_offered() {
        let capabilities =
            from_answers(|hash, algorithm| !(algorithm == RsaPss && hash == HashAlgorithm::Sha512));
        assert!(!capabilities.supports(RsaPss));
        assert!(capabilities.supports(RsaPkcs1v15));
    }
}
