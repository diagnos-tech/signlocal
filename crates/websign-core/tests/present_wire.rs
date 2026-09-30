//! SPEC §13: `present::wire::certificate_profile`, plus a round trip over the
//! total mappings that were already in place.

mod common;

use common::{blank_info, info};
use websign_core::present::wire::{
    algorithm_name, certificate_profile, curve_name, hash_algorithm, hash_name, key_description,
    signature_algorithm,
};
use websign_core::{
    CertInfo, Curve, HashAlgorithm, IcpBrasil, IcpLevel, PublicKeyKind, QcType, Qualified,
    SignatureAlgorithm,
};
use websign_protocol::types::{CertificateProfile, EidasProfile, EidasType, KeyStorage};

fn icp_info(level: Option<IcpLevel>) -> CertInfo {
    let mut info = blank_info();
    info.icp_brasil = Some(IcpBrasil {
        level,
        holder_name: None,
        cpf: None,
        cnpj: None,
    });
    info
}

fn qualified_info(compliance: bool, sscd: bool, types: Vec<QcType>) -> CertInfo {
    let mut info = blank_info();
    info.qualified = Some(Qualified {
        compliance,
        sscd,
        types,
    });
    info
}

fn eidas(qualified: bool, qscd: bool, types: Vec<EidasType>) -> Option<EidasProfile> {
    Some(EidasProfile {
        qualified,
        qscd,
        types,
    })
}

// --- icp_brasil ----------------------------------------------------------------------------------

#[test]
fn a_certificate_that_is_not_icp_brasil_has_no_icp_profile() {
    let profile = certificate_profile(&blank_info(), None);
    assert_eq!(profile.icp_brasil, None);
}

#[test]
fn names_the_icp_brasil_class() {
    let levels = [
        (IcpLevel::A1, "A1"),
        (IcpLevel::A2, "A2"),
        (IcpLevel::A3, "A3"),
        (IcpLevel::A4, "A4"),
        (IcpLevel::S1, "S1"),
        (IcpLevel::S2, "S2"),
        (IcpLevel::S3, "S3"),
        (IcpLevel::S4, "S4"),
        (IcpLevel::T3, "T3"),
        (IcpLevel::T4, "T4"),
    ];
    for (level, text) in levels {
        let profile = certificate_profile(&icp_info(Some(level)), None);
        assert_eq!(profile.icp_brasil.as_deref(), Some(text), "{level}");
    }
}

#[test]
fn an_unknown_or_missing_class_is_plain_icp_brasil() {
    for level in [
        None,
        Some(IcpLevel::Other(0)),
        Some(IcpLevel::Other(999)),
        Some(IcpLevel::Other(u32::MAX)),
    ] {
        let profile = certificate_profile(&icp_info(level), None);
        assert_eq!(
            profile.icp_brasil.as_deref(),
            Some("ICP-Brasil"),
            "{level:?}"
        );
    }
}

#[test]
fn reads_the_class_from_real_certificates() {
    let cases = [
        ("icp-pf-a3", Some("A3")),
        ("icp-pj-a1", Some("A1")),
        ("icp-level-303", Some("T3")),
        ("icp-level-102", Some("S2")),
        ("icp-level-999", Some("ICP-Brasil")),
        ("icp-level-huge", Some("ICP-Brasil")),
        ("icp-san-only", Some("ICP-Brasil")),
        ("icp-lookalike", None),
        ("p256", None),
    ];
    for (name, expected) in cases {
        let profile = certificate_profile(&info(name), None);
        assert_eq!(profile.icp_brasil.as_deref(), expected, "{name}");
    }
}

// --- eidas ---------------------------------------------------------------------------------------

#[test]
fn a_certificate_without_qc_statements_has_no_eidas_profile() {
    assert_eq!(certificate_profile(&blank_info(), None).eidas, None);
}

#[test]
fn maps_compliance_sscd_and_types() {
    let info = qualified_info(true, true, vec![QcType::ESign]);
    assert_eq!(
        certificate_profile(&info, None).eidas,
        eidas(true, true, vec![EidasType::Esign])
    );
}

#[test]
fn compliance_is_qualified_and_sscd_is_qscd() {
    let only_compliance = qualified_info(true, false, vec![]);
    assert_eq!(
        certificate_profile(&only_compliance, None).eidas,
        eidas(true, false, vec![])
    );
    let only_sscd = qualified_info(false, true, vec![]);
    assert_eq!(
        certificate_profile(&only_sscd, None).eidas,
        eidas(false, true, vec![])
    );
}

#[test]
fn present_but_empty_qc_statements_still_give_a_profile() {
    let info = qualified_info(false, false, vec![]);
    assert_eq!(
        certificate_profile(&info, None).eidas,
        eidas(false, false, vec![])
    );
}

#[test]
fn maps_every_type_keeping_order_and_repeats() {
    let info = qualified_info(
        true,
        false,
        vec![QcType::Web, QcType::ESign, QcType::ESeal, QcType::Web],
    );
    assert_eq!(
        certificate_profile(&info, None).eidas,
        eidas(
            true,
            false,
            vec![
                EidasType::Web,
                EidasType::Esign,
                EidasType::Eseal,
                EidasType::Web
            ]
        )
    );
}

#[test]
fn reads_the_profile_from_real_certificates() {
    let cases = [
        ("qc-esign-sscd", eidas(true, true, vec![EidasType::Esign])),
        ("qc-eseal", eidas(true, false, vec![EidasType::Eseal])),
        ("qc-web", eidas(false, false, vec![EidasType::Web])),
        (
            "qc-type-order",
            eidas(
                false,
                false,
                vec![EidasType::Web, EidasType::Esign, EidasType::Eseal],
            ),
        ),
        ("qc-sscd-only", eidas(false, true, vec![])),
        ("qc-unknown-only", eidas(false, false, vec![])),
        ("qc-empty", eidas(false, false, vec![])),
        ("p256", None),
        ("icp-pf-a3", None),
    ];
    for (name, expected) in cases {
        assert_eq!(
            certificate_profile(&info(name), None).eidas,
            expected,
            "{name}"
        );
    }
}

// --- key_storage ---------------------------------------------------------------------------------

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

// --- the whole profile ---------------------------------------------------------------------------

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

// --- total mappings (already implemented) ----------------------------------------------------------

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
