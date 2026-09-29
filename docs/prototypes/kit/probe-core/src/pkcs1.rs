//! PKCS#1 v1.5 `DigestInfo`, for signers that take an already-hashed input.

use crate::hash::{DigestLengthError, HashAlgorithm};

/// DER `DigestInfo` (RFC 8017 §9.2): the hash's fixed prefix followed by `digest`.
///
/// `CKM_RSA_PKCS` and other "raw" RSA signers pad whatever they receive, so
/// the caller must wrap the digest itself; hashing mechanisms such as
/// `CKM_SHA256_RSA_PKCS` would hash the digest a second time.
pub fn digest_info(hash: HashAlgorithm, digest: &[u8]) -> Result<Vec<u8>, DigestLengthError> {
    hash.check_digest(digest)?;
    let prefix = digest_info_prefix(hash);
    let mut out = Vec::with_capacity(prefix.len() + digest.len());
    out.extend_from_slice(prefix);
    out.extend_from_slice(digest);
    Ok(out)
}

/// The fixed DER bytes that precede the digest inside a `DigestInfo`.
///
/// Crate-private so the verifier can hand the exact same prefix to the RSA
/// implementation instead of keeping a second copy of these constants.
pub(crate) const fn digest_info_prefix(hash: HashAlgorithm) -> &'static [u8] {
    match hash {
        HashAlgorithm::Sha256 => &[
            0x30, 0x31, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02,
            0x01, 0x05, 0x00, 0x04, 0x20,
        ],
        HashAlgorithm::Sha384 => &[
            0x30, 0x41, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02,
            0x02, 0x05, 0x00, 0x04, 0x30,
        ],
        HashAlgorithm::Sha512 => &[
            0x30, 0x51, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02,
            0x03, 0x05, 0x00, 0x04, 0x40,
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixes_are_the_rfc_8017_constants() {
        let cases = [
            (
                HashAlgorithm::Sha256,
                "3031300d060960864801650304020105000420",
            ),
            (
                HashAlgorithm::Sha384,
                "3041300d060960864801650304020205000430",
            ),
            (
                HashAlgorithm::Sha512,
                "3051300d060960864801650304020305000440",
            ),
        ];
        for (hash, expected) in cases {
            assert_eq!(crate::hex::lower(digest_info_prefix(hash)), expected);
        }
    }

    #[test]
    fn digest_info_is_prefix_then_digest() {
        for hash in HashAlgorithm::ALL {
            let digest = hash.digest(b"payload");
            let info = digest_info(hash, &digest).unwrap();
            let prefix = digest_info_prefix(hash);
            assert_eq!(info.get(..prefix.len()), Some(prefix));
            assert_eq!(info.get(prefix.len()..), Some(digest.as_slice()));
            // The outer SEQUENCE length covers everything after its own header.
            assert_eq!(usize::from(info[1]) + 2, info.len());
        }
    }

    #[test]
    fn wrong_digest_length_is_rejected_before_wrapping() {
        let err = digest_info(HashAlgorithm::Sha384, &[0; 32]).unwrap_err();
        assert_eq!(err.expected, 48);
        assert_eq!(err.actual, 32);
        assert_eq!(err.algorithm, HashAlgorithm::Sha384);
    }
}
