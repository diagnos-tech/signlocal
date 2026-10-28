//! DigestLengthError.

use super::*;

#[test]
fn formats_digest_length_error() {
    let err = Sha256.check_digest(&[0; 31]).unwrap_err();
    assert_eq!(
        err.to_string(),
        "digest for SHA-256 must be 32 bytes, got 31"
    );
    let err = Sha384.check_digest(&[0; 32]).unwrap_err();
    assert_eq!(
        err.to_string(),
        "digest for SHA-384 must be 48 bytes, got 32"
    );
    let err = Sha512.check_digest(&[]).unwrap_err();
    assert_eq!(
        err.to_string(),
        "digest for SHA-512 must be 64 bytes, got 0"
    );
}

#[test]
fn digest_length_error_is_a_cloneable_comparable_std_error() {
    fn assert_error<E: std::error::Error + Clone + PartialEq + Send + Sync + 'static>() {}
    assert_error::<DigestLengthError>();
    let err = Sha256.check_digest(&[0; 3]).unwrap_err();
    assert_eq!(err.clone(), err);
    assert!(format!("{err:?}").contains("DigestLengthError"));
}
