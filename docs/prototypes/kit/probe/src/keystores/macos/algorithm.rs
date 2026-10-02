//! `SecKeyAlgorithm` for each (hash, signature algorithm) pair.
//!
//! Only the "Digest" variants: the caller hashed already, and the key must
//! sign those exact bytes. For RSASSA-PSS Apple fixes MGF1 to the same hash
//! and the salt length to the digest length (`SecKey.h`; the algorithm IDs
//! end in the salt length, see the tests), which is what the SDK promises.

use probe_core::{HashAlgorithm, SignatureAlgorithm};
use security_framework::key::Algorithm;

/// The Security.framework algorithm that signs a `hash` digest with `algorithm`.
pub fn for_digest(hash: HashAlgorithm, algorithm: SignatureAlgorithm) -> Algorithm {
    use HashAlgorithm::{Sha256, Sha384, Sha512};
    use SignatureAlgorithm::{Ecdsa, RsaPkcs1v15, RsaPss};

    match (algorithm, hash) {
        (Ecdsa, Sha256) => Algorithm::ECDSASignatureDigestX962SHA256,
        (Ecdsa, Sha384) => Algorithm::ECDSASignatureDigestX962SHA384,
        (Ecdsa, Sha512) => Algorithm::ECDSASignatureDigestX962SHA512,
        (RsaPkcs1v15, Sha256) => Algorithm::RSASignatureDigestPKCS1v15SHA256,
        (RsaPkcs1v15, Sha384) => Algorithm::RSASignatureDigestPKCS1v15SHA384,
        (RsaPkcs1v15, Sha512) => Algorithm::RSASignatureDigestPKCS1v15SHA512,
        (RsaPss, Sha256) => Algorithm::RSASignatureDigestPSSSHA256,
        (RsaPss, Sha384) => Algorithm::RSASignatureDigestPSSSHA384,
        (RsaPss, Sha512) => Algorithm::RSASignatureDigestPSSSHA512,
    }
}

#[cfg(test)]
mod tests {
    use core_foundation::base::TCFType;
    use core_foundation::string::{CFString, CFStringRef};

    use super::*;

    /// The identifier Security.framework gives an algorithm, e.g.
    /// `algid:sign:RSA:digest-PSS:SHA256:SHA256:32`.
    fn algorithm_id(algorithm: Algorithm) -> String {
        let raw: CFStringRef = algorithm.into();
        // SAFETY: the constant is an immutable CFString owned by
        // Security.framework; the get rule retains it for the wrapper's life.
        unsafe { CFString::wrap_under_get_rule(raw) }.to_string()
    }

    #[test]
    fn every_pair_maps_to_the_documented_system_algorithm() {
        use HashAlgorithm::{Sha256, Sha384, Sha512};
        use SignatureAlgorithm::{Ecdsa, RsaPkcs1v15, RsaPss};

        let expected = [
            (Sha256, Ecdsa, "algid:sign:ECDSA:digest-X962:SHA256"),
            (Sha384, Ecdsa, "algid:sign:ECDSA:digest-X962:SHA384"),
            (Sha512, Ecdsa, "algid:sign:ECDSA:digest-X962:SHA512"),
            (Sha256, RsaPkcs1v15, "algid:sign:RSA:digest-PKCS1v15:SHA256"),
            (Sha384, RsaPkcs1v15, "algid:sign:RSA:digest-PKCS1v15:SHA384"),
            (Sha512, RsaPkcs1v15, "algid:sign:RSA:digest-PKCS1v15:SHA512"),
            // Evidence for the PSS parameters: the last two fields are the
            // MGF1 hash and the salt length in bytes (= digest length).
            (Sha256, RsaPss, "algid:sign:RSA:digest-PSS:SHA256:SHA256:32"),
            (Sha384, RsaPss, "algid:sign:RSA:digest-PSS:SHA384:SHA384:48"),
            (Sha512, RsaPss, "algid:sign:RSA:digest-PSS:SHA512:SHA512:64"),
        ];
        for (hash, algorithm, id) in expected {
            assert_eq!(
                algorithm_id(for_digest(hash, algorithm)),
                id,
                "{hash} {algorithm}"
            );
        }
    }
}
