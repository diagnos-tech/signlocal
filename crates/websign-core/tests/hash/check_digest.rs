//! Check_digest.

use super::*;

#[test]
fn accepts_digest_of_exactly_the_declared_length() {
    for hash in HashAlgorithm::ALL {
        assert_eq!(
            hash.check_digest(&vec![0; hash.digest_len()]),
            Ok(()),
            "{hash}"
        );
        assert_eq!(
            hash.check_digest(&vec![0xFF; hash.digest_len()]),
            Ok(()),
            "{hash}"
        );
    }
}

#[test]
fn rejects_every_length_but_the_declared_one() {
    for hash in HashAlgorithm::ALL {
        for len in 0..=130 {
            let result = hash.check_digest(&vec![0x5A; len]);
            if len == hash.digest_len() {
                assert_eq!(result, Ok(()), "{hash} with {len} bytes");
            } else {
                assert_eq!(
                    result,
                    Err(DigestLengthError {
                        algorithm: hash,
                        expected: hash.digest_len(),
                        actual: len,
                    }),
                    "{hash} with {len} bytes"
                );
            }
        }
    }
}

#[test]
fn rejects_empty_digest() {
    let err = Sha256.check_digest(&[]).unwrap_err();
    assert_eq!(err.actual, 0);
    assert_eq!(err.expected, 32);
}

#[test]
fn rejects_digest_shorter_than_hash() {
    let err = Sha384.check_digest(&[0; 47]).unwrap_err();
    assert_eq!((err.algorithm, err.expected, err.actual), (Sha384, 48, 47));
}

#[test]
fn rejects_digest_longer_than_hash() {
    let err = Sha512.check_digest(&[0; 65]).unwrap_err();
    assert_eq!((err.algorithm, err.expected, err.actual), (Sha512, 64, 65));
}

#[test]
fn rejects_digest_of_another_hash() {
    for hash in HashAlgorithm::ALL {
        for other in HashAlgorithm::ALL {
            let digest = other.digest(b"x");
            assert_eq!(
                hash.check_digest(&digest).is_ok(),
                hash == other,
                "{hash} vs {other}"
            );
        }
    }
}
