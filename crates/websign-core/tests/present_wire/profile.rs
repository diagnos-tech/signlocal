//! Key storage, the whole profile and the total mappings.

use super::*;

#[test]
fn maps_the_key_store_answer_to_key_storage() {
    let info = blank_info();
    assert_eq!(
        certificate_profile(&info, Some(true)).key_storage,
        KeyStorage::Hardware
    );
    assert_eq!(
        certificate_profile(&info, Some(false)).key_storage,
        KeyStorage::Software
    );
    assert_eq!(
        certificate_profile(&info, None).key_storage,
        KeyStorage::Unknown
    );
}

#[test]
fn builds_the_whole_profile_of_a_qualified_icp_certificate() {
    let mut info = icp_info(Some(IcpLevel::A3));
    info.qualified = Some(Qualified {
        compliance: true,
        sscd: true,
        types: vec![QcType::ESign],
    });
    assert_eq!(
        certificate_profile(&info, Some(true)),
        CertificateProfile {
            icp_brasil: Some("A3".to_owned()),
            eidas: eidas(true, true, vec![EidasType::Esign]),
            key_storage: KeyStorage::Hardware,
        }
    );
}

#[test]
fn an_ordinary_certificate_has_only_the_storage() {
    assert_eq!(
        certificate_profile(&blank_info(), None),
        CertificateProfile {
            icp_brasil: None,
            eidas: None,
            key_storage: KeyStorage::Unknown,
        }
    );
}

#[test]
fn hash_and_algorithm_mappings_round_trip() {
    for hash in HashAlgorithm::ALL {
        assert_eq!(hash_algorithm(hash_name(hash)), hash);
    }
    for algorithm in SignatureAlgorithm::ALL {
        assert_eq!(signature_algorithm(algorithm_name(algorithm)), algorithm);
    }
}

#[test]
fn key_description_covers_supported_keys_only() {
    assert!(key_description(&PublicKeyKind::Rsa { bits: 2048 }).is_some());
    for curve in [
        Curve::P256,
        Curve::P384,
        Curve::P521,
        Curve::BrainpoolP256r1,
        Curve::BrainpoolP384r1,
        Curve::BrainpoolP512r1,
    ] {
        assert!(
            key_description(&PublicKeyKind::Ec { curve }).is_some(),
            "{curve:?}"
        );
        let _ = curve_name(curve);
    }
    let unsupported = PublicKeyKind::Unsupported {
        oid: "1.3.101.112".to_owned(),
    };
    assert_eq!(key_description(&unsupported), None);
}
