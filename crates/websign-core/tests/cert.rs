//! SPEC §6 and §6.0: parsing `CertInfo` from DER, and `CertError`. Field
//! rules live in `cert_names`, `cert_key`, `cert_usage` and `cert_validity`;
//! ICP-Brasil and eIDAS in `cert_icp_brasil` and `cert_qualified`.

mod common;

use common::{Rng, cert, cert_names, fixture_fingerprint, info, leaf_dn};
use websign_core::{CertError, CertInfo, Curve, DistinguishedName, Fingerprint, PublicKeyKind};

#[test]
fn parses_every_well_formed_fixture() {
    let mut parsed = 0;
    for name in cert_names() {
        if name.starts_with("bad-") {
            continue;
        }
        let der = cert(&name);
        let info = CertInfo::from_der(&der).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(info.fingerprint, Fingerprint::of(&der), "{name}");
        parsed += 1;
    }
    assert!(parsed > 100, "fixture set went missing");
}

#[test]
fn fingerprint_is_the_sha256_of_the_der() {
    for name in ["rsa2048", "ca", "p521", "icp-pf-a3", "qc-esign-sscd"] {
        assert_eq!(info(name).fingerprint, fixture_fingerprint(name), "{name}");
    }
}

#[test]
fn cert_info_is_cloneable_and_comparable() {
    let a = info("rsa2048");
    assert_eq!(a.clone(), a);
    assert_ne!(a, info("rsa3072"));
    assert_eq!(info("rsa2048"), a, "parsing twice gives the same summary");
}

#[test]
fn version_1_certificate_without_extensions_has_neutral_defaults() {
    let info = info("v1");
    assert_eq!(info.subject, leaf_dn("v1"));
    assert_eq!(info.key_usage, None);
    assert!(info.extended_key_usage.is_empty());
    assert!(info.policies.is_empty());
    assert!(!info.is_ca);
    assert!(info.icp_brasil.is_none());
    assert!(info.qualified.is_none());
    assert!(info.can_sign());
}

#[test]
fn parses_a_tiny_certificate_framed_with_the_one_byte_long_length_form() {
    let der = cert("tiny");
    assert!(der.len() < 256, "fixture is meant to be under 256 bytes");
    assert_eq!(der[..2], [0x30, 0x81], "outer SEQUENCE uses 30 81 xx");
    let info = CertInfo::from_der(&der).expect("valid certificate");
    assert_eq!(info.subject, DistinguishedName::default());
    assert_eq!(info.issuer, DistinguishedName::default());
    assert_eq!(info.serial_hex, "01");
    assert_eq!(info.key, PublicKeyKind::Ec { curve: Curve::P256 });
    assert_eq!(info.key_usage, None);
    assert!(info.can_sign());
}

fn assert_malformed(input: &[u8], label: &str) {
    match CertInfo::from_der(input) {
        Err(CertError::Malformed(_)) => {}
        other => panic!("{label}: expected Malformed, got {other:?}"),
    }
}

#[test]
fn rejects_bytes_that_are_not_a_certificate() {
    let pem = b"-----BEGIN CERTIFICATE-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA\n-----END CERTIFICATE-----\n";
    let cases: Vec<(&str, Vec<u8>)> = vec![
        ("empty", vec![]),
        ("one byte", vec![0x30]),
        ("empty SEQUENCE", vec![0x30, 0x00]),
        (
            "SEQUENCE with one INTEGER",
            vec![0x30, 0x03, 0x02, 0x01, 0x00],
        ),
        (
            "SEQUENCE with three INTEGERs",
            vec![0x30, 0x09, 2, 1, 0, 2, 1, 0, 2, 1, 0],
        ),
        ("zeros", vec![0; 64]),
        ("ones", vec![0xFF; 64]),
        ("ASCII text", b"hello world".to_vec()),
        ("PEM instead of DER", pem.to_vec()),
        ("INTEGER", vec![0x02, 0x01, 0x05]),
        (
            "length beyond the data",
            vec![0x30, 0x82, 0xFF, 0xFF, 0x30, 0x00],
        ),
        ("indefinite length", vec![0x30, 0x80, 0x00, 0x00]),
    ];
    for (label, bytes) in cases {
        assert_malformed(&bytes, label);
    }
}

#[test]
fn rejects_every_truncation_of_a_certificate() {
    for name in ["p256", "icp-pf-a3"] {
        let der = cert(name);
        for len in 0..der.len() {
            assert_malformed(&der[..len], &format!("{name} cut at {len}"));
        }
    }
}

#[test]
fn rejects_bytes_after_the_certificate() {
    let der = cert("rsa2048");
    let tails: [&[u8]; 5] = [&[0], &[0, 0, 0, 0], &[0x30, 0x00], b"\n", &der];
    for tail in tails {
        let mut padded = der.clone();
        padded.extend_from_slice(tail);
        assert_malformed(&padded, &format!("{} trailing bytes", tail.len()));
    }
}

#[test]
fn rejects_a_certificate_rewritten_with_indefinite_length() {
    // A well-formed certificate re-framed as BER (30 80 ... 00 00) is not DER.
    let der = cert("p256");
    let header = if der[1] == 0x82 { 4 } else { 3 };
    let mut ber = vec![0x30, 0x80];
    ber.extend_from_slice(&der[header..]);
    ber.extend_from_slice(&[0, 0]);
    assert_malformed(&ber, "indefinite length");
}

#[test]
fn rejects_known_extensions_that_are_malformed() {
    for name in [
        "bad-ext-key-usage",
        "bad-ext-extended-key-usage",
        "bad-ext-policies",
        "bad-ext-basic-constraints",
        "bad-ext-san",
        "bad-ext-qc-statements",
        "bad-qc-statement-element",
    ] {
        assert_malformed(&cert(name), name);
    }
}

#[test]
fn malformed_error_message_is_prefixed_and_carries_detail() {
    let err = CertInfo::from_der(&[0x30]).unwrap_err();
    assert!(
        err.to_string().starts_with("malformed certificate: "),
        "{err}"
    );
    let CertError::Malformed(detail) = &err;
    assert_eq!(err.to_string(), format!("malformed certificate: {detail}"));
}

#[test]
fn cert_error_is_a_cloneable_comparable_std_error() {
    fn assert_error<E: std::error::Error + Clone + PartialEq + Send + Sync + 'static>() {}
    assert_error::<CertError>();
    let err = CertError::Malformed("x".to_owned());
    assert_eq!(err.clone(), err);
    assert_ne!(err, CertError::Malformed("y".to_owned()));
}

#[test]
fn survives_a_flipped_byte_anywhere_in_a_certificate() {
    for name in ["icp-pf-a3", "qc-esign-sscd", "dn-bmp", "eku-multi"] {
        let der = cert(name);
        for index in 0..der.len() {
            for mask in [0xFF, 0x01, 0x80] {
                let mut broken = der.clone();
                broken[index] ^= mask;
                // Either outcome is fine; panicking is not.
                let _ = CertInfo::from_der(&broken);
            }
        }
    }
}

#[test]
fn survives_random_garbage() {
    let mut rng = Rng::new(0xC0FFEE);
    for _ in 0..2000 {
        let len = rng.below(400) as usize;
        let mut bytes = rng.bytes(len);
        if let Some(first) = bytes.first_mut()
            && rng.below(2) == 0
        {
            *first = 0x30;
        }
        let _ = CertInfo::from_der(&bytes);
    }
}

#[test]
fn survives_a_valid_certificate_with_its_body_replaced_by_garbage() {
    let der = cert("p256");
    let mut rng = Rng::new(7);
    for _ in 0..300 {
        let mut broken = der.clone();
        let start = 4 + rng.below((der.len() - 8) as u64) as usize;
        let end = (start + 1 + rng.below(20) as usize).min(der.len());
        broken[start..end].copy_from_slice(&rng.bytes(end - start));
        let _ = CertInfo::from_der(&broken);
    }
}
