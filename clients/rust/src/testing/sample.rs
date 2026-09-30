//! The certificate the fake app offers unless told otherwise.

use websign_protocol::types::{
    Base64Bytes, Certificate, CertificateProfile, CurveName, FingerprintHex, KeyDescription,
    KeyStorage, SignatureAlgorithmName,
};

/// An ECDSA P-256 certificate on a token; every field is obviously fictional.
/// Start from it and override fields to build your own.
///
/// ```
/// use websign_client::testing::sample_certificate;
///
/// let mut certificate = sample_certificate();
/// certificate.display_name = "Ana Test".into();
/// assert_eq!(certificate.algorithms.len(), 1);
/// ```
pub fn sample_certificate() -> Certificate {
    Certificate {
        der: Base64Bytes::new(vec![1, 2, 3]),
        chain: Vec::new(),
        fingerprint: sample_fingerprint(),
        display_name: "Test Holder".into(),
        issuer_name: "Test CA".into(),
        not_before: 1_741_000_000,
        not_after: 4_102_444_800,
        key: KeyDescription::Ec {
            curve: CurveName::P256,
        },
        algorithms: vec![SignatureAlgorithmName::Ecdsa],
        profile: CertificateProfile {
            icp_brasil: None,
            eidas: None,
            key_storage: KeyStorage::Hardware,
        },
    }
}

/// 64 lowercase hex digits, so the constructor cannot refuse; a unit test
/// keeps it that way.
#[allow(clippy::expect_used)]
fn sample_fingerprint() -> FingerprintHex {
    FingerprintHex::new("ab".repeat(32)).expect("a constant of 64 hex digits")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sample_certificate_builds() {
        assert_eq!(sample_certificate().fingerprint.as_str().len(), 64);
    }
}
