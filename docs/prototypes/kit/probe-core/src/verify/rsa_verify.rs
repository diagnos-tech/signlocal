//! RSA signature verification over a digest that is already computed.

use rsa::{BoxedUint, Pkcs1v15Sign, Pss, RsaPublicKey};
use sha2::{Sha256, Sha384, Sha512};

use super::VerifyError;
use crate::algorithm::SignatureAlgorithm;
use crate::cert::RsaComponents;
use crate::hash::HashAlgorithm;
use crate::pkcs1::digest_info_prefix;

/// Verifies an RSASSA-PKCS1-v1_5 or RSASSA-PSS signature over `digest`.
///
/// The signature must be exactly as long as the modulus and, read as a
/// big-endian integer, strictly below it (RFC 8017 §5.2.2 and §8.1.2). The
/// range check is done here because the PSS verifier of the `rsa` crate
/// reduces the signature modulo `n` instead of rejecting it, which would let
/// `s + n` pass as a second valid signature whenever it still fits the block.
pub(super) fn verify(
    key: RsaComponents<'_>,
    hash: HashAlgorithm,
    algorithm: SignatureAlgorithm,
    digest: &[u8],
    signature: &[u8],
) -> Result<(), VerifyError> {
    // `RsaPublicKey::new` validates the modulus and exponent (odd modulus,
    // exponent in range) and reports a bad pair as an error, never a panic.
    // SPEC: such a key is `UnsupportedKey`, like an EC point that is off-curve.
    let public_key = RsaPublicKey::new(
        BoxedUint::from_be_slice_vartime(key.modulus),
        BoxedUint::from_be_slice_vartime(key.exponent),
    )
    .map_err(|_| VerifyError::UnsupportedKey)?;

    // The modulus has no leading zeros, so its byte length is the block
    // size, and equal-length big-endian slices compare like the integers.
    if signature.len() != key.modulus.len() || signature >= key.modulus {
        return Err(VerifyError::InvalidSignature);
    }

    let outcome = match algorithm {
        SignatureAlgorithm::RsaPkcs1v15 => {
            let scheme = Pkcs1v15Sign {
                hash_len: Some(hash.digest_len()),
                prefix: digest_info_prefix(hash).into(),
            };
            public_key.verify(scheme, digest, signature)
        }
        SignatureAlgorithm::RsaPss => match hash {
            HashAlgorithm::Sha256 => public_key.verify(Pss::<Sha256>::new(), digest, signature),
            HashAlgorithm::Sha384 => public_key.verify(Pss::<Sha384>::new(), digest, signature),
            HashAlgorithm::Sha512 => public_key.verify(Pss::<Sha512>::new(), digest, signature),
        },
        // The caller has already checked that the key supports the algorithm.
        SignatureAlgorithm::Ecdsa => return Err(VerifyError::KeyMismatch(algorithm)),
    };
    outcome.map_err(|_| VerifyError::InvalidSignature)
}
