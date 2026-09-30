//! `VerifyError`.

use super::*;

#[test]
fn formats_each_error_variant() {
    let digest_err = Sha256.check_digest(&[0; 31]).unwrap_err();
    assert_eq!(
        VerifyError::Digest(digest_err).to_string(),
        "digest for SHA-256 must be 32 bytes, got 31"
    );
    assert_eq!(
        VerifyError::UnsupportedKey.to_string(),
        "unsupported public key"
    );
    assert_eq!(
        VerifyError::KeyMismatch(Ecdsa).to_string(),
        "ECDSA cannot be used with this key"
    );
    assert_eq!(
        VerifyError::KeyMismatch(RsaPss).to_string(),
        "RSASSA-PSS cannot be used with this key"
    );
    assert_eq!(
        VerifyError::KeyMismatch(RsaPkcs1v15).to_string(),
        "RSASSA-PKCS1-v1_5 cannot be used with this key"
    );
    assert_eq!(
        VerifyError::InvalidSignature.to_string(),
        "invalid signature"
    );
    let cert_err = CertError::Malformed("bad".to_owned());
    assert_eq!(
        VerifyError::Certificate(cert_err.clone()).to_string(),
        cert_err.to_string()
    );
}

#[test]
fn converts_from_the_errors_it_wraps() {
    let cert_err = CertError::Malformed("x".to_owned());
    assert_eq!(
        VerifyError::from(cert_err.clone()),
        VerifyError::Certificate(cert_err)
    );
    let digest_err = Sha512.check_digest(&[]).unwrap_err();
    assert_eq!(
        VerifyError::from(digest_err.clone()),
        VerifyError::Digest(digest_err)
    );
}

#[test]
fn verify_error_is_a_cloneable_comparable_std_error() {
    fn assert_error<E: std::error::Error + Clone + PartialEq + Send + Sync + 'static>() {}
    assert_error::<VerifyError>();
    assert_ne!(VerifyError::UnsupportedKey, VerifyError::InvalidSignature);
    assert_ne!(
        VerifyError::KeyMismatch(Ecdsa),
        VerifyError::KeyMismatch(RsaPss)
    );
}

#[test]
fn a_certificate_that_is_only_a_prefix_never_panics_verification() {
    let full = cert("rsa2048");
    let digest = fixture_digest(Sha256);
    let sig = signature("rsa2048", "pkcs1", Sha256);
    for len in (0..full.len()).step_by(7) {
        assert!(
            verify(&full[..len], Sha256, RsaPkcs1v15, &digest, &sig).is_err(),
            "prefix of {len} bytes"
        );
    }
}
