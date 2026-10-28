use super::*;

#[test]
fn rsa_key_sizes_are_exact_bit_lengths() {
    for bits in [1024, 2047, 2048, 3072, 4096] {
        let info = parse(&TestCert::new().spki(spki_rsa(bits)));
        assert_eq!(info.key, PublicKeyKind::Rsa { bits: bits as u32 }, "{bits}");
    }
}

#[test]
fn ec_curves_are_recognised_by_name() {
    for (oid, curve) in [
        ("1.2.840.10045.3.1.7", Curve::P256),
        ("1.3.132.0.34", Curve::P384),
        ("1.3.132.0.35", Curve::P521),
    ] {
        let info = parse(&TestCert::new().spki(spki_ec(oid)));
        assert_eq!(info.key, PublicKeyKind::Ec { curve });
    }
}

#[test]
fn unknown_curves_and_algorithms_are_unsupported_with_their_oid() {
    // secp256k1
    let info = parse(&TestCert::new().spki(spki_ec("1.3.132.0.10")));
    assert_eq!(
        info.key,
        PublicKeyKind::Unsupported {
            oid: "1.3.132.0.10".into()
        }
    );
    // Ed25519
    let info = parse(&TestCert::new().spki(spki_other("1.3.101.112")));
    assert_eq!(
        info.key,
        PublicKeyKind::Unsupported {
            oid: "1.3.101.112".into()
        }
    );
}

#[test]
fn ec_key_without_a_named_curve_is_reported_under_the_algorithm() {
    // id-ecPublicKey with NULL parameters instead of a curve OID.
    let algorithm = sequence(&[oid("1.2.840.10045.2.1"), tlv(0x05, &[])]);
    let spki = sequence(&[algorithm, tlv(0x03, &[0x00, 0x04, 0x01])]);
    let info = parse(&TestCert::new().spki(spki));
    assert_eq!(
        info.key,
        PublicKeyKind::Unsupported {
            oid: "1.2.840.10045.2.1".into()
        }
    );
}

#[test]
fn rsa_pss_keys_count_as_rsa() {
    let key = sequence(&[tlv(0x02, &[0x00, 0xc1, 0x02, 0x03]), tlv(0x02, &[0x03])]);
    let algorithm = sequence(&[oid("1.2.840.113549.1.1.10")]);
    let spki = sequence(&[algorithm, tlv(0x03, &[&[0u8][..], &key].concat())]);
    let info = parse(&TestCert::new().spki(spki));
    assert_eq!(info.key, PublicKeyKind::Rsa { bits: 24 });
}

#[test]
fn key_support_matrix() {
    let rsa = PublicKeyKind::Rsa { bits: 2048 };
    let ec = PublicKeyKind::Ec { curve: Curve::P256 };
    let other = PublicKeyKind::Unsupported {
        oid: "1.3.101.112".into(),
    };
    assert!(rsa.supports(SignatureAlgorithm::RsaPkcs1v15));
    assert!(rsa.supports(SignatureAlgorithm::RsaPss));
    assert!(!rsa.supports(SignatureAlgorithm::Ecdsa));
    assert!(ec.supports(SignatureAlgorithm::Ecdsa));
    assert!(!ec.supports(SignatureAlgorithm::RsaPkcs1v15));
    assert!(!ec.supports(SignatureAlgorithm::RsaPss));
    for algorithm in SignatureAlgorithm::ALL {
        assert!(!other.supports(algorithm));
    }
}
